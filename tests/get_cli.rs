use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use aget::{Session, SessionCookie, SessionStore};
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
args, _unknown = parser.parse_known_args()
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
fn get_session_uses_named_session_state_and_marks_sensitive() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    save_cookie_session(&aget_home, "local", "127.0.0.1", "sid", "secret-cookie");
    let fake_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
import argparse, json, pathlib
parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
args, _unknown = parser.parse_known_args()
state = json.loads(pathlib.Path(args.state).read_text(encoding='utf-8'))
assert state['origins'] == []
assert state['cookies'] == [{
    'name': 'sid',
    'value': 'secret-cookie',
    'domain': '127.0.0.1',
    'path': '/',
    'httpOnly': True,
    'secure': False,
    'sameSite': 'Lax',
}]
content = '# Session Fetch'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            "http://127.0.0.1/session",
            "--session",
            "local",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["sessions"], serde_json::json!(["local"]));
    assert_eq!(json["sensitive"], true);

    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["sessions"], serde_json::json!(["local"]));
    assert_eq!(metadata["sensitive"], true);
}

#[test]
fn get_session_marks_output_sensitive_even_if_session_metadata_is_false() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    save_cookie_session_with_sensitivity(
        &aget_home,
        "local",
        "127.0.0.1",
        "sid",
        "secret-cookie",
        false,
    );
    let fake_backend = write_success_backend(temp.path());

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            "http://127.0.0.1/session",
            "--session",
            "local",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["sessions"], serde_json::json!(["local"]));
    assert_eq!(json["sensitive"], true);
}

#[test]
fn get_forwards_supported_output_options_and_records_limits() {
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
parser.add_argument('--format', required=True)
parser.add_argument('--selector')
parser.add_argument('--exclude-selector')
parser.add_argument('--wait-for')
parser.add_argument('--extractor-option', action='append', default=[])
args, _unknown = parser.parse_known_args()
assert args.format == 'text'
assert args.selector == 'main.article'
assert args.exclude_selector == 'nav,.ad'
assert args.wait_for == 'css:.ready'
assert args.extractor_option == ['cache=bypass', 'magic=value']
assert '--only-main' not in _unknown
assert '--max-chars' not in _unknown
assert '--max-tokens' not in _unknown
content = 'aé💡bc'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url + '#done', 'content': content, 'warnings': []}))
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            "https://example.com/options",
            "--format",
            "text",
            "--selector",
            "main.article",
            "--exclude-selector",
            "nav,.ad",
            "--only-main",
            "--wait-for",
            "css:.ready",
            "--max-chars",
            "3",
            "--max-tokens",
            "17",
            "--extractor-option",
            "cache=bypass",
            "--extractor-option",
            "magic=value",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["format"], "text");
    assert_eq!(json["content"], "aé💡");
    assert_eq!(json["final_url"], "https://example.com/options#done");
    assert_eq!(json["limits"]["max_chars"], 3);
    assert_eq!(json["limits"]["max_tokens"], 17);
    assert_eq!(json["limits"]["truncated"], true);
    assert_eq!(json["limits"]["truncated_by"], "max_chars");
    assert_eq!(json["limits"]["content_chars_before_truncation"], 5);
    assert_eq!(json["limits"]["content_chars_after_truncation"], 3);
    assert_eq!(json["limits"]["max_tokens_enforced"], false);
    assert_eq!(json["output_options"]["format"], "text");
    assert_eq!(json["output_options"]["selector"], "main.article");
    assert_eq!(json["output_options"]["exclude_selector"], "nav,.ad");
    assert_eq!(json["output_options"]["only_main"], true);
    assert_eq!(json["output_options"]["wait_for"], "css:.ready");
    assert_eq!(
        json["output_options"]["extractor_options"],
        serde_json::json!({"cache": "bypass", "magic": "value"})
    );

    let markdown_path = PathBuf::from(json["artifacts"]["markdown"].as_str().unwrap());
    assert_eq!(fs::read_to_string(&markdown_path).unwrap(), "aé💡\n");
    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["content"], serde_json::Value::Null);
    assert_eq!(metadata["output_options"], json["output_options"]);
    assert_eq!(metadata["limits"], json["limits"]);
}

