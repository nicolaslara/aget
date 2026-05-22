use aget::session::SessionStore;
use aget::SessionCookie;
use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, named_session, nyt_state, session_origin};

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
