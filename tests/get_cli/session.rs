use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::json;

use crate::support::get_cli::{
    cookie_echo_server, metadata_files, mock_agent_browser, mock_backend_command,
    save_cookie_and_storage_session, save_cookie_session, save_cookie_session_with_sensitivity,
    success_backend, success_data, success_envelope,
};

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
            "--envelope",
            "json",
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
    assert!(!json.as_object().unwrap().contains_key("content"));

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
            "--envelope",
            "json",
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
            "--envelope",
            "json",
            "get",
            "http://127.0.0.1/multi",
            "--session",
            "provider",
            "--session",
            "app",
            "--inline-content",
            "always",
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
            "--envelope",
            "json",
            "http://127.0.0.1/multi-alias",
            "--session",
            "provider",
            "--session",
            "app",
            "--inline-content",
            "always",
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
            "--envelope",
            "json",
            "get",
            &local_url,
            "--session",
            "provider",
            "--session",
            "app",
            "--inline-content",
            "always",
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
            "--envelope",
            "json",
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
fn get_session_backend_failure_redacts_state_secrets_from_errors_metadata_and_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let missing_agent_browser = temp.path().join("missing-agent-browser");
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
        .env("AGET_AGENT_BROWSER_COMMAND", &missing_agent_browser)
        .args([
            "--envelope",
            "json",
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
            "--envelope",
            "json",
            "get",
            "http://127.0.0.1/fallback",
            "--session",
            "local",
            "--content-format",
            "markdown",
            "--inline-content",
            "always",
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
        .args([
            "--envelope",
            "json",
            "get",
            "http://127.0.0.1/public-failure",
        ])
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
            "--envelope",
            "json",
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
#[ignore = "requires local Crawl4AI and Playwright browser setup"]
fn real_crawl4ai_replays_named_session_cookie() {
    let temp = tempfile::tempdir().unwrap();
    let empty_home = temp.path().join("empty-home");
    let (empty_url, empty_server, empty_cookie) = cookie_echo_server("/cookie-empty");

    let mut empty_cmd = Command::cargo_bin("aget").unwrap();
    let empty_output = empty_cmd
        .env("AGET_HOME", &empty_home)
        .args(["--envelope", "json", "get", &empty_url, "--timeout", "60"])
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
            "--envelope",
            "json",
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
