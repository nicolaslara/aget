use assert_cmd::Command;
use serde_json::json;

use crate::support::get_cli::{mock_backend_command, success_data};

#[test]
fn get_noisy_backend_output_does_not_deadlock() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "noisy_success", "content": "# Noisy", "noise_repetitions": 20000}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--timeout",
            "2",
            "--envelope",
            "json",
            "get",
            "https://example.com/noisy",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(json["content"], "# Noisy");
}

#[test]
fn get_backend_stdout_logs_before_json_succeeds() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "stdout_logs_success",
            "content": "# Logged stdout",
            "stdout_lines": ["[INIT] Starting browser", "[FETCH] Fetching", "[COMPLETE] Crawling done"]
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/stdout-logs",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(json["content"], "# Logged stdout");
}
