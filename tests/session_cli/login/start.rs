use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, success_data, success_envelope};

#[test]
fn session_login_start_opens_aget_browser_profile_and_records_pending_flow() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let target_url = "https://www.nytimes.com/article";
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));
    let expected_profile = aget_home.join("tmp/agent-browser/aget-news");

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
            "news",
            "--url",
            target_url,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = success_envelope(&output, "session.login.start");
    assert!(envelope["warnings"][0]
        .as_str()
        .unwrap()
        .contains("prefer signing in with your real browser"));
    let json = envelope["data"].clone();
    assert_eq!(json["state"], "login_started");
    assert!(json.get("site").is_none());
    assert_eq!(json["name"], "news");
    assert_eq!(
        PathBuf::from(json["profile"].as_str().unwrap()),
        expected_profile
    );
    assert_eq!(json["url"], target_url);
    assert_eq!(
        json["allowed_domains"],
        serde_json::json!(["nytimes.com", "www.nytimes.com"])
    );
    assert_eq!(
        json["next_command"],
        serde_json::json!(["aget", "session", "login", "finish", "news"])
    );
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(&format!(
        r#""--profile", "{}", "--session", "aget-login-news", "open""#,
        expected_profile.display()
    )));
    assert!(log.contains(target_url));
    assert!(aget_home.join("tmp/login-news.json").exists());
    assert!(expected_profile.parent().unwrap().exists());
}

#[test]
fn session_login_start_rejects_http_url_before_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

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
            "news",
            "--url",
            "http://www.nytimes.com/login",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "session.login.start");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("login URL must use https"));
    assert!(!log_path.exists() || fs::read_to_string(&log_path).unwrap().is_empty());
    assert!(!aget_home.join("tmp/login-news.json").exists());
}

#[test]
fn session_login_start_uses_exact_non_www_url_host_scope() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

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
            "docs",
            "--url",
            "https://docs.example.com/login",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.login.start");
    assert_eq!(json["name"], "docs");
    assert_eq!(
        json["allowed_domains"],
        serde_json::json!(["docs.example.com"])
    );
    assert!(aget_home.join("tmp/login-docs.json").exists());
}

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
