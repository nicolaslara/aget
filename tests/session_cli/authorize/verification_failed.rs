use aget::session::SessionStore;
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, mock_backend_command};

#[test]
fn session_authorize_reports_verification_failed_without_inlining_content() {
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
            "--chrome-profile",
            "Default",
            "--allow-domain",
            "example.com",
            "--must-contain",
            "Welcome",
            "--must-not-contain",
            "Please sign in",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = crate::support::session_cli::success_envelope(&output, "session.authorize");
    assert!(envelope["warnings"][0]
        .as_str()
        .unwrap()
        .contains("predicates failed"));
    let data = envelope["data"].clone();
    assert_eq!(data["state"], "verification_failed");
    assert!(data["verification"].get("content").is_none());
    assert!(data["predicates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|predicate| predicate["kind"] == "must_contain"
            && predicate["value"] == "Welcome"
            && predicate["matched"] == false));
    assert!(data["predicates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|predicate| predicate["kind"] == "must_not_contain"
            && predicate["value"] == "Please sign in"
            && predicate["matched"] == false));
    assert_eq!(
        data["next_command"],
        json!([
            "aget",
            "session",
            "authorize",
            "news",
            "--url",
            "https://example.com/private",
            "--browser-profile",
            "<profile>",
            "--allow-domain",
            "<domain>"
        ])
    );

    let store = SessionStore::new(&aget_home).unwrap();
    assert!(store.load("news").is_ok());
}
