use std::fs;

use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, mock_backend_command};

#[test]
fn session_authorize_rejects_unsupported_browser_without_importing() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "content": "Please sign in before reading this page"
        }),
    );
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({
            "state": {
                "cookies": [{
                    "name": "sid",
                    "value": "allowed-secret",
                    "domain": "example.com",
                    "path": "/",
                    "httpOnly": true,
                    "secure": true
                }],
                "origins": []
            }
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "authorize",
            "news",
            "--url",
            "https://example.com/private",
            "--browser",
            "firefox",
            "--browser-profile",
            "/tmp/firefox-profile",
            "--allow-domain",
            "example.com",
            "--must-contain",
            "Welcome",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "session.authorize");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("does not support 'firefox' yet"));
    assert!(!log_path.exists() || fs::read_to_string(&log_path).unwrap().is_empty());
    assert!(!aget_home.join("sessions/news.json").exists());
}
