use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_includes_global_flags() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("--json")
            .and(predicate::str::contains("--envelope"))
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
            .and(predicate::str::contains("--format"))
            .and(predicate::str::contains("--selector"))
            .and(predicate::str::contains("--exclude-selector"))
            .and(predicate::str::contains("--wait-for"))
            .and(predicate::str::contains("--max-chars"))
            .and(predicate::str::contains("--extractor-option")),
    );
}

#[test]
fn get_rejects_invalid_format() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["get", "https://example.com", "--format", "pdf"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn get_json_parse_error_uses_structured_envelope() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .args(["--json", "get", "https://example.com", "--format", "pdf"])
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
        "--extractor-option",
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
        "--extractor-option",
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
        .args(["--json", &url, "--format", "html"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "get");
    assert_eq!(json["data"]["format"], "html");
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
        .args(["--envelope", "get", &url])
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

fn local_server(body: &str) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let body = body.to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, handle)
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
