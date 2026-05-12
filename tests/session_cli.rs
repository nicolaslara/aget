use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use aget::session::SessionStore;
use aget::{
    complete_login_session, finish_login_session, start_login_session, LoginCompleteOptions,
    LoginFinishOptions, LoginStartOptions, Session, SessionCookie, SessionOrigin, SessionSource,
    StorageEntry,
};
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn session_list_inspect_and_delete_use_aget_home() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let session = demo_session();
    store.save(&session).unwrap();

    let mut list = Command::cargo_bin("aget").unwrap();
    list.env("AGET_HOME", &aget_home)
        .args(["session", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo"));

    let mut inspect = Command::cargo_bin("aget").unwrap();
    inspect
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted>"))
        .stdout(predicate::str::contains("secret-cookie").not());

    let mut inspect_secrets = Command::cargo_bin("aget").unwrap();
    inspect_secrets
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "demo", "--show-secrets"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret-cookie"));

    let mut delete = Command::cargo_bin("aget").unwrap();
    delete
        .env("AGET_HOME", &aget_home)
        .args(["session", "delete", "demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted session demo"));

    assert!(store.list().unwrap().is_empty());
}

#[test]
fn session_list_json_has_stable_shape() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    store.save(&demo_session()).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args(["--json", "session", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["sessions"], serde_json::json!(["demo"]));
}

#[test]
fn session_import_cmux_saves_filtered_cookies() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_cmux = write_fake_cmux(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, sys
args = sys.argv[1:]
if args[:4] != ['--json', 'browser', '--surface', 'surface:1'] or args[4:7] != ['cookies', 'get', '--domain']:
    print('unexpected args: ' + repr(args), file=sys.stderr)
    raise SystemExit(2)
domain = args[7]
cookies = [
    {'name': 'sid', 'value': 'allowed-secret', 'domain': domain, 'path': '/', 'secure': True, 'session_only': False, 'expires': 1910000000},
    {'name': 'wide', 'value': 'suffix-secret', 'domain': '.' + domain, 'path': '/', 'secure': False, 'session_only': True, 'expires': None},
    {'name': 'evil', 'value': 'blocked-secret', 'domain': domain + '.evil', 'path': '/', 'secure': False, 'session_only': True, 'expires': None},
]
print(json.dumps({'cookies': cookies}))
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CMUX_COMMAND", &fake_cmux)
        .args([
            "--json",
            "session",
            "import",
            "cmux",
            "--surface",
            "surface:1",
            "--name",
            "imported",
            "--domain",
            "example.com",
            "--domain",
            "docs.example.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["name"], "imported");
    assert_eq!(json["source"], "cmux");
    assert_eq!(json["cookie_count"], 4);

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("imported").unwrap();
    assert_eq!(
        session.source,
        SessionSource::Cmux {
            surface: "surface:1".to_string()
        }
    );
    assert!(session.sensitive);
    assert_eq!(
        session.allowed_cookie_domains,
        vec!["example.com".to_string(), "docs.example.com".to_string()]
    );
    assert!(session.allowed_storage_origins.is_empty());
    assert!(session.origins.is_empty());
    assert_eq!(session.cookies.len(), 4);
    assert!(session
        .cookies
        .iter()
        .all(|cookie| cookie.source_session.as_deref() == Some("imported")));
    assert!(session.cookies.iter().all(|cookie| cookie.http_only));
    assert!(session
        .cookies
        .iter()
        .all(|cookie| cookie.same_site.is_none()));
    assert!(session
        .cookies
        .iter()
        .any(|cookie| cookie.name == "sid" && cookie.domain == "example.com"));
    assert!(session
        .cookies
        .iter()
        .any(|cookie| cookie.name == "wide" && cookie.domain == ".example.com"));
    assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));

    let mut inspect = Command::cargo_bin("aget").unwrap();
    inspect
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "imported"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted>"))
        .stdout(predicate::str::contains("allowed-secret").not())
        .stdout(predicate::str::contains("suffix-secret").not())
        .stdout(predicate::str::contains("blocked-secret").not());
}

#[test]
fn session_import_cmux_missing_backend_returns_backend_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CMUX_COMMAND", "definitely_missing_aget_cmux")
        .args([
            "--json",
            "session",
            "import",
            "cmux",
            "--surface",
            "surface:1",
            "--name",
            "imported",
            "--domain",
            "example.com",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "backend_unavailable");
}

#[test]
fn session_import_chrome_saves_filtered_state_and_cleans_raw_file() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if len(args) == 6 and args[:1] == ['--profile'] and args[2:3] == ['--session'] and args[4:] == ['open', 'about:blank']:
    raise SystemExit(0)
if len(args) == 5 and args[:1] == ['--session'] and args[2:4] == ['state', 'save']:
    state_path = pathlib.Path(args[4])
    with log.open('a', encoding='utf-8') as handle:
        handle.write('STATE_PATH=' + str(state_path) + '\n')
    state = {
        'cookies': [
            {'name': 'sid', 'value': 'allowed-secret', 'domain': 'example.com', 'path': '/', 'expires': 1812619153.69691, 'httpOnly': True, 'secure': True, 'sameSite': 'Lax'},
            {'name': 'sub', 'value': 'sub-secret', 'domain': 'docs.example.com', 'path': '/', 'httpOnly': False, 'secure': False},
            {'name': 'evil', 'value': 'blocked-secret', 'domain': 'example.com.evil', 'path': '/', 'httpOnly': False, 'secure': False},
        ],
        'origins': [
            {'origin': 'https://example.com', 'localStorage': [{'name': 'token', 'value': 'allowed-storage'}], 'sessionStorage': [{'name': 'ignored', 'value': 'session-only'}]},
            {'origin': 'https://docs.example.com:443', 'localStorage': [{'name': 'subtoken', 'value': 'sub-storage'}]},
            {'origin': 'https://example.com.evil', 'localStorage': [{'name': 'evil', 'value': 'blocked-storage'}]},
        ],
    }
    state_path.write_text(json.dumps(state), encoding='utf-8')
    raise SystemExit(0)
if len(args) == 3 and args[:1] == ['--session'] and args[2:] == ['close']:
    raise SystemExit(0)
print('unexpected args: ' + repr(args), file=sys.stderr)
raise SystemExit(2)
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--json",
            "session",
            "import",
            "chrome",
            "--profile",
            "Default",
            "--name",
            "chrome-imported",
            "--domain",
            "example.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["source"], "chrome");
    assert_eq!(json["name"], "chrome-imported");
    assert_eq!(json["cookie_count"], 2);
    assert_eq!(json["origin_count"], 2);

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("chrome-imported").unwrap();
    assert_eq!(
        session.source,
        SessionSource::ChromeProfile {
            profile: "Default".to_string()
        }
    );
    assert!(session.sensitive);
    assert_eq!(
        session.allowed_cookie_domains,
        vec!["example.com".to_string()]
    );
    assert_eq!(
        session.allowed_storage_origins,
        vec![
            "https://docs.example.com:443".to_string(),
            "https://example.com".to_string(),
        ]
    );
    assert_eq!(session.cookies.len(), 2);
    assert!(session.cookies.iter().all(|cookie| cookie
        .source_session
        .as_deref()
        .is_some_and(|source| source.starts_with("aget-import-"))));
    assert!(session.cookies.iter().any(|cookie| cookie.name == "sid"));
    assert!(session.cookies.iter().any(|cookie| cookie.name == "sub"));
    assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));
    assert_eq!(
        session
            .cookies
            .iter()
            .find(|cookie| cookie.name == "sid")
            .unwrap()
            .expires,
        Some(1812619153)
    );
    assert_eq!(session.origins.len(), 2);
    assert!(session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://example.com"
            && origin
                .local_storage
                .iter()
                .any(|entry| entry.name == "token")));
    assert!(!session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://example.com.evil"));

    let mut inspect = Command::cargo_bin("aget").unwrap();
    let inspect_output = inspect
        .env("AGET_HOME", &aget_home)
        .args(["--json", "session", "inspect", "chrome-imported"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect_json: serde_json::Value = serde_json::from_slice(&inspect_output).unwrap();
    assert_eq!(
        inspect_json["origins"][0]["local_storage"][0]["value"],
        "<redacted>"
    );
    assert!(!String::from_utf8_lossy(&inspect_output).contains("allowed-storage"));

    let mut inspect_secrets = Command::cargo_bin("aget").unwrap();
    inspect_secrets
        .env("AGET_HOME", &aget_home)
        .args([
            "--json",
            "session",
            "inspect",
            "chrome-imported",
            "--show-secrets",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("allowed-storage"));

    let log = fs::read_to_string(&log_path).unwrap();
    let calls = log
        .lines()
        .filter(|line| line.starts_with('['))
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 3);
    assert!(calls[0].contains(r#""--profile", "Default", "--session", "#));
    assert!(calls[1].contains(r#""state", "save""#));
    assert!(calls[2].contains(r#""close""#));
    let raw_state_path = log
        .lines()
        .find_map(|line| line.strip_prefix("STATE_PATH="))
        .map(PathBuf::from)
        .unwrap();
    assert!(!raw_state_path.exists());
}

#[test]
fn session_import_chrome_missing_backend_returns_backend_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env(
            "AGET_AGENT_BROWSER_COMMAND",
            "definitely_missing_aget_agent_browser",
        )
        .args([
            "--json",
            "session",
            "import",
            "chrome",
            "--profile",
            "Default",
            "--name",
            "imported",
            "--domain",
            "example.com",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "backend_unavailable");
}

#[test]
fn session_import_chrome_requires_user_action_for_profile_lock() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG']).write_text(json.dumps(args), encoding='utf-8')
print('Please quit Chrome before importing this profile', file=sys.stderr)
raise SystemExit(2)
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--json",
            "session",
            "import",
            "chrome",
            "--profile",
            "Default",
            "--name",
            "imported",
            "--domain",
            "example.com",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "requires_user_action");
}

#[test]
fn session_import_chrome_closes_and_cleans_raw_state_on_malformed_state() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if args[-2:] == ['open', 'about:blank']:
    raise SystemExit(0)
if args[2:4] == ['state', 'save']:
    with log.open('a', encoding='utf-8') as handle:
        handle.write('STATE_PATH=' + args[4] + '\n')
    pathlib.Path(args[4]).write_text('{bad json', encoding='utf-8')
    raise SystemExit(0)
if args[-1:] == ['close']:
    raise SystemExit(0)
raise SystemExit(2)
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--json",
            "session",
            "import",
            "chrome",
            "--profile",
            "Default",
            "--name",
            "imported",
            "--domain",
            "example.com",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "extraction_failed");
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""open", "about:blank""#));
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
fn session_login_start_opens_aget_owned_browser_and_records_pending_flow() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let target_url = "https://www.nytimes.com/article";
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if len(args) == 6 and args[:1] == ['--profile'] and args[2:3] == ['--session'] and args[4] == 'open':
    raise SystemExit(0)
print('unexpected args: ' + repr(args), file=sys.stderr)
raise SystemExit(2)
"#,
    );
    let expected_profile = aget_home.join("tmp/agent-browser/aget-news");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--json", "session", "login", "start", "news", "--url", target_url,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
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
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
raise SystemExit(0)
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--json",
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
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if args[-2:-1] == ['open']:
    raise SystemExit(0)
raise SystemExit(2)
"#,
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--json",
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

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
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
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if len(args) == 6 and args[:1] == ['--profile'] and args[2:3] == ['--session'] and args[4] == 'open':
    raise SystemExit(0)
if len(args) == 3 and args[:1] == ['--session'] and args[2:] == ['close']:
    raise SystemExit(0)
print('unexpected args: ' + repr(args), file=sys.stderr)
raise SystemExit(2)
"#,
    );

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
            "--json",
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
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if len(args) == 6 and args[:1] == ['--profile'] and args[2:3] == ['--session'] and args[4] == 'open':
    raise SystemExit(0)
if len(args) == 5 and args[:1] == ['--session'] and args[2:4] == ['state', 'save']:
    state_path = pathlib.Path(args[4])
    with log.open('a', encoding='utf-8') as handle:
        handle.write('STATE_PATH=' + str(state_path) + '\n')
    state = {
        'cookies': [
            {'name': 'nyt_session', 'value': 'nyt-secret', 'domain': '.nytimes.com', 'path': '/', 'expires': 1812619153.69691, 'httpOnly': True, 'secure': True, 'sameSite': 'Lax'},
            {'name': 'provider', 'value': 'google-secret', 'domain': 'accounts.google.com', 'path': '/', 'httpOnly': True, 'secure': True},
        ],
        'origins': [
            {'origin': 'https://www.nytimes.com', 'localStorage': [{'name': 'token', 'value': 'nyt-storage'}]},
            {'origin': 'https://accounts.google.com', 'localStorage': [{'name': 'provider', 'value': 'google-storage'}]},
        ],
    }
    state_path.write_text(json.dumps(state), encoding='utf-8')
    raise SystemExit(0)
if len(args) == 3 and args[:1] == ['--session'] and args[2:] == ['close']:
    raise SystemExit(0)
print('unexpected args: ' + repr(args), file=sys.stderr)
raise SystemExit(2)
"#,
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
        .args(["--json", "session", "login", "finish", "news"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
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

    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if args[-2:-1] == ['open']:
    raise SystemExit(0)
if args[2:4] == ['state', 'save']:
    state_path = pathlib.Path(args[4])
    state = {
        'cookies': [
            {'name': 'nyt_new', 'value': 'new-nyt-secret', 'domain': '.nytimes.com', 'path': '/', 'httpOnly': True, 'secure': True},
            {'name': 'provider', 'value': 'google-secret', 'domain': 'accounts.google.com', 'path': '/', 'httpOnly': True, 'secure': True},
        ],
        'origins': [
            {'origin': 'https://www.nytimes.com', 'localStorage': [{'name': 'new-token', 'value': 'new-storage'}]},
            {'origin': 'https://accounts.google.com', 'localStorage': [{'name': 'provider', 'value': 'google-storage'}]},
        ],
    }
    state_path.write_text(json.dumps(state), encoding='utf-8')
    raise SystemExit(0)
if args[-1:] == ['close']:
    raise SystemExit(0)
raise SystemExit(2)
"#,
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
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if len(args) == 6 and args[:1] == ['--profile'] and args[2:3] == ['--session'] and args[4] == 'open':
    raise SystemExit(0)
if len(args) == 5 and args[:1] == ['--session'] and args[2:4] == ['state', 'save']:
    state_path = pathlib.Path(args[4])
    state = {
        'cookies': [
            {'name': 'nyt_session', 'value': 'nyt-secret', 'domain': '.nytimes.com', 'path': '/', 'expires': 1812619153.69691, 'httpOnly': True, 'secure': True, 'sameSite': 'Lax'},
        ],
        'origins': [
            {'origin': 'https://www.nytimes.com', 'localStorage': [{'name': 'token', 'value': 'nyt-storage'}]},
        ],
    }
    state_path.write_text(json.dumps(state), encoding='utf-8')
    raise SystemExit(0)
if len(args) == 3 and args[:1] == ['--session'] and args[2:] == ['close']:
    print('close failed', file=sys.stderr)
    raise SystemExit(1)
print('unexpected args: ' + repr(args), file=sys.stderr)
raise SystemExit(2)
"#,
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
        .args(["--json", "session", "login", "finish", "news"])
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
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if len(args) == 6 and args[:1] == ['--profile'] and args[2:3] == ['--session'] and args[4] == 'open':
    raise SystemExit(0)
if len(args) == 5 and args[:1] == ['--session'] and args[2:4] == ['state', 'save']:
    state_path = pathlib.Path(args[4])
    state = {
        'cookies': [
            {'name': 'nyt_session', 'value': 'nyt-secret', 'domain': '.nytimes.com', 'path': '/', 'expires': 1812619153.69691, 'httpOnly': True, 'secure': True, 'sameSite': 'Lax'},
        ],
        'origins': [
            {'origin': 'https://www.nytimes.com', 'localStorage': [{'name': 'token', 'value': 'nyt-storage'}]},
        ],
    }
    state_path.write_text(json.dumps(state), encoding='utf-8')
    raise SystemExit(0)
if len(args) == 3 and args[:1] == ['--session'] and args[2:] == ['close']:
    raise SystemExit(0)
print('unexpected args: ' + repr(args), file=sys.stderr)
raise SystemExit(2)
"#,
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
}

#[test]
fn session_login_cancel_closes_only_pending_agent_browser_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
with pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG']).open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if args[-2:-1] == ['open'] or args[-1:] == ['close']:
    raise SystemExit(0)
raise SystemExit(2)
"#,
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

    let mut cancel = Command::cargo_bin("aget").unwrap();
    let output = cancel
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["--json", "session", "login", "cancel", "hellointerview"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["state"], "login_cancelled");
    assert_eq!(json["name"], "hellointerview");
    assert_eq!(json["agent_session"], "aget-login-hellointerview");
    assert!(!aget_home.join("tmp/login-hellointerview.json").exists());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""open""#));
    assert!(log.contains(r#"["--session", "aget-login-hellointerview", "close"]"#));
}

#[test]
fn session_login_finish_rejects_provider_only_state_without_saving_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = write_fake_agent_browser(
        temp.path(),
        r#"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
log = pathlib.Path(os.environ['AGET_FAKE_AGENT_BROWSER_LOG'])
with log.open('a', encoding='utf-8') as handle:
    handle.write(json.dumps(args) + '\n')
if args[-2:-1] == ['open']:
    raise SystemExit(0)
if args[2:4] == ['state', 'save']:
    state_path = pathlib.Path(args[4])
    state_path.write_text(json.dumps({'cookies': [{'name': 'provider', 'value': 'google-secret', 'domain': 'accounts.google.com', 'path': '/', 'httpOnly': True, 'secure': True}], 'origins': []}), encoding='utf-8')
    raise SystemExit(0)
if args[-1:] == ['close']:
    raise SystemExit(0)
raise SystemExit(2)
"#,
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
        .args(["--json", "session", "login", "finish", "hellointerview"])
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
            "--json",
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
        .args(["--json", "session", "login", "finish", "hellointerview"])
        .assert()
        .success();

    let mut get = Command::cargo_bin("aget").unwrap();
    let output = get
        .env("AGET_HOME", &aget_home)
        .args([
            "--json",
            "get",
            &target,
            "--session",
            "hellointerview",
            "--format",
            "markdown",
            "--timeout",
            "90",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    let content = json["content"].as_str().unwrap();
    assert!(!content.contains("Purchase Premium to Keep Reading"));
    assert!(!content.contains("Premium users can view this video once signed in"));
    assert!(content.len() > 500);
}

#[test]
fn session_compose_persists_composed_session_with_provenance_and_preserves_sources() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let mut provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    provider.allowed_storage_origins = vec!["https://accounts.example.com".to_string()];
    provider.origins.push(session_origin(
        "https://accounts.example.com",
        "provider-token",
        "provider-storage-secret",
    ));
    let app = named_session("app", "appsid", "app-secret", "app.example.com");
    let original_provider = provider.clone();
    let original_app = app.clone();
    store.save(&provider).unwrap();
    store.save(&app).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--json",
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["name"], "combined");
    assert_eq!(
        json["source_sessions"],
        serde_json::json!(["provider", "app"])
    );
    assert_eq!(json["cookie_count"], 2);
    assert_eq!(json["origin_count"], 1);

    let combined = store.load("combined").unwrap();
    assert_eq!(
        combined.source,
        SessionSource::Composed {
            sessions: vec!["provider".to_string(), "app".to_string()]
        }
    );
    assert_eq!(
        combined.allowed_cookie_domains,
        vec![
            "accounts.example.com".to_string(),
            "app.example.com".to_string()
        ]
    );
    assert!(combined.cookies.iter().any(|cookie| {
        cookie.name == "oauth" && cookie.source_session.as_deref() == Some("provider")
    }));
    assert!(combined.cookies.iter().any(|cookie| {
        cookie.name == "appsid" && cookie.source_session.as_deref() == Some("app")
    }));
    assert_eq!(
        combined.origins[0].source_session.as_deref(),
        Some("provider")
    );
    assert_eq!(store.load("provider").unwrap(), original_provider);
    assert_eq!(store.load("app").unwrap(), original_app);
}

#[test]
fn session_compose_rejects_source_name_target_without_mutating_sources() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    let app = named_session("app", "appsid", "app-secret", "app.example.com");
    store.save(&provider).unwrap();
    store.save(&app).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--json",
            "session",
            "compose",
            "provider",
            "--session",
            "provider",
            "--session",
            "app",
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
        .contains("must not match a source session"));
    assert_eq!(store.load("provider").unwrap(), provider);
    assert_eq!(store.load("app").unwrap(), app);
}

#[test]
fn session_compose_rejects_existing_target_without_mutating_existing_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    let app = named_session("app", "appsid", "app-secret", "app.example.com");
    let existing = named_session("combined", "existing", "existing-secret", "old.example.com");
    store.save(&provider).unwrap();
    store.save(&app).unwrap();
    store.save(&existing).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--json",
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
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
        .contains("already exists"));
    assert_eq!(store.load("combined").unwrap(), existing);
    assert_eq!(store.load("provider").unwrap(), provider);
    assert_eq!(store.load("app").unwrap(), app);
}

#[test]
fn session_compose_inspect_reports_cookie_source_session_and_redacts_values() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    store
        .save(&named_session(
            "provider",
            "oauth",
            "provider-secret",
            "accounts.example.com",
        ))
        .unwrap();
    store
        .save(&named_session(
            "app",
            "appsid",
            "app-secret",
            "app.example.com",
        ))
        .unwrap();

    let mut compose = Command::cargo_bin("aget").unwrap();
    compose
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success();

    let mut inspect = Command::cargo_bin("aget").unwrap();
    let inspect_output = inspect
        .env("AGET_HOME", &aget_home)
        .args(["--json", "session", "inspect", "combined"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect_json: serde_json::Value = serde_json::from_slice(&inspect_output).unwrap();
    assert_eq!(inspect_json["cookies"][0]["value"], "<redacted>");
    assert!(inspect_json["cookies"]
        .as_array()
        .unwrap()
        .iter()
        .any(|cookie| { cookie["name"] == "oauth" && cookie["source_session"] == "provider" }));
    assert!(!String::from_utf8_lossy(&inspect_output).contains("provider-secret"));
    assert!(!String::from_utf8_lossy(&inspect_output).contains("app-secret"));
}

#[test]
fn session_inspect_plain_reports_cookie_and_origin_provenance_with_redacted_values() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let mut provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    provider.allowed_storage_origins = vec!["https://accounts.example.com".to_string()];
    provider.origins.push(session_origin(
        "https://accounts.example.com",
        "provider-token",
        "provider-storage-secret",
    ));
    store.save(&provider).unwrap();
    store
        .save(&named_session(
            "app",
            "appsid",
            "app-secret",
            "app.example.com",
        ))
        .unwrap();

    let mut compose = Command::cargo_bin("aget").unwrap();
    compose
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success();

    let mut inspect = Command::cargo_bin("aget").unwrap();
    inspect
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "combined"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "accounts.example.com oauth=<redacted> source=provider",
        ))
        .stdout(predicate::str::contains(
            "https://accounts.example.com source=provider",
        ))
        .stdout(predicate::str::contains(
            "localStorage provider-token=<redacted>",
        ))
        .stdout(predicate::str::contains("provider-secret").not())
        .stdout(predicate::str::contains("provider-storage-secret").not());
}

