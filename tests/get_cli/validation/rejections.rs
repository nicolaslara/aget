use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::json;

use crate::support::get_cli::{metadata_files, mock_backend_command};

#[test]
fn get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "success", "validate_crawl4ai_options": true}),
    );

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/unsupported-option",
            "--backend-option",
            "crawl4ai.js_code=alert(1)",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    assert_failure_error_contains(
        &output,
        &aget_home,
        "unsupported extractor option 'js_code'",
        "unsupported extractor option 'js_code'",
    );
}

#[test]
fn get_real_helper_rejects_javascript_wait_before_crawl4ai_import() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "success", "validate_crawl4ai_options": true}),
    );

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/js-wait",
            "--wait-for-selector",
            "js:() => true",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    assert_failure_error_contains(
        &output,
        &aget_home,
        "--wait-for-selector only supports CSS selectors in v1",
        "JavaScript wait conditions are not allowed",
    );
}

fn assert_failure_error_contains(
    output: &[u8],
    aget_home: &Path,
    expected_error: &str,
    expected_metadata: &str,
) {
    let json: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains(expected_error));

    assert_metadata_error_contains(aget_home, expected_metadata);
}

fn assert_metadata_error_contains(aget_home: &Path, expected: &str) {
    let metadata_files = metadata_files(aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert!(metadata["error"]["message"]
        .as_str()
        .unwrap()
        .contains(expected));
}
