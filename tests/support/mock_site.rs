mod protocol;
mod routes;

use std::collections::BTreeMap;
use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use self::protocol::{cookie_contains, read_request};
use self::routes::response_for;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockRequest {
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
}

pub struct MockSite {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<MockRequest>>>,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockResponse {
    status: u16,
    content_type: String,
    headers: Vec<(String, String)>,
    body: String,
    delay: Option<Duration>,
}

impl MockResponse {
    pub fn html(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            content_type: "text/html; charset=utf-8".to_string(),
            headers: Vec::new(),
            body: body.into(),
            delay: None,
        }
    }

    pub fn redirect(location: impl Into<String>) -> Self {
        Self {
            status: 302,
            content_type: "text/html; charset=utf-8".to_string(),
            headers: vec![("Location".to_string(), location.into())],
            body: String::new(),
            delay: None,
        }
    }

    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }

    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

#[derive(Default)]
pub struct MockSiteBuilder {
    routes: BTreeMap<String, MockResponse>,
}

impl MockSiteBuilder {
    pub fn route(mut self, path: impl Into<String>, response: MockResponse) -> Self {
        self.routes.insert(path.into(), response);
        self
    }

    pub fn start(self) -> MockSite {
        MockSite::start_with_routes(self.routes)
    }
}

impl MockSite {
    pub fn start() -> Self {
        Self::builder().start()
    }

    pub fn builder() -> MockSiteBuilder {
        MockSiteBuilder::default()
    }

    fn start_with_routes(routes: BTreeMap<String, MockResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let shutdown = Arc::new(AtomicBool::new(false));
        let thread_requests = Arc::clone(&requests);
        let thread_shutdown = Arc::clone(&shutdown);
        let routes = Arc::new(routes);
        let thread_routes = Arc::clone(&routes);
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                if thread_shutdown.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(mut stream) = stream else {
                    continue;
                };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                if let Some(request) = read_request(&mut stream) {
                    thread_requests.lock().unwrap().push(request.clone());
                    let response = response_for(&request, &thread_routes);
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        });

        Self {
            addr,
            requests,
            shutdown,
            handle: Some(handle),
        }
    }

    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn origin(&self) -> String {
        self.base_url()
    }

    pub fn host(&self) -> String {
        self.addr.ip().to_string()
    }

    pub fn https_url(&self, path: &str) -> String {
        format!("https://{}{}", self.addr, path)
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url(), path)
    }

    pub fn requests(&self) -> Vec<MockRequest> {
        self.requests.lock().unwrap().clone()
    }

    pub fn received_cookie(&self, path: &str, name: &str, value: &str) -> bool {
        self.requests().iter().any(|request| {
            request.path == path
                && request
                    .headers
                    .get("cookie")
                    .is_some_and(|cookie| cookie_contains(cookie, name, value))
        })
    }

    pub fn received_header(&self, path: &str, name: &str, value: &str) -> bool {
        let name = name.to_ascii_lowercase();
        self.requests().iter().any(|request| {
            request.path == path
                && request
                    .headers
                    .get(&name)
                    .is_some_and(|header| header == value)
        })
    }
}

impl Drop for MockSite {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect_timeout(&self.addr, Duration::from_millis(100));
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl MockResponse {
    fn to_http(&self) -> String {
        if let Some(delay) = self.delay {
            thread::sleep(delay);
        }
        protocol::http_response(
            self.status,
            &self.content_type,
            &self
                .headers
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str()))
                .collect::<Vec<_>>(),
            &self.body,
        )
    }
}
