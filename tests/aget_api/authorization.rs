use aget::{Aget, AuthorizationState, AuthorizeSessionOptions, ErrorCode};

use crate::support::{
    cookie_session, AuthorizationExtractor, MemorySessionStore, TestBrowserBackend,
};

#[test]
fn authorize_chrome_session_fetches_baseline_imports_and_verifies() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let extractor = AuthorizationExtractor::new("Please sign in", "Welcome private content");
    let browser = TestBrowserBackend {
        login_session: Some(cookie_session("news", "example.com", "chrome-secret")),
        ..TestBrowserBackend::default()
    };

    let result = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_extractor_backend(extractor.clone())
        .with_browser_automation_backend(browser)
        .authorize_chrome_session(AuthorizeSessionOptions {
            name: "news".to_string(),
            url: "https://example.com/account".to_string(),
            chrome_profile: "Default".to_string(),
            allow_domains: vec!["example.com".to_string()],
            must_contain: vec!["Welcome".to_string()],
            must_not_contain: vec!["Please sign in".to_string()],
            output: None,
        })
        .unwrap();

    assert_eq!(result.state, AuthorizationState::Verified);
    assert_eq!(result.source, "chrome");
    assert_eq!(result.baseline.content, "Please sign in");
    assert!(result.baseline.sessions.is_empty());
    assert!(!result.baseline.sensitive);
    assert_eq!(result.verification.content, "Welcome private content");
    assert_eq!(result.verification.sessions, vec!["news"]);
    assert!(result.verification.sensitive);
    assert!(result.predicates.iter().all(|predicate| predicate.matched));
    assert_eq!(extractor.call_count(), 2);
    assert_eq!(
        store.load("news").unwrap().cookies[0].value,
        "chrome-secret"
    );
}

#[test]
fn authorize_chrome_session_reports_failed_verification_predicates() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let extractor = AuthorizationExtractor::new("Please sign in", "Please sign in");
    let browser = TestBrowserBackend {
        login_session: Some(cookie_session("news", "example.com", "chrome-secret")),
        ..TestBrowserBackend::default()
    };

    let result = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_extractor_backend(extractor)
        .with_browser_automation_backend(browser)
        .authorize_chrome_session(AuthorizeSessionOptions {
            name: "news".to_string(),
            url: "https://example.com/account".to_string(),
            chrome_profile: "Default".to_string(),
            allow_domains: vec!["example.com".to_string()],
            must_contain: vec!["Welcome".to_string()],
            must_not_contain: vec!["Please sign in".to_string()],
            output: None,
        })
        .unwrap();

    assert_eq!(result.state, AuthorizationState::VerificationFailed);
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.contains("predicates failed")));
    assert!(result.predicates.iter().any(|predicate| {
        predicate.kind == "must_contain" && predicate.value == "Welcome" && !predicate.matched
    }));
    assert!(store.load("news").is_ok());
}

#[test]
fn authorize_chrome_session_preserves_requires_user_action_after_baseline_fetch() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let extractor = AuthorizationExtractor::new("Please sign in", "Welcome private content");
    let browser = TestBrowserBackend {
        import_error: Some((
            ErrorCode::RequiresUserAction,
            "quit this browser/profile before import".to_string(),
        )),
        ..TestBrowserBackend::default()
    };

    let error = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_extractor_backend(extractor.clone())
        .with_browser_automation_backend(browser)
        .authorize_chrome_session(AuthorizeSessionOptions {
            name: "news".to_string(),
            url: "https://example.com/account".to_string(),
            chrome_profile: "Default".to_string(),
            allow_domains: vec!["example.com".to_string()],
            must_contain: vec!["Welcome".to_string()],
            must_not_contain: Vec::new(),
            output: None,
        })
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::RequiresUserAction);
    assert!(error.to_string().contains("quit this browser/profile"));
    assert_eq!(extractor.call_count(), 1);
    assert!(store.load("news").is_err());
}
