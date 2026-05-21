use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use aget::{Session, SessionCookie, SessionOrigin, StorageEntry};
use serde_json::json;

#[path = "mock_tools.rs"]
mod mock_tools;
use mock_tools::mock_agent_browser;

pub(crate) fn mock_backend_command(dir: &Path, config: serde_json::Value) -> String {
    mock_tools::mock_backend_command(dir, config)
}

pub(crate) fn mock_cmux(dir: &Path, config: serde_json::Value) -> PathBuf {
    mock_tools::mock_cmux(dir, config)
}

pub(crate) fn agent_browser_tool(
    dir: &Path,
    log_path: &Path,
    mut config: serde_json::Value,
) -> PathBuf {
    config["log_path"] = serde_json::Value::String(log_path.to_string_lossy().into_owned());
    mock_agent_browser(dir, config)
}

pub(crate) fn chrome_import_state() -> serde_json::Value {
    json!({
        "cookies": [
            {"name": "sid", "value": "allowed-secret", "domain": "example.com", "path": "/", "expires": 1812619153.69691_f64, "httpOnly": true, "secure": true, "sameSite": "Lax"},
            {"name": "sub", "value": "sub-secret", "domain": "docs.example.com", "path": "/", "httpOnly": false, "secure": false},
            {"name": "evil", "value": "blocked-secret", "domain": "example.com.evil", "path": "/", "httpOnly": false, "secure": false}
        ],
        "origins": [
            {"origin": "https://example.com", "localStorage": [{"name": "token", "value": "allowed-storage"}], "sessionStorage": [{"name": "session-token", "value": "session-only"}]},
            {"origin": "https://docs.example.com:443", "localStorage": [{"name": "subtoken", "value": "sub-storage"}]},
            {"origin": "https://example.com.evil", "localStorage": [{"name": "evil", "value": "blocked-storage"}]}
        ]
    })
}

pub(crate) fn nyt_state(cookie_name: &str, storage_name: &str) -> serde_json::Value {
    json!({
        "cookies": [
            {"name": cookie_name, "value": "nyt-secret", "domain": ".nytimes.com", "path": "/", "expires": 1812619153.69691_f64, "httpOnly": true, "secure": true, "sameSite": "Lax"},
            {"name": "provider", "value": "google-secret", "domain": "accounts.google.com", "path": "/", "httpOnly": true, "secure": true}
        ],
        "origins": [
            {"origin": "https://www.nytimes.com", "localStorage": [{"name": storage_name, "value": "nyt-storage"}]},
            {"origin": "https://accounts.google.com", "localStorage": [{"name": "provider", "value": "google-storage"}]}
        ]
    })
}

pub(crate) fn provider_only_state() -> serde_json::Value {
    json!({
        "cookies": [{"name": "provider", "value": "google-secret", "domain": "accounts.google.com", "path": "/", "httpOnly": true, "secure": true}],
        "origins": []
    })
}

pub(crate) fn success_envelope(output: &[u8], command: &str) -> serde_json::Value {
    let json: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], command);
    assert!(json["timing_ms"]["total"].as_u64().is_some());
    json
}

pub(crate) fn success_data(output: &[u8], command: &str) -> serde_json::Value {
    success_envelope(output, command)["data"].clone()
}

pub(crate) fn demo_session() -> Session {
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

pub(crate) fn named_session(name: &str, cookie_name: &str, value: &str, domain: &str) -> Session {
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

pub(crate) fn session_origin(origin: &str, name: &str, value: &str) -> SessionOrigin {
    SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: name.to_string(),
            value: value.to_string(),
        }],
        session_storage: Vec::new(),
        source_session: None,
    }
}

pub(crate) fn loopback_cookie_server() -> (String, thread::JoinHandle<()>) {
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

pub(crate) fn loopback_cookie_echo_server() -> (String, thread::JoinHandle<()>) {
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

pub(crate) fn crawl4ai_cookie_echo_server() -> (String, thread::JoinHandle<()>, Receiver<String>) {
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
