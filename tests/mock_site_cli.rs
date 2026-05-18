mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::OnceLock;

use aget::{Aget, OutputFormat, Session, SessionCookie, SessionOrigin, SessionStore, StorageEntry};
use assert_cmd::Command;
use support::mock_site::{MockResponse, MockSite};

#[test]
fn mock_site_fetch_handles_redirect_output_shaping_and_waits() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            &site.url("/redirect"),
            "--format",
            "text",
            "--selector",
            "main",
            "--exclude-selector",
            "nav",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["final_url"], site.url("/public"));
    assert!(json["content"].as_str().unwrap().contains("Public Main"));
    assert!(!json["content"]
        .as_str()
        .unwrap()
        .contains("In-Article Navigation"));

    let delayed = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            &site.url("/delayed"),
            "--format",
            "text",
            "--wait-for",
            "#ready",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let delayed_json = success_data(&delayed, "get");
    assert!(delayed_json["content"]
        .as_str()
        .unwrap()
        .contains("Delayed Ready"));
}

#[test]
fn mock_site_replays_cookie_and_storage_sessions() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");
    save_storage_session(
        &aget_home,
        "storage",
        &site.origin(),
        "local_token",
        "storage-secret",
    );

    let protected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            &site.url("/protected"),
            "--session",
            "app",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let protected_json = success_data(&protected, "get");
    assert!(protected_json["content"]
        .as_str()
        .unwrap()
        .contains("Protected Account"));
    assert_eq!(protected_json["sensitive"], true);
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));

    let storage = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            &site.url("/storage-protected"),
            "--session",
            "storage",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let storage_json = success_data(&storage, "get");
    assert!(storage_json["content"]
        .as_str()
        .unwrap()
        .contains("Storage Protected"));
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}

