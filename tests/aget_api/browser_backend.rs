use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};

use aget::{Aget, AgetBrowserBackend, CurrentTabOptions, ErrorCode, OutputFormat};
use serde_json::{json, Value};
use tungstenite::{Message, WebSocket};

use crate::support::pending_login;

#[test]
fn aget_browser_backend_does_not_save_failed_profile_path_import() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let missing_profile = temp.path().join("missing-profile");

    let error = Aget::new(&home)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .import_chrome_session(
            missing_profile.to_string_lossy().to_string(),
            "chrome",
            vec!["example.com".to_string()],
        )
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::RequiresUserAction);
    assert!(error.to_string().contains("does not exist"));
    assert!(!home.join("sessions/chrome.json").exists());
}

#[test]
fn aget_browser_backend_cancels_pending_login_without_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let tmp_dir = home.join("tmp");
    let profile = tmp_dir.join("owned-login/aget-docs");
    fs::create_dir_all(&profile).unwrap();
    fs::create_dir_all(&tmp_dir).unwrap();
    fs::write(
        tmp_dir.join("login-docs.json"),
        serde_json::to_vec_pretty(&pending_login(
            "docs".to_string(),
            profile.to_string_lossy().into_owned(),
            "https://example.com/login".to_string(),
        ))
        .unwrap(),
    )
    .unwrap();

    let cancelled = Aget::new(&home)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .cancel_login_session("docs")
        .unwrap();

    assert_eq!(cancelled.pending.name, "docs");
    assert!(!profile.exists());
    assert!(!tmp_dir.join("login-docs.json").exists());
}

#[test]
fn aget_current_tab_api_renders_mock_cdp_page_without_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let (port, server) = mock_current_tab_cdp_server(
        "https://example.com/current",
        "<html><body><main><h1>API Tab</h1><p>Owned current-tab content.</p></main></body></html>",
    );

    let mut options = CurrentTabOptions::new(port);
    options.allow_private_content = true;
    options.content_format = OutputFormat::Text;
    options.selector = Some("main".to_string());
    let result = Aget::new(&home)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .current_tab(options)
        .unwrap();

    assert_eq!(result.url, "current-tab");
    assert_eq!(result.final_url, "https://example.com/current");
    assert_eq!(result.extractor, "aget-owned-current-tab");
    assert_eq!(result.content_format, "text");
    assert_eq!(result.content, "API Tab Owned current-tab content.");
    assert!(result.sensitive);
    assert!(result.sessions.is_empty());
    server.join().unwrap();
}

fn mock_current_tab_cdp_server(final_url: &str, html: &str) -> (u16, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let final_url = final_url.to_string();
    let html = html.to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0; 512];
        let _ = stream.read(&mut request);
        let body =
            format!(r#"{{"webSocketDebuggerUrl":"ws://localhost:9999/devtools/browser/current"}}"#);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();

        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        serve_attached_page_cdp(&mut websocket, &final_url, &html);
        let _ = websocket.close(None);
    });
    (port, handle)
}

fn serve_attached_page_cdp(websocket: &mut WebSocket<TcpStream>, final_url: &str, html: &str) {
    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setDiscoverTargets");
    reply_ok(websocket, &request, json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.getTargets");
    reply_ok(
        websocket,
        &request,
        json!({
            "targetInfos": [{
                "targetId": "page-1",
                "type": "page",
                "url": final_url,
                "title": "Current"
            }]
        }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.attachToTarget");
    reply_ok(websocket, &request, json!({ "sessionId": "session-1" }));

    for expected_method in [
        "Page.enable",
        "Runtime.enable",
        "Runtime.runIfWaitingForDebugger",
        "Network.enable",
    ] {
        let request = read_cdp_request(websocket);
        assert_eq!(request["method"], expected_method);
        reply_ok(websocket, &request, json!({}));
    }

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setAutoAttach");
    reply_ok(websocket, &request, json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    reply_ok(
        websocket,
        &request,
        json!({ "result": { "type": "undefined" } }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(request["params"]["expression"], "location.href");
    reply_ok(
        websocket,
        &request,
        json!({ "result": { "type": "string", "value": final_url } }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(
        request["params"]["expression"],
        "document.documentElement.outerHTML || ''"
    );
    reply_ok(
        websocket,
        &request,
        json!({ "result": { "type": "string", "value": html } }),
    );
}

fn read_cdp_request(websocket: &mut WebSocket<TcpStream>) -> Value {
    let Message::Text(text) = websocket.read().unwrap() else {
        panic!("expected text CDP command");
    };
    serde_json::from_str(text.as_ref()).unwrap()
}

fn reply_ok(websocket: &mut WebSocket<TcpStream>, request: &Value, result: Value) {
    let id = request.get("id").and_then(Value::as_u64).unwrap();
    websocket
        .send(Message::Text(
            json!({ "id": id, "result": result }).to_string().into(),
        ))
        .unwrap();
}
