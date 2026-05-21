use std::fs;

use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, success_data};

#[test]
fn session_login_cancel_closes_only_pending_agent_browser_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

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

    let mut cancel = Command::cargo_bin("aget").unwrap();
    let output = cancel
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "cancel",
            "hellointerview",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.login.cancel");
    assert_eq!(json["state"], "login_cancelled");
    assert_eq!(json["name"], "hellointerview");
    assert_eq!(json["agent_session"], "aget-login-hellointerview");
    assert!(!aget_home.join("tmp/login-hellointerview.json").exists());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""open""#));
    assert!(log.contains(r#"["--session", "aget-login-hellointerview", "close"]"#));
}

#[test]
fn session_login_cancel_cleans_profile_and_pending_when_close_fails() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"behavior": "close_failure", "close_stderr": "close failed", "close_exit_code": 2}),
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
            "https://www.nytimes.com/login",
        ])
        .assert()
        .success();
    let profile = aget_home.join("tmp/agent-browser/aget-news");
    fs::create_dir_all(&profile).unwrap();

    let mut cancel = Command::cargo_bin("aget").unwrap();
    let output = cancel
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["--envelope", "json", "session", "login", "cancel", "news"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "session.login.cancel");
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(!aget_home.join("tmp/login-news.json").exists());
    assert!(!profile.exists());
}
