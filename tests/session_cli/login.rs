use std::fs;
use std::path::PathBuf;

use aget::session::SessionStore;
use aget::{
    complete_login_session, finish_login_session, start_login_session, LoginCompleteOptions,
    LoginFinishOptions, LoginStartOptions, SessionCookie, SessionSource,
};
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{
    agent_browser_tool, named_session, nyt_state, provider_only_state, session_origin,
    success_data, success_envelope,
};

#[test]
fn session_login_start_opens_aget_browser_profile_and_records_pending_flow() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let target_url = "https://www.nytimes.com/article";
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));
    let expected_profile = aget_home.join("tmp/agent-browser/aget-news");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "news",
            "--url",
            target_url,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = success_envelope(&output, "session.login.start");
    assert!(envelope["warnings"][0]
        .as_str()
        .unwrap()
        .contains("prefer signing in with your real browser"));
    let json = envelope["data"].clone();
    assert_eq!(json["state"], "login_started");
    assert!(json.get("site").is_none());
    assert_eq!(json["name"], "news");
    assert_eq!(
        PathBuf::from(json["profile"].as_str().unwrap()),
        expected_profile
    );
    assert_eq!(json["url"], target_url);
    assert_eq!(
        json["allowed_domains"],
        serde_json::json!(["nytimes.com", "www.nytimes.com"])
    );
    assert_eq!(
        json["next_command"],
        serde_json::json!(["aget", "session", "login", "finish", "news"])
    );
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(&format!(
        r#""--profile", "{}", "--session", "aget-login-news", "open""#,
        expected_profile.display()
    )));
    assert!(log.contains(target_url));
    assert!(aget_home.join("tmp/login-news.json").exists());
    assert!(expected_profile.parent().unwrap().exists());
}

#[test]
fn session_login_start_rejects_http_url_before_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "news",
            "--url",
            "http://www.nytimes.com/login",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "session.login.start");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("login URL must use https"));
    assert!(!log_path.exists() || fs::read_to_string(&log_path).unwrap().is_empty());
    assert!(!aget_home.join("tmp/login-news.json").exists());
}

#[test]
fn session_login_start_uses_exact_non_www_url_host_scope() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "docs",
            "--url",
            "https://docs.example.com/login",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.login.start");
    assert_eq!(json["name"], "docs");
    assert_eq!(
        json["allowed_domains"],
        serde_json::json!(["docs.example.com"])
    );
    assert!(aget_home.join("tmp/login-docs.json").exists());
}

#[test]
fn session_login_start_rejects_duplicate_pending_flow_without_overwriting() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

    let first_url = "https://www.nytimes.com/article";
    let second_url = "https://www.nytimes.com/login";

    let mut first = Command::cargo_bin("aget").unwrap();
    first
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            first_url,
            "--profile",
            "aget-hi",
        ])
        .assert()
        .success();

    let pending_path = aget_home.join("tmp/login-news.json");
    let original_pending = fs::read_to_string(&pending_path).unwrap();

    let mut second = Command::cargo_bin("aget").unwrap();
    let output = second
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "news",
            "--url",
            second_url,
            "--profile",
            "aget-hi-2",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("pending login flow named 'news' already exists"));
    assert_eq!(fs::read_to_string(&pending_path).unwrap(), original_pending);
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""--profile", "aget-hi", "--session", "aget-login-news", "open""#));
    assert!(!log.contains(r#"aget-hi-2"#));
    assert!(!log.contains(r#"["--session", "aget-login-news", "close"]"#));
}

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
fn finish_login_session_leaves_pending_until_complete_login_session_runs() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let tmp_dir = aget_home.join("tmp");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": nyt_state("nyt_session", "token")}),
    );
    unsafe {
        std::env::set_var("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser);
        std::env::set_var("AGET_FAKE_AGENT_BROWSER_LOG", &log_path);
    }

    let start_result = start_login_session(LoginStartOptions {
        name: "hi".to_string(),
        profile: None,
        url: "https://www.nytimes.com/article".to_string(),
        tmp_dir: tmp_dir.clone(),
    })
    .unwrap();
    assert!(tmp_dir.join("login-hi.json").exists());
    assert_eq!(
        PathBuf::from(&start_result.pending.profile),
        tmp_dir.join("agent-browser/aget-hi")
    );
    let profile_dir = PathBuf::from(&start_result.pending.profile);
    fs::create_dir_all(&profile_dir).unwrap();

    let finish_result = finish_login_session(LoginFinishOptions {
        name: "hi".to_string(),
        tmp_dir: tmp_dir.clone(),
    })
    .unwrap();
    assert_eq!(finish_result.pending, start_result.pending);
    assert_eq!(finish_result.session.cookies[0].expires, Some(1812619153));
    assert!(tmp_dir.join("login-hi.json").exists());

    complete_login_session(LoginCompleteOptions {
        pending: finish_result.pending,
        tmp_dir,
    })
    .unwrap();

    assert!(!aget_home.join("tmp/login-hi.json").exists());
    assert!(!profile_dir.exists());
}

#[test]
fn session_login_cancel_closes_only_pending_agent_browser_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

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

    let mut cancel = Command::cargo_bin("aget").unwrap();
    let output = cancel
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "cancel",
            "hellointerview",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.login.cancel");
    assert_eq!(json["state"], "login_cancelled");
    assert_eq!(json["name"], "hellointerview");
    assert_eq!(json["agent_session"], "aget-login-hellointerview");
    assert!(!aget_home.join("tmp/login-hellointerview.json").exists());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""open""#));
    assert!(log.contains(r#"["--session", "aget-login-hellointerview", "close"]"#));
}

#[test]
fn session_login_cancel_cleans_profile_and_pending_when_close_fails() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"behavior": "close_failure", "close_stderr": "close failed", "close_exit_code": 2}),
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
            "https://www.nytimes.com/login",
        ])
        .assert()
        .success();
    let profile = aget_home.join("tmp/agent-browser/aget-news");
    fs::create_dir_all(&profile).unwrap();

    let mut cancel = Command::cargo_bin("aget").unwrap();
    let output = cancel
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["--envelope", "json", "session", "login", "cancel", "news"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "session.login.cancel");
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(!aget_home.join("tmp/login-news.json").exists());
    assert!(!profile.exists());
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

#[test]
#[ignore = "requires local agent-browser, Crawl4AI setup, and manual authorized HelloInterview login"]
fn real_hellointerview_login_flow_fetches_paywalled_markdown() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let target = std::env::var("AGET_REAL_HELLOINTERVIEW_URL").unwrap_or_else(|_| {
        "https://www.hellointerview.com/learn/behavioral/course/select-choosing-responses-strategically".to_string()
    });

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "hellointerview",
            "--url",
            &target,
        ])
        .assert()
        .success();

    eprintln!(
        "Complete the HelloInterview/Google login in the opened browser, then press Enter here."
    );
    let mut confirmation = String::new();
    std::io::stdin().read_line(&mut confirmation).unwrap();

    let mut finish = Command::cargo_bin("aget").unwrap();
    finish
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "hellointerview",
        ])
        .assert()
        .success();

    let mut get = Command::cargo_bin("aget").unwrap();
    let output = get
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            &target,
            "--session",
            "hellointerview",
            "--content-format",
            "markdown",
            "--timeout",
            "90",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    let content = json["content"].as_str().unwrap();
    assert!(!content.contains("Purchase Premium to Keep Reading"));
    assert!(!content.contains("Premium users can view this video once signed in"));
    assert!(content.len() > 500);
}
