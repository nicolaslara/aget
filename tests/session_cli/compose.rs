use aget::session::SessionStore;

use aget::SessionSource;

use assert_cmd::Command;

use crate::support::session_cli::{named_session, session_origin, success_data};

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
