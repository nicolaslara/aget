use aget::session::SessionStore;
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{
    agent_browser_tool, crawl4ai_cookie_echo_server, mock_backend_command, success_data,
};

#[test]
fn session_authorize_imports_verifies_and_omits_sensitive_inline_content() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let (url, server, cookie_receiver) = crawl4ai_cookie_echo_server();
    let fake_backend = mock_backend_command(temp.path(), json!({}));
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({
            "state": {
                "cookies": [{
                    "name": "app_session",
                    "value": "valid-app",
                    "domain": "127.0.0.1",
                    "path": "/",
                    "httpOnly": true,
                    "secure": false,
                    "sameSite": "Lax"
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
            "app",
            "--url",
            &url,
            "--browser-profile",
            "Default",
            "--allow-domain",
            "127.0.0.1",
            "--must-contain",
            "app_session=valid-app",
            "--must-not-contain",
            "Cookie: none",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output, "session.authorize");
    assert_eq!(data["state"], "verified");
    assert_eq!(data["name"], "app");
    assert_eq!(data["source"], "chrome");
    assert_eq!(data["allowed_domains"], json!(["127.0.0.1"]));
    assert!(data["baseline"].get("content").is_none());
    assert!(data["verification"].get("content").is_none());
    assert_eq!(data["baseline"]["sensitive"], false);
    assert_eq!(data["verification"]["sensitive"], true);
    assert_eq!(data["verification"]["sessions"], json!(["app"]));
    assert_eq!(data["verification_content_inlined"], false);
    assert_eq!(data["verification_sensitive"], true);
    assert!(data["predicates"]
        .as_array()
        .unwrap()
        .iter()
        .all(|predicate| predicate["matched"] == true));

    assert_eq!(cookie_receiver.recv().unwrap(), "none");
    assert_eq!(cookie_receiver.recv().unwrap(), "app_session=valid-app");
    server.join().unwrap();

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("app").unwrap();
    assert!(session.sensitive);
    assert_eq!(session.cookies[0].value, "valid-app");
}

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
