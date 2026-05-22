use std::fs;

use aget::session::SessionStore;
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, nyt_state, provider_only_state};

#[test]
fn session_login_finish_reports_close_failure_and_keeps_pending_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({
            "behavior": "close_failure",
            "state": nyt_state("nyt_session", "token"),
            "close_stderr": "close failed"
        }),
    );

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            "https://www.nytimes.com/article",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    let output = finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["--envelope", "json", "session", "login", "finish", "news"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("close failed"));
    assert!(aget_home.join("tmp/login-news.json").exists());
    assert!(SessionStore::new(&aget_home).unwrap().load("news").is_err());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#"["--session", "aget-login-news", "close"]"#));
}

#[test]
fn session_login_finish_rejects_provider_only_state_without_saving_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": provider_only_state()}),
    );
    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "hellointerview",
            "--url",
            "https://www.hellointerview.com/login",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    let output = finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "hellointerview",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "requires_user_action");
    assert!(SessionStore::new(&aget_home)
        .unwrap()
        .load("hellointerview")
        .is_err());
    assert!(aget_home.join("tmp/login-hellointerview.json").exists());
}
