use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::json;
use support::get_cli::{
    metadata_files, mock_backend_command, success_backend, success_data, success_envelope,
};

#[path = "get_cli/session.rs"]
mod session;
mod support;

#[test]
fn get_json_success_writes_run_artifacts_with_empty_state() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let local_url = "https://example.com/public-fetch".to_string();
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "expect_state": {"cookies": [], "origins": []},
            "content": "# Example\n\nFetched locally.",
            "final_url_suffix": "/final",
            "warnings": ["fake warning"]
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args(["--envelope", "json", "get", &local_url])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = success_envelope(&output, "get");
    let json = &envelope["data"];
    assert_eq!(json["url"].as_str().unwrap(), local_url);
    assert_eq!(
        json["final_url"].as_str().unwrap(),
        format!("{local_url}/final")
    );
    assert_eq!(json["content_format"], "markdown");
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(json["content"], "# Example\n\nFetched locally.");
    assert_eq!(json["sessions"], serde_json::json!([]));
    assert_eq!(json["sensitive"], false);
    assert_eq!(envelope["warnings"], serde_json::json!(["fake warning"]));
    assert_eq!(json["limits"]["max_chars"], serde_json::Value::Null);
    assert_eq!(json["limits"]["truncated"], false);

    let content_path = PathBuf::from(json["artifacts"]["content"].as_str().unwrap());
    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    assert_eq!(
        fs::read_to_string(&content_path).unwrap(),
        "# Example\n\nFetched locally.\n"
    );
    assert!(metadata_path.starts_with(aget_home.join("runs")));

    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["ok"], true);
    assert_eq!(metadata["extractor"], "crawl4ai");
    assert_eq!(
        metadata["artifacts"]["content"],
        content_path.to_string_lossy().as_ref()
    );
    assert!(fs::read_dir(aget_home.join("tmp"))
        .unwrap()
        .next()
        .is_none());
}

#[test]
fn get_out_writes_markdown_to_requested_path_and_metadata_to_run_dir() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let out_path = temp.path().join("requested.md");
    let fake_backend = success_backend(temp.path());

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/out",
            "--output",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(
        json["artifacts"]["content"].as_str().unwrap(),
        out_path.to_string_lossy().as_ref()
    );
    assert_eq!(fs::read_to_string(&out_path).unwrap(), "# Fake\n");
    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    assert!(metadata_path.starts_with(aget_home.join("runs")));
}

#[test]
fn get_inline_content_never_omits_content_for_public_fetches() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "content": "# Public But Artifact Only"
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/artifact-only",
            "--inline-content",
            "never",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["sensitive"], false);
    assert!(!json.as_object().unwrap().contains_key("content"));
    let content_path = PathBuf::from(json["artifacts"]["content"].as_str().unwrap());
    assert_eq!(
        fs::read_to_string(content_path).unwrap(),
        "# Public But Artifact Only\n"
    );
}

#[test]
fn get_backend_does_not_inherit_unneeded_parent_environment() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "content": "# Env Scrubbed",
            "expect_env_absent": ["SECRET_TOKEN", "AGET_HOME"]
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .env("SECRET_TOKEN", "do-not-pass")
        .args(["--envelope", "json", "get", "https://example.com/env"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["content"], "# Env Scrubbed");
}

#[test]
fn get_forwards_supported_output_options_and_records_limits() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "content": "aé💡bc",
            "final_url_suffix": "#done",
            "expect_format": "text",
            "expect_selector": "main.article",
            "expect_exclude_selector": "nav,.ad",
            "expect_wait_for": "css:.ready",
            "expect_extractor_options": ["crawl4ai.cache=bypass", "crawl4ai.magic=value"]
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/options",
            "--content-format",
            "text",
            "--selector",
            "main.article",
            "--exclude-selector",
            "nav,.ad",
            "--wait-for-selector",
            "css:.ready",
            "--max-chars",
            "3",
            "--backend-option",
            "crawl4ai.cache=bypass",
            "--backend-option",
            "crawl4ai.magic=value",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["content_format"], "text");
    assert_eq!(json["content"], "aé💡");
    assert_eq!(json["final_url"], "https://example.com/options#done");
    assert_eq!(json["limits"]["max_chars"], 3);
    assert_eq!(json["limits"]["truncated"], true);
    assert_eq!(json["limits"]["truncated_by"], "max_chars");
    assert_eq!(json["limits"]["content_chars_before_truncation"], 5);
    assert_eq!(json["limits"]["content_chars_after_truncation"], 3);
    assert_eq!(json["output_options"]["content_format"], "text");
    assert_eq!(json["output_options"]["selector"], "main.article");
    assert_eq!(json["output_options"]["exclude_selector"], "nav,.ad");
    assert_eq!(json["output_options"]["wait_for_selector"], "css:.ready");
    assert_eq!(
        json["output_options"]["backend_options"],
        serde_json::json!({"crawl4ai.cache": "bypass", "crawl4ai.magic": "value"})
    );

    let content_path = PathBuf::from(json["artifacts"]["content"].as_str().unwrap());
    assert_eq!(fs::read_to_string(&content_path).unwrap(), "aé💡\n");
    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["content"], serde_json::Value::Null);
    assert_eq!(metadata["output_options"], json["output_options"]);
    assert_eq!(metadata["limits"], json["limits"]);
}

#[test]
fn get_max_chars_success_sanitizes_backend_stdout_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "success", "content": "SECRET-UNTRUNCATED-CONTENT"}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/secret",
            "--max-chars",
            "6",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["content"], "SECRET");
    assert_eq!(json["limits"]["truncated"], true);

    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    let backend_stdout_path = metadata_path.with_file_name("backend-stdout.json");
    let backend_stdout = fs::read_to_string(backend_stdout_path).unwrap();
    assert!(!backend_stdout.contains("SECRET-UNTRUNCATED-CONTENT"));
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