#[test]
fn get_max_chars_success_sanitizes_backend_stdout_artifact() {
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
args, _unknown = parser.parse_known_args()
content = 'SECRET-UNTRUNCATED-CONTENT'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            "https://example.com/secret",
            "--max-chars",
            "6",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["content"], "SECRET");
    assert_eq!(json["limits"]["truncated"], true);

    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    let backend_stdout_path = metadata_path.with_file_name("backend-stdout.json");
    let backend_stdout = fs::read_to_string(backend_stdout_path).unwrap();
    assert!(!backend_stdout.contains("SECRET-UNTRUNCATED-CONTENT"));
}

#[test]
fn get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let helper = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/crawl4ai_extract.py");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&helper))
        .args([
            "--json",
            "get",
            "https://example.com/unsupported-option",
            "--extractor-option",
            "js_code=alert(1)",
        ])
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
        .contains("unsupported extractor option 'js_code'"));

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert!(metadata["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unsupported extractor option 'js_code'"));
}

#[test]
fn get_real_helper_rejects_javascript_wait_before_crawl4ai_import() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let helper = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/crawl4ai_extract.py");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&helper))
        .args([
            "--json",
            "get",
            "https://example.com/js-wait",
            "--wait-for",
            "js:() => true",
        ])
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
        .contains("--wait-for only supports CSS selectors in v1"));

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert!(metadata["error"]["message"]
        .as_str()
        .unwrap()
        .contains("JavaScript wait conditions are not allowed"));
}

#[test]
fn get_json_format_truncates_only_content_not_response_envelope() {
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
parser.add_argument('--format', required=True)
args, _unknown = parser.parse_known_args()
assert args.format == 'json'
content = '{"title":"Example","body":"abcdef"}'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            "https://example.com/json",
            "--format",
            "json",
            "--max-chars",
            "12",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["format"], "json");
    assert_eq!(json["content"], "{\"title\":\"Ex");
    assert_eq!(json["limits"]["truncated"], true);
    assert_eq!(
        json["artifacts"]["metadata"].as_str().unwrap().is_empty(),
        false
    );
    assert!(json["timing_ms"]["total"].as_u64().is_some());
}

