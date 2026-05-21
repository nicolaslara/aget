use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::json;

use crate::support::get_cli::{
    mock_backend_command, save_cookie_session, save_cookie_session_with_sensitivity,
    success_backend, success_data,
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
