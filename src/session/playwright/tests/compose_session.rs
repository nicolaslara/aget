use super::super::compose_session;
use super::support::{origin, session_with_cookie};
use crate::error::ErrorCode;
use crate::session::{Session, SessionSource, StorageEntry};

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
