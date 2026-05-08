use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use aget::session::SessionStore;
use aget::{Session, SessionCookie, SessionSource};
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
