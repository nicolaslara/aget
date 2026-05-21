use std::fs;

use assert_cmd::Command;
use serde_json::json;

use crate::support::get_cli::{metadata_files, mock_backend_command};

#[test]
fn get_command_backend_accepts_scan_full_page_options() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "validate_crawl4ai_options": true,
            "expect_extractor_options": [
                "crawl4ai.scan_full_page=true",
                "crawl4ai.scroll_delay=0.3",
                "crawl4ai.max_scroll_steps=5",
                "crawl4ai.remove_forms=true",
                "crawl4ai.keep_data_attributes=true",
                "crawl4ai.exclude_all_images=true",
                "crawl4ai.exclude_external_images=true",
                "crawl4ai.exclude_external_links=true"
            ]
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    cmd.env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/scan-options",
            "--backend-option",
            "crawl4ai.scan_full_page=true",
            "--backend-option",
            "crawl4ai.scroll_delay=0.3",
            "--backend-option",
            "crawl4ai.max_scroll_steps=5",
            "--backend-option",
            "crawl4ai.remove_forms=true",
            "--backend-option",
            "crawl4ai.keep_data_attributes=true",
            "--backend-option",
            "crawl4ai.exclude_all_images=true",
            "--backend-option",
            "crawl4ai.exclude_external_images=true",
            "--backend-option",
            "crawl4ai.exclude_external_links=true",
        ])
        .assert()
        .success();
}

#[test]
fn get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "success", "validate_crawl4ai_options": true}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
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

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unsupported extractor option 'js_code'"));

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert!(metadata["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unsupported extractor option 'js_code'"));
}

#[test]
fn get_real_helper_rejects_javascript_wait_before_crawl4ai_import() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "success", "validate_crawl4ai_options": true}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
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

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--wait-for-selector only supports CSS selectors in v1"));

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert!(metadata["error"]["message"]
        .as_str()
        .unwrap()
        .contains("JavaScript wait conditions are not allowed"));
}
