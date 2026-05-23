use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, success_data};

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
