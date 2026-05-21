use super::*;
use crate::error::ErrorCode;
use crate::session::{Session, SessionCookie, SessionOrigin, SessionSource, StorageEntry};

#[test]
fn empty_sessions_produce_empty_state() {
    let state = compose_playwright_state(&[]).unwrap();

    assert!(state.cookies.is_empty());
    assert!(state.origins.is_empty());
}

#[test]
fn maps_one_session_to_playwright_state() {
    let session = session_with_cookie("demo", "sid", "secret");
    let state = compose_playwright_state(&[session]).unwrap();

    assert_eq!(state.cookies.len(), 1);
    assert_eq!(state.cookies[0].name, "sid");
    assert_eq!(state.cookies[0].value, "secret");
}

#[test]
fn deduplicates_identical_cookies() {
    let session_a = session_with_cookie("a", "sid", "same");
    let session_b = session_with_cookie("b", "sid", "same");
    let state = compose_playwright_state(&[session_a, session_b]).unwrap();

    assert_eq!(state.cookies.len(), 1);
    assert_eq!(state.cookies[0].name, "sid");
    assert_eq!(state.cookies[0].domain, "example.com");
    assert_eq!(state.cookies[0].path, "/");
}

#[test]
fn deduplicates_cookies_with_normalized_identity() {
    let mut session_a = session_with_cookie("a", " sid ", "same");
    session_a.cookies[0].domain = ".Example.COM".to_string();
    session_a.cookies[0].path = String::new();
    let mut session_b = session_with_cookie("b", "sid", "same");
    session_b.cookies[0].domain = "example.com".to_string();
    session_b.cookies[0].path = "/".to_string();

    let state = compose_playwright_state(&[session_a, session_b]).unwrap();

    assert_eq!(state.cookies.len(), 1);
}

#[test]
fn rejects_conflicting_cookies() {
    let session_a = session_with_cookie("a", "sid", "first");
    let session_b = session_with_cookie("b", "sid", "second");
    let error = compose_playwright_state(&[session_a, session_b]).unwrap_err();

    assert_eq!(error.code(), ErrorCode::SessionConflict);
}

#[test]
fn deduplicates_identical_origins() {
    let mut session_a = Session::new("a");
    session_a
        .origins
        .push(origin("https://example.com", "token", "same"));
    let mut session_b = Session::new("b");
    session_b
        .origins
        .push(origin("https://example.com", "token", "same"));

    let state = compose_playwright_state(&[session_a, session_b]).unwrap();
    assert_eq!(state.origins.len(), 1);
}

#[test]
fn rejects_conflicting_origins() {
    let mut session_a = Session::new("a");
    session_a
        .origins
        .push(origin("https://example.com", "token", "first"));
    let mut session_b = Session::new("b");
    session_b
        .origins
        .push(origin("https://example.com", "token", "second"));

    let error = compose_playwright_state(&[session_a, session_b]).unwrap_err();
    assert_eq!(error.code(), ErrorCode::SessionConflict);
}

#[test]
fn merges_disjoint_local_storage_keys_for_same_origin() {
    let mut session_a = Session::new("a");
    session_a
        .origins
        .push(origin("https://example.com", "token", "first"));
    let mut session_b = Session::new("b");
    session_b
        .origins
        .push(origin("https://example.com", "theme", "dark"));

    let state = compose_playwright_state(&[session_a, session_b]).unwrap();

    assert_eq!(state.origins.len(), 1);
    assert_eq!(state.origins[0].origin, "https://example.com");
    assert_eq!(state.origins[0].local_storage.len(), 2);
    assert!(state.origins[0]
        .local_storage
        .iter()
        .any(|entry| entry.name == "token" && entry.value == "first"));
    assert!(state.origins[0]
        .local_storage
        .iter()
        .any(|entry| entry.name == "theme" && entry.value == "dark"));
}

#[test]
fn composes_session_storage_into_playwright_state() {
    let mut session = Session::new("a");
    let mut origin = origin("https://example.com", "token", "first");
    origin.session_storage.push(StorageEntry {
        name: "session-token".to_string(),
        value: "session-secret".to_string(),
    });
    session.origins.push(origin);

    let state = compose_playwright_state(&[session]).unwrap();

    assert_eq!(state.origins.len(), 1);
    assert_eq!(state.origins[0].session_storage.len(), 1);
    assert_eq!(state.origins[0].session_storage[0].name, "session-token");
}

#[test]
fn temp_state_file_is_removed_on_drop() {
    let temp = tempfile::tempdir().unwrap();
    let state = compose_playwright_state(&[]).unwrap();
    let temp_state = TempStateFile::write(temp.path(), &state).unwrap();
    let path = temp_state.path().to_path_buf();

    assert!(path.exists());
    drop(temp_state);
    assert!(!path.exists());
}

#[cfg(unix)]
#[test]
fn temp_state_file_uses_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let state = compose_playwright_state(&[]).unwrap();
    let temp_state = TempStateFile::write(temp.path(), &state).unwrap();
    let mode = std::fs::metadata(temp_state.path())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(mode, 0o600);
}

