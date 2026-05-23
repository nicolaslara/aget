use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, success_envelope};

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
    assert_eq!(json["injected_sessions"], serde_json::json!([]));
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