#[test]
fn session_compose_rejects_conflicts_with_redacted_error_and_does_not_save() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    store
        .save(&named_session(
            "first",
            "sid",
            "first-secret",
            "example.com",
        ))
        .unwrap();
    store
        .save(&named_session(
            "second",
            "sid",
            "second-secret",
            "example.com",
        ))
        .unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--json",
            "session",
            "compose",
            "combined",
            "--session",
            "first",
            "--session",
            "second",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "session_conflict");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("conflicting cookie 'sid'"));
    assert!(!String::from_utf8_lossy(&output).contains("first-secret"));
    assert!(!String::from_utf8_lossy(&output).contains("second-secret"));
    assert!(store.load("combined").is_err());
}

#[test]
#[ignore = "requires a running cmux browser surface named by AGET_REAL_CMUX_SURFACE"]
fn real_cmux_imports_loopback_cookie() {
    let surface = match std::env::var("AGET_REAL_CMUX_SURFACE") {
        Ok(surface) => surface,
        Err(_) => {
            eprintln!("set AGET_REAL_CMUX_SURFACE to a disposable cmux browser surface");
            return;
        }
    };
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (url, server) = loopback_cookie_server();

    let goto = StdCommand::new("cmux")
        .args(["browser", "--surface", &surface, "goto", &url])
        .status()
        .expect("cmux must be installed and runnable");
    assert!(goto.success(), "cmux goto failed with {goto}");
    server.join().unwrap();

    let mut import = Command::cargo_bin("aget").unwrap();
    import
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "import",
            "cmux",
            "--surface",
            &surface,
            "--name",
            "cmux-loopback",
            "--domain",
            "127.0.0.1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Imported cmux session cmux-loopback",
        ));

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("cmux-loopback").unwrap();
    assert!(session.cookies.iter().any(|cookie| {
        cookie.name == "aget_cmux_e2e"
            && cookie.value == "loopback-secret"
            && cookie.domain == "127.0.0.1"
    }));

    let (echo_url, echo_server) = loopback_cookie_echo_server();
    let replay_backend = write_fake_backend(
        temp.path(),
        r#"#!/usr/bin/env python3
import argparse, json, pathlib, sys, urllib.request
parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
args = parser.parse_args()
state = json.loads(pathlib.Path(args.state).read_text(encoding='utf-8'))
cookies = state.get('cookies', [])
cookie_header = '; '.join(
    f"{cookie['name']}={cookie['value']}"
    for cookie in cookies
    if cookie.get('domain') == '127.0.0.1'
)
expected = 'aget_cmux_e2e=loopback-secret'
if expected not in cookie_header:
    print('imported cookie missing from generated Playwright state', file=sys.stderr)
    raise SystemExit(2)
request = urllib.request.Request(args.url, headers={'Cookie': cookie_header})
body = urllib.request.urlopen(request, timeout=5).read().decode('utf-8')
if expected not in body:
    print('imported cookie was not replayed to loopback echo server', file=sys.stderr)
    raise SystemExit(3)
content = '# cmux replay ok\n\n' + body
pathlib.Path(args.output).write_text(content, encoding='utf-8')
pathlib.Path(args.metadata).write_text(json.dumps({'backend': 'fake-cmux-replay'}), encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    );

    let mut replay = Command::cargo_bin("aget").unwrap();
    replay
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&replay_backend))
        .args(["get", &echo_url, "--session", "cmux-loopback"])
        .assert()
        .success()
        .stdout(predicate::str::contains("aget_cmux_e2e=loopback-secret"));
    echo_server.join().unwrap();
}

