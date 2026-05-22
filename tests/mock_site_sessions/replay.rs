use assert_cmd::Command;

use crate::support::mock_site::MockSite;
use crate::support::mock_site_cli::{
    mock_backend_command, save_cookie_session, save_storage_session, success_data,
};

#[test]
fn mock_site_replays_cookie_and_storage_sessions() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");
    save_storage_session(
        &aget_home,
        "storage",
        &site.origin(),
        "local_token",
        "storage-secret",
    );

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
            "app",
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
    assert_eq!(protected_json["sensitive"], true);
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));

    let storage = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/storage-protected"),
            "--session",
            "storage",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let storage_json = success_data(&storage, "get");
    assert!(storage_json["content"]
        .as_str()
        .unwrap()
        .contains("Storage Protected"));
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}
