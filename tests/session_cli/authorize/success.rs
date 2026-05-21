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
