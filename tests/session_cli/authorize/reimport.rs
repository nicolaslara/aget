use aget::session::SessionStore;
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{
    agent_browser_tool, crawl4ai_cookie_echo_server, mock_backend_command, success_data,
};

#[test]
fn session_authorize_reimport_after_user_login_replaces_failed_verification_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let (url, server, cookie_receiver) = crawl4ai_cookie_echo_server();
    let fake_backend = mock_backend_command(temp.path(), json!({}));
    let stale_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({
            "state": {
                "cookies": [{
                    "name": "app_session",
                    "value": "stale-app",
                    "domain": "127.0.0.1",
                    "path": "/",
                    "httpOnly": true,
                    "secure": false
                }],
                "origins": []
            }
        }),
    );
    let logged_in_agent_browser = agent_browser_tool(
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
                    "secure": false
                }],
                "origins": []
            }
        }),
    );

    let mut first = Command::cargo_bin("aget").unwrap();
    let first_output = first
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .env("AGET_AGENT_BROWSER_COMMAND", &stale_agent_browser)
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
    let first_data = success_data(&first_output, "session.authorize");
    assert_eq!(first_data["state"], "verification_failed");

    let store = SessionStore::new(&aget_home).unwrap();
    assert_eq!(store.load("app").unwrap().cookies[0].value, "stale-app");

    let mut second = Command::cargo_bin("aget").unwrap();
    let second_output = second
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .env("AGET_AGENT_BROWSER_COMMAND", &logged_in_agent_browser)
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
    let second_data = success_data(&second_output, "session.authorize");
    assert_eq!(second_data["state"], "verified");
    assert!(second_data["verification"].get("content").is_none());
    assert_eq!(store.load("app").unwrap().cookies[0].value, "valid-app");

    assert_eq!(cookie_receiver.recv().unwrap(), "none");
    assert_eq!(cookie_receiver.recv().unwrap(), "app_session=stale-app");
    assert_eq!(cookie_receiver.recv().unwrap(), "none");
    assert_eq!(cookie_receiver.recv().unwrap(), "app_session=valid-app");
    server.join().unwrap();
}
