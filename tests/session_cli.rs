use aget::session::SessionStore;
use aget::{Session, SessionCookie};
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn session_list_inspect_and_delete_use_aget_home() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let session = demo_session();
    store.save(&session).unwrap();

    let mut list = Command::cargo_bin("aget").unwrap();
    list.env("AGET_HOME", &aget_home)
        .args(["session", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo"));

    let mut inspect = Command::cargo_bin("aget").unwrap();
    inspect
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted>"))
        .stdout(predicate::str::contains("secret-cookie").not());

    let mut inspect_secrets = Command::cargo_bin("aget").unwrap();
    inspect_secrets
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "demo", "--show-secrets"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret-cookie"));

    let mut delete = Command::cargo_bin("aget").unwrap();
    delete
        .env("AGET_HOME", &aget_home)
        .args(["session", "delete", "demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted session demo"));

    assert!(store.list().unwrap().is_empty());
}

#[test]
fn session_list_json_has_stable_shape() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    store.save(&demo_session()).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args(["--json", "session", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["sessions"], serde_json::json!(["demo"]));
}

fn demo_session() -> Session {
    let mut session = Session::new("demo");
    session.allowed_cookie_domains = vec!["example.com".to_string()];
    session.cookies.push(SessionCookie {
        name: "session".to_string(),
        value: "secret-cookie".to_string(),
        domain: "example.com".to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
        source_session: None,
    });
    session
}
