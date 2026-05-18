use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use aget::{Session, SessionCookie, SessionOrigin, SessionStore, StorageEntry};
use assert_cmd::Command;
use serde_json::json;

#[path = "support/mock_tools.rs"]
mod mock_tools;
use mock_tools::{mock_agent_browser, mock_backend_command};

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
        .args(["--json", "get", &local_url])
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
    assert_eq!(json["format"], "markdown");
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
            "--json",
            "get",
            "https://example.com/out",
            "--out",
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
fn get_session_uses_named_session_state_and_marks_sensitive() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    save_cookie_session(&aget_home, "local", "127.0.0.1", "sid", "secret-cookie");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "content": "# Session Fetch",
            "expect_state": {
                "cookies": [{
                    "name": "sid",
                    "value": "secret-cookie",
                    "domain": "127.0.0.1",
                    "path": "/",
                    "httpOnly": true,
                    "secure": false,
                    "sameSite": "Lax"
                }],
                "origins": []
            }
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            "http://127.0.0.1/session",
            "--session",
            "local",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["sessions"], serde_json::json!(["local"]));
    assert_eq!(json["sensitive"], true);

    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["sessions"], serde_json::json!(["local"]));
    assert_eq!(metadata["sensitive"], true);
}

#[test]
fn get_rejects_session_replay_outside_saved_scope() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    save_cookie_session(
        &aget_home,
        "private-docs",
        "private.example.com",
        "sid",
        "secret-cookie",
    );
    let fake_backend = mock_backend_command(temp.path(), json!({"behavior": "exit"}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            "https://unrelated.example/page",
            "--session",
            "private-docs",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "get");
    assert_eq!(json["error"]["code"], "privacy_policy_blocked");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("outside request host"));
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
        .args(["--json", "get", "https://example.com/env"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["content"], "# Env Scrubbed");
}

#[test]
fn get_repeated_sessions_compose_request_state_in_order_for_command_and_alias() {
    let temp = tempfile::tempdir().unwrap();
    let command_home = temp.path().join("command-home");
    save_cookie_session(
        &command_home,
        "provider",
        "127.0.0.1",
        "oauth",
        "provider-secret",
    );
    save_cookie_session(&command_home, "app", "127.0.0.1", "appsid", "app-secret");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "content": "# Multi Session Fetch",
            "expect_state_cookies": [["appsid", "app-secret"], ["oauth", "provider-secret"]]
        }),
    );

    let mut command = Command::cargo_bin("aget").unwrap();
    let command_output = command
        .env("AGET_HOME", &command_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            "http://127.0.0.1/multi",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let command_json = success_data(&command_output, "get");
    assert_eq!(
        command_json["sessions"],
        serde_json::json!(["provider", "app"])
    );
    assert_eq!(command_json["sensitive"], true);

    let alias_home = temp.path().join("alias-home");
    save_cookie_session(
        &alias_home,
        "provider",
        "127.0.0.1",
        "oauth",
        "provider-secret",
    );
    save_cookie_session(&alias_home, "app", "127.0.0.1", "appsid", "app-secret");
    let mut alias = Command::cargo_bin("aget").unwrap();
    let alias_output = alias
        .env("AGET_HOME", &alias_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "http://127.0.0.1/multi-alias",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let alias_json = success_data(&alias_output, "get");
    assert_eq!(
        alias_json["sessions"],
        serde_json::json!(["provider", "app"])
    );
}

#[test]
fn get_repeated_sessions_satisfy_local_app_provider_cookie_flow() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    save_cookie_session(
        &aget_home,
        "provider",
        "127.0.0.1",
        "oauth",
        "provider-secret",
    );
    save_cookie_session(&aget_home, "app", "127.0.0.1", "appsid", "app-secret");
    let local_url = "http://127.0.0.1/app-provider".to_string();
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "content": "# App Provider OK\n\nboth cookies accepted",
            "expect_state_cookies": [["appsid", "app-secret"], ["oauth", "provider-secret"]]
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            &local_url,
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json = success_data(&output, "get");
    assert_eq!(
        json["content"],
        "# App Provider OK\n\nboth cookies accepted"
    );
}

