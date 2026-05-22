use assert_cmd::Command;

use crate::support::mock_site::MockSite;
use crate::support::mock_site_cli::{
    mock_agent_browser_command, mock_backend_command, success_data,
};

#[test]
fn mock_site_imported_chrome_session_can_fetch_protected_page() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let fake_agent_browser = mock_agent_browser_command();

    let imported = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "chrome",
            "--chrome-profile",
            "Default",
            "--name",
            "imported",
            "--allow-domain",
            &site.host(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let import_json = success_data(&imported, "session.import.chrome");
    assert_eq!(import_json["name"], "imported");
    assert_eq!(import_json["cookie_count"], 1);

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
            "imported",
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
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));
}
