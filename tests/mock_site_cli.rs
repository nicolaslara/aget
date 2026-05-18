mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use aget::{Session, SessionCookie, SessionOrigin, SessionStore, StorageEntry};
use assert_cmd::Command;
use support::mock_site::MockSite;

#[test]
fn mock_site_fetch_handles_redirect_output_shaping_and_waits() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = write_mock_site_backend(temp.path());

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            &site.url("/redirect"),
            "--format",
            "text",
            "--selector",
            "main",
            "--exclude-selector",
            "nav",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["final_url"], site.url("/public"));
    assert!(json["content"].as_str().unwrap().contains("Public Main"));
    assert!(!json["content"]
        .as_str()
        .unwrap()
        .contains("In-Article Navigation"));

    let delayed = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            &site.url("/delayed"),
            "--format",
            "text",
            "--wait-for",
            "#ready",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let delayed_json = success_data(&delayed, "get");
    assert!(delayed_json["content"]
        .as_str()
        .unwrap()
        .contains("Delayed Ready"));
}

#[test]
fn mock_site_replays_cookie_and_storage_sessions() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = write_mock_site_backend(temp.path());
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");
    save_storage_session(
        &aget_home,
        "storage",
        &site.origin(),
        "local_token",
        "storage-secret",
    );

    let protected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            &site.url("/protected"),
            "--session",
            "app",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let protected_json = success_data(&protected, "get");
    assert!(protected_json["content"]
        .as_str()
        .unwrap()
        .contains("Protected Account"));
    assert_eq!(protected_json["sensitive"], true);
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));

    let storage = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            &site.url("/storage-protected"),
            "--session",
            "storage",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let storage_json = success_data(&storage, "get");
    assert!(storage_json["content"]
        .as_str()
        .unwrap()
        .contains("Storage Protected"));
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}

#[test]
fn mock_site_covers_unauthenticated_expired_and_logout_states() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = write_mock_site_backend(temp.path());
    save_cookie_session(
        &aget_home,
        "expired",
        &site.host(),
        "app_session",
        "expired",
    );
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let unauthenticated = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args(["--json", "get", &site.url("/protected"), "--format", "text"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let unauthenticated_json = success_data(&unauthenticated, "get");
    assert_eq!(
        unauthenticated_json["final_url"],
        site.url("/login?next=/protected")
    );
    assert!(unauthenticated_json["content"]
        .as_str()
        .unwrap()
        .contains("Login Form"));
    assert!(!site.received_cookie("/protected", "app_session", "valid-app"));

    let expired = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            &site.url("/protected"),
            "--session",
            "expired",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let expired_json = success_data(&expired, "get");
    assert!(expired_json["content"]
        .as_str()
        .unwrap()
        .contains("Session Expired"));

    let logout = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            &site.url("/logout"),
            "--session",
            "app",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let logout_json = success_data(&logout, "get");
    assert!(logout_json["content"]
        .as_str()
        .unwrap()
        .contains("Logged Out"));
}

#[test]
fn mock_site_composes_sessions_and_rejects_mixed_scope_replay() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = write_mock_site_backend(temp.path());
    save_cookie_session(
        &aget_home,
        "provider",
        &site.host(),
        "provider_session",
        "valid-provider",
    );
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let compose = Command::cargo_bin("aget")
        .unwrap()
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
    let compose_json = success_data(&compose, "session.compose");
    assert_eq!(compose_json["cookie_count"], 2);

    let composed_fetch = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            &site.url("/requires-two"),
            "--session",
            "combined",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let composed_json = success_data(&composed_fetch, "get");
    assert!(composed_json["content"]
        .as_str()
        .unwrap()
        .contains("Composed Session"));
    assert!(site.received_cookie("/requires-two", "provider_session", "valid-provider"));
    assert!(site.received_cookie("/requires-two", "app_session", "valid-app"));

    save_mixed_scope_session(&aget_home, "mixed", &site.host());
    let requests_before_rejection = site.requests().len();
    let rejected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            &site.url("/protected"),
            "--session",
            "mixed",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let rejected_json: serde_json::Value = serde_json::from_slice(&rejected).unwrap();
    assert_eq!(rejected_json["command"], "get");
    assert_eq!(rejected_json["error"]["code"], "privacy_policy_blocked");
    assert_eq!(site.requests().len(), requests_before_rejection);
}

