use assert_cmd::Command;
use serde_json::json;

use crate::support::get_cli::{mock_backend_command, save_cookie_session};

#[test]
fn get_rejects_session_replay_outside_saved_scope() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    save_cookie_session(
        &aget_home,
        "private-docs",
        "private.example.com",
        "sid",
        "secret-cookie",
    );
    let fake_backend = mock_backend_command(temp.path(), json!({"behavior": "exit"}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://unrelated.example/page",
            "--session",
            "private-docs",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "get");
    assert_eq!(json["error"]["code"], "privacy_policy_blocked");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("outside request host"));
}