#[test]
#[ignore = "requires a running cmux browser surface and local Crawl4AI setup"]
fn real_cmux_import_replays_loopback_cookie_through_crawl4ai() {
    let surface = match std::env::var("AGET_REAL_CMUX_SURFACE") {
        Ok(surface) => surface,
        Err(_) => {
            eprintln!("set AGET_REAL_CMUX_SURFACE to a disposable cmux browser surface");
            return;
        }
    };
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (url, server) = loopback_cookie_server();

    let goto = StdCommand::new("cmux")
        .args(["browser", "--surface", &surface, "goto", &url])
        .status()
        .expect("cmux must be installed and runnable");
    assert!(goto.success(), "cmux goto failed with {goto}");
    server.join().unwrap();

    let mut import = Command::cargo_bin("aget").unwrap();
    import
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "import",
            "cmux",
            "--surface",
            &surface,
            "--name",
            "cmux-loopback",
            "--domain",
            "127.0.0.1",
        ])
        .assert()
        .success();

    let (echo_url, echo_server, cookies) = crawl4ai_cookie_echo_server();
    let mut replay = Command::cargo_bin("aget").unwrap();
    replay
        .env("AGET_HOME", &aget_home)
        .args([
            "--json",
            "get",
            &echo_url,
            "--session",
            "cmux-loopback",
            "--timeout",
            "60",
        ])
        .assert()
        .success();
    echo_server.join().unwrap();

    assert!(cookies
        .try_iter()
        .any(|cookie| cookie.contains("aget_cmux_e2e=loopback-secret")));
}

