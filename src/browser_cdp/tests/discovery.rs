use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::error::{AgetError, ErrorCode};
use crate::process::{create_private_file, TempOutputFile};
use serde_json::{json, Value};
use tungstenite::Message;

use super::super::discovery::{
    classify_chrome_startup_error, connect_existing_profile_browser, devtools_ws_url_from_stderr,
    discover_cdp_ws_url, read_devtools_active_port, relevant_chrome_stderr, rewrite_cdp_ws_host,
    wait_for_devtools_active_port,
};
use super::super::process::terminate_child;

#[test]
fn parses_devtools_ws_url_from_chrome_stderr() {
    let stderr = "\
noise
DevTools listening on ws://127.0.0.1:9222/devtools/browser/fallback
more noise";

    assert_eq!(
        devtools_ws_url_from_stderr(stderr).as_deref(),
        Some("ws://127.0.0.1:9222/devtools/browser/fallback")
    );
}

#[test]
fn chrome_stderr_detail_includes_sandbox_hint() {
    let detail = relevant_chrome_stderr(
        "Failed to move to new namespace: PID namespaces supported, Network namespace supported, but failed: errno = Operation not permitted",
    );

    assert!(detail.contains("Failed to move to new namespace"));
    assert!(detail.contains("Chrome sandbox/namespace startup failure"));
    assert!(detail.contains("AGET_CHROME_COMMAND"));
}

#[test]
fn chrome_stderr_detail_labels_generic_tail_lines() {
    let detail = relevant_chrome_stderr(
        "info: startup preparing\n\
         info: still warming\n\
         note: first generic line\n\
         trace: second generic line\n\
         debug: third generic line\n\
         debug: fourth generic line",
    );

    assert!(detail.contains("Chrome stderr (last 5 lines):"));
    assert!(!detail.contains("startup preparing"));
    assert!(detail.contains("info: still warming"));
    assert!(detail.contains("note: first generic line"));
    assert!(detail.contains("trace: second generic line"));
    assert!(detail.contains("debug: third generic line"));
    assert!(detail.contains("debug: fourth generic line"));
}

#[test]
fn chrome_startup_error_adds_silent_exit_hint_without_stderr() {
    let temp = tempfile::tempdir().unwrap();
    let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
    let error = AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: "owned browser fallback Chrome exited before CDP startup".to_string(),
    };

    let classified = classify_chrome_startup_error("owned Chrome test", error, &stderr_capture);

    assert_eq!(classified.code(), ErrorCode::BackendUnavailable);
    let message = classified.to_string();
    assert!(message.contains("Chrome exited without startup diagnostics"));
    assert!(message.contains("AGET_CHROME_COMMAND"));
}

#[test]
fn chrome_startup_error_includes_labeled_generic_stderr() {
    let temp = tempfile::tempdir().unwrap();
    let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
    fs::write(
        stderr_capture.path(),
        "startup line one\nstartup line two\nstartup line three\nstartup line four\nstartup line five\nstartup line six\n",
    )
    .unwrap();
    let error = AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: "owned browser fallback Chrome exited before CDP startup".to_string(),
    };

    let classified = classify_chrome_startup_error("owned Chrome test", error, &stderr_capture);

    assert_eq!(classified.code(), ErrorCode::BackendUnavailable);
    let message = classified.to_string();
    assert!(message.contains("Chrome stderr (last 5 lines):"));
    assert!(!message.contains("startup line one"));
    assert!(message.contains("startup line two"));
    assert!(message.contains("startup line three"));
    assert!(message.contains("startup line four"));
    assert!(message.contains("startup line five"));
    assert!(message.contains("startup line six"));
}

#[cfg(unix)]
#[test]
fn wait_for_devtools_active_port_uses_stderr_fallback() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
    let stderr = create_private_file(stderr_capture.path()).unwrap();
    let fake_chrome = temp.path().join("fake-chrome");
    fs::write(
        &fake_chrome,
        "#!/bin/sh\n\
         echo 'DevTools listening on ws://127.0.0.1:9222/devtools/browser/fallback' >&2\n\
         sleep 5\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_chrome).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&fake_chrome, permissions).unwrap();
    let mut child = Command::new(&fake_chrome)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr))
        .spawn()
        .unwrap();

    let ws_url = wait_for_devtools_active_port(
        &mut child,
        temp.path(),
        &stderr_capture,
        Duration::from_secs(5),
    )
    .unwrap();
    terminate_child(&mut child);

    assert_eq!(ws_url, "ws://127.0.0.1:9222/devtools/browser/fallback");
}