#[test]
fn mock_site_covers_unauthenticated_expired_and_logout_states() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    save_cookie_session(
        &aget_home,
        "expired",
        &site.host(),
        "app_session",
        "expired",
    );
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let unauthenticated = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args(["--json", "get", &site.url("/protected"), "--format", "text"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let unauthenticated_json = success_data(&unauthenticated, "get");
    assert_eq!(
        unauthenticated_json["final_url"],
        site.url("/login?next=/protected")
    );
    assert!(unauthenticated_json["content"]
        .as_str()
        .unwrap()
        .contains("Login Form"));
    assert!(!site.received_cookie("/protected", "app_session", "valid-app"));

    let expired = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            &site.url("/protected"),
            "--session",
            "expired",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let expired_json = success_data(&expired, "get");
    assert!(expired_json["content"]
        .as_str()
        .unwrap()
        .contains("Session Expired"));

    let logout = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            &site.url("/logout"),
            "--session",
            "app",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let logout_json = success_data(&logout, "get");
    assert!(logout_json["content"]
        .as_str()
        .unwrap()
        .contains("Logged Out"));
}

#[test]
fn mock_site_imported_chrome_session_can_fetch_protected_page() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let fake_agent_browser = mock_agent_browser_command();

    let imported = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--json",
            "session",
            "import",
            "chrome",
            "--profile",
            "Default",
            "--name",
            "imported",
            "--domain",
            &site.host(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let import_json = success_data(&imported, "session.import.chrome");
    assert_eq!(import_json["name"], "imported");
    assert_eq!(import_json["cookie_count"], 1);

    let protected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            &site.url("/protected"),
            "--session",
            "imported",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let protected_json = success_data(&protected, "get");
    assert!(protected_json["content"]
        .as_str()
        .unwrap()
        .contains("Protected Account"));
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));
}

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
            "--json",
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
            "--json",
            "get",
            &site.url("/requires-two"),
            "--session",
            "combined",
            "--format",
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
            "--json",
            "get",
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
fn mock_site_login_bootstrap_can_fetch_protected_page_without_manual_action() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let fake_agent_browser = mock_agent_browser_command();

    Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--json",
            "session",
            "login",
            "start",
            "mock-app",
            "--url",
            &site.https_url("/login"),
        ])
        .assert()
        .success();

    let finish = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args(["--json", "session", "login", "finish", "mock-app"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let finish_json = success_data(&finish, "session.login.finish");
    assert_eq!(finish_json["name"], "mock-app");

    let protected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--json",
            "get",
            &site.url("/protected"),
            "--session",
            "mock-app",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let protected_json = success_data(&protected, "get");
    assert!(protected_json["content"]
        .as_str()
        .unwrap()
        .contains("Protected Account"));
}

#[test]
fn documents_public_get_json_contract_for_agents() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();

    let result = aget(&aget_home, &fake_backend)
        .get(site.url("/public"))
        .format(OutputFormat::Text)
        .selector("main")
        .exclude_selector("nav")
        .run()
        .unwrap();

    assert_eq!(result.url, site.url("/public"));
    assert_eq!(result.final_url, site.url("/public"));
    assert_eq!(result.format, "text");
    assert_eq!(result.extractor, "crawl4ai");
    assert_eq!(result.content, "Public Main Visible public article.");
    assert!(result.sessions.is_empty());
    assert!(!result.sensitive);
    assert!(result.warnings.is_empty());
    assert!(result.timing_ms.total > 0);
    assert_eq!(result.limits.max_chars, None);
    assert!(!result.limits.truncated);
    assert_eq!(result.output_options.format, OutputFormat::Text);
    assert_eq!(result.output_options.selector.as_deref(), Some("main"));
    assert_eq!(
        result.output_options.exclude_selector.as_deref(),
        Some("nav")
    );
    assert!(result.output_options.extractor_options.is_empty());

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
    assert_eq!(metadata["format"], "text");
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
        .format(OutputFormat::Text)
        .out(&out_path)
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
        .format(OutputFormat::Text)
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
            "--json",
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
        .args(["--json", "session", "login", "finish", "mock-app"])
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
        .args(["--json", "session", "list"])
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
        .args(["--json", "session", "inspect", "mock-app"])
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
        .format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(fetch.sessions, vec!["mock-app"]);
    assert!(fetch.sensitive);
    assert!(fetch.content.contains("Protected Account"));

    let delete = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--json", "session", "delete", "mock-app"])
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
        .args(["--json", "session", "list"])
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
    assert_eq!(post_delete_fetch.code(), aget::ErrorCode::IoError);
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn mock_backend_command() -> String {
    shell_quote(&mock_tool_path("aget-mock-backend").to_string_lossy())
}

fn mock_agent_browser_command() -> PathBuf {
    mock_tool_path("aget-mock-agent-browser")
}

fn mock_tool_path(name: &str) -> PathBuf {
    let tools = MOCK_TOOLS.get_or_init(build_mock_tools);
    let binary = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    tools.target_dir.join("debug").join(binary)
}

struct MockTools {
    target_dir: PathBuf,
}

static MOCK_TOOLS: OnceLock<MockTools> = OnceLock::new();

fn build_mock_tools() -> MockTools {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("tests/fixtures/mock-tools/Cargo.toml");
    let target_dir = root.join("target/aget-mock-tools");
    let status = StdCommand::new("cargo")
        .args([
            "build",
            "--quiet",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--target-dir",
            target_dir.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "failed to build mocked e2e helper tools");
    MockTools { target_dir }
}

fn aget(home: &Path, backend: &str) -> Aget {
    Aget::new(home).with_backend_command(backend.to_string())
}

fn success_data(output: &[u8], command: &str) -> serde_json::Value {
    let json: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], command);
    json["data"].clone()
}

fn save_cookie_session(home: &Path, name: &str, domain: &str, cookie_name: &str, value: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
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

fn save_storage_session(home: &Path, name: &str, origin: &str, key: &str, value: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_storage_origins.push(origin.to_string());
    session.origins.push(SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: key.to_string(),
            value: value.to_string(),
        }],
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}

fn save_mixed_scope_session(home: &Path, name: &str, domain: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_cookie_domains.push(domain.to_string());
    session.cookies.push(SessionCookie {
        name: "app_session".to_string(),
        value: "valid-app".to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    session.cookies.push(SessionCookie {
        name: "other_session".to_string(),
        value: "other-secret".to_string(),
        domain: "unrelated.example".to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}
