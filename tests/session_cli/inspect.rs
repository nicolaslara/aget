use aget::session::SessionStore;

use assert_cmd::Command;

use predicates::prelude::*;

use crate::support::session_cli::{named_session, session_origin, success_data};

#[test]
fn session_compose_inspect_reports_cookie_source_session_and_redacts_values() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    store
        .save(&named_session(
            "provider",
            "oauth",
            "provider-secret",
            "accounts.example.com",
        ))
        .unwrap();
    store
        .save(&named_session(
            "app",
            "appsid",
            "app-secret",
            "app.example.com",
        ))
        .unwrap();

    let mut compose = Command::cargo_bin("aget").unwrap();
    compose
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success();

    let mut inspect = Command::cargo_bin("aget").unwrap();
    let inspect_output = inspect
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "inspect", "combined"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect_json = success_data(&inspect_output, "session.inspect");
    assert_eq!(inspect_json["cookies"][0]["value"], "<redacted>");
    assert!(inspect_json["cookies"]
        .as_array()
        .unwrap()
        .iter()
        .any(|cookie| { cookie["name"] == "oauth" && cookie["source_session"] == "provider" }));
    assert!(!String::from_utf8_lossy(&inspect_output).contains("provider-secret"));
    assert!(!String::from_utf8_lossy(&inspect_output).contains("app-secret"));
}

#[test]
fn session_inspect_plain_reports_cookie_and_origin_provenance_with_redacted_values() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let mut provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    provider.allowed_storage_origins = vec!["https://accounts.example.com".to_string()];
    provider.origins.push(session_origin(
        "https://accounts.example.com",
        "provider-token",
        "provider-storage-secret",
    ));
    store.save(&provider).unwrap();
    store
        .save(&named_session(
            "app",
            "appsid",
            "app-secret",
            "app.example.com",
        ))
        .unwrap();

    let mut compose = Command::cargo_bin("aget").unwrap();
    compose
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success();

    let mut inspect = Command::cargo_bin("aget").unwrap();
    inspect
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "combined"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "accounts.example.com oauth=<redacted> source=provider",
        ))
        .stdout(predicate::str::contains(
            "https://accounts.example.com source=provider",
        ))
        .stdout(predicate::str::contains(
            "localStorage provider-token=<redacted>",
        ))
        .stdout(predicate::str::contains("provider-secret").not())
        .stdout(predicate::str::contains("provider-storage-secret").not());
}
