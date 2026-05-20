use std::env;
use std::fs;
use std::io;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

use crate::error::{AgetError, ErrorCode};
use crate::process::configure_local_command;
use crate::session::{PlaywrightCookie, PlaywrightState};

const CDP_READ_POLL: Duration = Duration::from_millis(100);
const CHROME_SHUTDOWN_WAIT: Duration = Duration::from_secs(1);

pub(crate) struct BrowserRenderRequest<'a> {
    pub(crate) tmp_dir: &'a Path,
    pub(crate) url: &'a str,
    pub(crate) state: &'a PlaywrightState,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) timeout: Duration,
}

pub(crate) struct RenderedPage {
    pub(crate) final_url: String,
    pub(crate) html: String,
}

pub(crate) fn render_page(request: BrowserRenderRequest<'_>) -> Result<RenderedPage, AgetError> {
    let mut chrome = ChromeProcess::launch(request.tmp_dir, request.timeout)?;
    let mut client = CdpClient::connect(&chrome.ws_url, request.timeout)?;
    let page = client.create_page(request.timeout)?;
    client.enable_page_domains(&page.session_id, request.timeout)?;
    client.load_state(&page.session_id, request.state, request.timeout)?;
    client.navigate_and_wait(&page.session_id, request.url, request.timeout)?;
    if let Some(selector) = request.wait_for_selector {
        client.wait_for_selector(&page.session_id, selector, request.timeout)?;
    }
    let final_url = client.evaluate_string(&page.session_id, "location.href", request.timeout)?;
    let html = client.evaluate_string(
        &page.session_id,
        "document.documentElement.outerHTML || ''",
        request.timeout,
    )?;
    let _ = client.send(
        "Target.closeTarget",
        Some(json!({ "targetId": page.target_id })),
        None,
        Duration::from_secs(1),
    );
    let _ = client.send("Browser.close", None, None, Duration::from_secs(1));
    chrome.wait_or_kill(CHROME_SHUTDOWN_WAIT);
    Ok(RenderedPage { final_url, html })
}

struct ChromeProcess {
    child: Child,
    ws_url: String,
    user_data_dir: PathBuf,
    terminated: bool,
}

impl ChromeProcess {
    fn launch(tmp_dir: &Path, timeout: Duration) -> Result<Self, AgetError> {
        let executable = find_chrome_binary().ok_or_else(|| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: "owned browser fallback could not find Chrome; set AGET_CHROME_COMMAND to a Chrome/Chromium executable for localStorage-backed rendering".to_string(),
        })?;
        let user_data_dir = unique_profile_dir(tmp_dir)?;
        let mut command = Command::new(&executable);
        command
            .arg("--remote-debugging-port=0")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-background-networking")
            .arg("--disable-backgrounding-occluded-windows")
            .arg("--disable-component-update")
            .arg("--disable-default-apps")
            .arg("--disable-popup-blocking")
            .arg("--disable-sync")
            .arg("--disable-features=Translate")
            .arg("--headless=new")
            .arg("--enable-unsafe-swiftshader")
            .arg("--password-store=basic")
            .arg("--use-mock-keychain")
            .arg("--window-size=1280,720")
            .arg(format!("--user-data-dir={}", user_data_dir.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if cfg!(target_os = "linux") {
            command.arg("--no-sandbox").arg("--disable-dev-shm-usage");
        }
        configure_local_command(&mut command);
        let mut child = command.spawn().map_err(|error| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!(
                "owned browser fallback could not launch Chrome at '{}': {error}",
                executable.display()
            ),
        })?;
        let ws_url = match wait_for_devtools_active_port(&mut child, &user_data_dir, timeout) {
            Ok(ws_url) => ws_url,
            Err(error) => {
                terminate_child(&mut child);
                let _ = fs::remove_dir_all(&user_data_dir);
                return Err(error);
            }
        };
        Ok(Self {
            child,
            ws_url,
            user_data_dir,
            terminated: false,
        })
    }

    fn wait_or_kill(&mut self, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(_)) => {
                    self.terminated = true;
                    return;
                }
                Ok(None) => thread::sleep(Duration::from_millis(25)),
                Err(_) => {
                    self.terminated = true;
                    return;
                }
            }
        }
        terminate_child(&mut self.child);
        self.terminated = true;
    }
}

impl Drop for ChromeProcess {
    fn drop(&mut self) {
        if !self.terminated {
            terminate_child(&mut self.child);
            self.terminated = true;
        }
        let _ = fs::remove_dir_all(&self.user_data_dir);
    }
}

struct PageSession {
    target_id: String,
    session_id: String,
}

