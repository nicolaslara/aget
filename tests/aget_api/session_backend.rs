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
        )
        .unwrap();
    assert_eq!(started.pending.name, "docs");
    assert_eq!(started.pending.profile, "custom-profile");
    assert_eq!(started.pending.url, "https://example.com/login");

    let cancelled = aget.cancel_login_session("docs").unwrap();
    assert_eq!(cancelled.pending.name, "docs");
    assert_eq!(cancelled.pending.profile, "test-profile");
}