#[test]
fn mock_site_login_bootstrap_can_fetch_protected_page_without_manual_action() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = write_mock_site_backend(temp.path());
    let fake_agent_browser = write_fake_agent_browser(temp.path(), &site.host());

    Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--json",
            "session",
            "login",
            "start",
            "mock-app",
            "--url",
            &site.https_url("/login"),
        ])
        .assert()
        .success();

    let finish = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args(["--json", "session", "login", "finish", "mock-app"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let finish_json = success_data(&finish, "session.login.finish");
    assert_eq!(finish_json["name"], "mock-app");

    let protected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", python_command(&fake_backend))
        .args([
            "--json",
            "get",
            &site.url("/protected"),
            "--session",
            "mock-app",
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let protected_json = success_data(&protected, "get");
    assert!(protected_json["content"]
        .as_str()
        .unwrap()
        .contains("Protected Account"));
}

fn write_mock_site_backend(dir: &Path) -> PathBuf {
    write_script(
        dir,
        "mock-site-backend",
        r#"#!/usr/bin/env python3
import argparse, html, json, pathlib, re, urllib.error, urllib.parse, urllib.request

parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
parser.add_argument('--format', default='markdown')
parser.add_argument('--selector')
parser.add_argument('--exclude-selector')
parser.add_argument('--wait-for')
args, _unknown = parser.parse_known_args()

state = json.loads(pathlib.Path(args.state).read_text(encoding='utf-8'))
parsed = urllib.parse.urlparse(args.url)
host = parsed.hostname or ''
origin = f"{parsed.scheme}://{parsed.netloc}"

cookies = []
for cookie in state.get('cookies', []):
    domain = cookie.get('domain', '').strip('.').lower()
    if host == domain or host.endswith('.' + domain):
        cookies.append(f"{cookie.get('name')}={cookie.get('value')}")

headers = {}
if cookies:
    headers['Cookie'] = '; '.join(cookies)

for item in state.get('origins', []):
    if item.get('origin') == origin:
        for entry in item.get('localStorage', []):
            if entry.get('name') == 'local_token':
                headers['X-Local-Token'] = entry.get('value', '')

def fetch(url, headers):
    request = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=5) as response:
            return response.geturl(), response.read().decode('utf-8')
    except urllib.error.HTTPError as error:
        return error.geturl(), error.read().decode('utf-8')

final_url, body = fetch(args.url, headers)

if parsed.path == '/storage-protected' and 'X-Local-Token' in headers:
    api_url = urllib.parse.urljoin(args.url, '/storage-api')
    _api_final_url, body = fetch(api_url, {'X-Local-Token': headers['X-Local-Token']})

if args.wait_for == '#ready' and '/delayed' in parsed.path:
    body += '<div id="ready">Delayed Ready</div>'

if args.exclude_selector == 'nav':
    body = re.sub(r'<nav\b[^>]*>.*?</nav>', '', body, flags=re.S | re.I)
if args.selector == 'main':
    match = re.search(r'<main\b[^>]*>(.*?)</main>', body, flags=re.S | re.I)
    if match:
        body = match.group(1)

def textify(value):
    value = re.sub(r'<script\b[^>]*>.*?</script>', '', value, flags=re.S | re.I)
    value = re.sub(r'<style\b[^>]*>.*?</style>', '', value, flags=re.S | re.I)
    value = re.sub(r'<[^>]+>', ' ', value)
    return ' '.join(html.unescape(value).split())

if args.format == 'html':
    content = body
elif args.format == 'json':
    content = json.dumps({'url': final_url, 'content': textify(body)})
else:
    content = textify(body)

pathlib.Path(args.output).write_text(content, encoding='utf-8')
pathlib.Path(args.metadata).write_text(json.dumps({'mock_site': True, 'url': args.url}), encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': final_url, 'content': content, 'warnings': []}))
"#,
    )
}

fn write_fake_agent_browser(dir: &Path, host: &str) -> PathBuf {
    write_script(
        dir,
        "mock-agent-browser",
        &format!(
            r#"#!/usr/bin/env python3
import json, pathlib, sys
args = sys.argv[1:]
if args[-2:-1] == ['open']:
    raise SystemExit(0)
if len(args) == 5 and args[:1] == ['--session'] and args[2:4] == ['state', 'save']:
    state_path = pathlib.Path(args[4])
    state = {{
        'cookies': [
            {{'name': 'app_session', 'value': 'valid-app', 'domain': '{host}', 'path': '/', 'httpOnly': True, 'secure': False, 'sameSite': 'Lax'}},
        ],
        'origins': [],
    }}
    state_path.write_text(json.dumps(state), encoding='utf-8')
    raise SystemExit(0)
if args[-1:] == ['close']:
    raise SystemExit(0)
print('unexpected args: ' + repr(args), file=sys.stderr)
raise SystemExit(2)
"#
        ),
    )
}

fn write_script(dir: &Path, prefix: &str, content: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = dir.join(format!("{prefix}-{nanos}.py"));
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

fn python_command(path: &Path) -> String {
    format!("python3 {}", shell_quote(&path.to_string_lossy()))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn success_data(output: &[u8], command: &str) -> serde_json::Value {
    let json: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], command);
    json["data"].clone()
}

fn save_cookie_session(home: &Path, name: &str, domain: &str, cookie_name: &str, value: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
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

fn save_storage_session(home: &Path, name: &str, origin: &str, key: &str, value: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_storage_origins.push(origin.to_string());
    session.origins.push(SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: key.to_string(),
            value: value.to_string(),
        }],
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}

fn save_mixed_scope_session(home: &Path, name: &str, domain: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_cookie_domains.push(domain.to_string());
    session.cookies.push(SessionCookie {
        name: "app_session".to_string(),
        value: "valid-app".to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    session.cookies.push(SessionCookie {
        name: "other_session".to_string(),
        value: "other-secret".to_string(),
        domain: "unrelated.example".to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}
