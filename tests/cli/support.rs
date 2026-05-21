use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub fn local_server(body: &str) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let body = body.to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let mut request = Vec::new();
        let mut buffer = [0; 512];
        while request.len() < 16 * 1024 {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    request.extend_from_slice(&buffer[..read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
        let _ = stream.flush();
        let _ = stream.shutdown(Shutdown::Both);
    });
    (url, handle)
}

pub fn mock_current_tab_cdp_server(final_url: &str, html: &str) -> (u16, JoinHandle<()>) {
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

fn serve_attached_page_cdp(
    websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
    final_url: &str,
    html: &str,
) {
    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setDiscoverTargets");
    reply_ok(websocket, &request, serde_json::json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.getTargets");
    reply_ok(
        websocket,
        &request,
        serde_json::json!({
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
    reply_ok(
        websocket,
        &request,
        serde_json::json!({ "sessionId": "session-1" }),
    );

    for expected_method in [
        "Page.enable",
        "Runtime.enable",
        "Runtime.runIfWaitingForDebugger",
        "Network.enable",
    ] {
        let request = read_cdp_request(websocket);
        assert_eq!(request["method"], expected_method);
        reply_ok(websocket, &request, serde_json::json!({}));
    }

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setAutoAttach");
    reply_ok(websocket, &request, serde_json::json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    reply_ok(
        websocket,
        &request,
        serde_json::json!({ "result": { "type": "undefined" } }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(request["params"]["expression"], "location.href");
    reply_ok(
        websocket,
        &request,
        serde_json::json!({ "result": { "type": "string", "value": final_url } }),
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
        serde_json::json!({ "result": { "type": "string", "value": html } }),
    );
}

fn read_cdp_request(
    websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
) -> serde_json::Value {
    let tungstenite::Message::Text(text) = websocket.read().unwrap() else {
        panic!("expected text CDP command");
    };
    serde_json::from_str(text.as_ref()).unwrap()
}

fn reply_ok(
    websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
    request: &serde_json::Value,
    result: serde_json::Value,
) {
    let id = request
        .get("id")
        .and_then(serde_json::Value::as_u64)
        .unwrap();
    websocket
        .send(tungstenite::Message::Text(
            serde_json::json!({ "id": id, "result": result })
                .to_string()
                .into(),
        ))
        .unwrap();
}

pub fn mock_backend_command() -> String {
    shell_quote(&mock_tool_path("aget-mock-backend").to_string_lossy())
}

fn mock_tool_path(name: &str) -> PathBuf {
    let tools = MOCK_TOOLS.get_or_init(build_mock_tools);
    let binary = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    tools.target_dir.join("debug").join(binary)
}

struct MockTools {
    target_dir: PathBuf,
}

static MOCK_TOOLS: OnceLock<MockTools> = OnceLock::new();

fn build_mock_tools() -> MockTools {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("tests/fixtures/mock-tools/Cargo.toml");
    let target_dir = root.join("target/aget-mock-tools");
    let status = StdCommand::new("cargo")
        .args([
            "build",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--target-dir",
            target_dir.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    MockTools { target_dir }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
