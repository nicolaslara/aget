use std::path::{Path, PathBuf};

use super::support::local_server;
use aget::{Session, SessionCookie, SessionStore};
use assert_cmd::Command;

#[test]
fn extract_reads_tables_links_and_headings_from_page_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let run_id = create_url_artifact(
        &aget_home,
        r#"raw:<main>
          <h1>Pricing</h1>
          <a href="/docs/install">Install</a>
          <table><tr><th>Plan</th><th>Price</th></tr><tr><td>Pro</td><td>$19</td></tr></table>
        </main>"#,
        &["--content-format", "html"],
    );

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--artifact",
            &run_id,
            "--field",
            "headings",
            "--field",
            "links",
            "--field",
            "tables",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["summary"]["sources"], 1);
    assert_eq!(
        data["records"][0]["values"]["headings"][0]["text"],
        "Pricing"
    );
    assert_eq!(data["records"][0]["values"]["links"][0]["text"], "Install");
    assert_eq!(
        data["records"][0]["values"]["tables"][0]["rows"][0]["Plan"],
        "Pro"
    );
    assert_eq!(
        data["records"][0]["values"]["tables"][0]["rows"][0]["Price"],
        "$19"
    );
    assert!(path_from(&data["artifacts"]["json"]).exists());
    assert!(path_from(&data["artifacts"]["markdown"]).exists());
    assert!(path_from(&data["artifacts"]["metadata"]).exists());
}

#[test]
fn extract_uses_schema_selectors_for_html_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let schema_path = temp.path().join("schema.json");
    std::fs::write(
        &schema_path,
        r#"{
          "fields": [
            {"name": "price", "type": "selector", "selector": ".price", "attribute": "text", "multiple": false},
            {"name": "plans", "type": "selector", "selector": "a.plan", "attribute": "href", "multiple": true}
          ]
        }"#,
    )
    .unwrap();
    let run_id = create_url_artifact(
        &aget_home,
        r#"raw:<main><div class="price">$19</div><a class="plan" href="/pro">Pro</a></main>"#,
        &["--content-format", "html"],
    );

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--artifact",
            &run_id,
            "--schema",
            schema_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["records"][0]["values"]["price"], "$19");
    assert_eq!(data["records"][0]["values"]["plans"][0], "/pro");
    assert_eq!(data["records"][0]["provenance"][0]["selector"], ".price");
}

#[test]
fn extract_reads_successful_items_from_batch_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let output_dir = temp.path().join("batch-output");
    let batch_output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "batch",
            "raw:<main><h1>One</h1></main>",
            "raw:<main><h1>Two</h1></main>",
            "--content-format",
            "html",
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let manifest = batch_success_data(&batch_output)["artifacts"]["manifest"]
        .as_str()
        .unwrap()
        .to_string();

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--manifest",
            &manifest,
            "--field",
            "headings",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["summary"]["sources"], 2);
    let headings = data["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["values"]["headings"][0]["text"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(headings.contains(&"One"));
    assert!(headings.contains(&"Two"));
}

#[test]
fn extract_reads_successful_items_from_crawl_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let output_dir = temp.path().join("crawl-output");
    let docs_dir = temp.path().join("docs");
    std::fs::create_dir_all(&docs_dir).unwrap();
    let index = docs_dir.join("index.html");
    let page = docs_dir.join("page.html");
    std::fs::write(
        &index,
        r#"<main><h1>Index</h1><a href="page.html">Page</a></main>"#,
    )
    .unwrap();
    std::fs::write(&page, r#"<main><h1>Page</h1></main>"#).unwrap();
    let start = url::Url::from_file_path(&index).unwrap().to_string();

    let crawl_output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "crawl",
            &start,
            "--limit",
            "2",
            "--any-path",
            "--content-format",
            "html",
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let manifest = crawl_success_data(&crawl_output)["artifacts"]["manifest"]
        .as_str()
        .unwrap()
        .to_string();

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--manifest",
            &manifest,
            "--field",
            "headings",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["summary"]["sources"], 2);
    let headings = data["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["values"]["headings"][0]["text"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(headings.contains(&"Index"));
    assert!(headings.contains(&"Page"));
}

#[test]
fn extract_supports_json_and_metadata_schema_paths() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let source_path = temp.path().join("data.json");
    let schema_path = temp.path().join("schema.json");
    std::fs::write(&source_path, r#"{"plan":"pro","nested":{"seats":5}}"#).unwrap();
    std::fs::write(
        &schema_path,
        r#"{
          "fields": [
            {"name": "plan", "type": "json", "path": "plan"},
            {"name": "format", "type": "metadata", "path": "content_format"}
          ]
        }"#,
    )
    .unwrap();
    let source = url::Url::from_file_path(&source_path).unwrap().to_string();
    let run_id = create_url_artifact(&aget_home, &source, &["--content-format", "text"]);

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--artifact",
            &run_id,
            "--schema",
            schema_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["records"][0]["values"]["plan"], "pro");
    assert_eq!(data["records"][0]["values"]["format"], "text");
}

