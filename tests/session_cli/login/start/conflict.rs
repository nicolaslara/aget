use std::fs;

use aget::session::SessionStore;
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, named_session};

#[test]
fn session_login_start_rejects_conflicting_injected_sessions_before_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));
    let store = SessionStore::new(&aget_home).unwrap();
    store
        .save(&named_session(
            "oauth-a",
            "sid",
            "first-secret",
            "accounts.example.com",
        ))
        .unwrap();
    store
        .save(&named_session(
            "oauth-b",
            "sid",
            "second-secret",
            "accounts.example.com",
        ))
        .unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "target",
            "--url",
            "https://example.com/login",
            "--session",
            "oauth-a",
            "--session",
            "oauth-b",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "session.login.start");
    assert_eq!(json["error"]["code"], "session_conflict");
    assert!(!log_path.exists() || fs::read_to_string(&log_path).unwrap().is_empty());
    assert!(!aget_home.join("tmp/login-target.json").exists());
}
