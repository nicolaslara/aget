use assert_cmd::Command;

use crate::support::mock_site::MockSite;
use crate::support::mock_site_cli::{
    mock_agent_browser_command, mock_backend_command, success_data,
};

#[test]
fn mock_site_login_bootstrap_can_fetch_protected_page_without_manual_action() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let fake_agent_browser = mock_agent_browser_command();

    Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "mock-app",
            "--url",
            &site.https_url("/login"),
        ])
        .assert()
        .success();

    let finish = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "mock-app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let finish_json = success_data(&finish, "session.login.finish");
    assert_eq!(finish_json["name"], "mock-app");

    let protected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "mock-app",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let protected_json = success_data(&protected, "get");
    assert!(protected_json["content"]
        .as_str()
        .unwrap()
        .contains("Protected Account"));
}