fn demo_session() -> Session {
    let mut session = Session::new("demo");
    session.allowed_cookie_domains = vec!["example.com".to_string()];
    session.cookies.push(SessionCookie {
        name: "session".to_string(),
        value: "secret-cookie".to_string(),
        domain: "example.com".to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
        source_session: None,
    });
    session
}

fn named_session(name: &str, cookie_name: &str, value: &str, domain: &str) -> Session {
    let mut session = Session::new(name);
    session.allowed_cookie_domains = vec![domain.to_string()];
    session.cookies.push(SessionCookie {
        name: cookie_name.to_string(),
        value: value.to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
        source_session: None,
    });
    session
}

fn session_origin(origin: &str, name: &str, value: &str) -> SessionOrigin {
    SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: name.to_string(),
            value: value.to_string(),
        }],
        source_session: None,
    }
}

fn loopback_cookie_server() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{addr}/cmux-cookie");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).unwrap_or_default();
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        assert!(request.starts_with("GET /cmux-cookie HTTP/1.1"));
        let body = "cmux loopback cookie set";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nSet-Cookie: aget_cmux_e2e=loopback-secret; Path=/; SameSite=Lax\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    (url, handle)
}

fn loopback_cookie_echo_server() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{addr}/cmux-cookie-echo");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0; 4096];
        let bytes_read = stream.read(&mut buffer).unwrap_or_default();
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        assert!(request.starts_with("GET /cmux-cookie-echo HTTP/1.1"));
        let cookie = request
            .lines()
            .find_map(|line| line.strip_prefix("Cookie: "))
            .unwrap_or("none");
        let body = format!("Cookie: {cookie}");
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    (url, handle)
}

fn crawl4ai_cookie_echo_server() -> (String, thread::JoinHandle<()>, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{addr}/cmux-cookie-crawl4ai-echo");
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
                    let filler = "Local cmux import replay verification content. ".repeat(40);
                    let body = format!(
                        "<!doctype html><html><body><main><h1>cmux Cookie Echo</h1><p>Cookie: {cookie}</p><article>{filler}</article></main></body></html>"
                    );
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if handled_request
                        && matches!(idle_after_request, Some(end) if Instant::now() >= end)
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

fn write_fake_cmux(dir: &Path, content: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = dir.join(format!("fake-cmux-{nanos}.py"));
    fs::write(&path, content).unwrap();
    make_executable(&path);
    path
}

fn write_fake_agent_browser(dir: &Path, content: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = dir.join(format!("fake-agent-browser-{nanos}.py"));
    fs::write(&path, content).unwrap();
    make_executable(&path);
    path
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

fn write_fake_backend(dir: &Path, content: &str) -> PathBuf {
    write_fake_cmux(dir, content)
}

fn python_command(path: &Path) -> String {
    format!("python3 {}", shell_quote(&path.to_string_lossy()))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
