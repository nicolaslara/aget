use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
    headers: Vec<(String, String)>,
    body: String,
}

impl MockResponse {
    pub fn html(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub fn redirect(location: impl Into<String>) -> Self {
        Self {
            status: 302,
            headers: vec![("Location".to_string(), location.into())],
            body: String::new(),
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

fn read_request(stream: &mut TcpStream) -> Option<MockRequest> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    while buffer.len() < 16 * 1024 {
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    if buffer.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(&buffer);
    let mut lines = text.lines();
    let request_line = lines.next()?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next()?.to_string();
    let raw_path = request_parts.next()?.to_string();
    let path = raw_path
        .split('?')
        .next()
        .unwrap_or(raw_path.as_str())
        .to_string();
    let mut headers = BTreeMap::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    Some(MockRequest {
        method,
        path,
        headers,
    })
}

fn response_for(request: &MockRequest, routes: &BTreeMap<String, MockResponse>) -> String {
    if let Some(response) = routes.get(&request.path) {
        return response.to_http();
    }

    match request.path.as_str() {
        "/public" => html(
            200,
            &[],
            r#"
<html>
  <body>
    <main id="content">
      <nav>In-Article Navigation</nav>
      <h1>Public Main</h1>
      <p>Visible public article.</p>
    </main>
    <footer>Footer chrome</footer>
  </body>
</html>
"#,
        ),
        "/warning" => html(
            200,
            &[],
            r#"<html><body><main><h1>Warning Page</h1><p>Visible warning body.</p></main></body></html>"#,
        ),
        "/protected" if has_cookie(request, "app_session", "valid-app") => html(
            200,
            &[],
            r#"<html><body><main><h1>Protected Account</h1><p>Private account body.</p></main></body></html>"#,
        ),
        "/protected" if has_cookie(request, "app_session", "expired") => html(
            401,
            &[],
            r#"<html><body><main><h1>Session Expired</h1></main></body></html>"#,
        ),
        "/protected" => html(302, &[("Location", "/login?next=/protected")], ""),
        "/requires-two"
            if has_cookie(request, "app_session", "valid-app")
                && has_cookie(request, "provider_session", "valid-provider") =>
        {
            html(
                200,
                &[],
                r#"<html><body><main><h1>Composed Session</h1><p>Both cookies arrived.</p></main></body></html>"#,
            )
        }
        "/requires-two" => html(401, &[], r#"<main><h1>Missing composed auth</h1></main>"#),
        "/storage-protected" => html(
            200,
            &[],
            r#"
<html>
  <body>
    <main><h1>Storage App Shell</h1><div id="storage-result"></div></main>
    <script>
      fetch("/storage-api", { headers: { "X-Local-Token": localStorage.getItem("local_token") }})
    </script>
  </body>
</html>
"#,
        ),
        "/storage-api"
            if request
                .headers
                .get("x-local-token")
                .is_some_and(|token| token == "storage-secret") =>
        {
            html(
                200,
                &[],
                r#"<html><body><main><h1>Storage Protected</h1></main></body></html>"#,
            )
        }
        "/storage-api" => html(401, &[], r#"<main><h1>Missing storage token</h1></main>"#),
        "/delayed" => html(
            200,
            &[],
            r#"
<html>
  <body>
    <main><h1>Delayed Shell</h1><div id="initial">Loading</div></main>
    <script>
      setTimeout(() => {
        const ready = document.createElement("div")
        ready.id = "ready"
        ready.textContent = "Delayed Ready"
        document.body.appendChild(ready)
      }, 25)
    </script>
  </body>
</html>
"#,
        ),
        "/redirect" => html(302, &[("Location", "/public")], ""),
        "/login" => html(
            200,
            &[],
            r#"<html><body><main><h1>Login Form</h1><form action="/login/callback"></form></main></body></html>"#,
        ),
        "/login/callback" => html(
            200,
            &[("Set-Cookie", "app_session=valid-app; Path=/; HttpOnly")],
            r#"<html><body><main><h1>Login Complete</h1></main></body></html>"#,
        ),
        "/logout" => html(
            200,
            &[("Set-Cookie", "app_session=expired; Path=/; Max-Age=0")],
            r#"<html><body><main><h1>Logged Out</h1></main></body></html>"#,
        ),
        _ => MockResponse::html(r#"<main><h1>Not Found</h1></main>"#)
            .status(404)
            .to_http(),
    }
}

fn has_cookie(request: &MockRequest, name: &str, value: &str) -> bool {
    request
        .headers
        .get("cookie")
        .is_some_and(|cookie| cookie_contains(cookie, name, value))
}

fn cookie_contains(cookie_header: &str, name: &str, value: &str) -> bool {
    cookie_header
        .split(';')
        .map(str::trim)
        .any(|cookie| cookie == format!("{name}={value}"))
}

impl MockResponse {
    fn to_http(&self) -> String {
        html(
            self.status,
            &self
                .headers
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str()))
                .collect::<Vec<_>>(),
            &self.body,
        )
    }
}

fn html(status: u16, headers: &[(&str, &str)], body: &str) -> String {
    let reason = match status {
        200 => "OK",
        302 => "Found",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "OK",
    };
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    response.push_str(body);
    response
}
