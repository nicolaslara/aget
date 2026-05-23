use assert_cmd::Command;
use serde_json::json;

use crate::support::get_cli::{mock_backend_command, save_cookie_session, success_data};

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
