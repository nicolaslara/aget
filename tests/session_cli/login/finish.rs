use std::fs;
use std::path::PathBuf;

use aget::session::SessionStore;
use aget::{SessionCookie, SessionSource};
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{
    agent_browser_tool, named_session, nyt_state, provider_only_state, session_origin, success_data,
};

#[test]
fn session_login_finish_saves_only_url_scoped_state_and_cleans_temp_files() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": nyt_state("nyt_session", "token")}),
    );

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            "https://www.nytimes.com/article",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    let output = finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["--envelope", "json", "session", "login", "finish", "news"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.login.finish");
    assert_eq!(json["state"], "login_finished");
    assert_eq!(json["name"], "news");
    assert_eq!(json["cookie_count"], 1);
    assert_eq!(json["origin_count"], 1);

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("news").unwrap();
    assert_eq!(
        session.source,
        SessionSource::AgentBrowser {
            session: "aget-login-news".to_string()
        }
    );
    assert_eq!(
        session.allowed_cookie_domains,
        vec!["nytimes.com".to_string(), "www.nytimes.com".to_string()]
    );
    assert_eq!(session.cookies.len(), 1);
    assert_eq!(session.cookies[0].name, "nyt_session");
    assert_eq!(session.origins.len(), 1);
    assert_eq!(session.origins[0].origin, "https://www.nytimes.com");
    assert!(!session
        .cookies
        .iter()
        .any(|cookie| cookie.domain.contains("google")));
    assert!(!session
        .origins
        .iter()
        .any(|origin| origin.origin.contains("google")));
    assert!(!aget_home.join("tmp/login-news.json").exists());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""state", "save""#));
    assert!(log.contains(r#""close""#));
    let raw_state_path = log
        .lines()
        .find_map(|line| line.strip_prefix("STATE_PATH="))
        .map(PathBuf::from)
        .unwrap();
    assert!(!raw_state_path.exists());
}

#[test]
fn session_login_finish_merges_new_scope_into_existing_bucket() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let store = SessionStore::new(&aget_home).unwrap();
    let mut existing = named_session("news", "nyt_old", "old-nyt-secret", ".nytimes.com");
    existing.allowed_cookie_domains.push("ft.com".to_string());
    existing.cookies.push(SessionCookie {
        name: "ft_session".to_string(),
        value: "ft-secret".to_string(),
        domain: "ft.com".to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
        source_session: None,
    });
    existing.allowed_storage_origins = vec![
        "https://www.nytimes.com".to_string(),
        "https://www.ft.com".to_string(),
    ];
    existing.origins.push(session_origin(
        "https://www.nytimes.com",
        "old-token",
        "old-storage",
    ));
    existing.origins.push(session_origin(
        "https://www.ft.com",
        "ft-token",
        "ft-storage",
    ));
    store.save(&existing).unwrap();

    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": nyt_state("nyt_new", "new-token")}),
    );

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            "https://www.nytimes.com/article",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["session", "login", "finish", "news"])
        .assert()
        .success();

    let merged = store.load("news").unwrap();
    assert!(merged.cookies.iter().any(|cookie| cookie.name == "nyt_new"));
    assert!(merged
        .cookies
        .iter()
        .any(|cookie| cookie.name == "ft_session"));
    assert!(!merged.cookies.iter().any(|cookie| cookie.name == "nyt_old"));
    assert!(merged.origins.iter().any(|origin| {
        origin.origin == "https://www.nytimes.com"
            && origin
                .local_storage
                .iter()
                .any(|entry| entry.name == "new-token")
    }));
    assert!(merged
        .origins
        .iter()
        .any(|origin| origin.origin == "https://www.ft.com"));
    assert!(!merged.origins.iter().any(|origin| {
        origin.origin == "https://www.nytimes.com"
            && origin
                .local_storage
                .iter()
                .any(|entry| entry.name == "old-token")
    }));
    assert_eq!(
        merged.allowed_cookie_domains,
        vec![
            "ft.com".to_string(),
            "nytimes.com".to_string(),
            "www.nytimes.com".to_string()
        ]
    );
}

#[test]
fn session_login_finish_reports_close_failure_and_keeps_pending_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({
            "behavior": "close_failure",
            "state": nyt_state("nyt_session", "token"),
            "close_stderr": "close failed"
        }),
    );

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            "https://www.nytimes.com/article",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    let output = finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["--envelope", "json", "session", "login", "finish", "news"])
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
        .contains("close failed"));
    assert!(aget_home.join("tmp/login-news.json").exists());
    assert!(SessionStore::new(&aget_home).unwrap().load("news").is_err());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#"["--session", "aget-login-news", "close"]"#));
}

#[test]
fn session_login_finish_rejects_provider_only_state_without_saving_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": provider_only_state()}),
    );
    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "hellointerview",
            "--url",
            "https://www.hellointerview.com/login",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    let output = finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "hellointerview",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "requires_user_action");
    assert!(SessionStore::new(&aget_home)
        .unwrap()
        .load("hellointerview")
        .is_err());
    assert!(aget_home.join("tmp/login-hellointerview.json").exists());
}
