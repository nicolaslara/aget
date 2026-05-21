use aget::session::SessionStore;
use aget::SessionSource;
use assert_cmd::Command;
use predicates::prelude::*;
use support::session_cli::{demo_session, named_session, session_origin, success_data};
#[path = "session_cli/authorize.rs"]
mod authorize;
#[path = "session_cli/imports.rs"]
mod imports;
#[path = "session_cli/login.rs"]
mod login;
mod support;

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
        .args(["--envelope", "json", "session", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.list");
    assert_eq!(json["sessions"], serde_json::json!(["demo"]));
}

#[test]
fn session_compose_persists_composed_session_with_provenance_and_preserves_sources() {
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
    let app = named_session("app", "appsid", "app-secret", "app.example.com");
    let original_provider = provider.clone();
    let original_app = app.clone();
    store.save(&provider).unwrap();
    store.save(&app).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
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

    let json = success_data(&output, "session.compose");
    assert_eq!(json["name"], "combined");
    assert_eq!(
        json["source_sessions"],
        serde_json::json!(["provider", "app"])
    );
    assert_eq!(json["cookie_count"], 2);
    assert_eq!(json["origin_count"], 1);

    let combined = store.load("combined").unwrap();
    assert_eq!(
        combined.source,
        SessionSource::Composed {
            sessions: vec!["provider".to_string(), "app".to_string()]
        }
    );
    assert_eq!(
        combined.allowed_cookie_domains,
        vec![
            "accounts.example.com".to_string(),
            "app.example.com".to_string()
        ]
    );
    assert!(combined.cookies.iter().any(|cookie| {
        cookie.name == "oauth" && cookie.source_session.as_deref() == Some("provider")
    }));
    assert!(combined.cookies.iter().any(|cookie| {
        cookie.name == "appsid" && cookie.source_session.as_deref() == Some("app")
    }));
    assert_eq!(
        combined.origins[0].source_session.as_deref(),
        Some("provider")
    );
    assert_eq!(store.load("provider").unwrap(), original_provider);
    assert_eq!(store.load("app").unwrap(), original_app);
}

#[test]
fn session_compose_rejects_source_name_target_without_mutating_sources() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    let app = named_session("app", "appsid", "app-secret", "app.example.com");
    store.save(&provider).unwrap();
    store.save(&app).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "compose",
            "provider",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("must not match a source session"));
    assert_eq!(store.load("provider").unwrap(), provider);
    assert_eq!(store.load("app").unwrap(), app);
}

#[test]
fn session_compose_rejects_existing_target_without_mutating_existing_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    let app = named_session("app", "appsid", "app-secret", "app.example.com");
    let existing = named_session("combined", "existing", "existing-secret", "old.example.com");
    store.save(&provider).unwrap();
    store.save(&app).unwrap();
    store.save(&existing).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
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
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("already exists"));
    assert_eq!(store.load("combined").unwrap(), existing);
    assert_eq!(store.load("provider").unwrap(), provider);
    assert_eq!(store.load("app").unwrap(), app);
}

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

#[test]
fn session_compose_rejects_conflicts_with_redacted_error_and_does_not_save() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    store
        .save(&named_session(
            "first",
            "sid",
            "first-secret",
            "example.com",
        ))
        .unwrap();
    store
        .save(&named_session(
            "second",
            "sid",
            "second-secret",
            "example.com",
        ))
        .unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "compose",
            "combined",
            "--session",
            "first",
            "--session",
            "second",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "session_conflict");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("conflicting cookie 'sid'"));
    assert!(!String::from_utf8_lossy(&output).contains("first-secret"));
    assert!(!String::from_utf8_lossy(&output).contains("second-secret"));
    assert!(store.load("combined").is_err());
}
