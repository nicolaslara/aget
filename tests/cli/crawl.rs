use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use assert_cmd::Command;

#[test]
fn crawl_fetches_bounded_same_path_pages_and_writes_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let output_dir = temp.path().join("crawl-output");
    let (base_url, server) = crawl_site();
    let start = format!("{base_url}/docs/index.html");

    let output = crawl_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "crawl",
            &start,
            "--limit",
            "3",
            "--max-depth",
            "2",
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_eq!(data["summary"]["limit"], 3);
    assert_eq!(data["summary"]["fetched"], 3);
    assert_eq!(data["summary"]["succeeded"], 3);
    let urls = item_urls(&data);
    assert!(urls.iter().any(|url| url.ends_with("/docs/index.html")));
    assert!(urls.iter().any(|url| url.ends_with("/docs/page-a.html")));
    assert!(urls.iter().any(|url| url.ends_with("/docs/page-b.html")));
    assert!(!urls.iter().any(|url| url.ends_with("/outside.html")));
    assert!(path_from(&data["artifacts"]["manifest"]).exists());
    assert!(path_from(&data["artifacts"]["markdown"]).exists());
    assert!(path_from(&data["items"][0]["artifacts"]["content"]).exists());
    server.join().unwrap();
}

#[test]
fn crawl_requires_limit_and_validates_bounds() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let output = crawl_command(&aget_home)
        .args(["--envelope", "json", "crawl", "https://example.com/docs/"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "crawl");
    assert_eq!(error["error"]["code"], "usage_error");

    let output = crawl_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "crawl",
            "https://example.com/docs/",
            "--limit",
            "101",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "crawl");
    assert_eq!(error["error"]["code"], "usage_error");
}

#[test]
fn crawl_records_partial_failures() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let output_dir = temp.path().join("crawl-output");
    let missing = format!("file://{}", temp.path().join("missing.html").display());

    let output = crawl_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "crawl",
            &missing,
            "--limit",
            "1",
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["command"], "crawl");
    let data = envelope["data"].clone();
    assert_eq!(data["summary"]["failed"], 1);
    assert_eq!(data["items"][0]["status"], "failed");
    assert!(path_from(&data["artifacts"]["manifest"]).exists());
}

fn crawl_command(aget_home: &Path) -> Command {
    let mut command = Command::cargo_bin("aget").unwrap();
    command.env("AGET_HOME", aget_home);
    command
}

fn success_data(output: &[u8]) -> serde_json::Value {
    let envelope: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "crawl");
    envelope["data"].clone()
}

fn item_urls(data: &serde_json::Value) -> Vec<String> {
    data["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["url"].as_str().unwrap().to_string())
        .collect()
}

fn path_from(value: &serde_json::Value) -> PathBuf {
    PathBuf::from(value.as_str().unwrap())
}

fn crawl_site() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let mut pages = BTreeMap::new();
    pages.insert(
        "/docs/index.html".to_string(),
        r#"<main>
          <a href="/docs/page-a.html">A</a>
          <a href="/docs/page-b.html">B</a>
          <a href="/outside.html">Outside</a>
        </main>"#
            .to_string(),
    );
    pages.insert(
        "/docs/page-a.html".to_string(),
        r#"<main><h1>A</h1><a href="/docs/page-b.html">B</a></main>"#.to_string(),
    );
    pages.insert(
        "/docs/page-b.html".to_string(),
        r#"<main><h1>B</h1></main>"#.to_string(),
    );
    let handle = thread::spawn(move || {
        for _ in 0..6 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let mut request = Vec::new();
            let mut buffer = [0; 512];
            while request.len() < 16 * 1024 {
                match stream.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => {
                        request.extend_from_slice(&buffer[..read]);
                        if request.windows(4).any(|window| window == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            let request = String::from_utf8_lossy(&request);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");
            let body = pages
                .get(path)
                .cloned()
                .unwrap_or_else(|| "<main>missing</main>".to_string());
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
            let _ = stream.flush();
            let _ = stream.shutdown(Shutdown::Both);
        }
    });
    (base_url, handle)
}
