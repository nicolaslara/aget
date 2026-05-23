use aget::session::SessionStore;
use aget::SessionSource;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;

use crate::support::session_cli::{mock_cmux, success_data};

#[test]
fn session_import_cmux_saves_filtered_cookies() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_cmux = mock_cmux(temp.path(), json!({"surface": "surface:1"}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CMUX_COMMAND", &fake_cmux)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "cmux",
            "--surface",
            "surface:1",
            "--name",
            "imported",
            "--allow-domain",
            "example.com",
            "--allow-domain",
            "docs.example.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.import.cmux");
    assert_eq!(json["name"], "imported");
    assert_eq!(json["source"], "cmux");
    assert_eq!(json["cookie_count"], 4);

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("imported").unwrap();
    assert_eq!(
        session.source,
        SessionSource::Cmux {
            surface: "surface:1".to_string()
        }
    );
    assert!(session.sensitive);
    assert_eq!(
        session.allowed_cookie_domains,
        vec!["example.com".to_string(), "docs.example.com".to_string()]
    );
    assert!(session.allowed_storage_origins.is_empty());
    assert!(session.origins.is_empty());
    assert_eq!(session.cookies.len(), 4);
    assert!(session
        .cookies
        .iter()
        .all(|cookie| cookie.source_session.as_deref() == Some("imported")));
    assert!(session.cookies.iter().all(|cookie| cookie.http_only));
    assert!(session
        .cookies
        .iter()
        .all(|cookie| cookie.same_site.is_none()));
    assert!(session
        .cookies
        .iter()
        .any(|cookie| cookie.name == "sid" && cookie.domain == "example.com"));
    assert!(session
        .cookies
        .iter()
        .any(|cookie| cookie.name == "wide" && cookie.domain == ".example.com"));
    assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));

    let mut inspect = Command::cargo_bin("aget").unwrap();
    inspect
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "imported"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted>"))
        .stdout(predicate::str::contains("allowed-secret").not())
        .stdout(predicate::str::contains("suffix-secret").not())
        .stdout(predicate::str::contains("blocked-secret").not());
}

#[test]
fn session_import_cmux_missing_backend_returns_backend_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CMUX_COMMAND", "definitely_missing_aget_cmux")
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "cmux",
            "--surface",
            "surface:1",
            "--name",
            "imported",
            "--allow-domain",
            "example.com",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "session.import.cmux");
    assert_eq!(json["error"]["code"], "backend_unavailable");
}