#[test]
fn extract_reports_malformed_schema_and_input_errors() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let run_id = create_url_artifact(
        &aget_home,
        "raw:<main><h1>Schema</h1></main>",
        &["--content-format", "html"],
    );
    let bad_schema = temp.path().join("bad-schema.json");
    std::fs::write(&bad_schema, r#"{"fields": [{"name": "missing-type"}]}"#).unwrap();

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--artifact",
            &run_id,
            "--schema",
            bad_schema.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "extract");
    assert_eq!(error["error"]["code"], "usage_error");

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--artifact",
            &run_id,
            "--manifest",
            "/tmp/manifest.json",
            "--field",
            "links",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "extract");
    assert_eq!(error["error"]["code"], "usage_error");
}

#[test]
fn extract_rejects_external_artifact_and_mismatched_manifest_content_paths() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let external_output = temp.path().join("caller-output.html");
    let run_id = create_url_artifact(
        &aget_home,
        "raw:<main><h1>External</h1></main>",
        &[
            "--content-format",
            "html",
            "--output",
            external_output.to_str().unwrap(),
        ],
    );

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--artifact",
            &run_id,
            "--field",
            "headings",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "extract");
    assert_eq!(error["error"]["code"], "usage_error");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("external caller-owned"));

    let output_dir = temp.path().join("batch-output");
    let batch_output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "batch",
            "raw:<main><h1>One</h1></main>",
            "--content-format",
            "html",
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let manifest = PathBuf::from(
        batch_success_data(&batch_output)["artifacts"]["manifest"]
            .as_str()
            .unwrap(),
    );
    let secret = temp.path().join("secret.txt");
    std::fs::write(&secret, "# Secret\n").unwrap();
    let mut manifest_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest).unwrap()).unwrap();
    manifest_json["items"][0]["artifacts"]["content"] =
        serde_json::Value::String(secret.to_string_lossy().into_owned());
    std::fs::write(
        &manifest,
        serde_json::to_vec_pretty(&manifest_json).unwrap(),
    )
    .unwrap();

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--manifest",
            manifest.to_str().unwrap(),
            "--field",
            "headings",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "extract");
    assert_eq!(error["error"]["code"], "usage_error");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("does not match item metadata"));
}

#[test]
fn extract_requires_private_content_consent_for_sensitive_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (source, server) = local_server("<main><h1>Private Pricing</h1><p>Secret tier.</p></main>");
    save_cookie_session(&aget_home, "docs", "127.0.0.1");
    let run_id = create_url_artifact(
        &aget_home,
        &source,
        &["--session", "docs", "--content-format", "html"],
    );
    server.join().unwrap();

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--artifact",
            &run_id,
            "--field",
            "headings",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "extract");
    assert_eq!(error["error"]["code"], "requires_user_action");

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--artifact",
            &run_id,
            "--field",
            "headings",
            "--allow-private-content",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let data = success_data(&output);
    assert_eq!(data["summary"]["sensitive_sources"], 1);
    assert_eq!(
        data["records"][0]["values"]["headings"][0]["text"],
        "Private Pricing"
    );
}

#[test]
fn extract_requires_private_content_consent_for_sensitive_manifest_items() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let output_dir = temp.path().join("batch-output");
    let (source, server) = local_server("<main><h1>Private Batch</h1></main>");
    save_cookie_session(&aget_home, "docs", "127.0.0.1");
    let batch_output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "batch",
            &source,
            "--session",
            "docs",
            "--content-format",
            "html",
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    server.join().unwrap();
    let manifest = batch_success_data(&batch_output)["artifacts"]["manifest"]
        .as_str()
        .unwrap()
        .to_string();

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--manifest",
            &manifest,
            "--field",
            "headings",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "extract");
    assert_eq!(error["error"]["code"], "requires_user_action");

    let output = extract_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "extract",
            "--manifest",
            &manifest,
            "--field",
            "headings",
            "--allow-private-content",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let data = success_data(&output);
    assert_eq!(data["summary"]["sensitive_sources"], 1);
    assert_eq!(
        data["records"][0]["values"]["headings"][0]["text"],
        "Private Batch"
    );
}

fn create_url_artifact(aget_home: &Path, url: &str, extra_args: &[&str]) -> String {
    let output = extract_command(aget_home)
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

fn extract_command(aget_home: &Path) -> Command {
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

fn batch_success_data(output: &[u8]) -> serde_json::Value {
    let envelope: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "batch");
    envelope["data"].clone()
}

fn crawl_success_data(output: &[u8]) -> serde_json::Value {
    let envelope: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "crawl");
    envelope["data"].clone()
}

fn success_data(output: &[u8]) -> serde_json::Value {
    let envelope: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "extract");
    envelope["data"].clone()
}

fn path_from(value: &serde_json::Value) -> PathBuf {
    PathBuf::from(value.as_str().unwrap())
}
