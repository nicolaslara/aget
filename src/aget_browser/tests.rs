use std::fs;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use tungstenite::{Message, WebSocket};

use super::current_tab::{AttachedPageRequest, CdpEndpointRequest, CurrentTabRequest};
use super::*;
use crate::session::login::PendingLogin;

fn read_cdp_request(websocket: &mut WebSocket<TcpStream>) -> Value {
    let Message::Text(text) = websocket.read().unwrap() else {
        panic!("expected text CDP command");
    };
    serde_json::from_str(text.as_ref()).unwrap()
}

fn reply_ok(websocket: &mut WebSocket<TcpStream>, request: &Value, result: Value) {
    let id = request
        .get("id")
        .and_then(serde_json::Value::as_u64)
        .unwrap();
    websocket
        .send(Message::Text(
            json!({ "id": id, "result": result }).to_string().into(),
        ))
        .unwrap();
}

fn serve_cdp_discovery_responses(responses: Vec<String>) -> (u16, thread::JoinHandle<()>) {
    use std::io::{Read as _, Write as _};

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
            "targetInfos": [
                {
                    "targetId": "page-1",
                    "type": "page",
                    "url": final_url,
                    "title": "Current"
                }
            ]
        }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.attachToTarget");
    assert_eq!(request["params"]["targetId"], "page-1");
    reply_ok(websocket, &request, json!({ "sessionId": "session-1" }));

    for expected_method in [
        "Page.enable",
        "Runtime.enable",
        "Runtime.runIfWaitingForDebugger",
        "Network.enable",
    ] {
        let request = read_cdp_request(websocket);
        assert_eq!(request["method"], expected_method);
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(websocket, &request, json!({}));
    }

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setAutoAttach");
    assert_eq!(request["sessionId"], "session-1");
    reply_ok(websocket, &request, json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(request["sessionId"], "session-1");
    assert!(request["params"]["expression"]
        .as_str()
        .unwrap()
        .contains("querySelectorAll"));
    reply_ok(
        websocket,
        &request,
        json!({ "result": { "type": "undefined" } }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(request["sessionId"], "session-1");
    assert_eq!(request["params"]["expression"], "location.href");
    reply_ok(
        websocket,
        &request,
        json!({ "result": { "type": "string", "value": final_url } }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(request["sessionId"], "session-1");
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

#[test]
fn cancels_pending_login_without_aget_facade_or_command_backend() {
    let temp = tempfile::tempdir().unwrap();
    let tmp_dir = temp.path().join("tmp");
    let profile = tmp_dir.join("owned-login/aget-docs");
    fs::create_dir_all(&profile).unwrap();
    fs::write(
        tmp_dir.join("login-docs.json"),
        serde_json::to_vec_pretty(&PendingLogin {
            name: "docs".to_string(),
            profile: profile.to_string_lossy().into_owned(),
            agent_session: "aget-login-docs".to_string(),
            url: "https://example.com/login".to_string(),
            allowed_domains: vec!["example.com".to_string()],
            browser_pid: None,
        })
        .unwrap(),
    )
    .unwrap();

    let cancelled = AgetBrowser::default()
        .cancel_login_session(LoginCancelOptions {
            name: "docs".to_string(),
            tmp_dir: tmp_dir.clone(),
        })
        .unwrap();

    assert_eq!(cancelled.pending.name, "docs");
    assert!(!profile.exists());
    assert!(!tmp_dir.join("login-docs.json").exists());
}

#[test]
fn discovers_cdp_endpoint_without_aget_facade_or_command_backend() {
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

    let endpoint = AgetBrowser::default()
        .discover_cdp_endpoint(CdpEndpointRequest {
            port,
            timeout: Duration::from_secs(2),
        })
        .unwrap();

    assert_eq!(
        endpoint.ws_url,
        format!("ws://127.0.0.1:{port}/devtools/browser/list")
    );
    handle.join().unwrap();
}

#[test]
fn renders_current_tab_from_explicit_cdp_port_without_aget_facade_or_command_backend() {
    use std::io::{Read as _, Write as _};

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0u8; 512];
        let _ = stream.read(&mut request);
        let body = r#"{"webSocketDebuggerUrl":"ws://localhost:9999/devtools/browser/current"}"#;
        stream
            .write_all(http_json_response(200, body).as_bytes())
            .unwrap();

        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        serve_attached_page_cdp(
            &mut websocket,
            "https://example.com/current",
            "<html><body>Current composed page</body></html>",
        );
        let _ = websocket.close(None);
    });

    let rendered = AgetBrowser::default()
        .render_current_tab(CurrentTabRequest {
            port,
            wait_for_selector: None,
            wait_for_images: false,
            flatten_shadow_dom: false,
            settle_delay: Duration::ZERO,
            discovery_timeout: Duration::from_secs(2),
            page_timeout: Duration::from_secs(2),
            wait_for_timeout: None,
            timeout: Duration::from_secs(2),
        })
        .unwrap();

    assert_eq!(
        rendered.cdp_ws_url,
        format!("ws://127.0.0.1:{port}/devtools/browser/current")
    );
    assert_eq!(rendered.final_url, "https://example.com/current");
    assert_eq!(
        rendered.html,
        "<html><body>Current composed page</body></html>"
    );
    assert!(rendered.warnings.is_empty());
    handle.join().unwrap();
}

#[test]
fn renders_attached_page_without_aget_facade_or_command_backend() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        serve_attached_page_cdp(
            &mut websocket,
            "https://example.com/current",
            "<html><body>Current engine page</body></html>",
        );
        let _ = websocket.close(None);
    });

    let rendered = AgetBrowser::default()
        .render_attached_page(AttachedPageRequest {
            ws_url: &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
            wait_for_selector: None,
            wait_for_images: false,
            flatten_shadow_dom: false,
            settle_delay: Duration::ZERO,
            page_timeout: Duration::from_secs(2),
            wait_for_timeout: None,
            timeout: Duration::from_secs(2),
        })
        .unwrap();

    assert_eq!(rendered.final_url, "https://example.com/current");
    assert_eq!(
        rendered.html,
        "<html><body>Current engine page</body></html>"
    );
    assert!(rendered.warnings.is_empty());
    handle.join().unwrap();
}
