use std::fs;
use std::path::PathBuf;

use aget::session::SessionStore;
use aget::SessionSource;
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, nyt_state, success_data};

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
