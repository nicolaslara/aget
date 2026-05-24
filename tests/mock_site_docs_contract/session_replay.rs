use assert_cmd::Command;

use crate::support::mock_site::MockSite;
use crate::support::mock_site_cli::{save_cookie_session, save_mixed_scope_session, success_data};

#[test]
fn documents_session_compose_replay_and_scope_rejection_contract() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    save_cookie_session(
        &aget_home,
        "provider",
        &site.host(),
        "provider_session",
        "valid-provider",
    );
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let compose = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let compose_json = success_data(&compose, "session.compose");
    assert_eq!(compose_json["cookie_count"], 2);

    let composed_fetch = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/requires-two"),
            "--session",
            "combined",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let composed_json = success_data(&composed_fetch, "get");
    assert!(composed_json["content"]
        .as_str()
        .unwrap()
        .contains("Composed Session"));
    assert!(site.received_cookie("/requires-two", "provider_session", "valid-provider"));
    assert!(site.received_cookie("/requires-two", "app_session", "valid-app"));

    save_mixed_scope_session(&aget_home, "mixed", &site.host());
    let requests_before_rejection = site.requests().len();
    let rejected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "mixed",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let rejected_json: serde_json::Value = serde_json::from_slice(&rejected).unwrap();
    assert_eq!(rejected_json["command"], "get");
    assert_eq!(rejected_json["error"]["code"], "privacy_policy_blocked");
    assert_eq!(site.requests().len(), requests_before_rejection);
}