struct CdpClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl CdpClient {
    fn connect(ws_url: &str, timeout: Duration) -> Result<Self, AgetError> {
        let (mut socket, _) = connect(ws_url).map_err(|error| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!("owned browser fallback could not connect to Chrome CDP: {error}"),
        })?;
        configure_socket_timeout(&mut socket, timeout)?;
        Ok(Self { socket, next_id: 1 })
    }

    fn create_page(&mut self, timeout: Duration) -> Result<PageSession, AgetError> {
        let created = self.send(
            "Target.createTarget",
            Some(json!({ "url": "about:blank" })),
            None,
            timeout,
        )?;
        let target_id = string_field(&created, "targetId", "Target.createTarget")?;
        let attached = self.send(
            "Target.attachToTarget",
            Some(json!({
                "targetId": target_id,
                "flatten": true,
            })),
            None,
            timeout,
        )?;
        let session_id = string_field(&attached, "sessionId", "Target.attachToTarget")?;
        Ok(PageSession {
            target_id,
            session_id,
        })
    }

    fn enable_page_domains(
        &mut self,
        session_id: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        self.send("Page.enable", None, Some(session_id), timeout)?;
        self.send("Runtime.enable", None, Some(session_id), timeout)?;
        self.send("Network.enable", None, Some(session_id), timeout)?;
        Ok(())
    }

    fn load_state(
        &mut self,
        session_id: &str,
        state: &PlaywrightState,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        if !state.cookies.is_empty() {
            self.send(
                "Network.setCookies",
                Some(json!({ "cookies": cdp_cookies(&state.cookies) })),
                Some(session_id),
                timeout,
            )?;
        }

        for origin in &state.origins {
            if origin.local_storage.is_empty() {
                continue;
            }
            let navigate_url = format!("{}/", origin.origin.trim_end_matches('/'));
            self.navigate_and_wait(session_id, &navigate_url, timeout)?;
            for entry in &origin.local_storage {
                let expression = local_storage_set_expression(&entry.name, &entry.value)?;
                self.send(
                    "Runtime.evaluate",
                    Some(json!({
                        "expression": expression,
                        "returnByValue": true,
                        "awaitPromise": false,
                    })),
                    Some(session_id),
                    timeout,
                )?;
            }
        }
        Ok(())
    }

    fn navigate_and_wait(
        &mut self,
        session_id: &str,
        url: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        self.send(
            "Page.navigate",
            Some(json!({ "url": url })),
            Some(session_id),
            timeout,
        )?;
        self.wait_for_event("Page.loadEventFired", Some(session_id), timeout)
    }

    fn wait_for_selector(
        &mut self,
        session_id: &str,
        selector: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let expression = selector_exists_expression(selector)?;
        let deadline = Instant::now() + timeout;
        loop {
            let result = self.send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": false,
                })),
                Some(session_id),
                remaining(deadline),
            )?;
            if result
                .get("result")
                .and_then(|result| result.get("value"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(AgetError::Stable {
                    code: ErrorCode::Timeout,
                    message: format!(
                        "owned browser fallback timed out waiting for selector '{selector}'"
                    ),
                });
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn evaluate_string(
        &mut self,
        session_id: &str,
        expression: &str,
        timeout: Duration,
    ) -> Result<String, AgetError> {
        let result = self.send(
            "Runtime.evaluate",
            Some(json!({
                "expression": expression,
                "returnByValue": true,
                "awaitPromise": false,
            })),
            Some(session_id),
            timeout,
        )?;
        Ok(result
            .get("result")
            .and_then(|result| result.get("value"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string())
    }

    fn send(
        &mut self,
        method: &str,
        params: Option<Value>,
        session_id: Option<&str>,
        timeout: Duration,
    ) -> Result<Value, AgetError> {
        let id = self.next_id;
        self.next_id += 1;
        let mut command = serde_json::Map::new();
        command.insert("id".to_string(), json!(id));
        command.insert("method".to_string(), json!(method));
        if let Some(params) = params {
            command.insert("params".to_string(), params);
        }
        if let Some(session_id) = session_id {
            command.insert("sessionId".to_string(), json!(session_id));
        }
        let text = serde_json::to_string(&Value::Object(command)).map_err(io_aget_error)?;
        self.socket
            .send(Message::Text(text.into()))
            .map_err(cdp_io_error)?;
        self.wait_for_response(id, method, timeout)
    }

    fn wait_for_response(
        &mut self,
        id: u64,
        method: &str,
        timeout: Duration,
    ) -> Result<Value, AgetError> {
        let deadline = Instant::now() + timeout;
        loop {
            let message = self.read_message(deadline)?;
            if message.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = message.get("error") {
                let text = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("CDP command failed");
                return Err(AgetError::Stable {
                    code: ErrorCode::ExtractionFailed,
                    message: format!("owned browser fallback CDP {method} failed: {text}"),
                });
            }
            return Ok(message.get("result").cloned().unwrap_or(Value::Null));
        }
    }

    fn wait_for_event(
        &mut self,
        method: &str,
        session_id: Option<&str>,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let deadline = Instant::now() + timeout;
        loop {
            let message = self.read_message(deadline)?;
            if message.get("method").and_then(Value::as_str) != Some(method) {
                continue;
            }
            if let Some(expected_session_id) = session_id {
                if message.get("sessionId").and_then(Value::as_str) != Some(expected_session_id) {
                    continue;
                }
            }
            return Ok(());
        }
    }

    fn read_message(&mut self, deadline: Instant) -> Result<Value, AgetError> {
        loop {
            if Instant::now() >= deadline {
                return Err(AgetError::Stable {
                    code: ErrorCode::Timeout,
                    message: "owned browser fallback timed out waiting for Chrome CDP".to_string(),
                });
            }
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    let text: &str = text.as_ref();
                    return serde_json::from_str(text).map_err(|error| AgetError::Stable {
                        code: ErrorCode::ExtractionFailed,
                        message: format!(
                            "owned browser fallback received malformed CDP JSON: {error}"
                        ),
                    });
                }
                Ok(Message::Binary(bytes)) => {
                    let text =
                        String::from_utf8(bytes.to_vec()).map_err(|error| AgetError::Stable {
                            code: ErrorCode::ExtractionFailed,
                            message: format!(
                                "owned browser fallback received non-UTF8 CDP JSON: {error}"
                            ),
                        })?;
                    return serde_json::from_str(&text).map_err(|error| AgetError::Stable {
                        code: ErrorCode::ExtractionFailed,
                        message: format!(
                            "owned browser fallback received malformed CDP JSON: {error}"
                        ),
                    });
                }
                Ok(Message::Ping(bytes)) => self
                    .socket
                    .send(Message::Pong(bytes))
                    .map_err(cdp_io_error)?,
                Ok(Message::Pong(_)) => {}
                Ok(Message::Frame(_)) => {}
                Ok(Message::Close(_)) => {
                    return Err(AgetError::Stable {
                        code: ErrorCode::ExtractionFailed,
                        message: "owned browser fallback Chrome CDP connection closed".to_string(),
                    });
                }
                Err(tungstenite::Error::Io(error))
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) => {}
                Err(error) => return Err(cdp_io_error(error)),
            }
        }
    }
}

