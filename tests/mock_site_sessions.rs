mod support;

use assert_cmd::Command;
use support::mock_site::MockSite;
use support::mock_site_cli::{
    mock_agent_browser_command, mock_backend_command, save_cookie_session, save_storage_session,
    success_data,
};

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
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "app",
            "--content-format",
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
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/storage-protected"),
            "--session",
            "storage",
            "--content-format",
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
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--content-format",
            "text",
        ])
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
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "expired",
            "--content-format",
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
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/logout"),
            "--session",
            "app",
            "--content-format",
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
            "--envelope",
            "json",
            "session",
            "import",
            "chrome",
            "--chrome-profile",
            "Default",
            "--name",
            "imported",
            "--allow-domain",
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
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "imported",
            "--content-format",
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
        .success();

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
    let finish_json = success_data(&finish, "session.login.finish");
    assert_eq!(finish_json["name"], "mock-app");

    let protected = Command::cargo_bin("aget")
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
            "mock-app",
            "--content-format",
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