#[test]
fn get_session_marks_output_sensitive_even_if_session_metadata_is_false() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    save_cookie_session_with_sensitivity(
        &aget_home,
        "local",
        "127.0.0.1",
        "sid",
        "secret-cookie",
        false,
    );
    let fake_backend = success_backend(temp.path());

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            "http://127.0.0.1/session",
            "--session",
            "local",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["sessions"], serde_json::json!(["local"]));
    assert_eq!(json["sensitive"], true);
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
            "--json",
            "get",
            "https://example.com/options",
            "--format",
            "text",
            "--selector",
            "main.article",
            "--exclude-selector",
            "nav,.ad",
            "--wait-for",
            "css:.ready",
            "--max-chars",
            "3",
            "--extractor-option",
            "crawl4ai.cache=bypass",
            "--extractor-option",
            "crawl4ai.magic=value",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["format"], "text");
    assert_eq!(json["content"], "aé💡");
    assert_eq!(json["final_url"], "https://example.com/options#done");
    assert_eq!(json["limits"]["max_chars"], 3);
    assert_eq!(json["limits"]["truncated"], true);
    assert_eq!(json["limits"]["truncated_by"], "max_chars");
    assert_eq!(json["limits"]["content_chars_before_truncation"], 5);
    assert_eq!(json["limits"]["content_chars_after_truncation"], 3);
    assert_eq!(json["output_options"]["format"], "text");
    assert_eq!(json["output_options"]["selector"], "main.article");
    assert_eq!(json["output_options"]["exclude_selector"], "nav,.ad");
    assert_eq!(json["output_options"]["wait_for"], "css:.ready");
    assert_eq!(
        json["output_options"]["extractor_options"],
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
            "--json",
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
fn get_session_backend_failure_redacts_state_secrets_from_errors_metadata_and_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    save_cookie_and_storage_session(
        &aget_home,
        "local",
        "127.0.0.1",
        "sid",
        "cookie-secret-value",
        "https://127.0.0.1",
        "token",
        "storage-secret-value",
    );
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "structured_failure",
            "stderr": "stderr leaked {first_cookie_value} and {first_storage_value}",
            "error": "backend returned {first_cookie_value} and {first_storage_value}"
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            "http://127.0.0.1/sensitive-fail",
            "--session",
            "local",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let stderr = String::from_utf8_lossy(&output);
    assert!(!stderr.contains("cookie-secret-value"));
    assert!(!stderr.contains("storage-secret-value"));
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert_eq!(json["command"], "get");
    assert_eq!(
        json["error"]["message"],
        "Crawl4AI extraction failed for session-backed request"
    );

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata_text = fs::read_to_string(&metadata_files[0]).unwrap();
    assert!(!metadata_text.contains("cookie-secret-value"));
    assert!(!metadata_text.contains("storage-secret-value"));
    let metadata: serde_json::Value = serde_json::from_str(&metadata_text).unwrap();
    assert_eq!(
        metadata["error"]["message"],
        "Crawl4AI extraction failed for session-backed request"
    );

    let backend_stdout =
        fs::read_to_string(metadata_files[0].with_file_name("backend-stdout.json")).unwrap();
    let backend_stderr =
        fs::read_to_string(metadata_files[0].with_file_name("backend-stderr.txt")).unwrap();
    assert!(!backend_stdout.contains("cookie-secret-value"));
    assert!(!backend_stdout.contains("storage-secret-value"));
    assert!(!backend_stderr.contains("cookie-secret-value"));
    assert!(!backend_stderr.contains("storage-secret-value"));
    assert!(backend_stdout.contains("<redacted>"));
    assert!(backend_stderr.contains("<redacted>"));
}

