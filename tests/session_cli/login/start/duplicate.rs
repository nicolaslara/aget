use std::fs;

use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::agent_browser_tool;

#[test]
fn session_login_start_rejects_duplicate_pending_flow_without_overwriting() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

    let first_url = "https://www.nytimes.com/article";
    let second_url = "https://www.nytimes.com/login";

    let mut first = Command::cargo_bin("aget").unwrap();
    first
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            first_url,
            "--profile",
            "aget-hi",
        ])
        .assert()
        .success();

    let pending_path = aget_home.join("tmp/login-news.json");
    let original_pending = fs::read_to_string(&pending_path).unwrap();

    let mut second = Command::cargo_bin("aget").unwrap();
    let output = second
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "news",
            "--url",
            second_url,
            "--profile",
            "aget-hi-2",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("pending login flow named 'news' already exists"));
    assert_eq!(fs::read_to_string(&pending_path).unwrap(), original_pending);
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""--profile", "aget-hi", "--session", "aget-login-news", "open""#));
    assert!(!log.contains(r#"aget-hi-2"#));
    assert!(!log.contains(r#"["--session", "aget-login-news", "close"]"#));
}
