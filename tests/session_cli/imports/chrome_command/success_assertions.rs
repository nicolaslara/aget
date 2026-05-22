use std::fs;
use std::path::{Path, PathBuf};

use aget::session::SessionStore;
use aget::SessionSource;
use assert_cmd::Command;
use predicates::prelude::*;

use crate::support::session_cli::success_data;

pub(super) fn assert_saved_session(aget_home: &Path) {
    let store = SessionStore::new(aget_home).unwrap();
    let session = store.load("chrome-imported").unwrap();
    assert_eq!(
        session.source,
        SessionSource::ChromeProfile {
            profile: "Default".to_string()
        }
    );
    assert!(session.sensitive);
    assert_eq!(
        session.allowed_cookie_domains,
        vec!["example.com".to_string()]
    );
    assert_eq!(
        session.allowed_storage_origins,
        vec![
            "https://docs.example.com:443".to_string(),
            "https://example.com".to_string(),
        ]
    );
    assert_eq!(session.cookies.len(), 2);
    assert!(session.cookies.iter().all(|cookie| cookie
        .source_session
        .as_deref()
        .is_some_and(|source| source.starts_with("aget-import-"))));
    assert!(session.cookies.iter().any(|cookie| cookie.name == "sid"));
    assert!(session.cookies.iter().any(|cookie| cookie.name == "sub"));
    assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));
    assert_eq!(
        session
            .cookies
            .iter()
            .find(|cookie| cookie.name == "sid")
            .unwrap()
            .expires,
        Some(1812619153)
    );
    assert_eq!(session.origins.len(), 2);
    assert!(session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://example.com"
            && origin
                .local_storage
                .iter()
                .any(|entry| entry.name == "token")));
    assert!(session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://example.com"
            && origin
                .session_storage
                .iter()
                .any(|entry| entry.name == "session-token")));
    assert!(!session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://example.com.evil"));
}

pub(super) fn assert_inspect_redaction(aget_home: &Path) {
    let mut inspect = Command::cargo_bin("aget").unwrap();
    let inspect_output = inspect
        .env("AGET_HOME", aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "inspect",
            "chrome-imported",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect_json = success_data(&inspect_output, "session.inspect");
    let inspect_origin = inspect_json["origins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|origin| origin["origin"] == "https://example.com")
        .unwrap();
    assert_eq!(inspect_origin["local_storage"][0]["value"], "<redacted>");
    assert_eq!(inspect_origin["session_storage"][0]["value"], "<redacted>");
    assert!(!String::from_utf8_lossy(&inspect_output).contains("allowed-storage"));
    assert!(!String::from_utf8_lossy(&inspect_output).contains("session-only"));

    let mut inspect_secrets = Command::cargo_bin("aget").unwrap();
    inspect_secrets
        .env("AGET_HOME", aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "inspect",
            "chrome-imported",
            "--show-secrets",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("allowed-storage"))
        .stdout(predicate::str::contains("session-only"));
}

pub(super) fn assert_backend_calls_cleaned(log_path: &Path) {
    let log = fs::read_to_string(log_path).unwrap();
    let calls = log
        .lines()
        .filter(|line| line.starts_with('['))
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 3);
    assert!(calls[0].contains(r#""--profile", "Default", "--session", "#));
    assert!(calls[1].contains(r#""state", "save""#));
    assert!(calls[2].contains(r#""close""#));
    let raw_state_path = log
        .lines()
        .find_map(|line| line.strip_prefix("STATE_PATH="))
        .map(PathBuf::from)
        .unwrap();
    assert!(!raw_state_path.exists());
}
