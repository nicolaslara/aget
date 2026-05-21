use aget::{Aget, OutputFormat};

use crate::support::{
    cookie_session, FailingExtractor, InspectingExtractor, MemorySessionStore, TestBrowserBackend,
};

#[test]
fn aget_with_static_backends_uses_custom_session_store_and_extractor() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    store
        .save(&cookie_session("auth", "example.com", "memory-secret"))
        .unwrap();

    let result = Aget::new(&home)
        .with_session_store_backend(store)
        .with_extractor_backend(InspectingExtractor)
        .with_browser_automation_backend(TestBrowserBackend::default())
        .get("https://example.com/private")
        .session("auth")
        .content_format(OutputFormat::Text)
        .selector("main")
        .run()
        .unwrap();

    assert_eq!(result.extractor, "inspect-extractor");
    assert_eq!(result.final_url, "https://example.com/private/final");
    assert_eq!(result.content, "typed extractor content");
    assert_eq!(result.sessions, vec!["auth"]);
    assert!(result.sensitive);
    assert_eq!(result.warnings, vec!["custom extractor"]);
}

#[test]
fn aget_with_static_backends_uses_custom_browser_fallback_after_extractor_failure() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    store
        .save(&cookie_session("auth", "example.com", "fallback-secret"))
        .unwrap();

    let result = Aget::new(&home)
        .with_session_store_backend(store)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(TestBrowserBackend {
            fallback_content: Some("browser fallback content".to_string()),
            ..TestBrowserBackend::default()
        })
        .get("https://example.com/private")
        .session("auth")
        .run()
        .unwrap();

    assert_eq!(result.extractor, "test-browser-fallback");
    assert_eq!(result.content, "browser fallback content");
    assert_eq!(result.warnings, vec!["custom browser fallback"]);
    assert!(result.sensitive);
}
