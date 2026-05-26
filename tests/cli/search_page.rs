use std::path::{Path, PathBuf};

use aget::{Session, SessionCookie, SessionStore};
use assert_cmd::Command;

use super::support::local_server;

#[test]
fn search_page_ranks_heading_and_keyword_matches() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let run_id = create_markdown_artifact(
        &aget_home,
        r#"<main>
          <h1>Overview</h1><p>General project notes.</p>
          <h2>Install</h2><p>Run cargo install --git to install the CLI.</p>
          <h2>Auth Setup</h2><p>Import a session for private docs.</p>
        </main>"#,
    );

    let output = search_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "search-page",
            "--artifact",
            &run_id,
            "--query",
            "cargo install",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["summary"]["query"], "cargo install");
    assert_eq!(data["summary"]["matches"], 1);
    assert_eq!(data["matches"][0]["heading"], "Install");
    assert!(data["matches"][0]["snippet"]
        .as_str()
        .unwrap()
        .contains("cargo install"));
    assert!(data["matches"][0]["char_start"].as_u64().unwrap() > 0);
    assert!(path_from(&data["artifacts"]["json"]).exists());
    assert!(path_from(&data["artifacts"]["markdown"]).exists());
    assert!(path_from(&data["artifacts"]["metadata"]).exists());
}

#[test]
fn search_page_returns_empty_matches_for_no_match() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let run_id = create_markdown_artifact(
        &aget_home,
        "<main><h1>Overview</h1><p>No relevant content here.</p></main>",
    );

    let output = search_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "search-page",
            "--artifact",
            &run_id,
            "--query",
            "oauth",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["summary"]["matches"], 0);
    assert!(data["matches"].as_array().unwrap().is_empty());
}

#[test]
fn search_page_requires_private_content_consent_for_sensitive_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (source, server) =
        local_server("<main><h1>Private Install</h1><p>Secret setup notes.</p></main>");
    save_cookie_session(&aget_home, "docs", "127.0.0.1");
    let run_id = create_url_artifact(&aget_home, &source, &["--session", "docs"]);
    server.join().unwrap();

    let output = search_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "search-page",
            "--artifact",
            &run_id,
            "--query",
            "secret",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "search-page");
    assert_eq!(error["error"]["code"], "requires_user_action");

    let output = search_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "search-page",
            "--artifact",
            &run_id,
            "--query",
            "secret",
            "--allow-private-content",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let data = success_data(&output);
    assert_eq!(data["source"]["sensitive"], true);
    assert_eq!(data["matches"][0]["heading"], "Private Install");
}

fn create_markdown_artifact(aget_home: &Path, html: &str) -> String {
    create_url_artifact(aget_home, &format!("raw:{html}"), &[])
}

fn create_url_artifact(aget_home: &Path, url: &str, extra_args: &[&str]) -> String {
    let output = search_command(aget_home)
        .args(["--envelope", "json", "get", url])
        .args(extra_args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    run_id_from_get_output(&output)
}

fn save_cookie_session(home: &Path, name: &str, domain: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_cookie_domains.push(domain.to_string());
    session.cookies.push(SessionCookie {
        name: "sid".to_string(),
        value: "secret".to_string(),
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

fn search_command(aget_home: &Path) -> Command {
    let mut command = Command::cargo_bin("aget").unwrap();
    command.env("AGET_HOME", aget_home);
    command
}

fn run_id_from_get_output(output: &[u8]) -> String {
    let envelope: serde_json::Value = serde_json::from_slice(output).unwrap();
    let metadata = PathBuf::from(envelope["data"]["artifacts"]["metadata"].as_str().unwrap());
    metadata
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap()
        .to_string()
}

fn success_data(output: &[u8]) -> serde_json::Value {
    let envelope: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "search-page");
    envelope["data"].clone()
}

fn path_from(value: &serde_json::Value) -> PathBuf {
    PathBuf::from(value.as_str().unwrap())
}
