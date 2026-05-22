use std::fs;

use crate::browser_cdp::discovery::{
    classify_chrome_startup_error, devtools_ws_url_from_stderr, relevant_chrome_stderr,
};
use crate::error::{AgetError, ErrorCode};
use crate::process::TempOutputFile;

#[test]
fn parses_devtools_ws_url_from_chrome_stderr() {
    let stderr = "\
noise
DevTools listening on ws://127.0.0.1:9222/devtools/browser/fallback
more noise";

    assert_eq!(
        devtools_ws_url_from_stderr(stderr).as_deref(),
        Some("ws://127.0.0.1:9222/devtools/browser/fallback")
    );
}

#[test]
fn chrome_stderr_detail_includes_sandbox_hint() {
    let detail = relevant_chrome_stderr(
        "Failed to move to new namespace: PID namespaces supported, Network namespace supported, but failed: errno = Operation not permitted",
    );

    assert!(detail.contains("Failed to move to new namespace"));
    assert!(detail.contains("Chrome sandbox/namespace startup failure"));
    assert!(detail.contains("AGET_CHROME_COMMAND"));
}

#[test]
fn chrome_stderr_detail_labels_generic_tail_lines() {
    let detail = relevant_chrome_stderr(
        "info: startup preparing\n\
         info: still warming\n\
         note: first generic line\n\
         trace: second generic line\n\
         debug: third generic line\n\
         debug: fourth generic line",
    );

    assert!(detail.contains("Chrome stderr (last 5 lines):"));
    assert!(!detail.contains("startup preparing"));
    assert!(detail.contains("info: still warming"));
    assert!(detail.contains("note: first generic line"));
    assert!(detail.contains("trace: second generic line"));
    assert!(detail.contains("debug: third generic line"));
    assert!(detail.contains("debug: fourth generic line"));
}

#[test]
fn chrome_startup_error_adds_silent_exit_hint_without_stderr() {
    let temp = tempfile::tempdir().unwrap();
    let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
    let error = AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: "owned browser fallback Chrome exited before CDP startup".to_string(),
    };

    let classified = classify_chrome_startup_error("owned Chrome test", error, &stderr_capture);

    assert_eq!(classified.code(), ErrorCode::BackendUnavailable);
    let message = classified.to_string();
    assert!(message.contains("Chrome exited without startup diagnostics"));
    assert!(message.contains("AGET_CHROME_COMMAND"));
}

#[test]
fn chrome_startup_error_includes_labeled_generic_stderr() {
    let temp = tempfile::tempdir().unwrap();
    let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
    fs::write(
        stderr_capture.path(),
        "startup line one\nstartup line two\nstartup line three\nstartup line four\nstartup line five\nstartup line six\n",
    )
    .unwrap();
    let error = AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: "owned browser fallback Chrome exited before CDP startup".to_string(),
    };

    let classified = classify_chrome_startup_error("owned Chrome test", error, &stderr_capture);

    assert_eq!(classified.code(), ErrorCode::BackendUnavailable);
    let message = classified.to_string();
    assert!(message.contains("Chrome stderr (last 5 lines):"));
    assert!(!message.contains("startup line one"));
    assert!(message.contains("startup line two"));
    assert!(message.contains("startup line three"));
    assert!(message.contains("startup line four"));
    assert!(message.contains("startup line five"));
    assert!(message.contains("startup line six"));
}