#[test]
fn get_session_backend_failure_uses_agent_browser_fallback_with_composed_state() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let agent_log = temp.path().join("agent-browser.log");
    save_cookie_and_storage_session(
        &aget_home,
        "local",
        "127.0.0.1",
        "sid",
        "cookie-secret-value",
        "https://127.0.0.1",
        "token",
        "storage-secret-value",
    );
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "structured_failure",
            "stderr": "backend saw {first_cookie_value}",
            "error": "crawl4ai failed after state load {first_storage_value}",
            "exit_code": 1
        }),
    );
    let fake_agent_browser = mock_agent_browser(
        temp.path(),
        json!({"log_path": agent_log.to_string_lossy(), "html": "<main><h1>Fallback Title</h1><p>Useful &amp; local content</p></main>"}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGENT_BROWSER_LOG", &agent_log)
        .args([
            "--json",
            "get",
            "http://127.0.0.1/fallback",
            "--session",
            "local",
            "--format",
            "markdown",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = success_envelope(&output, "get");
    let json = &envelope["data"];
    assert_eq!(json["extractor"], "agent-browser-fallback");
    assert_eq!(json["sessions"], serde_json::json!(["local"]));
    assert_eq!(json["sensitive"], true);
    assert_eq!(
        envelope["warnings"],
        serde_json::json!(["agent-browser fallback used after Crawl4AI failed"])
    );
    assert_eq!(json["content"], "Fallback Title\n\nUseful & local content");

    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["extractor"], "agent-browser-fallback");
    let backend_stderr =
        fs::read_to_string(metadata_path.with_file_name("backend-stderr.txt")).unwrap();
    assert!(!backend_stderr.contains("cookie-secret-value"));
    assert!(backend_stderr.contains("<redacted>"));

    let log = fs::read_to_string(&agent_log).unwrap();
    assert!(log.contains("\"state\", \"load\""));
    assert!(log.contains("\"open\""));
    assert!(log.contains("\"get\", \"html\", \"body\""));
    assert!(log.contains("\"close\""));
    assert!(log.contains(
        &aget_home
            .join("tmp/agent-browser")
            .to_string_lossy()
            .to_string()
    ));
    assert!(fs::read_dir(aget_home.join("tmp/agent-browser"))
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(true));
}

#[test]
fn get_unauthenticated_backend_failure_does_not_use_agent_browser_fallback() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let agent_log = temp.path().join("agent-browser.log");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "structured_failure", "error": "public crawl4ai failure", "exit_code": 1}),
    );
    let fake_agent_browser = mock_agent_browser(
        temp.path(),
        json!({"log_path": agent_log.to_string_lossy()}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGENT_BROWSER_LOG", &agent_log)
        .args(["--json", "get", "http://127.0.0.1/public-failure"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert_eq!(json["error"]["message"], "public crawl4ai failure");
    assert!(!agent_log.exists());
}

#[test]
fn get_session_fallback_close_failure_preserves_original_sanitized_crawl4ai_error() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    save_cookie_session(
        &aget_home,
        "local",
        "127.0.0.1",
        "sid",
        "cookie-secret-value",
    );
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "structured_failure",
            "error": "crawl4ai leaked {first_cookie_value}",
            "exit_code": 1
        }),
    );
    let fake_agent_browser = mock_agent_browser(
        temp.path(),
        json!({"behavior": "close_failure", "html": "<main>fallback content</main>"}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--json",
            "get",
            "http://127.0.0.1/fallback-close-fails",
            "--session",
            "local",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert_eq!(
        json["error"]["message"],
        "Crawl4AI extraction failed for session-backed request"
    );
    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert_eq!(metadata["extractor"], "crawl4ai");
    assert!(!fs::read_to_string(&metadata_files[0])
        .unwrap()
        .contains("cookie-secret-value"));
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
            "--json",
            "get",
            "https://example.com/unsupported-option",
            "--extractor-option",
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
            "--json",
            "get",
            "https://example.com/js-wait",
            "--wait-for",
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
        .contains("--wait-for only supports CSS selectors in v1"));

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
            "--json",
            "get",
            "https://example.com/json",
            "--format",
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
    assert_eq!(json["format"], "json");
    assert_eq!(json["content"], "{\"title\":\"Ex");
    assert_eq!(json["limits"]["truncated"], true);
    assert!(!json["artifacts"]["metadata"].as_str().unwrap().is_empty());
    assert!(envelope["timing_ms"]["total"].as_u64().is_some());
}

