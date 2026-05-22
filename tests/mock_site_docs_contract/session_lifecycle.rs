use aget::{ErrorCode, OutputFormat};
use assert_cmd::Command;

use crate::support::mock_site::MockSite;
use crate::support::mock_site_cli::{
    aget, mock_agent_browser_command, mock_backend_command, success_data,
};

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
