use std::{fs, path::PathBuf};

use aget::{Aget, ErrorCode, OutputFormat};

use crate::support::{
    cookie_session, FailingExtractor, InspectingExtractor, MemorySessionStore,
    SecretLeakingExtractor, TestBrowserBackend,
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

#[test]
fn aget_session_failure_metadata_redacts_sensitive_backend_error() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    store
        .save(&cookie_session("auth", "example.com", "session-secret"))
        .unwrap();

    let error = Aget::new(&home)
        .with_session_store_backend(store)
        .with_extractor_backend(SecretLeakingExtractor)
        .with_browser_automation_backend(TestBrowserBackend::default())
        .get("https://example.com/private")
        .session("auth")
        .run()
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::ExtractionFailed);
    assert!(!error.to_string().contains("session-secret"));

    let metadata_path = only_metadata_path(home.join("runs"));
    let metadata = fs::read_to_string(metadata_path).unwrap();
    assert!(!metadata.contains("session-secret"));
    let json: serde_json::Value = serde_json::from_str(&metadata).unwrap();
    assert_eq!(json["sensitive"], true);
    assert_eq!(json["sessions"], serde_json::json!(["auth"]));
    assert_eq!(
        json["error"]["message"],
        "primary extraction failed for session-backed request"
    );
}

fn only_metadata_path(runs_dir: PathBuf) -> PathBuf {
    let paths = fs::read_dir(runs_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path().join("metadata.json"))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 1);
    paths.into_iter().next().unwrap()
}
