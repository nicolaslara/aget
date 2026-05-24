use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use aget::{Session, SessionCookie, SessionOrigin, SessionStore, StorageEntry};
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

pub(crate) fn cookie_echo_server(path: &str) -> (String, JoinHandle<()>, Receiver<String>) {
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
                        .find_map(|line| {
                            line.strip_prefix("Cookie: ")
                                .or_else(|| line.strip_prefix("cookie: "))
                        })
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

pub(crate) fn metadata_files(aget_home: &Path) -> Vec<PathBuf> {
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

pub(crate) fn save_cookie_session(
    home: &Path,
    name: &str,
    domain: &str,
    cookie_name: &str,
    value: &str,
) {
    save_cookie_session_with_sensitivity(home, name, domain, cookie_name, value, true);
}

pub(crate) fn save_cookie_session_with_sensitivity(
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn save_cookie_and_storage_session(
    home: &Path,
    name: &str,
    domain: &str,
    cookie_name: &str,
    cookie_value: &str,
    origin: &str,
    storage_name: &str,
    storage_value: &str,
) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_cookie_domains.push(domain.to_string());
    session.allowed_storage_origins.push(origin.to_string());
    session.cookies.push(SessionCookie {
        name: cookie_name.to_string(),
        value: cookie_value.to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    session.origins.push(SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: storage_name.to_string(),
            value: storage_value.to_string(),
        }],
        session_storage: Vec::new(),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}
