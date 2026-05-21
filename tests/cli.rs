use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_includes_global_flags() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("--envelope")
            .and(predicate::str::contains("--timeout"))
            .and(predicate::str::contains("get")),
    );
}

#[test]
fn get_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["get", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget get")
            .and(predicate::str::contains("<URL>"))
            .and(predicate::str::contains("--content-format"))
            .and(predicate::str::contains("--inline-content"))
            .and(predicate::str::contains("--selector"))
            .and(predicate::str::contains("--exclude-selector"))
            .and(predicate::str::contains("--wait-for-selector"))
            .and(predicate::str::contains("--max-chars"))
            .and(predicate::str::contains("--backend-option")),
    );
}

#[test]
fn current_tab_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["current-tab", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Usage: aget current-tab")
                .and(predicate::str::contains("--cdp-port"))
                .and(predicate::str::contains("--allow-private-content"))
                .and(predicate::str::contains("--content-format"))
                .and(predicate::str::contains("--inline-content")),
        );
}

#[test]
fn get_rejects_invalid_format() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["get", "https://example.com", "--content-format", "pdf"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn get_json_parse_error_uses_structured_envelope() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com",
            "--content-format",
            "pdf",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "get");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("invalid value"));
}

#[test]
fn get_rejects_malformed_extractor_option() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args([
        "get",
        "https://example.com",
        "--backend-option",
        "missing-equals",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("key=value"));
}

#[test]
fn get_rejects_unnamespaced_extractor_option() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args([
        "get",
        "https://example.com",
        "--backend-option",
        "wait_until=networkidle",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("must be namespaced"));
}

#[test]
fn top_level_url_runs_get_command() {
    let temp = tempfile::tempdir().unwrap();
    let (url, server) = local_server("<main># Example</main>");
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.env("AGET_HOME", temp.path().join("aget-home"))
        .env("AGET_CRAWL4AI_COMMAND", mock_backend_command())
        .arg(url)
        .assert()
        .success()
        .stdout(predicate::str::contains("# Example"));
    server.join().unwrap();
}

#[test]
fn top_level_url_alias_preserves_get_output_flags() {
    let temp = tempfile::tempdir().unwrap();
    let (url, server) = local_server("<main>Example</main>");
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .env("AGET_CRAWL4AI_COMMAND", mock_backend_command())
        .args(["--envelope", "json", &url, "--content-format", "html"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "get");
    assert_eq!(json["data"]["content_format"], "html");
    assert_eq!(json["data"]["content"], "<main>Example</main>");
    server.join().unwrap();
}

#[test]
fn envelope_global_flag_emits_structured_output() {
    let temp = tempfile::tempdir().unwrap();
    let (url, server) = local_server("<main># Example</main>");
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .env("AGET_CRAWL4AI_COMMAND", mock_backend_command())
        .args(["--envelope", "json", "get", &url])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "get");
    assert_eq!(json["data"]["content"], "# Example");
    server.join().unwrap();
}

#[test]
fn current_tab_requires_private_content_consent() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .args(["--envelope", "json", "current-tab", "--cdp-port", "9222"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "current-tab");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("private content"));
}

#[test]
fn current_tab_renders_mock_cdp_page_without_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let (port, server) = mock_current_tab_cdp_server(
        "https://example.com/current",
        "<html><body><nav>skip</nav><main><h1>Tab Title</h1><p>Private tab body.</p></main></body></html>",
    );
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .args([
            "--envelope",
            "json",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--allow-private-content",
            "--inline-content",
            "always",
            "--selector",
            "main",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "current-tab");
    assert_eq!(json["data"]["url"], "current-tab");
    assert_eq!(json["data"]["final_url"], "https://example.com/current");
    assert_eq!(json["data"]["extractor"], "aget-owned-current-tab");
    assert_eq!(json["data"]["sensitive"], true);
    assert!(json["data"]["content"]
        .as_str()
        .unwrap()
        .contains("Tab Title"));
    assert!(json["data"]["content"]
        .as_str()
        .unwrap()
        .contains("Private tab body."));
    assert!(json["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning.as_str().unwrap().contains("private")));
    server.join().unwrap();
}

fn local_server(body: &str) -> (String, JoinHandle<()>) {
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

fn mock_backend_command() -> String {
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