#[test]
fn composed_session_preserves_and_fills_provenance_without_mutating_sources() {
    let mut provider = session_with_cookie("provider", "oauth", "provider-secret");
    provider.cookies[0].source_session = Some("browser-import".to_string());
    provider.allowed_cookie_domains = vec!["accounts.example.com".to_string()];
    provider.origins.push(origin(
        "https://accounts.example.com",
        "token",
        "provider-storage",
    ));
    let app = session_with_cookie("app", "appsid", "app-secret");
    let original_provider = provider.clone();
    let original_app = app.clone();

    let composed = compose_session("combined", &[provider.clone(), app.clone()]).unwrap();

    assert_eq!(
        composed.source,
        SessionSource::Composed {
            sessions: vec!["provider".to_string(), "app".to_string()]
        }
    );
    assert!(composed.cookies.iter().any(|cookie| {
        cookie.name == "oauth" && cookie.source_session.as_deref() == Some("browser-import")
    }));
    assert!(composed.cookies.iter().any(|cookie| {
        cookie.name == "appsid" && cookie.source_session.as_deref() == Some("app")
    }));
    assert_eq!(
        composed.origins[0].source_session.as_deref(),
        Some("provider")
    );
    assert_eq!(provider, original_provider);
    assert_eq!(app, original_app);
}

#[test]
fn composed_session_merges_disjoint_local_storage_keys() {
    let mut session_a = Session::new("a");
    session_a
        .origins
        .push(origin("https://example.com", "token", "first"));
    let mut session_b = Session::new("b");
    session_b
        .origins
        .push(origin("https://example.com", "theme", "dark"));

    let composed = compose_session("combined", &[session_a, session_b]).unwrap();

    assert_eq!(composed.origins.len(), 1);
    assert_eq!(composed.origins[0].origin, "https://example.com");
    assert_eq!(composed.origins[0].local_storage.len(), 2);
    assert_eq!(composed.origins[0].source_session, None);
}

#[test]
fn composed_session_merges_disjoint_session_storage_keys() {
    let mut session_a = Session::new("a");
    let mut origin_a = origin("https://example.com", "token", "first");
    origin_a.session_storage.push(StorageEntry {
        name: "session-token".to_string(),
        value: "first-session".to_string(),
    });
    session_a.origins.push(origin_a);
    let mut session_b = Session::new("b");
    let mut origin_b = origin("https://example.com", "theme", "dark");
    origin_b.session_storage.push(StorageEntry {
        name: "session-theme".to_string(),
        value: "dark-session".to_string(),
    });
    session_b.origins.push(origin_b);

    let composed = compose_session("combined", &[session_a, session_b]).unwrap();

    assert_eq!(composed.origins.len(), 1);
    assert_eq!(composed.origins[0].session_storage.len(), 2);
    assert_eq!(composed.origins[0].source_session, None);
}

#[test]
fn composed_session_deduplicates_and_sorts_allowed_scopes() {
    let mut session_a = session_with_cookie("a", "sid", "same");
    session_a.allowed_cookie_domains =
        vec!["z.example.com".to_string(), "a.example.com".to_string()];
    session_a.allowed_storage_origins = vec!["https://z.example.com".to_string()];
    let mut session_b = session_with_cookie("b", "sid", "same");
    session_b.allowed_cookie_domains = vec!["a.example.com".to_string()];
    session_b.allowed_storage_origins = vec![
        "https://a.example.com".to_string(),
        "https://z.example.com".to_string(),
    ];

    let composed = compose_session("combined", &[session_a, session_b]).unwrap();

    assert_eq!(composed.cookies.len(), 1);
    assert_eq!(
        composed.allowed_cookie_domains,
        vec!["a.example.com".to_string(), "z.example.com".to_string()]
    );
    assert_eq!(
        composed.allowed_storage_origins,
        vec![
            "https://a.example.com".to_string(),
            "https://z.example.com".to_string()
        ]
    );
}

#[test]
fn composed_session_deduplicates_cookies_with_normalized_identity() {
    let mut session_a = session_with_cookie("a", " sid ", "same");
    session_a.cookies[0].domain = ".Example.COM".to_string();
    session_a.cookies[0].path = String::new();
    let mut session_b = session_with_cookie("b", "sid", "same");
    session_b.cookies[0].domain = "example.com".to_string();
    session_b.cookies[0].path = "/".to_string();

    let composed = compose_session("combined", &[session_a, session_b]).unwrap();

    assert_eq!(composed.cookies.len(), 1);
    assert_eq!(composed.cookies[0].name, "sid");
    assert_eq!(composed.cookies[0].domain, "example.com");
    assert_eq!(composed.cookies[0].path, "/");
}

#[test]
fn composed_session_rejects_conflicts_without_secret_values() {
    let session_a = session_with_cookie("a", "sid", "first-secret");
    let session_b = session_with_cookie("b", "sid", "second-secret");

    let error = compose_session("combined", &[session_a, session_b]).unwrap_err();
    let message = error.to_string();

    assert_eq!(error.code(), ErrorCode::SessionConflict);
    assert!(message.contains("conflicting cookie 'sid'"));
    assert!(!message.contains("first-secret"));
    assert!(!message.contains("second-secret"));
}

fn session_with_cookie(name: &str, cookie_name: &str, value: &str) -> Session {
    let mut session = Session::new(name);
    session.cookies.push(SessionCookie {
        name: cookie_name.to_string(),
        value: value.to_string(),
        domain: "example.com".to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    session
}

fn origin(origin: &str, name: &str, value: &str) -> SessionOrigin {
    SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: name.to_string(),
            value: value.to_string(),
        }],
        session_storage: Vec::new(),
        source_session: None,
    }
}
