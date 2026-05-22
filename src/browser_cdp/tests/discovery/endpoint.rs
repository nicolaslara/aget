use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use tungstenite::Message;

use crate::browser_cdp::discovery::{discover_cdp_ws_url, rewrite_cdp_ws_host};

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
