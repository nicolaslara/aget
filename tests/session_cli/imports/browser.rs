use std::fs;

use aget::session::SessionStore;
use aget::SessionSource;
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, chrome_import_state, success_data};

#[test]
fn session_import_browser_chrome_saves_filtered_state() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": chrome_import_state()}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "browser",
            "--browser",
            "chrome",
            "--browser-profile",
            "Default",
            "--name",
            "browser-imported",
            "--allow-domain",
            "example.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.import.browser");
    assert_eq!(json["source"], "browser_profile");
    assert_eq!(json["browser"], "chrome");
    assert_eq!(json["name"], "browser-imported");
    assert_eq!(json["cookie_count"], 2);
    assert_eq!(json["origin_count"], 2);

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("browser-imported").unwrap();
    assert_eq!(
        session.source,
        SessionSource::ChromeProfile {
            profile: "Default".to_string()
        }
    );
    assert!(session.sensitive);
    assert_eq!(session.cookies.len(), 2);
    assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));
}

#[test]
fn session_import_browser_rejects_unsupported_browser_without_importing() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": chrome_import_state()}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "browser",
            "--browser",
            "firefox",
            "--profile-path",
            "/tmp/firefox-profile",
            "--name",
            "firefox-imported",
            "--allow-domain",
            "example.com",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "session.import.browser");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("does not support 'firefox' yet"));
    assert!(!log_path.exists() || fs::read_to_string(&log_path).unwrap().is_empty());
    assert!(!aget_home.join("sessions/firefox-imported.json").exists());
}
