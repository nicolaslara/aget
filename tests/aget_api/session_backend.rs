use aget::Aget;

use crate::support::{cookie_session, MemorySessionStore, TestBrowserBackend};

#[test]
fn aget_with_static_browser_backend_saves_login_session_to_custom_store() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let browser = TestBrowserBackend {
        login_session: Some(cookie_session("login", "example.com", "login-secret")),
        ..TestBrowserBackend::default()
    };

    let aget = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_browser_automation_backend(browser);
    let session = aget.finish_login_session("login").unwrap();

    assert_eq!(session.name, "login");
    assert_eq!(
        store.load("login").unwrap().cookies[0].value,
        "login-secret"
    );
}

#[test]
fn aget_with_static_browser_backend_imports_chrome_session_to_custom_store() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let browser = TestBrowserBackend {
        login_session: Some(cookie_session("chrome", "example.com", "chrome-secret")),
        ..TestBrowserBackend::default()
    };

    let aget = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_browser_automation_backend(browser);
    let session = aget
        .import_chrome_session("Default", "chrome", vec!["example.com".to_string()])
        .unwrap();

    assert_eq!(session.name, "chrome");
    assert_eq!(
        store.load("chrome").unwrap().cookies[0].value,
        "chrome-secret"
    );
}

#[test]
fn aget_with_static_browser_backend_starts_and_cancels_login_without_command_backend() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let aget = Aget::new(&home)
        .with_session_store_backend(MemorySessionStore::new(&home))
        .with_browser_automation_backend(TestBrowserBackend::default());

    let started = aget
        .start_login_session(
            "docs",
            Some("custom-profile".to_string()),
            "https://example.com/login",
            Vec::new(),
        )
        .unwrap();
    assert_eq!(started.pending.name, "docs");
    assert_eq!(started.pending.profile, "custom-profile");
    assert_eq!(started.pending.url, "https://example.com/login");

    let cancelled = aget.cancel_login_session("docs").unwrap();
    assert_eq!(cancelled.pending.name, "docs");
    assert_eq!(cancelled.pending.profile, "test-profile");
}

#[test]
fn aget_start_login_session_injects_named_sessions_into_browser_backend() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    store
        .save(&cookie_session(
            "oauth",
            "accounts.example.com",
            "oauth-secret",
        ))
        .unwrap();
    store
        .save(&cookie_session(
            "target-helper",
            "login.example.com",
            "helper-secret",
        ))
        .unwrap();
    let browser = TestBrowserBackend::default();
    let calls = browser.login_starts.clone();

    let aget = Aget::new(&home)
        .with_session_store_backend(store)
        .with_browser_automation_backend(browser);
    let started = aget
        .start_login_session(
            "target",
            None,
            "https://example.com/login",
            vec!["oauth".to_string(), "target-helper".to_string()],
        )
        .unwrap();

    assert_eq!(
        started.pending.injected_sessions,
        vec!["oauth".to_string(), "target-helper".to_string()]
    );
    let calls = calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].injected_sessions.len(), 2);
    assert_eq!(
        calls[0].injected_sessions[0].cookies[0].value,
        "oauth-secret"
    );
    assert_eq!(
        calls[0].injected_sessions[1].cookies[0].value,
        "helper-secret"
    );
}

#[test]
fn aget_start_login_session_rejects_conflicting_injected_sessions_before_browser_backend() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    store
        .save(&cookie_session(
            "oauth-a",
            "accounts.example.com",
            "a-secret",
        ))
        .unwrap();
    store
        .save(&cookie_session(
            "oauth-b",
            "accounts.example.com",
            "b-secret",
        ))
        .unwrap();
    let browser = TestBrowserBackend::default();
    let calls = browser.login_starts.clone();

    let aget = Aget::new(&home)
        .with_session_store_backend(store)
        .with_browser_automation_backend(browser);
    let error = aget
        .start_login_session(
            "target",
            None,
            "https://example.com/login",
            vec!["oauth-a".to_string(), "oauth-b".to_string()],
        )
        .unwrap_err();

    assert_eq!(error.code(), aget::ErrorCode::SessionConflict);
    assert!(calls.borrow().is_empty());
}