#[test]
fn get_json_format_truncates_only_content_not_response_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "content": "{\"title\":\"Example\",\"body\":\"abcdef\"}",
            "expect_format": "json"
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/json",
            "--content-format",
            "json",
            "--max-chars",
            "12",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = success_envelope(&output, "get");
    let json = &envelope["data"];
    assert_eq!(json["content_format"], "json");
    assert_eq!(json["content"], "{\"title\":\"Ex");
    assert_eq!(json["limits"]["truncated"], true);
    assert!(!json["artifacts"]["metadata"].as_str().unwrap().is_empty());
    assert!(envelope["timing_ms"]["total"].as_u64().is_some());
}

#[test]
fn get_timeout_returns_stable_error_and_error_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend =
        mock_backend_command(temp.path(), json!({"behavior": "sleep", "seconds": 3}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--timeout",
            "1",
            "--envelope",
            "json",
            "get",
            "https://example.com/slow",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "get");
    assert_eq!(json["error"]["code"], "timeout");

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert_eq!(metadata["extractor"], "crawl4ai");
    assert_eq!(metadata["error"]["code"], "timeout");
}

#[test]
fn get_noisy_backend_output_does_not_deadlock() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "noisy_success", "content": "# Noisy", "noise_repetitions": 20000}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--timeout",
            "2",
            "--envelope",
            "json",
            "get",
            "https://example.com/noisy",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(json["content"], "# Noisy");
}

#[test]
fn get_backend_stdout_logs_before_json_succeeds() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "stdout_logs_success",
            "content": "# Logged stdout",
            "stdout_lines": ["[INIT] Starting browser", "[FETCH] Fetching", "[COMPLETE] Crawling done"]
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/stdout-logs",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(json["content"], "# Logged stdout");
}

#[cfg(unix)]
#[test]
fn get_timeout_terminates_backend_descendants() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let marker = temp.path().join("descendant-survived.txt");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "spawn_descendant_and_sleep",
            "marker": marker,
            "seconds": 5
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--timeout",
            "1",
            "--envelope",
            "json",
            "get",
            "https://example.com/slow",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "timeout");
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(!marker.exists());
}

#[test]
fn get_nonzero_and_malformed_backend_results_are_extraction_failed() {
    let temp = tempfile::tempdir().unwrap();
    let nonzero_home = temp.path().join("nonzero-home");
    let nonzero_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "exit", "stderr": "backend exploded", "exit_code": 2}),
    );

    let mut nonzero = Command::cargo_bin("aget").unwrap();
    let nonzero_output = nonzero
        .env("AGET_HOME", &nonzero_home)
        .env("AGET_CRAWL4AI_COMMAND", &nonzero_backend)
        .args(["--envelope", "json", "get", "https://example.com/error"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let nonzero_json: serde_json::Value = serde_json::from_slice(&nonzero_output).unwrap();
    assert_eq!(nonzero_json["error"]["code"], "extraction_failed");

    let malformed_home = temp.path().join("malformed-home");
    let malformed_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "malformed", "stdout": "not json"}),
    );

    let mut malformed = Command::cargo_bin("aget").unwrap();
    let malformed_output = malformed
        .env("AGET_HOME", &malformed_home)
        .env("AGET_CRAWL4AI_COMMAND", &malformed_backend)
        .args(["--envelope", "json", "get", "https://example.com/malformed"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let malformed_json: serde_json::Value = serde_json::from_slice(&malformed_output).unwrap();
    assert_eq!(malformed_json["error"]["code"], "extraction_failed");
}

#[test]
fn get_nonzero_backend_preserves_structured_failure_but_not_structured_success() {
    let temp = tempfile::tempdir().unwrap();
    let structured_failure_home = temp.path().join("structured-failure-home");
    let structured_failure_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "structured_failure", "error": "structured backend failure", "exit_code": 2}),
    );

    let mut structured_failure = Command::cargo_bin("aget").unwrap();
    let structured_failure_output = structured_failure
        .env("AGET_HOME", &structured_failure_home)
        .env("AGET_CRAWL4AI_COMMAND", &structured_failure_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/structured-failure",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let structured_failure_json: serde_json::Value =
        serde_json::from_slice(&structured_failure_output).unwrap();
    assert_eq!(
        structured_failure_json["error"]["code"],
        "extraction_failed"
    );
    assert_eq!(
        structured_failure_json["error"]["message"],
        "structured backend failure"
    );

    let structured_success_home = temp.path().join("structured-success-home");
    let structured_success_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "final_url": "https://example.com",
            "content": "# Wrong",
            "exit_code": 2
        }),
    );

    let mut structured_success = Command::cargo_bin("aget").unwrap();
    let structured_success_output = structured_success
        .env("AGET_HOME", &structured_success_home)
        .env("AGET_CRAWL4AI_COMMAND", &structured_success_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/structured-success",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let structured_success_json: serde_json::Value =
        serde_json::from_slice(&structured_success_output).unwrap();
    assert_eq!(
        structured_success_json["error"]["code"],
        "extraction_failed"
    );
    assert_ne!(structured_success_json["content"], "# Wrong");
}

#[test]
fn missing_backend_returns_backend_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", "definitely_missing_aget_backend")
        .args(["--envelope", "json", "get", "https://example.com/missing"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "backend_unavailable");

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["extractor"], "crawl4ai");
}
