use super::super::compose_playwright_state;
use super::support::{origin, session_with_cookie};
use crate::error::ErrorCode;
use crate::session::{Session, StorageEntry};

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
