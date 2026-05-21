mod support;

use std::fs;
use std::path::PathBuf;

use aget::{ErrorCode, OutputFormat};
use assert_cmd::Command;
use support::mock_site::{MockResponse, MockSite};
use support::mock_site_cli::{
    aget, mock_agent_browser_command, mock_backend_command, save_cookie_session,
    save_mixed_scope_session, success_data,
};

#[test]
fn documents_session_compose_replay_and_scope_rejection_contract() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    save_cookie_session(
        &aget_home,
        "provider",
        &site.host(),
        "provider_session",
        "valid-provider",
    );
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let compose = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "compose",
            "combined",
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
    let compose_json = success_data(&compose, "session.compose");
    assert_eq!(compose_json["cookie_count"], 2);

    let composed_fetch = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/requires-two"),
            "--session",
            "combined",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let composed_json = success_data(&composed_fetch, "get");
    assert!(composed_json["content"]
        .as_str()
        .unwrap()
        .contains("Composed Session"));
    assert!(site.received_cookie("/requires-two", "provider_session", "valid-provider"));
    assert!(site.received_cookie("/requires-two", "app_session", "valid-app"));

    save_mixed_scope_session(&aget_home, "mixed", &site.host());
    let requests_before_rejection = site.requests().len();
    let rejected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "mixed",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let rejected_json: serde_json::Value = serde_json::from_slice(&rejected).unwrap();
    assert_eq!(rejected_json["command"], "get");
    assert_eq!(rejected_json["error"]["code"], "privacy_policy_blocked");
    assert_eq!(site.requests().len(), requests_before_rejection);
}

#[test]
fn documents_public_get_json_contract_for_agents() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();

    let result = aget(&aget_home, &fake_backend)
        .get(site.url("/public"))
        .content_format(OutputFormat::Text)
        .selector("main")
        .exclude_selector("nav")
        .run()
        .unwrap();

    assert_eq!(result.url, site.url("/public"));
    assert_eq!(result.final_url, site.url("/public"));
    assert_eq!(result.content_format, "text");
    assert_eq!(result.extractor, "crawl4ai");
    assert_eq!(result.content, "Public Main Visible public article.");
    assert!(result.sessions.is_empty());
    assert!(!result.sensitive);
    assert!(result.warnings.is_empty());
    assert!(result.timing_ms.total > 0);
    assert_eq!(result.limits.max_chars, None);
    assert!(!result.limits.truncated);
    assert_eq!(result.output_options.content_format, OutputFormat::Text);
    assert_eq!(result.output_options.selector.as_deref(), Some("main"));
    assert_eq!(
        result.output_options.exclude_selector.as_deref(),
        Some("nav")
    );
    assert!(result.output_options.backend_options.is_empty());

    let content_path = PathBuf::from(&result.artifacts.content);
    let metadata_path = PathBuf::from(&result.artifacts.metadata);
    assert!(content_path.starts_with(aget_home.join("runs")));
    assert!(metadata_path.starts_with(aget_home.join("runs")));
    assert_eq!(
        fs::read_to_string(content_path).unwrap(),
        "Public Main Visible public article.\n"
    );

    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["url"], site.url("/public"));
    assert_eq!(metadata["content_format"], "text");
    assert_eq!(metadata["sensitive"], false);
}

#[test]
fn documents_output_limits_out_file_and_warning_contract() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let out_path = temp.path().join("agent-context.txt");

    let result = aget(&aget_home, &fake_backend)
        .get(site.url("/warning"))
        .content_format(OutputFormat::Text)
        .output(&out_path)
        .max_chars(12)
        .run()
        .unwrap();

    assert_eq!(result.warnings, vec!["mock warning"]);
    assert_eq!(result.content, "Warning Page");
    assert_eq!(result.artifacts.content, out_path.to_string_lossy());
    assert_eq!(fs::read_to_string(&out_path).unwrap(), "Warning Page\n");
    assert_eq!(result.limits.max_chars, Some(12));
    assert!(result.limits.truncated);
    assert_eq!(result.limits.truncated_by.as_deref(), Some("max_chars"));
    assert!(result.limits.content_chars_before_truncation > 12);
    assert_eq!(result.limits.content_chars_after_truncation, 12);
}

#[test]
fn documents_custom_site_routes_for_extraction_features() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command();
    let site = MockSite::builder()
        .route(
            "/guide",
            MockResponse::html(
                r#"
<html>
  <body>
    <main>
      <h1>Custom Guide</h1>
      <p>Feature-specific extraction fixture.</p>
      <aside>Remove this sidebar</aside>
    </main>
  </body>
</html>
"#,
            )
            .header("X-Fixture", "custom-guide"),
        )
        .route("/guide/latest", MockResponse::redirect("/guide"))
        .start();

    let result = aget(&aget_home, &fake_backend)
        .get(site.url("/guide/latest"))
        .content_format(OutputFormat::Text)
        .selector("main")
        .run()
        .unwrap();

    assert_eq!(result.final_url, site.url("/guide"));
    assert_eq!(
        result.content,
        "Custom Guide Feature-specific extraction fixture. Remove this sidebar"
    );
}

#[test]
fn documents_session_lifecycle_contract_for_agents() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let fake_agent_browser = mock_agent_browser_command();

    let start = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "mock-app",
            "--url",
            &site.https_url("/login"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_data = success_data(&start, "session.login.start");
    assert_eq!(start_data["state"], "login_started");
    assert_eq!(start_data["name"], "mock-app");
    assert_eq!(
        start_data["allowed_domains"],
        serde_json::json!([site.host()])
    );
    assert_eq!(
        start_data["next_command"],
        serde_json::json!(["aget", "session", "login", "finish", "mock-app"])
    );

    let finish = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "mock-app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let finish_data = success_data(&finish, "session.login.finish");
    assert_eq!(finish_data["state"], "login_finished");
    assert_eq!(finish_data["source"], "agent_browser");
    assert_eq!(finish_data["cookie_count"], 1);

    let list = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_data = success_data(&list, "session.list");
    assert_eq!(list_data["sessions"], serde_json::json!(["mock-app"]));

    let inspect = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "inspect", "mock-app"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect_data = success_data(&inspect, "session.inspect");
    assert_eq!(inspect_data["name"], "mock-app");
    assert_eq!(inspect_data["sensitive"], true);
    assert_eq!(inspect_data["cookies"][0]["name"], "app_session");
    assert_eq!(inspect_data["cookies"][0]["value"], "<redacted>");

    let fetch = aget(&aget_home, &fake_backend)
        .get(site.url("/protected"))
        .session("mock-app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(fetch.sessions, vec!["mock-app"]);
    assert!(fetch.sensitive);
    assert!(fetch.content.contains("Protected Account"));

    let delete = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "delete", "mock-app"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let delete_data = success_data(&delete, "session.delete");
    assert_eq!(delete_data["deleted"], true);

    let post_delete_list = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let post_delete_data = success_data(&post_delete_list, "session.list");
    assert_eq!(post_delete_data["sessions"], serde_json::json!([]));

    let post_delete_fetch = aget(&aget_home, &fake_backend)
        .get(site.url("/protected"))
        .session("mock-app")
        .run()
        .unwrap_err();
    assert_eq!(post_delete_fetch.code(), ErrorCode::IoError);
}