#[test]
#[ignore = "requires local Crawl4AI and Playwright browser setup"]
fn real_crawl4ai_replays_named_session_cookie() {
    let temp = tempfile::tempdir().unwrap();
    let empty_home = temp.path().join("empty-home");
    let (empty_url, empty_server, empty_cookie) = cookie_echo_server("/cookie-empty");

    let mut empty_cmd = Command::cargo_bin("aget").unwrap();
    let empty_output = empty_cmd
        .env("AGET_HOME", &empty_home)
        .args(["--json", "get", &empty_url, "--timeout", "60"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    empty_server.join().unwrap();

    let empty_json = success_data(&empty_output, "get");
    assert_eq!(empty_json["sessions"], serde_json::json!([]));
    assert_eq!(empty_json["sensitive"], false);
    assert!(empty_cookie
        .try_iter()
        .all(|cookie| !cookie.contains("sid=secret-cookie")));

    let session_home = temp.path().join("session-home");
    save_cookie_session(&session_home, "local", "127.0.0.1", "sid", "secret-cookie");
    let (session_url, session_server, session_cookie) = cookie_echo_server("/cookie-session");

    let mut session_cmd = Command::cargo_bin("aget").unwrap();
    let session_output = session_cmd
        .env("AGET_HOME", &session_home)
        .args([
            "--json",
            "get",
            &session_url,
            "--session",
            "local",
            "--timeout",
            "60",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    session_server.join().unwrap();

    let session_json = success_data(&session_output, "get");
    assert_eq!(session_json["sessions"], serde_json::json!(["local"]));
    assert_eq!(session_json["sensitive"], true);
    assert!(session_cookie
        .try_iter()
        .any(|cookie| cookie.contains("sid=secret-cookie")));
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
            "--json",
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
            "--json",
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
        .args(["--json", "get", "https://example.com/stdout-logs"])
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
            "--json",
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
        .args(["--json", "get", "https://example.com/error"])
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
        .args(["--json", "get", "https://example.com/malformed"])
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
        .args(["--json", "get", "https://example.com/structured-failure"])
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
        .args(["--json", "get", "https://example.com/structured-success"])
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
        .args(["--json", "get", "https://example.com/missing"])
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

fn success_backend(dir: &Path) -> String {
    mock_backend_command(dir, json!({"behavior": "success", "content": "# Fake"}))
}

fn success_envelope(output: &[u8], command: &str) -> serde_json::Value {
    let json: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], command);
    assert!(json["timing_ms"]["total"].as_u64().is_some());
    json
}

fn success_data(output: &[u8], command: &str) -> serde_json::Value {
    success_envelope(output, command)["data"].clone()
}

fn cookie_echo_server(path: &str) -> (String, JoinHandle<()>, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let path = path.to_string();
    let url = format!("http://{addr}{path}");
    let (cookie_sender, cookie_receiver) = mpsc::channel();

    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut handled_request = false;
        let mut idle_after_request = None;

        while Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buffer = [0; 4096];
                    let bytes_read = stream.read(&mut buffer).unwrap_or_default();
                    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                    handled_request = true;
                    idle_after_request = Some(Instant::now() + Duration::from_secs(2));
                    let cookie = request
                        .lines()
                        .find_map(|line| line.strip_prefix("Cookie: "))
                        .unwrap_or("none");
                    let _ = cookie_sender.send(cookie.to_string());
                    let filler = "Local cookie replay verification content. ".repeat(40);
                    let body = format!(
                        "<!doctype html><html><body><main><h1>Cookie Echo</h1><p>Path: {path}</p><p>Cookie: {cookie}</p><article>{filler}</article></main></body></html>"
                    );
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nSet-Cookie: server_set=ok; Path=/; SameSite=Lax\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if handled_request
                        && matches!(idle_after_request, Some(end) if Instant::now() >= end)
                    {
                        break;
                    }
                    thread::sleep(Duration::from_millis(20));
                }
                Err(error) => panic!("cookie echo server failed: {error}"),
            }
        }

        assert!(handled_request);
    });

    (url, handle, cookie_receiver)
}

fn metadata_files(aget_home: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(aget_home.join("runs")).unwrap() {
        let entry = entry.unwrap();
        let metadata = entry.path().join("metadata.json");
        if metadata.exists() {
            files.push(metadata);
        }
    }
    files
}

fn save_cookie_session(home: &Path, name: &str, domain: &str, cookie_name: &str, value: &str) {
    save_cookie_session_with_sensitivity(home, name, domain, cookie_name, value, true);
}

fn save_cookie_session_with_sensitivity(
    home: &Path,
    name: &str,
    domain: &str,
    cookie_name: &str,
    value: &str,
    sensitive: bool,
) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.sensitive = sensitive;
    session.allowed_cookie_domains.push(domain.to_string());
    session.cookies.push(SessionCookie {
        name: cookie_name.to_string(),
        value: value.to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}

#[allow(clippy::too_many_arguments)]
fn save_cookie_and_storage_session(
    home: &Path,
    name: &str,
    domain: &str,
    cookie_name: &str,
    cookie_value: &str,
    origin: &str,
    storage_name: &str,
    storage_value: &str,
) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_cookie_domains.push(domain.to_string());
    session.allowed_storage_origins.push(origin.to_string());
    session.cookies.push(SessionCookie {
        name: cookie_name.to_string(),
        value: cookie_value.to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    session.origins.push(SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: storage_name.to_string(),
            value: storage_value.to_string(),
        }],
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}
