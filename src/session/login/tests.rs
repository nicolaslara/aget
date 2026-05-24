use std::fs;
use std::path::PathBuf;

use super::owned::{cancel_owned_login_session, start_owned_login_session};
use super::pending::{default_owned_login_profile_path, pending_login_path, write_pending_login};
use super::types::{LoginCancelOptions, LoginCompleteOptions, LoginStartOptions, PendingLogin};
use super::{complete_login_session, merge_login_session};
use crate::error::ErrorCode;
use crate::session::{Session, SessionCookie, SessionOrigin, StorageEntry};

fn pending(name: &str, profile: PathBuf) -> PendingLogin {
    PendingLogin {
        name: name.to_string(),
        profile: profile.to_string_lossy().into_owned(),
        agent_session: format!("aget-login-{name}"),
        url: "https://example.com/login".to_string(),
        allowed_domains: vec!["example.com".to_string()],
        injected_sessions: Vec::new(),
        browser_pid: None,
    }
}

#[test]
fn complete_login_session_removes_owned_default_profile() {
    let temp = tempfile::tempdir().unwrap();
    let tmp_dir = temp.path().join("tmp");
    fs::create_dir_all(&tmp_dir).unwrap();
    let profile = default_owned_login_profile_path(&tmp_dir, "docs");
    fs::create_dir_all(&profile).unwrap();
    let pending = pending("docs", profile.clone());
    write_pending_login(&tmp_dir, &pending).unwrap();

    complete_login_session(LoginCompleteOptions {
        pending,
        tmp_dir: tmp_dir.clone(),
    })
    .unwrap();

    assert!(!profile.exists());
    assert!(!pending_login_path(&tmp_dir, "docs").exists());
}

#[test]
fn complete_login_session_preserves_custom_profile_path() {
    let temp = tempfile::tempdir().unwrap();
    let tmp_dir = temp.path().join("tmp");
    let profile = temp.path().join("custom-profile");
    fs::create_dir_all(&tmp_dir).unwrap();
    fs::create_dir_all(&profile).unwrap();
    let pending = pending("docs", profile.clone());
    write_pending_login(&tmp_dir, &pending).unwrap();

    complete_login_session(LoginCompleteOptions {
        pending,
        tmp_dir: tmp_dir.clone(),
    })
    .unwrap();

    assert!(profile.exists());
    assert!(!pending_login_path(&tmp_dir, "docs").exists());
}

#[test]
fn cancel_owned_login_session_cleans_pending_and_profile_when_browser_is_closed() {
    let temp = tempfile::tempdir().unwrap();
    let tmp_dir = temp.path().join("tmp");
    fs::create_dir_all(&tmp_dir).unwrap();
    let profile = default_owned_login_profile_path(&tmp_dir, "docs");
    fs::create_dir_all(&profile).unwrap();
    write_pending_login(&tmp_dir, &pending("docs", profile.clone())).unwrap();

    let result = cancel_owned_login_session(LoginCancelOptions {
        name: "docs".to_string(),
        tmp_dir: tmp_dir.clone(),
    })
    .unwrap();

    assert_eq!(result.pending.name, "docs");
    assert!(!profile.exists());
    assert!(!pending_login_path(&tmp_dir, "docs").exists());
}

#[test]
fn start_owned_login_session_rejects_non_https_before_launching_browser() {
    let temp = tempfile::tempdir().unwrap();
    let tmp_dir = temp.path().join("tmp");

    let error = start_owned_login_session(LoginStartOptions {
        name: "docs".to_string(),
        profile: None,
        url: "http://example.com/login".to_string(),
        tmp_dir: tmp_dir.clone(),
        injected_sessions: Vec::new(),
    })
    .unwrap_err();

    assert_eq!(error.code(), ErrorCode::UsageError);
    assert!(!pending_login_path(&tmp_dir, "docs").exists());
}

#[test]
fn start_owned_login_session_rejects_injected_sessions_with_custom_profile() {
    let temp = tempfile::tempdir().unwrap();
    let tmp_dir = temp.path().join("tmp");
    let profile = temp.path().join("custom-profile");

    let error = start_owned_login_session(LoginStartOptions {
        name: "docs".to_string(),
        profile: Some(profile.to_string_lossy().into_owned()),
        url: "https://example.com/login".to_string(),
        tmp_dir: tmp_dir.clone(),
        injected_sessions: vec![Session::new("oauth")],
    })
    .unwrap_err();

    assert_eq!(error.code(), ErrorCode::UsageError);
    assert!(error.to_string().contains("omit --profile"));
    assert!(!profile.exists());
    assert!(!pending_login_path(&tmp_dir, "docs").exists());
}

#[test]
fn merge_login_session_replaces_stale_same_scope_state() {
    let mut existing = Session::new("docs");
    existing.allowed_cookie_domains = vec![
        "example.com".to_string(),
        "other.example".to_string(),
        "example.com.evil".to_string(),
    ];
    existing
        .allowed_storage_origins
        .push("https://example.com".to_string());
    existing.cookies = vec![
        session_cookie("sid", "stale", "example.com"),
        session_cookie("other", "keep", "other.example"),
        session_cookie("evil", "keep-evil", "example.com.evil"),
    ];
    existing.origins = vec![
        session_origin("https://example.com", "stale-token"),
        session_origin("https://other.example", "keep-token"),
    ];

    let mut fresh = Session::new("docs");
    fresh.allowed_cookie_domains = vec!["example.com".to_string()];
    fresh
        .allowed_storage_origins
        .push("https://example.com".to_string());
    fresh.cookies = vec![session_cookie("sid", "fresh", "example.com")];
    fresh.origins = vec![session_origin("https://example.com", "fresh-token")];

    let merged = merge_login_session(existing, fresh);

    assert!(merged
        .cookies
        .iter()
        .any(|cookie| cookie.name == "sid" && cookie.value == "fresh"));
    assert!(!merged
        .cookies
        .iter()
        .any(|cookie| cookie.name == "sid" && cookie.value == "stale"));
    assert!(merged
        .cookies
        .iter()
        .any(|cookie| cookie.name == "other" && cookie.value == "keep"));
    assert!(merged
        .cookies
        .iter()
        .any(|cookie| cookie.name == "evil" && cookie.value == "keep-evil"));
    assert!(merged.origins.iter().any(|origin| {
        origin.origin == "https://example.com" && origin.local_storage[0].value == "fresh-token"
    }));
    assert!(merged.origins.iter().any(|origin| {
        origin.origin == "https://other.example" && origin.local_storage[0].value == "keep-token"
    }));
}

fn session_cookie(name: &str, value: &str, domain: &str) -> SessionCookie {
    SessionCookie {
        name: name.to_string(),
        value: value.to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
        source_session: Some("docs".to_string()),
    }
}

fn session_origin(origin: &str, value: &str) -> SessionOrigin {
    SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: "token".to_string(),
            value: value.to_string(),
        }],
        session_storage: Vec::new(),
        source_session: Some("docs".to_string()),
    }
}