#[test]
fn parses_devtools_active_port_file() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join("DevToolsActivePort"),
        "49152\n/devtools/browser/abc\n",
    )
    .unwrap();

    assert_eq!(
        read_devtools_active_port(temp.path()),
        Some((49152, "/devtools/browser/abc".to_string()))
    );
}

#[test]
fn existing_profile_attach_removes_stale_devtools_active_port() {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let temp = tempfile::tempdir().unwrap();
    let active_port = temp.path().join("DevToolsActivePort");
    fs::write(&active_port, format!("{port}\n/devtools/browser/stale\n")).unwrap();

    let client = connect_existing_profile_browser(temp.path(), Duration::from_millis(100))
        .expect("stale CDP attach should not be fatal");

    assert!(client.is_none());
    assert!(!active_port.exists());
}

#[test]
fn rewrites_discovered_cdp_websocket_host_and_port() {
    assert_eq!(
        rewrite_cdp_ws_host("ws://localhost:9222/devtools/browser/abc", 49152).as_deref(),
        Some("ws://127.0.0.1:49152/devtools/browser/abc")
    );
    assert!(rewrite_cdp_ws_host("http://localhost:9222/json/version", 49152).is_none());
}

#[test]
fn discovers_cdp_websocket_url_from_json_version() {
    let body = r#"{"webSocketDebuggerUrl":"ws://localhost:9999/devtools/browser/discovered"}"#;
    let (port, handle) = serve_cdp_discovery_responses(vec![http_json_response(200, body)]);

    let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
    handle.join().unwrap();

    assert_eq!(
        ws_url,
        format!("ws://127.0.0.1:{port}/devtools/browser/discovered")
    );
}

#[test]
fn discovers_cdp_websocket_url_from_json_list_fallback() {
    let list_body = r#"[
        {
            "type": "page",
            "webSocketDebuggerUrl": "ws://localhost:9999/devtools/page/ignored"
        },
        {
            "type": "browser",
            "webSocketDebuggerUrl": "ws://localhost:9999/devtools/browser/list"
        }
    ]"#;
    let (port, handle) = serve_cdp_discovery_responses(vec![
        http_json_response(200, r#"{}"#),
        http_json_response(200, list_body),
    ]);

    let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
    handle.join().unwrap();

    assert_eq!(
        ws_url,
        format!("ws://127.0.0.1:{port}/devtools/browser/list")
    );
}

#[test]
fn discovers_cdp_websocket_url_from_first_list_target_with_ws() {
    let list_body = r#"[
        {
            "type": "page"
        },
        {
            "type": "page",
            "webSocketDebuggerUrl": "ws://localhost:9999/devtools/page/fallback"
        }
    ]"#;
    let (port, handle) = serve_cdp_discovery_responses(vec![
        http_json_response(404, r#"{"error":"missing"}"#),
        http_json_response(200, list_body),
    ]);

    let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
    handle.join().unwrap();

    assert_eq!(
        ws_url,
        format!("ws://127.0.0.1:{port}/devtools/page/fallback")
    );
}

#[test]
fn discovers_cdp_websocket_url_from_direct_websocket_fallback() {
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 512];
            let _ = stream.read(&mut request);
            let response = http_json_response(404, r#"{"error":"missing"}"#);
            stream.write_all(response.as_bytes()).unwrap();
        }

        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let Message::Text(text) = websocket.read().unwrap() else {
            panic!("expected text CDP command");
        };
        let request: Value = serde_json::from_str(text.as_ref()).unwrap();
        let id = request.get("id").and_then(Value::as_u64).unwrap();
        let reply = json!({
            "id": id,
            "result": {
                "protocolVersion": "1.3",
                "product": "Chrome/136"
            }
        })
        .to_string();
        websocket.send(Message::Text(reply.into())).unwrap();
        let _ = websocket.close(None);
    });

    let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
    handle.join().unwrap();

    assert_eq!(ws_url, format!("ws://127.0.0.1:{port}/devtools/browser"));
}

fn serve_cdp_discovery_responses(responses: Vec<String>) -> (u16, thread::JoinHandle<()>) {
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 512];
            let _ = stream.read(&mut request);
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (port, handle)
}

fn http_json_response(status: u16, body: &str) -> String {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Status",
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}