fn cdp_cookies(cookies: &[PlaywrightCookie]) -> Vec<Value> {
    cookies
        .iter()
        .map(|cookie| {
            let mut value = json!({
                "name": cookie.name,
                "value": cookie.value,
                "domain": cookie.domain,
                "path": cookie.path,
                "httpOnly": cookie.http_only,
                "secure": cookie.secure,
            });
            if let Some(expires) = cookie.expires {
                value["expires"] = json!(expires);
            }
            if let Some(same_site) = &cookie.same_site {
                value["sameSite"] = json!(same_site);
            }
            value
        })
        .collect()
}

fn local_storage_set_expression(name: &str, value: &str) -> Result<String, AgetError> {
    Ok(format!(
        "localStorage.setItem({}, {})",
        serde_json::to_string(name).map_err(io_aget_error)?,
        serde_json::to_string(value).map_err(io_aget_error)?
    ))
}

fn selector_exists_expression(selector: &str) -> Result<String, AgetError> {
    Ok(format!(
        "document.querySelector({}) !== null",
        serde_json::to_string(selector).map_err(io_aget_error)?
    ))
}

fn string_field(value: &Value, field: &str, context: &str) -> Result<String, AgetError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!("owned browser fallback CDP {context} response lacked '{field}'"),
        })
}

fn wait_for_devtools_active_port(
    child: &mut Child,
    user_data_dir: &Path,
    timeout: Duration,
) -> Result<String, AgetError> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(AgetError::Stable {
                code: ErrorCode::BackendUnavailable,
                message: format!(
                    "owned browser fallback Chrome exited before CDP startup with {status}"
                ),
            });
        }
        if let Some((port, path)) = read_devtools_active_port(user_data_dir) {
            return Ok(format!("ws://127.0.0.1:{port}{path}"));
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(AgetError::Stable {
        code: ErrorCode::Timeout,
        message: "owned browser fallback timed out waiting for Chrome CDP startup".to_string(),
    })
}

fn read_devtools_active_port(user_data_dir: &Path) -> Option<(u16, String)> {
    let content = fs::read_to_string(user_data_dir.join("DevToolsActivePort")).ok()?;
    let mut lines = content.lines();
    let port = lines.next()?.trim().parse::<u16>().ok()?;
    let path = lines
        .next()
        .unwrap_or("/devtools/browser")
        .trim()
        .to_string();
    Some((port, path))
}

