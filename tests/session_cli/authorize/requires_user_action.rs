use aget::session::SessionStore;
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, mock_backend_command};

#[test]
fn session_authorize_preserves_requires_user_action_and_does_not_save_session() {
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
    let fake_agent_browser =
        agent_browser_tool(temp.path(), &log_path, json!({"behavior": "profile_lock"}));

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
            "--browser-profile",
            "Default",
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
    assert_eq!(json["error"]["code"], "requires_user_action");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("quit Chrome"));

    let store = SessionStore::new(&aget_home).unwrap();
    assert!(store.load("news").is_err());
}
