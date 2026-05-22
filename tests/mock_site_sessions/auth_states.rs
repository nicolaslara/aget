use assert_cmd::Command;

use crate::support::mock_site::MockSite;
use crate::support::mock_site_cli::{mock_backend_command, save_cookie_session, success_data};

#[test]
fn mock_site_covers_unauthenticated_expired_and_logout_states() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    save_cookie_session(
        &aget_home,
        "expired",
        &site.host(),
        "app_session",
        "expired",
    );
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let unauthenticated = Command::cargo_bin("aget")
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
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let unauthenticated_json = success_data(&unauthenticated, "get");
    assert_eq!(
        unauthenticated_json["final_url"],
        site.url("/login?next=/protected")
    );
    assert!(unauthenticated_json["content"]
        .as_str()
        .unwrap()
        .contains("Login Form"));
    assert!(!site.received_cookie("/protected", "app_session", "valid-app"));

    let expired = Command::cargo_bin("aget")
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
            "expired",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let expired_json = success_data(&expired, "get");
    assert!(expired_json["content"]
        .as_str()
        .unwrap()
        .contains("Session Expired"));

    let logout = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/logout"),
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
    let logout_json = success_data(&logout, "get");
    assert!(logout_json["content"]
        .as_str()
        .unwrap()
        .contains("Logged Out"));
}