fn unique_profile_dir(tmp_dir: &Path) -> Result<PathBuf, AgetError> {
    let owned_dir = tmp_dir.join("owned-chrome");
    create_private_dir(&owned_dir).map_err(io_aget_error)?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = owned_dir.join(format!("aget-chrome-{}-{nanos}", std::process::id()));
    create_private_dir(&path).map_err(io_aget_error)?;
    Ok(path)
}

fn find_chrome_binary() -> Option<PathBuf> {
    if let Some(command) = env::var_os("AGET_CHROME_COMMAND") {
        if !command.is_empty() {
            return Some(PathBuf::from(command));
        }
    }

    for path in platform_chrome_candidates() {
        if path.is_file() {
            return Some(path);
        }
    }

    for command in [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "chrome",
        "brave-browser",
        "brave-browser-stable",
    ] {
        if let Some(path) = find_on_path(command) {
            return Some(path);
        }
    }

    None
}

fn platform_chrome_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "macos")]
    {
        candidates.extend([
            PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            PathBuf::from(
                "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            ),
            PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium"),
            PathBuf::from("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
        ]);
        if let Some(home) = env::var_os("HOME") {
            let home = PathBuf::from(home);
            candidates.extend([
                home.join("Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
                home.join("Applications/Chromium.app/Contents/MacOS/Chromium"),
                home.join("Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
            ]);
        }
    }

    #[cfg(target_os = "windows")]
    {
        candidates.extend([
            PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
        ]);
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            let local = PathBuf::from(local);
            candidates.extend([
                local.join(r"Google\Chrome\Application\chrome.exe"),
                local.join(r"BraveSoftware\Brave-Browser\Application\brave.exe"),
            ]);
        }
    }

    candidates
}

fn find_on_path(command: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(command))
        .find(|path| path.is_file())
}

fn configure_socket_timeout(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    timeout: Duration,
) -> Result<(), AgetError> {
    let timeout = timeout.min(CDP_READ_POLL).max(Duration::from_millis(10));
    match socket.get_mut() {
        MaybeTlsStream::Plain(stream) => {
            stream
                .set_read_timeout(Some(timeout))
                .map_err(io_aget_error)?;
            stream
                .set_write_timeout(Some(timeout))
                .map_err(io_aget_error)?;
        }
        #[allow(unreachable_patterns)]
        _ => {}
    }
    Ok(())
}

fn remaining(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}

fn cdp_io_error(error: tungstenite::Error) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: format!("owned browser fallback Chrome CDP I/O failed: {error}"),
    }
}

fn io_aget_error(error: impl ToString) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}

fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    set_private_dir_permissions(path)
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    {
        let process_group = format!("-{}", child.id());
        let _ = Command::new("kill")
            .args(["-TERM", &process_group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        thread::sleep(Duration::from_millis(50));
        let _ = Command::new("kill")
            .args(["-KILL", &process_group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::PlaywrightCookie;

    #[test]
    fn cdp_cookie_payload_preserves_browser_cookie_fields() {
        let payload = cdp_cookies(&[PlaywrightCookie {
            name: "sid".to_string(),
            value: "secret".to_string(),
            domain: ".example.com".to_string(),
            path: "/account".to_string(),
            expires: Some(1_800_000_000),
            http_only: true,
            secure: true,
            same_site: Some("Lax".to_string()),
        }]);

        assert_eq!(
            payload,
            vec![json!({
                "name": "sid",
                "value": "secret",
                "domain": ".example.com",
                "path": "/account",
                "expires": 1_800_000_000,
                "httpOnly": true,
                "secure": true,
                "sameSite": "Lax",
            })]
        );
    }

    #[test]
    fn local_storage_expression_json_quotes_key_and_value() {
        let expression =
            local_storage_set_expression("token\"name", "line\nvalue</script>").unwrap();

        assert_eq!(
            expression,
            "localStorage.setItem(\"token\\\"name\", \"line\\nvalue</script>\")"
        );
    }

    #[test]
    fn selector_wait_expression_json_quotes_css_selector() {
        let expression = selector_exists_expression(r#"main[data-name="a b"]"#).unwrap();

        assert_eq!(
            expression,
            r#"document.querySelector("main[data-name=\"a b\"]") !== null"#
        );
    }

    #[test]
    fn parses_devtools_active_port_file() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join("DevToolsActivePort"),
            "49152\n/devtools/browser/abc\n",
        )
        .unwrap();

        assert_eq!(
            read_devtools_active_port(temp.path()),
            Some((49152, "/devtools/browser/abc".to_string()))
        );
    }
}
