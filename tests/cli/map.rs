use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

use super::support::local_server;

#[test]
fn map_url_extracts_dedupes_and_filters_links() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let html = r#"
        <main>
          <a href="/docs/intro">Intro</a>
          <a href="/docs/intro#section">Intro duplicate</a>
          <a href="/docs/reference.html">Reference</a>
          <a href="/docs/manual.pdf">Manual</a>
          <a href="/account">Account</a>
          <a href="https://other.example/docs">External</a>
        </main>
    "#;
    let (source, server) = local_server(html);

    let output = map_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "map",
            &source,
            "--any-path",
            "--content-type",
            "text/html",
            "--exclude",
            "*/account",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["summary"]["emitted"], 2);
    let urls = link_urls(&data);
    assert!(urls.iter().any(|url| url.ends_with("/docs/intro")));
    assert!(urls.iter().any(|url| url.ends_with("/docs/reference.html")));
    assert!(!urls.iter().any(|url| url.contains("manual.pdf")));
    assert!(!urls.iter().any(|url| url.contains("other.example")));
    assert!(path_from(&data["artifacts"]["json"]).exists());
    assert!(path_from(&data["artifacts"]["markdown"]).exists());
    server.join().unwrap();
}

#[test]
fn map_artifact_reads_internal_html_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (source, server) =
        local_server(r#"<main><a href="/docs/a">A</a><a href="/docs/b">B</a></main>"#);
    let run_id = create_html_artifact(&aget_home, &source);
    server.join().unwrap();

    let output = map_command(&aget_home)
        .args(["--envelope", "json", "map", "--artifact", &run_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["source"]["artifact"], run_id);
    assert_eq!(data["summary"]["emitted"], 2);
}

#[test]
fn map_rejects_external_artifact_content() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let external = temp.path().join("external.html");
    fs::write(&external, "<a href='/docs'>Docs</a>").unwrap();

    let run_id =
        create_html_artifact_with_output(&aget_home, "<a href='/docs'>Docs</a>", &external);
    let output = map_command(&aget_home)
        .args(["--envelope", "json", "map", "--artifact", &run_id])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "map");
    assert_eq!(error["error"]["code"], "usage_error");
}

#[test]
fn map_rejects_conflicting_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let output = map_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "map",
            "raw:<a href='/docs'>Docs</a>",
            "--artifact",
            "run-123",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "map");
    assert_eq!(error["error"]["code"], "usage_error");
}

fn map_command(aget_home: &Path) -> Command {
    let mut command = Command::cargo_bin("aget").unwrap();
    command.env("AGET_HOME", aget_home);
    command
}

fn create_html_artifact(aget_home: &Path, source: &str) -> String {
    let mut command = map_command(aget_home);
    let output = command
        .args([
            "--envelope",
            "json",
            "get",
            source,
            "--content-format",
            "html",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    run_id_from_get_output(&output)
}

fn create_html_artifact_with_output(aget_home: &Path, html: &str, output_path: &Path) -> String {
    let mut command = map_command(aget_home);
    let output = command
        .args([
            "--envelope",
            "json",
            "get",
            &raw_url(html),
            "--content-format",
            "html",
            "--output",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    run_id_from_get_output(&output)
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
    assert_eq!(envelope["command"], "map");
    envelope["data"].clone()
}

fn link_urls(data: &serde_json::Value) -> Vec<String> {
    data["links"]
        .as_array()
        .unwrap()
        .iter()
        .map(|link| link["url"].as_str().unwrap().to_string())
        .collect()
}

fn path_from(value: &serde_json::Value) -> PathBuf {
    PathBuf::from(value.as_str().unwrap())
}

fn raw_url(html: &str) -> String {
    format!("raw:{html}")
}