#[test]
#[ignore = "requires local Crawl4AI and Playwright browser setup"]
fn real_crawl4ai_replays_named_session_cookie() {
    let temp = tempfile::tempdir().unwrap();
    let empty_home = temp.path().join("empty-home");
    let (empty_url, empty_server, empty_cookie) = cookie_echo_server("/cookie-empty");

    let mut empty_cmd = Command::cargo_bin("aget").unwrap();
    let empty_output = empty_cmd
        .env("AGET_HOME", &empty_home)
        .args(["--json", "get", &empty_url, "--timeout", "60"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    empty_server.join().unwrap();

    let empty_json: serde_json::Value = serde_json::from_slice(&empty_output).unwrap();
    assert_eq!(empty_json["sessions"], serde_json::json!([]));
    assert_eq!(empty_json["sensitive"], false);
    assert!(empty_cookie
        .try_iter()
        .all(|cookie| !cookie.contains("sid=secret-cookie")));

    let session_home = temp.path().join("session-home");
    save_cookie_session(&session_home, "local", "127.0.0.1", "sid", "secret-cookie");
    let (session_url, session_server, session_cookie) = cookie_echo_server("/cookie-session");

    let mut session_cmd = Command::cargo_bin("aget").unwrap();
    let session_output = session_cmd
        .env("AGET_HOME", &session_home)
        .args([
            "--json",
            "get",
            &session_url,
            "--session",
            "local",
            "--timeout",
            "60",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    session_server.join().unwrap();

    let session_json: serde_json::Value = serde_json::from_slice(&session_output).unwrap();
    assert_eq!(session_json["sessions"], serde_json::json!(["local"]));
    assert_eq!(session_json["sensitive"], true);
    assert!(session_cookie
        .try_iter()
        .any(|cookie| cookie.contains("sid=secret-cookie")));
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
args, _unknown = parser.parse_known_args()
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
args, _unknown = parser.parse_known_args()
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
fn get_nonzero_backend_preserves_structured_failure_but_not_structured_success() {
    let temp = tempfile::tempdir().unwrap();
    let structured_failure_home = temp.path().join("structured-failure-home");
    let structured_failure_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, sys
print(json.dumps({'ok': False, 'error': 'structured backend failure'}))
raise SystemExit(2)
"#,
    );

    let mut structured_failure = Command::cargo_bin("aget").unwrap();
    let structured_failure_output = structured_failure
        .env("AGET_HOME", &structured_failure_home)
        .env(
            "AGET_CRAWL4AI_COMMAND",
            python_command(&structured_failure_backend),
        )
        .args(["--json", "get", "https://example.com/structured-failure"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let structured_failure_json: serde_json::Value =
        serde_json::from_slice(&structured_failure_output).unwrap();
    assert_eq!(
        structured_failure_json["error"]["code"],
        "extraction_failed"
    );
    assert_eq!(
        structured_failure_json["error"]["message"],
        "structured backend failure"
    );

    let structured_success_home = temp.path().join("structured-success-home");
    let structured_success_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
import json
print(json.dumps({'ok': True, 'final_url': 'https://example.com', 'content': '# Wrong', 'warnings': []}))
raise SystemExit(2)
"#,
    );

    let mut structured_success = Command::cargo_bin("aget").unwrap();
    let structured_success_output = structured_success
        .env("AGET_HOME", &structured_success_home)
        .env(
            "AGET_CRAWL4AI_COMMAND",
            python_command(&structured_success_backend),
        )
        .args(["--json", "get", "https://example.com/structured-success"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let structured_success_json: serde_json::Value =
        serde_json::from_slice(&structured_success_output).unwrap();
    assert_eq!(
        structured_success_json["error"]["code"],
        "extraction_failed"
    );
    assert_ne!(structured_success_json["content"], "# Wrong");
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
args, _unknown = parser.parse_known_args()
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

fn cookie_echo_server(path: &str) -> (String, JoinHandle<()>, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let path = path.to_string();
    let url = format!("http://{addr}{path}");
    let (cookie_sender, cookie_receiver) = mpsc::channel();

    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut handled_request = false;
        let mut idle_after_request = None;

        while Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buffer = [0; 4096];
                    let bytes_read = stream.read(&mut buffer).unwrap_or_default();
                    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                    handled_request = true;
                    idle_after_request = Some(Instant::now() + Duration::from_secs(2));
                    let cookie = request
                        .lines()
                        .find_map(|line| line.strip_prefix("Cookie: "))
                        .unwrap_or("none");
                    let _ = cookie_sender.send(cookie.to_string());
                    let filler = "Local cookie replay verification content. ".repeat(40);
                    let body = format!(
                        "<!doctype html><html><body><main><h1>Cookie Echo</h1><p>Path: {path}</p><p>Cookie: {cookie}</p><article>{filler}</article></main></body></html>"
                    );
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nSet-Cookie: server_set=ok; Path=/; SameSite=Lax\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if handled_request
                        && idle_after_request.is_some_and(|end| Instant::now() >= end)
                    {
                        break;
                    }
                    thread::sleep(Duration::from_millis(20));
                }
                Err(error) => panic!("cookie echo server failed: {error}"),
            }
        }

        assert!(handled_request);
    });

    (url, handle, cookie_receiver)
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

fn save_cookie_session(home: &Path, name: &str, domain: &str, cookie_name: &str, value: &str) {
    save_cookie_session_with_sensitivity(home, name, domain, cookie_name, value, true);
}

fn save_cookie_session_with_sensitivity(
    home: &Path,
    name: &str,
    domain: &str,
    cookie_name: &str,
    value: &str,
    sensitive: bool,
) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.sensitive = sensitive;
    session.allowed_cookie_domains.push(domain.to_string());
    session.cookies.push(SessionCookie {
        name: cookie_name.to_string(),
        value: value.to_string(),
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
