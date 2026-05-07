use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};

use assert_cmd::Command;

#[test]
fn get_json_success_writes_run_artifacts_with_empty_state() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (local_url, server) = local_server("/public-fetch");
    let fake_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
import argparse, json, pathlib
import urllib.request
parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
args = parser.parse_args()
urllib.request.urlopen(args.url, timeout=2).read()
state = json.loads(pathlib.Path(args.state).read_text(encoding='utf-8'))
assert state == {'cookies': [], 'origins': []}
content = '# Example\n\nFetched locally.'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
backend_metadata = {'backend': 'fake', 'state_path': args.state}
pathlib.Path(args.metadata).write_text(json.dumps(backend_metadata), encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url + '/final', 'content': content, 'warnings': ['fake warning']}))
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args(["--json", "get", &local_url])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["url"].as_str().unwrap(), local_url);
    assert_eq!(
        json["final_url"].as_str().unwrap(),
        format!("{local_url}/final")
    );
    assert_eq!(json["format"], "markdown");
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(json["content"], "# Example\n\nFetched locally.");
    assert_eq!(json["sessions"], serde_json::json!([]));
    assert_eq!(json["sensitive"], false);
    assert_eq!(json["warnings"], serde_json::json!(["fake warning"]));
    assert_eq!(json["limits"]["max_chars"], serde_json::Value::Null);
    assert_eq!(json["limits"]["max_tokens"], serde_json::Value::Null);
    assert_eq!(json["limits"]["truncated"], false);
    assert!(json["timing_ms"]["total"].as_u64().is_some());

    let markdown_path = PathBuf::from(json["artifacts"]["markdown"].as_str().unwrap());
    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    assert_eq!(
        fs::read_to_string(&markdown_path).unwrap(),
        "# Example\n\nFetched locally.\n"
    );
    assert!(metadata_path.starts_with(aget_home.join("runs")));

    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["ok"], true);
    assert_eq!(metadata["extractor"], "crawl4ai");
    assert_eq!(
        metadata["artifacts"]["markdown"],
        markdown_path.to_string_lossy().as_ref()
    );
    assert!(fs::read_dir(aget_home.join("tmp"))
        .unwrap()
        .next()
        .is_none());
    server.join().unwrap();
}

#[test]
fn get_out_writes_markdown_to_requested_path_and_metadata_to_run_dir() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let out_path = temp.path().join("requested.md");
    let fake_backend = write_success_backend(temp.path());

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            "https://example.com/out",
            "--out",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(
        json["artifacts"]["markdown"].as_str().unwrap(),
        out_path.to_string_lossy().as_ref()
    );
    assert_eq!(fs::read_to_string(&out_path).unwrap(), "# Fake\n");
    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    assert!(metadata_path.starts_with(aget_home.join("runs")));
}

#[test]
fn get_timeout_returns_stable_error_and_error_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
import time
time.sleep(3)
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--timeout",
            "1",
            "--json",
            "get",
            "https://example.com/slow",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "timeout");

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert_eq!(metadata["extractor"], "crawl4ai");
    assert_eq!(metadata["error"]["code"], "timeout");
}

#[test]
fn get_noisy_backend_output_does_not_deadlock() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
import argparse, json, pathlib, sys
parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
args = parser.parse_args()
sys.stderr.write('noise' * 20000)
sys.stderr.flush()
content = '# Noisy'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--timeout",
            "2",
            "--json",
            "get",
            "https://example.com/noisy",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(json["content"], "# Noisy");
}

#[test]
fn get_backend_stdout_logs_before_json_succeeds() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
import argparse, json, pathlib
parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
args = parser.parse_args()
content = '# Logged stdout'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print('[INIT] Starting browser')
print('[FETCH] Fetching ' + args.url)
print('[COMPLETE] Crawling done')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args(["--json", "get", "https://example.com/stdout-logs"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["extractor"], "crawl4ai");
    assert_eq!(json["content"], "# Logged stdout");
}

#[cfg(unix)]
#[test]
fn get_timeout_terminates_backend_descendants() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let marker = temp.path().join("descendant-survived.txt");
    let fake_backend = write_fake_backend(
        temp.path(),
        &format!(
            r#"#!/usr/bin/env python3
import subprocess, sys, time
subprocess.Popen([sys.executable, '-c', "import pathlib, time; time.sleep(2); pathlib.Path('{marker}').write_text('survived', encoding='utf-8')"])
time.sleep(5)
"#,
            marker = marker
                .to_string_lossy()
                .replace('\\', "\\\\")
                .replace('\'', "\\'")
        ),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--timeout",
            "1",
            "--json",
            "get",
            "https://example.com/slow",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "timeout");
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(!marker.exists());
}

#[test]
fn get_nonzero_and_malformed_backend_results_are_extraction_failed() {
    let temp = tempfile::tempdir().unwrap();
    let nonzero_home = temp.path().join("nonzero-home");
    let nonzero_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
import sys
print('backend exploded', file=sys.stderr)
raise SystemExit(2)
"#,
    );

    let mut nonzero = Command::cargo_bin("aget").unwrap();
    let nonzero_output = nonzero
        .env("AGET_HOME", &nonzero_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&nonzero_backend))
        .args(["--json", "get", "https://example.com/error"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let nonzero_json: serde_json::Value = serde_json::from_slice(&nonzero_output).unwrap();
    assert_eq!(nonzero_json["error"]["code"], "extraction_failed");

    let malformed_home = temp.path().join("malformed-home");
    let malformed_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
print('not json')
"#,
    );

    let mut malformed = Command::cargo_bin("aget").unwrap();
    let malformed_output = malformed
        .env("AGET_HOME", &malformed_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&malformed_backend))
        .args(["--json", "get", "https://example.com/malformed"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let malformed_json: serde_json::Value = serde_json::from_slice(&malformed_output).unwrap();
    assert_eq!(malformed_json["error"]["code"], "extraction_failed");
}

#[test]
fn missing_backend_returns_backend_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", "definitely_missing_aget_backend")
        .args(["--json", "get", "https://example.com/missing"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "backend_unavailable");

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["extractor"], "crawl4ai");
}

fn write_success_backend(dir: &Path) -> PathBuf {
    write_fake_backend(
        dir,
        r#"#!/usr/bin/env python3
import argparse, json, pathlib
parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
args = parser.parse_args()
content = '# Fake'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    )
}

fn local_server(path: &str) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let path = path.to_string();
    let url = format!("http://{addr}{path}");

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0; 1024];
            let bytes_read = stream.read(&mut buffer).unwrap_or_default();
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);
            assert!(request.starts_with(&format!("GET {path} HTTP/1.1")));
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
            let _ = stream.write_all(response.as_bytes());
        }
    });

    (url, handle)
}

fn write_fake_backend(dir: &Path, content: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = dir.join(format!("fake-backend-{nanos}.py"));
    fs::write(&path, content).unwrap();
    path
}

fn python_command(path: &Path) -> String {
    format!("python3 {}", shell_quote(&path.to_string_lossy()))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn metadata_files(aget_home: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(aget_home.join("runs")).unwrap() {
        let entry = entry.unwrap();
        let metadata = entry.path().join("metadata.json");
        if metadata.exists() {
            files.push(metadata);
        }
    }
    files
}
