use std::collections::{BTreeMap, BTreeSet};
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
use url::Url;

use crate::error::{AgetError, ErrorCode};
use crate::process::{configure_local_command, create_private_file, TempOutputFile};
use crate::session::{PlaywrightCookie, PlaywrightOrigin, PlaywrightState, StorageEntry};

const CDP_READ_POLL: Duration = Duration::from_millis(100);
const CHROME_SHUTDOWN_WAIT: Duration = Duration::from_secs(1);
const NETWORK_IDLE_DURATION: Duration = Duration::from_millis(500);
const CHROME_SANDBOX_STARTUP_HINT: &str = "Hint: Chrome sandbox/namespace startup failure; in containers or VMs, set AGET_CHROME_COMMAND to a Chrome/Chromium executable that can run with --no-sandbox";

pub(crate) struct BrowserRenderRequest<'a> {
    pub(crate) tmp_dir: &'a Path,
    pub(crate) url: &'a str,
    pub(crate) state: &'a PlaywrightState,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) wait_until: PageWaitUntil,
    pub(crate) wait_for_images: bool,
    pub(crate) settle_delay: Duration,
    pub(crate) page_timeout: Duration,
    pub(crate) wait_for_timeout: Option<Duration>,
    pub(crate) timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PageWaitUntil {
    DomContentLoaded,
    Load,
    NetworkIdle,
}

impl PageWaitUntil {
    fn lifecycle_event_name(self) -> Option<&'static str> {
        match self {
            Self::DomContentLoaded => Some("Page.domContentEventFired"),
            Self::Load => Some("Page.loadEventFired"),
            Self::NetworkIdle => None,
        }
    }
}

pub(crate) struct BrowserStateExportRequest<'a> {
    pub(crate) profile_dir: &'a Path,
    pub(crate) profile_directory: Option<&'a str>,
    pub(crate) use_real_keychain: bool,
    pub(crate) allowed_domains: &'a [String],
    pub(crate) timeout: Duration,
}

pub(crate) struct BrowserLoginStartRequest<'a> {
    pub(crate) profile_dir: &'a Path,
    pub(crate) url: &'a str,
    pub(crate) timeout: Duration,
}

pub(crate) struct StartedLoginBrowser {
    pub(crate) pid: u32,
}

pub(crate) struct BrowserLoginStateExportRequest<'a> {
    pub(crate) profile_dir: &'a Path,
    pub(crate) allowed_domains: &'a [String],
    pub(crate) pid: Option<u32>,
    pub(crate) timeout: Duration,
}

pub(crate) struct BrowserLoginCloseRequest<'a> {
    pub(crate) profile_dir: &'a Path,
    pub(crate) pid: Option<u32>,
    pub(crate) timeout: Duration,
}

pub(crate) struct RenderedPage {
    pub(crate) final_url: String,
    pub(crate) html: String,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn render_page(request: BrowserRenderRequest<'_>) -> Result<RenderedPage, AgetError> {
    let mut chrome =
        ChromeProcess::launch_temp(request.tmp_dir, request.timeout, "owned browser fallback")?;
    let mut client = CdpClient::connect(&chrome.ws_url, request.timeout)?;
    let page = client.create_page(request.page_timeout)?;
    client.enable_page_domains(&page.session_id, request.page_timeout)?;
    client.load_state(&page.session_id, request.state, request.page_timeout)?;
    client.navigate_and_wait(
        &page.session_id,
        request.url,
        request.wait_until,
        request.page_timeout,
    )?;
    if let Some(selector) = request.wait_for_selector {
        client.wait_for_selector(
            &page.session_id,
            selector,
            request.wait_for_timeout.unwrap_or(request.page_timeout),
        )?;
    }
    let mut warnings = Vec::new();
    if request.wait_for_images && !client.wait_for_images_complete(&page.session_id)? {
        warnings.push(
            "some images did not finish loading before crawl4ai.wait_for_images timeout"
                .to_string(),
        );
    }
    if !request.settle_delay.is_zero() {
        thread::sleep(request.settle_delay);
    }
    let final_url =
        client.evaluate_string(&page.session_id, "location.href", request.page_timeout)?;
    let html = client.evaluate_string(
        &page.session_id,
        "document.documentElement.outerHTML || ''",
        request.page_timeout,
    )?;
    let _ = client.send(
        "Target.closeTarget",
        Some(json!({ "targetId": page.target_id })),
        None,
        Duration::from_secs(1),
    );
    let _ = client.send("Browser.close", None, None, Duration::from_secs(1));
    chrome.wait_or_kill(CHROME_SHUTDOWN_WAIT);
    Ok(RenderedPage {
        final_url,
        html,
        warnings,
    })
}

pub(crate) fn export_browser_state(
    request: BrowserStateExportRequest<'_>,
) -> Result<PlaywrightState, AgetError> {
    let mut chrome = ChromeProcess::launch_profile(
        request.profile_dir,
        request.profile_directory,
        request.use_real_keychain,
        request.timeout,
        "owned Chrome import",
    )?;
    let mut client = CdpClient::connect(&chrome.ws_url, request.timeout)?;
    let page = client.create_page(request.timeout)?;
    client.enable_page_domains(&page.session_id, request.timeout)?;
    let state = client.export_state(&page.session_id, request.allowed_domains, request.timeout)?;
    let _ = client.send(
        "Target.closeTarget",
        Some(json!({ "targetId": page.target_id })),
        None,
        Duration::from_secs(1),
    );
    let _ = client.send("Browser.close", None, None, Duration::from_secs(1));
    chrome.wait_or_kill(CHROME_SHUTDOWN_WAIT);
    Ok(state)
}

pub(crate) fn start_login_browser(
    request: BrowserLoginStartRequest<'_>,
) -> Result<StartedLoginBrowser, AgetError> {
    create_private_dir(request.profile_dir).map_err(io_aget_error)?;
    let chrome = ChromeProcess::launch_login(
        request.profile_dir,
        request.url,
        request.timeout,
        "owned login start",
    )?;
    let pid = chrome.id();
    chrome.detach();
    Ok(StartedLoginBrowser { pid })
}

pub(crate) fn export_login_browser_state(
    request: BrowserLoginStateExportRequest<'_>,
) -> Result<PlaywrightState, AgetError> {
    if let Some(mut client) =
        connect_existing_profile_browser(request.profile_dir, request.timeout)?
    {
        let existing_page = client.attach_existing_page(request.timeout)?;
        let created_page = existing_page.is_none();
        let page = match existing_page {
            Some(page) => page,
            None => client.create_page(request.timeout)?,
        };
        client.enable_page_domains(&page.session_id, request.timeout)?;
        let state =
            client.export_state(&page.session_id, request.allowed_domains, request.timeout)?;
        if created_page {
            let _ = client.send(
                "Target.closeTarget",
                Some(json!({ "targetId": page.target_id })),
                None,
                Duration::from_secs(1),
            );
        }
        client.close_browser(request.timeout)?;
        wait_for_profile_browser_shutdown(request.profile_dir, Duration::from_secs(5));
        ensure_login_browser_exited(request.profile_dir, request.pid, Duration::from_secs(5))?;
        return Ok(state);
    }

    ensure_login_browser_exited(request.profile_dir, request.pid, Duration::from_secs(5))?;

    export_browser_state(BrowserStateExportRequest {
        profile_dir: request.profile_dir,
        profile_directory: None,
        use_real_keychain: false,
        allowed_domains: request.allowed_domains,
        timeout: request.timeout,
    })
}

pub(crate) fn close_login_browser(request: BrowserLoginCloseRequest<'_>) -> Result<(), AgetError> {
    if let Some(mut client) =
        connect_existing_profile_browser(request.profile_dir, request.timeout)?
    {
        client.close_browser(request.timeout)?;
        wait_for_profile_browser_shutdown(request.profile_dir, Duration::from_secs(5));
    }
    ensure_login_browser_exited(request.profile_dir, request.pid, Duration::from_secs(5))?;
    Ok(())
}

struct ChromeProcess {
    child: Child,
    ws_url: String,
    user_data_dir: PathBuf,
    remove_user_data_dir: bool,
    terminated: bool,
    _stderr_capture: TempOutputFile,
}

impl ChromeProcess {
    fn launch_temp(
        tmp_dir: &Path,
        timeout: Duration,
        operation: &'static str,
    ) -> Result<Self, AgetError> {
        let user_data_dir = unique_profile_dir(tmp_dir)?;
        Self::launch_with_user_data_dir(
            user_data_dir,
            true,
            None,
            false,
            true,
            None,
            timeout,
            operation,
        )
    }

    fn launch_profile(
        user_data_dir: &Path,
        profile_directory: Option<&str>,
        use_real_keychain: bool,
        timeout: Duration,
        operation: &'static str,
    ) -> Result<Self, AgetError> {
        Self::launch_with_user_data_dir(
            user_data_dir.to_path_buf(),
            false,
            profile_directory,
            use_real_keychain,
            true,
            None,
            timeout,
            operation,
        )
    }

    fn launch_login(
        user_data_dir: &Path,
        url: &str,
        timeout: Duration,
        operation: &'static str,
    ) -> Result<Self, AgetError> {
        Self::launch_with_user_data_dir(
            user_data_dir.to_path_buf(),
            false,
            None,
            false,
            false,
            Some(url),
            timeout,
            operation,
        )
    }

    fn launch_with_user_data_dir(
        user_data_dir: PathBuf,
        remove_user_data_dir: bool,
        profile_directory: Option<&str>,
        use_real_keychain: bool,
        headless: bool,
        startup_url: Option<&str>,
        timeout: Duration,
        operation: &'static str,
    ) -> Result<Self, AgetError> {
        let executable = find_chrome_binary().ok_or_else(|| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!(
                "{operation} could not find Chrome; set AGET_CHROME_COMMAND to a Chrome/Chromium executable"
            ),
        })?;
        let stderr_capture =
            TempOutputFile::new(&env::temp_dir(), "aget-chrome-stderr").map_err(io_aget_error)?;
        let stderr = create_private_file(stderr_capture.path()).map_err(io_aget_error)?;
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
            .arg("--window-size=1280,720")
            .arg(format!("--user-data-dir={}", user_data_dir.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(stderr));
        if headless {
            command
                .arg("--headless=new")
                .arg("--enable-unsafe-swiftshader");
        }
        if !use_real_keychain {
            command
                .arg("--password-store=basic")
                .arg("--use-mock-keychain");
        }
        if let Some(profile_directory) = profile_directory {
            command.arg(format!("--profile-directory={profile_directory}"));
        }
        if cfg!(target_os = "linux") {
            command.arg("--no-sandbox").arg("--disable-dev-shm-usage");
        }
        if let Some(startup_url) = startup_url {
            command.arg("--new-window").arg(startup_url);
        }
        configure_local_command(&mut command);
        configure_chrome_process_group(&mut command);
        let _ = fs::remove_file(user_data_dir.join("DevToolsActivePort"));
        let mut child = command.spawn().map_err(|error| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!(
                "{operation} could not launch Chrome at '{}': {error}",
                executable.display()
            ),
        })?;
        let ws_url = match wait_for_devtools_active_port(
            &mut child,
            &user_data_dir,
            &stderr_capture,
            timeout,
        ) {
            Ok(ws_url) => ws_url,
            Err(error) => {
                terminate_child(&mut child);
                if remove_user_data_dir {
                    let _ = fs::remove_dir_all(&user_data_dir);
                }
                return Err(classify_chrome_startup_error(
                    operation,
                    error,
                    &stderr_capture,
                ));
            }
        };
        Ok(Self {
            child,
            ws_url,
            user_data_dir,
            remove_user_data_dir,
            terminated: false,
            _stderr_capture: stderr_capture,
        })
    }

    fn detach(mut self) {
        self.terminated = true;
    }

    fn id(&self) -> u32 {
        self.child.id()
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
        if self.remove_user_data_dir {
            let _ = fs::remove_dir_all(&self.user_data_dir);
        }
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
        self.attach_to_target(target_id, timeout)
    }

    fn attach_existing_page(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<PageSession>, AgetError> {
        let targets = self.send("Target.getTargets", Some(json!({})), None, timeout)?;
        let Some(target_infos) = targets.get("targetInfos").and_then(Value::as_array) else {
            return Ok(None);
        };
        let Some(target_id) = preferred_page_target_id(target_infos) else {
            return Ok(None);
        };
        self.attach_to_target(target_id, timeout).map(Some)
    }

    fn attach_to_target(
        &mut self,
        target_id: String,
        timeout: Duration,
    ) -> Result<PageSession, AgetError> {
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
        let _ = self.send(
            "Runtime.runIfWaitingForDebugger",
            None,
            Some(session_id),
            Duration::from_secs(1),
        );
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
            if origin.local_storage.is_empty() && origin.session_storage.is_empty() {
                continue;
            }
            let navigate_url = format!("{}/", origin.origin.trim_end_matches('/'));
            self.navigate_and_wait(session_id, &navigate_url, PageWaitUntil::Load, timeout)?;
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
            for entry in &origin.session_storage {
                let expression = session_storage_set_expression(&entry.name, &entry.value)?;
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

    fn export_state(
        &mut self,
        session_id: &str,
        allowed_domains: &[String],
        timeout: Duration,
    ) -> Result<PlaywrightState, AgetError> {
        let cookies = self.collect_cookies(session_id, allowed_domains, timeout)?;
        let origins = self.collect_storage_origins(session_id, allowed_domains, timeout)?;
        Ok(PlaywrightState { cookies, origins })
    }

    fn collect_cookies(
        &mut self,
        session_id: &str,
        allowed_domains: &[String],
        timeout: Duration,
    ) -> Result<Vec<PlaywrightCookie>, AgetError> {
        let result = self.send("Network.getAllCookies", None, Some(session_id), timeout)?;
        let mut cookies = playwright_cookies_from_cdp(&result);

        let urls = storage_candidate_origins(allowed_domains)
            .into_iter()
            .map(|origin| format!("{}/", origin.trim_end_matches('/')))
            .collect::<Vec<_>>();
        if !urls.is_empty() {
            let result = self.send(
                "Network.getCookies",
                Some(json!({ "urls": urls })),
                Some(session_id),
                timeout,
            )?;
            cookies.extend(playwright_cookies_from_cdp(&result));
        }

        if cookies.is_empty() {
            let result = self.send("Storage.getCookies", None, Some(session_id), timeout)?;
            cookies.extend(playwright_cookies_from_cdp(&result));
        }

        Ok(dedupe_playwright_cookies(cookies))
    }

    fn collect_storage_origins(
        &mut self,
        session_id: &str,
        allowed_domains: &[String],
        timeout: Duration,
    ) -> Result<Vec<PlaywrightOrigin>, AgetError> {
        let candidate_origins = storage_candidate_origins(allowed_domains);
        if candidate_origins.is_empty() {
            return Ok(Vec::new());
        }

        self.send(
            "Fetch.enable",
            Some(json!({ "patterns": [{ "urlPattern": "*" }] })),
            Some(session_id),
            timeout,
        )?;

        let mut origins = Vec::new();
        for origin in candidate_origins {
            let navigate_url = format!("{}/", origin.trim_end_matches('/'));
            self.navigate_with_blank_response(session_id, &navigate_url, timeout)?;
            let result = self.send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": origin_storage_expression(),
                    "returnByValue": true,
                    "awaitPromise": false,
                })),
                Some(session_id),
                timeout,
            )?;
            let Some(origin) = origin_storage_from_runtime_result(&result) else {
                continue;
            };
            if !origin.local_storage.is_empty() || !origin.session_storage.is_empty() {
                origins.push(origin);
            }
        }

        let _ = self.send(
            "Fetch.disable",
            None,
            Some(session_id),
            Duration::from_secs(1),
        );
        Ok(origins)
    }

    fn navigate_with_blank_response(
        &mut self,
        session_id: &str,
        url: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let navigate_id = self.send_no_wait(
            "Page.navigate",
            Some(json!({ "url": url })),
            Some(session_id),
        )?;

        let deadline = Instant::now() + timeout;
        loop {
            let message = self.read_message(deadline)?;
            if message.get("id").and_then(Value::as_u64) == Some(navigate_id) {
                if let Some(error) = message.get("error") {
                    let text = error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("CDP command failed");
                    return Err(AgetError::Stable {
                        code: ErrorCode::ExtractionFailed,
                        message: format!("owned browser fallback CDP Page.navigate failed: {text}"),
                    });
                }
                continue;
            }
            if message.get("sessionId").and_then(Value::as_str) != Some(session_id) {
                continue;
            }

            match message.get("method").and_then(Value::as_str) {
                Some("Fetch.requestPaused") => {
                    if let Some(request_id) = message
                        .get("params")
                        .and_then(|params| params.get("requestId"))
                        .and_then(Value::as_str)
                    {
                        self.send_no_wait(
                            "Fetch.fulfillRequest",
                            Some(json!({
                                "requestId": request_id,
                                "responseCode": 200,
                                "responseHeaders": [
                                    { "name": "Content-Type", "value": "text/html" }
                                ],
                                "body": "PGh0bWw+PC9odG1sPg=="
                            })),
                            Some(session_id),
                        )?;
                    }
                }
                Some("Page.loadEventFired") => return Ok(()),
                _ => {}
            }
        }
    }

    fn navigate_and_wait(
        &mut self,
        session_id: &str,
        url: &str,
        wait_until: PageWaitUntil,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let navigate_id = self.send_no_wait(
            "Page.navigate",
            Some(json!({ "url": url })),
            Some(session_id),
        )?;
        match wait_until.lifecycle_event_name() {
            Some(event_name) => {
                self.wait_for_navigation_event(navigate_id, event_name, session_id, timeout)
            }
            None => self.wait_for_network_idle(navigate_id, session_id, timeout),
        }
    }

    fn wait_for_navigation_event(
        &mut self,
        navigate_id: u64,
        event_name: &str,
        session_id: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let deadline = Instant::now() + timeout;
        loop {
            let message = self.read_message(deadline)?;
            if message.get("id").and_then(Value::as_u64) == Some(navigate_id) {
                self.handle_navigate_response(&message)?;
                continue;
            }
            if message.get("method").and_then(Value::as_str) != Some(event_name) {
                continue;
            }
            if message.get("sessionId").and_then(Value::as_str) == Some(session_id) {
                return Ok(());
            }
        }
    }

    fn wait_for_network_idle(
        &mut self,
        navigate_id: u64,
        session_id: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let deadline = Instant::now() + timeout;
        let mut navigate_response_seen = false;
        let mut dom_content_loaded = false;
        let mut inflight_requests = BTreeSet::new();
        let mut idle_since = Instant::now();

        loop {
            if navigate_response_seen
                && dom_content_loaded
                && inflight_requests.is_empty()
                && idle_since.elapsed() >= NETWORK_IDLE_DURATION
            {
                return Ok(());
            }

            let Some(message) = self.try_read_message(deadline)? else {
                continue;
            };

            if message.get("id").and_then(Value::as_u64) == Some(navigate_id) {
                self.handle_navigate_response(&message)?;
                navigate_response_seen = true;
                if inflight_requests.is_empty() {
                    idle_since = Instant::now();
                }
                continue;
            }
            if message.get("sessionId").and_then(Value::as_str) != Some(session_id) {
                continue;
            }

            match message.get("method").and_then(Value::as_str) {
                Some("Page.domContentEventFired") => {
                    dom_content_loaded = true;
                    if inflight_requests.is_empty() {
                        idle_since = Instant::now();
                    }
                }
                Some("Network.requestWillBeSent") => {
                    if let Some(request_id) = network_request_id(&message) {
                        inflight_requests.insert(request_id.to_string());
                        idle_since = Instant::now();
                    }
                }
                Some("Network.loadingFinished" | "Network.loadingFailed") => {
                    if let Some(request_id) = network_request_id(&message) {
                        inflight_requests.remove(request_id);
                        if inflight_requests.is_empty() {
                            idle_since = Instant::now();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_navigate_response(&self, message: &Value) -> Result<(), AgetError> {
        if let Some(error) = message.get("error") {
            let text = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("CDP command failed");
            return Err(AgetError::Stable {
                code: ErrorCode::ExtractionFailed,
                message: format!("owned browser fallback CDP Page.navigate failed: {text}"),
            });
        }
        Ok(())
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

    fn wait_for_images_complete(&mut self, session_id: &str) -> Result<bool, AgetError> {
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if Instant::now() >= deadline {
                return Ok(false);
            }
            let result = self.send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": "Array.from(document.images).every((img) => img.complete)",
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
                return Ok(true);
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

    fn close_browser(&mut self, timeout: Duration) -> Result<(), AgetError> {
        self.send("Browser.close", None, None, timeout)?;
        Ok(())
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

    fn send_no_wait(
        &mut self,
        method: &str,
        params: Option<Value>,
        session_id: Option<&str>,
    ) -> Result<u64, AgetError> {
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
        Ok(id)
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

    fn read_message(&mut self, deadline: Instant) -> Result<Value, AgetError> {
        loop {
            if let Some(message) = self.try_read_message(deadline)? {
                return Ok(message);
            }
        }
    }

    fn try_read_message(&mut self, deadline: Instant) -> Result<Option<Value>, AgetError> {
        if Instant::now() >= deadline {
            return Err(AgetError::Stable {
                code: ErrorCode::Timeout,
                message: "owned browser fallback timed out waiting for Chrome CDP".to_string(),
            });
        }
        match self.socket.read() {
            Ok(Message::Text(text)) => {
                let text: &str = text.as_ref();
                serde_json::from_str(text)
                    .map(Some)
                    .map_err(|error| AgetError::Stable {
                        code: ErrorCode::ExtractionFailed,
                        message: format!(
                            "owned browser fallback received malformed CDP JSON: {error}"
                        ),
                    })
            }
            Ok(Message::Binary(bytes)) => {
                let text =
                    String::from_utf8(bytes.to_vec()).map_err(|error| AgetError::Stable {
                        code: ErrorCode::ExtractionFailed,
                        message: format!(
                            "owned browser fallback received non-UTF8 CDP JSON: {error}"
                        ),
                    })?;
                serde_json::from_str(&text)
                    .map(Some)
                    .map_err(|error| AgetError::Stable {
                        code: ErrorCode::ExtractionFailed,
                        message: format!(
                            "owned browser fallback received malformed CDP JSON: {error}"
                        ),
                    })
            }
            Ok(Message::Ping(bytes)) => {
                self.socket
                    .send(Message::Pong(bytes))
                    .map_err(cdp_io_error)?;
                Ok(None)
            }
            Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => Ok(None),
            Ok(Message::Close(_)) => Err(AgetError::Stable {
                code: ErrorCode::ExtractionFailed,
                message: "owned browser fallback Chrome CDP connection closed".to_string(),
            }),
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                Ok(None)
            }
            Err(error) => Err(cdp_io_error(error)),
        }
    }
}

fn network_request_id(message: &Value) -> Option<&str> {
    message
        .get("params")
        .and_then(|params| params.get("requestId"))
        .and_then(Value::as_str)
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

fn playwright_cookies_from_cdp(result: &Value) -> Vec<PlaywrightCookie> {
    result
        .get("cookies")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(playwright_cookie_from_cdp)
        .collect()
}

fn playwright_cookie_from_cdp(cookie: &Value) -> Option<PlaywrightCookie> {
    Some(PlaywrightCookie {
        name: cookie.get("name")?.as_str()?.to_string(),
        value: cookie.get("value")?.as_str()?.to_string(),
        domain: cookie.get("domain")?.as_str()?.to_string(),
        path: cookie
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("/")
            .to_string(),
        expires: cdp_cookie_expires(cookie.get("expires")),
        http_only: cookie
            .get("httpOnly")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        secure: cookie
            .get("secure")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        same_site: cookie
            .get("sameSite")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

fn dedupe_playwright_cookies(cookies: Vec<PlaywrightCookie>) -> Vec<PlaywrightCookie> {
    let mut by_key = BTreeMap::new();
    for cookie in cookies {
        let key = (
            cookie.name.trim().to_string(),
            cookie
                .domain
                .trim()
                .trim_start_matches('.')
                .trim_end_matches('.')
                .to_ascii_lowercase(),
            if cookie.path.trim().is_empty() {
                "/".to_string()
            } else {
                cookie.path.trim().to_string()
            },
        );
        by_key.entry(key).or_insert(cookie);
    }
    by_key.into_values().collect()
}

fn cdp_cookie_expires(value: Option<&Value>) -> Option<i64> {
    let expires = value.and_then(Value::as_i64).or_else(|| {
        value
            .and_then(Value::as_f64)
            .map(|value| value.trunc() as i64)
    })?;
    (expires > 0).then_some(expires)
}

fn storage_candidate_origins(allowed_domains: &[String]) -> Vec<String> {
    let mut origins = BTreeSet::new();
    for domain in allowed_domains {
        let domain = domain
            .trim()
            .trim_start_matches('.')
            .trim_end_matches('/')
            .trim_end_matches('.');
        if domain.is_empty() {
            continue;
        }
        if domain.starts_with("http://") || domain.starts_with("https://") {
            origins.insert(domain.to_ascii_lowercase());
        } else {
            let domain = domain.to_ascii_lowercase();
            origins.insert(format!("https://{domain}"));
            origins.insert(format!("http://{domain}"));
        }
    }
    origins.into_iter().collect()
}

fn origin_storage_expression() -> &'static str {
    r#"(() => {
        const result = { origin: location.origin, localStorage: [], sessionStorage: [] };
        try {
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                result.localStorage.push({ name: key, value: localStorage.getItem(key) });
            }
        } catch(e) {}
        try {
            for (let i = 0; i < sessionStorage.length; i++) {
                const key = sessionStorage.key(i);
                result.sessionStorage.push({ name: key, value: sessionStorage.getItem(key) });
            }
        } catch(e) {}
        return result;
    })()"#
}

fn origin_storage_from_runtime_result(result: &Value) -> Option<PlaywrightOrigin> {
    let value = result
        .get("result")
        .and_then(|result| result.get("value"))?;
    let origin = value.get("origin")?.as_str()?;
    if origin.is_empty() || origin == "null" {
        return None;
    }
    let local_storage = value
        .get("localStorage")
        .and_then(|storage| serde_json::from_value::<Vec<StorageEntry>>(storage.clone()).ok())
        .unwrap_or_default();
    let session_storage = value
        .get("sessionStorage")
        .and_then(|storage| serde_json::from_value::<Vec<StorageEntry>>(storage.clone()).ok())
        .unwrap_or_default();
    Some(PlaywrightOrigin {
        origin: origin.to_string(),
        local_storage,
        session_storage,
    })
}

fn local_storage_set_expression(name: &str, value: &str) -> Result<String, AgetError> {
    Ok(format!(
        "localStorage.setItem({}, {})",
        serde_json::to_string(name).map_err(io_aget_error)?,
        serde_json::to_string(value).map_err(io_aget_error)?
    ))
}

fn session_storage_set_expression(name: &str, value: &str) -> Result<String, AgetError> {
    Ok(format!(
        "sessionStorage.setItem({}, {})",
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

fn preferred_page_target_id(target_infos: &[Value]) -> Option<String> {
    let mut fallback = None;
    for target in target_infos {
        let Some((target_id, url)) = trackable_page_target(target) else {
            continue;
        };
        fallback.get_or_insert_with(|| target_id.to_string());
        if !url.is_empty() && url != "about:blank" {
            return Some(target_id.to_string());
        }
    }
    fallback
}

fn trackable_page_target(target: &Value) -> Option<(&str, &str)> {
    let target_type = target.get("type").and_then(Value::as_str)?;
    if target_type != "page" && target_type != "webview" {
        return None;
    }
    let url = target
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if is_internal_chrome_target(url) {
        return None;
    }
    let target_id = target.get("targetId").and_then(Value::as_str)?;
    Some((target_id, url))
}

fn is_internal_chrome_target(url: &str) -> bool {
    url.starts_with("chrome://")
        || url.starts_with("chrome-extension://")
        || url.starts_with("devtools://")
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
    stderr_capture: &TempOutputFile,
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
        if let Some(ws_url) = devtools_ws_url_from_chrome_stderr(stderr_capture) {
            return Ok(ws_url);
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(AgetError::Stable {
        code: ErrorCode::Timeout,
        message: "owned browser fallback timed out waiting for Chrome CDP startup".to_string(),
    })
}

fn devtools_ws_url_from_chrome_stderr(stderr_capture: &TempOutputFile) -> Option<String> {
    let stderr = stderr_capture.read_to_string().ok()?;
    devtools_ws_url_from_stderr(&stderr)
}

fn devtools_ws_url_from_stderr(stderr: &str) -> Option<String> {
    stderr.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("DevTools listening on ")?;
        let url = rest.split_whitespace().next().unwrap_or(rest);
        if url.starts_with("ws://") || url.starts_with("wss://") {
            Some(url.to_string())
        } else {
            None
        }
    })
}

fn classify_chrome_startup_error(
    operation: &str,
    error: AgetError,
    stderr_capture: &TempOutputFile,
) -> AgetError {
    let stderr = stderr_capture.read_to_string().unwrap_or_default();
    let detail = relevant_chrome_stderr(&stderr);
    let combined = format!("{}\n{}", error, detail);
    if chrome_startup_requires_user_action(&combined) {
        return AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: if detail.is_empty() {
                format!("{operation} requires user action: {error}")
            } else {
                format!("{operation} requires user action: {error}; Chrome stderr: {detail}")
            },
        };
    }

    if detail.is_empty() {
        return error;
    }

    match error {
        AgetError::Stable { code, message } => AgetError::Stable {
            code,
            message: format!("{message}; Chrome stderr: {detail}"),
        },
    }
}

fn chrome_startup_requires_user_action(output: &str) -> bool {
    let output = output.to_ascii_lowercase();
    [
        "opening in existing browser session",
        "profile lock",
        "profile is locked",
        "profile in use",
        "profile appears to be in use",
        "already running",
        "processsingleton",
        "singletonlock",
        "chrome must be quit",
        "quit chrome",
        "close chrome",
    ]
    .iter()
    .any(|needle| output.contains(needle))
}

fn relevant_chrome_stderr(stderr: &str) -> String {
    let relevant = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            [
                "error",
                "fatal",
                "sandbox",
                "namespace",
                "permission",
                "cannot",
                "failed",
                "abort",
                "profile",
                "singleton",
                "already running",
                "opening in existing browser session",
            ]
            .iter()
            .any(|needle| lower.contains(needle))
        })
        .take(5)
        .collect::<Vec<_>>();
    if relevant.is_empty() {
        stderr
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n  ")
    } else {
        append_chrome_startup_hint(relevant.join("\n  "))
    }
}

fn append_chrome_startup_hint(detail: String) -> String {
    let lower = detail.to_ascii_lowercase();
    if lower.contains("sandbox") || lower.contains("namespace") {
        format!("{detail}\n  {CHROME_SANDBOX_STARTUP_HINT}")
    } else {
        detail
    }
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

fn connect_existing_profile_browser(
    profile_dir: &Path,
    timeout: Duration,
) -> Result<Option<CdpClient>, AgetError> {
    let Some((port, path)) = read_devtools_active_port(profile_dir) else {
        return Ok(None);
    };
    let ws_url = format!("ws://127.0.0.1:{port}{path}");
    match CdpClient::connect(&ws_url, timeout) {
        Ok(client) => Ok(Some(client)),
        Err(AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            ..
        }) => {
            let discovered_ws_url = match discover_cdp_ws_url(port, timeout) {
                Ok(ws_url) => ws_url,
                Err(_) => return Ok(None),
            };
            match CdpClient::connect(&discovered_ws_url, timeout) {
                Ok(client) => Ok(Some(client)),
                Err(AgetError::Stable {
                    code: ErrorCode::BackendUnavailable,
                    ..
                }) => Ok(None),
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

fn discover_cdp_ws_url(port: u16, timeout: Duration) -> Result<String, AgetError> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout.min(Duration::from_secs(2))))
        .http_status_as_error(false)
        .build();
    let agent: ureq::Agent = config.into();

    let version_error = match discover_cdp_ws_url_from_version(&agent, port) {
        Ok(ws_url) => return Ok(ws_url),
        Err(error) => error,
    };
    match discover_cdp_ws_url_from_list(&agent, port) {
        Ok(ws_url) => Ok(ws_url),
        Err(list_error) => match discover_cdp_ws_url_from_direct_ws(port, timeout) {
            Ok(ws_url) => Ok(ws_url),
            Err(ws_error) => Err(AgetError::Stable {
                code: ErrorCode::BackendUnavailable,
                message: format!(
                    "owned browser CDP discovery failed: /json/version: {version_error}; /json/list: {list_error}; /devtools/browser: {ws_error}",
                ),
            }),
        },
    }
}

fn discover_cdp_ws_url_from_version(agent: &ureq::Agent, port: u16) -> Result<String, AgetError> {
    let body = fetch_cdp_discovery_body(agent, port, "/json/version")?;
    let json: Value = serde_json::from_str(&body).map_err(|error| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!(
            "owned browser could not parse Chrome CDP /json/version response: {error}"
        ),
    })?;
    let ws_url = json
        .get("webSocketDebuggerUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: "owned browser CDP /json/version response lacked webSocketDebuggerUrl"
                .to_string(),
        })?;
    rewrite_cdp_ws_host(ws_url, port).ok_or_else(|| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser CDP discovery returned invalid WebSocket URL: {ws_url}"),
    })
}

fn discover_cdp_ws_url_from_list(agent: &ureq::Agent, port: u16) -> Result<String, AgetError> {
    let body = fetch_cdp_discovery_body(agent, port, "/json/list")?;
    let targets: Vec<Value> = serde_json::from_str(&body).map_err(|error| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser could not parse Chrome CDP /json/list response: {error}"),
    })?;
    let target = targets
        .iter()
        .find(|target| {
            target.get("type").and_then(Value::as_str) == Some("browser")
                && target
                    .get("webSocketDebuggerUrl")
                    .and_then(Value::as_str)
                    .is_some()
        })
        .or_else(|| {
            targets.iter().find(|target| {
                target
                    .get("webSocketDebuggerUrl")
                    .and_then(Value::as_str)
                    .is_some()
            })
        });
    let ws_url = target
        .and_then(|target| target.get("webSocketDebuggerUrl"))
        .and_then(Value::as_str)
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: "owned browser CDP /json/list response lacked webSocketDebuggerUrl"
                .to_string(),
        })?;
    rewrite_cdp_ws_host(ws_url, port).ok_or_else(|| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser CDP discovery returned invalid WebSocket URL: {ws_url}"),
    })
}

fn fetch_cdp_discovery_body(
    agent: &ureq::Agent,
    port: u16,
    path: &str,
) -> Result<String, AgetError> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let mut response = agent.get(&url).call().map_err(cdp_discovery_error)?;
    let status = response.status();
    if !status.is_success() {
        return Err(AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!("owned browser CDP discovery {path} returned HTTP {status}"),
        });
    }
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(cdp_discovery_error)?;
    Ok(body)
}

fn discover_cdp_ws_url_from_direct_ws(port: u16, timeout: Duration) -> Result<String, AgetError> {
    let ws_url = format!("ws://127.0.0.1:{port}/devtools/browser");
    let mut client = CdpClient::connect(&ws_url, timeout).map_err(|error| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser direct CDP WebSocket discovery failed: {error}"),
    })?;
    client
        .send("Browser.getVersion", None, None, timeout)
        .map_err(|error| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!("owned browser direct CDP WebSocket verification failed: {error}"),
        })?;
    Ok(ws_url)
}

fn rewrite_cdp_ws_host(ws_url: &str, port: u16) -> Option<String> {
    let mut url = Url::parse(ws_url).ok()?;
    match url.scheme() {
        "ws" | "wss" => {}
        _ => return None,
    }
    url.set_host(Some("127.0.0.1")).ok()?;
    url.set_port(Some(port)).ok()?;
    Some(url.to_string())
}

fn cdp_discovery_error(error: ureq::Error) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser CDP discovery failed: {error}"),
    }
}

fn wait_for_profile_browser_shutdown(profile_dir: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let Some((port, _)) = read_devtools_active_port(profile_dir) else {
            return;
        };
        if TcpStream::connect(("127.0.0.1", port)).is_err() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn ensure_login_browser_exited(
    profile_dir: &Path,
    pid: Option<u32>,
    timeout: Duration,
) -> Result<(), AgetError> {
    let Some(pid) = pid else {
        return Ok(());
    };

    wait_for_process_exit(pid, timeout);
    if !process_is_running(pid) {
        return Ok(());
    }

    if !process_command_mentions(pid, profile_dir) {
        return Ok(());
    }

    terminate_process_group_or_pid(pid);
    wait_for_process_exit(pid, Duration::from_secs(2));
    if process_is_running(pid) {
        return Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!("owned login browser process {pid} did not exit after Browser.close"),
        });
    }
    Ok(())
}

fn wait_for_process_exit(pid: u32, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !process_is_running(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(unix)]
fn process_is_running(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(not(unix))]
fn process_is_running(_pid: u32) -> bool {
    false
}

#[cfg(unix)]
fn process_command_mentions(pid: u32, needle: &Path) -> bool {
    Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .is_some_and(|command| command.contains(&needle.to_string_lossy().into_owned()))
}

#[cfg(not(unix))]
fn process_command_mentions(_pid: u32, _needle: &Path) -> bool {
    false
}

#[cfg(unix)]
fn terminate_process_group_or_pid(pid: u32) {
    let process_group = format!("-{pid}");
    let pid = pid.to_string();
    let _ = Command::new("kill")
        .args(["-TERM", &process_group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("kill")
        .args(["-TERM", &pid])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    thread::sleep(Duration::from_millis(100));
    let _ = Command::new("kill")
        .args(["-KILL", &process_group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("kill")
        .args(["-KILL", &pid])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(unix))]
fn terminate_process_group_or_pid(_pid: u32) {}

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

fn configure_chrome_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    #[cfg(not(unix))]
    {
        let _ = command;
    }
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
    fn parses_cdp_cookies_into_playwright_state_shape() {
        let cookies = playwright_cookies_from_cdp(&json!({
            "cookies": [
                {
                    "name": "sid",
                    "value": "secret",
                    "domain": ".example.com",
                    "path": "/",
                    "expires": 1800000000.9,
                    "httpOnly": true,
                    "secure": true,
                    "sameSite": "Lax"
                },
                {
                    "name": "session",
                    "value": "secret",
                    "domain": "example.com",
                    "path": "/",
                    "expires": -1
                }
            ]
        }));

        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies[0].expires, Some(1_800_000_000));
        assert_eq!(cookies[1].expires, None);
        assert!(cookies[0].http_only);
        assert!(cookies[0].secure);
        assert_eq!(cookies[0].same_site.as_deref(), Some("Lax"));
    }

    #[test]
    fn storage_candidate_origins_cover_http_and_https_domains() {
        let origins = storage_candidate_origins(&[
            "Example.COM".to_string(),
            ".docs.example.com/".to_string(),
            "https://app.example.com".to_string(),
        ]);

        assert_eq!(
            origins,
            vec![
                "http://docs.example.com".to_string(),
                "http://example.com".to_string(),
                "https://app.example.com".to_string(),
                "https://docs.example.com".to_string(),
                "https://example.com".to_string(),
            ]
        );
    }

    #[test]
    fn parses_origin_storage_runtime_value() {
        let origin = origin_storage_from_runtime_result(&json!({
            "result": {
                "value": {
                    "origin": "https://example.com",
                    "localStorage": [
                        {"name": "token", "value": "secret"}
                    ],
                    "sessionStorage": [
                        {"name": "ignored", "value": "session-only"}
                    ]
                }
            }
        }))
        .unwrap();

        assert_eq!(origin.origin, "https://example.com");
        assert_eq!(origin.local_storage.len(), 1);
        assert_eq!(origin.local_storage[0].name, "token");
        assert_eq!(origin.session_storage.len(), 1);
        assert_eq!(origin.session_storage[0].name, "ignored");
    }

    #[test]
    #[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
    fn owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory() {
        let temp = tempfile::tempdir().unwrap();
        let profile = temp.path().join("profile");
        fs::create_dir_all(&profile).unwrap();
        let profile_directory = "Profile 1";
        let timeout = Duration::from_secs(20);
        let expires = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;

        {
            let mut chrome = ChromeProcess::launch_profile(
                &profile,
                Some(profile_directory),
                false,
                timeout,
                "owned Chrome test",
            )
            .expect("Chrome should launch for test profile");
            let mut client = CdpClient::connect(&chrome.ws_url, timeout).unwrap();
            let page = client.create_page(timeout).unwrap();
            client
                .enable_page_domains(&page.session_id, timeout)
                .unwrap();
            client
                .send(
                    "Fetch.enable",
                    Some(json!({ "patterns": [{ "urlPattern": "*" }] })),
                    Some(&page.session_id),
                    timeout,
                )
                .unwrap();
            client
                .navigate_with_blank_response(&page.session_id, "https://example.com/", timeout)
                .unwrap();
            client
                .send(
                    "Network.setCookie",
                    Some(json!({
                        "name": "sid",
                        "value": "secret",
                        "url": "https://example.com/",
                        "path": "/",
                        "expires": expires,
                        "secure": true,
                        "sameSite": "Lax",
                    })),
                    Some(&page.session_id),
                    timeout,
                )
                .unwrap();
            client
                .send(
                    "Runtime.evaluate",
                    Some(json!({
                        "expression": local_storage_set_expression("token", "storage-secret").unwrap(),
                        "returnByValue": true,
                        "awaitPromise": false,
                    })),
                    Some(&page.session_id),
                    timeout,
                )
                .unwrap();
            client
                .send(
                    "Runtime.evaluate",
                    Some(json!({
                        "expression": session_storage_set_expression("session-token", "session-secret").unwrap(),
                        "returnByValue": true,
                        "awaitPromise": false,
                    })),
                    Some(&page.session_id),
                    timeout,
                )
                .unwrap();
            let _ = client.send("Browser.close", None, None, Duration::from_secs(1));
            chrome.wait_or_kill(Duration::from_secs(5));
        }

        let exported = export_browser_state(BrowserStateExportRequest {
            profile_dir: &profile,
            profile_directory: Some(profile_directory),
            use_real_keychain: false,
            allowed_domains: &["example.com".to_string()],
            timeout,
        })
        .unwrap();

        assert!(exported
            .cookies
            .iter()
            .any(|cookie| cookie.name == "sid" && cookie.value == "secret"));
        assert!(exported.origins.iter().any(|origin| {
            origin.origin == "https://example.com"
                && origin
                    .local_storage
                    .iter()
                    .any(|entry| entry.name == "token" && entry.value == "storage-secret")
        }));
    }

    #[test]
    #[ignore = "requires local Chrome/Chromium and opens a visible browser window"]
    fn owned_login_browser_exports_state_from_headed_profile_and_closes() {
        let temp = tempfile::tempdir().unwrap();
        let profile = temp.path().join("login-profile");
        let timeout = Duration::from_secs(20);
        let expires = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;

        let started = start_login_browser(BrowserLoginStartRequest {
            profile_dir: &profile,
            url: "https://example.com/",
            timeout,
        })
        .expect("Chrome should launch for headed login test");

        {
            let mut client = connect_existing_profile_browser(&profile, timeout)
                .unwrap()
                .expect("headed login browser should expose CDP");
            let page = client
                .attach_existing_page(timeout)
                .unwrap()
                .unwrap_or_else(|| client.create_page(timeout).unwrap());
            client
                .enable_page_domains(&page.session_id, timeout)
                .unwrap();
            client
                .send(
                    "Fetch.enable",
                    Some(json!({ "patterns": [{ "urlPattern": "*" }] })),
                    Some(&page.session_id),
                    timeout,
                )
                .unwrap();
            client
                .navigate_with_blank_response(&page.session_id, "https://example.com/", timeout)
                .unwrap();
            client
                .send(
                    "Network.setCookie",
                    Some(json!({
                        "name": "sid",
                        "value": "login-secret",
                        "url": "https://example.com/",
                        "path": "/",
                        "expires": expires,
                        "secure": true,
                        "sameSite": "Lax",
                    })),
                    Some(&page.session_id),
                    timeout,
                )
                .unwrap();
            client
                .send(
                    "Runtime.evaluate",
                    Some(json!({
                        "expression": local_storage_set_expression("token", "login-storage").unwrap(),
                        "returnByValue": true,
                        "awaitPromise": false,
                    })),
                    Some(&page.session_id),
                    timeout,
                )
                .unwrap();
            client
                .send(
                    "Runtime.evaluate",
                    Some(json!({
                        "expression": session_storage_set_expression("session-token", "login-session").unwrap(),
                        "returnByValue": true,
                        "awaitPromise": false,
                    })),
                    Some(&page.session_id),
                    timeout,
                )
                .unwrap();
        }

        let exported = export_login_browser_state(BrowserLoginStateExportRequest {
            profile_dir: &profile,
            allowed_domains: &["example.com".to_string()],
            pid: Some(started.pid),
            timeout,
        })
        .unwrap();

        assert!(exported
            .cookies
            .iter()
            .any(|cookie| cookie.name == "sid" && cookie.value == "login-secret"));
        assert!(exported.origins.iter().any(|origin| {
            origin.origin == "https://example.com"
                && origin
                    .local_storage
                    .iter()
                    .any(|entry| entry.name == "token" && entry.value == "login-storage")
                && origin
                    .session_storage
                    .iter()
                    .any(|entry| entry.name == "session-token" && entry.value == "login-session")
        }));
        remove_dir_all_with_retries(&profile).unwrap();
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
    fn session_storage_expression_json_quotes_key_and_value() {
        let expression =
            session_storage_set_expression("token\"name", "line\nvalue</script>").unwrap();

        assert_eq!(
            expression,
            "sessionStorage.setItem(\"token\\\"name\", \"line\\nvalue</script>\")"
        );
    }

    #[test]
    fn prefers_existing_non_internal_page_target() {
        let target_id = preferred_page_target_id(&[
            json!({
                "targetId": "chrome",
                "type": "page",
                "url": "chrome://new-tab-page/"
            }),
            json!({
                "targetId": "blank",
                "type": "page",
                "url": "about:blank"
            }),
            json!({
                "targetId": "login",
                "type": "page",
                "url": "https://example.com/login"
            }),
        ]);

        assert_eq!(target_id.as_deref(), Some("login"));
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
    fn parses_devtools_ws_url_from_chrome_stderr() {
        let stderr = "\
noise
DevTools listening on ws://127.0.0.1:9222/devtools/browser/fallback
more noise";

        assert_eq!(
            devtools_ws_url_from_stderr(stderr).as_deref(),
            Some("ws://127.0.0.1:9222/devtools/browser/fallback")
        );
    }

    #[test]
    fn chrome_stderr_detail_includes_sandbox_hint() {
        let detail = relevant_chrome_stderr(
            "Failed to move to new namespace: PID namespaces supported, Network namespace supported, but failed: errno = Operation not permitted",
        );

        assert!(detail.contains("Failed to move to new namespace"));
        assert!(detail.contains("Chrome sandbox/namespace startup failure"));
        assert!(detail.contains("AGET_CHROME_COMMAND"));
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_devtools_active_port_uses_stderr_fallback() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
        let stderr = create_private_file(stderr_capture.path()).unwrap();
        let fake_chrome = temp.path().join("fake-chrome");
        fs::write(
            &fake_chrome,
            "#!/bin/sh\n\
             echo 'DevTools listening on ws://127.0.0.1:9222/devtools/browser/fallback' >&2\n\
             sleep 5\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_chrome).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&fake_chrome, permissions).unwrap();
        let mut child = Command::new(&fake_chrome)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(stderr))
            .spawn()
            .unwrap();

        let ws_url = wait_for_devtools_active_port(
            &mut child,
            temp.path(),
            &stderr_capture,
            Duration::from_secs(2),
        )
        .unwrap();
        terminate_child(&mut child);

        assert_eq!(ws_url, "ws://127.0.0.1:9222/devtools/browser/fallback");
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

    #[test]
    fn rewrites_discovered_cdp_websocket_host_and_port() {
        assert_eq!(
            rewrite_cdp_ws_host("ws://localhost:9222/devtools/browser/abc", 49152).as_deref(),
            Some("ws://127.0.0.1:49152/devtools/browser/abc")
        );
        assert!(rewrite_cdp_ws_host("http://localhost:9222/json/version", 49152).is_none());
    }

    #[test]
    fn discovers_cdp_websocket_url_from_json_version() {
        let body = r#"{"webSocketDebuggerUrl":"ws://localhost:9999/devtools/browser/discovered"}"#;
        let (port, handle) = serve_cdp_discovery_responses(vec![http_json_response(200, body)]);

        let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
        handle.join().unwrap();

        assert_eq!(
            ws_url,
            format!("ws://127.0.0.1:{port}/devtools/browser/discovered")
        );
    }

    #[test]
    fn discovers_cdp_websocket_url_from_json_list_fallback() {
        let list_body = r#"[
            {
                "type": "page",
                "webSocketDebuggerUrl": "ws://localhost:9999/devtools/page/ignored"
            },
            {
                "type": "browser",
                "webSocketDebuggerUrl": "ws://localhost:9999/devtools/browser/list"
            }
        ]"#;
        let (port, handle) = serve_cdp_discovery_responses(vec![
            http_json_response(200, r#"{}"#),
            http_json_response(200, list_body),
        ]);

        let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
        handle.join().unwrap();

        assert_eq!(
            ws_url,
            format!("ws://127.0.0.1:{port}/devtools/browser/list")
        );
    }

    #[test]
    fn discovers_cdp_websocket_url_from_first_list_target_with_ws() {
        let list_body = r#"[
            {
                "type": "page"
            },
            {
                "type": "page",
                "webSocketDebuggerUrl": "ws://localhost:9999/devtools/page/fallback"
            }
        ]"#;
        let (port, handle) = serve_cdp_discovery_responses(vec![
            http_json_response(404, r#"{"error":"missing"}"#),
            http_json_response(200, list_body),
        ]);

        let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
        handle.join().unwrap();

        assert_eq!(
            ws_url,
            format!("ws://127.0.0.1:{port}/devtools/page/fallback")
        );
    }

    #[test]
    fn discovers_cdp_websocket_url_from_direct_websocket_fallback() {
        use std::io::{Read as _, Write as _};
        use std::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0u8; 512];
                let _ = stream.read(&mut request);
                let response = http_json_response(404, r#"{"error":"missing"}"#);
                stream.write_all(response.as_bytes()).unwrap();
            }

            let (stream, _) = listener.accept().unwrap();
            let mut websocket = tungstenite::accept(stream).unwrap();
            let Message::Text(text) = websocket.read().unwrap() else {
                panic!("expected text CDP command");
            };
            let request: Value = serde_json::from_str(text.as_ref()).unwrap();
            let id = request.get("id").and_then(Value::as_u64).unwrap();
            let reply = json!({
                "id": id,
                "result": {
                    "protocolVersion": "1.3",
                    "product": "Chrome/136"
                }
            })
            .to_string();
            websocket.send(Message::Text(reply.into())).unwrap();
            let _ = websocket.close(None);
        });

        let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
        handle.join().unwrap();

        assert_eq!(ws_url, format!("ws://127.0.0.1:{port}/devtools/browser"));
    }

    fn serve_cdp_discovery_responses(responses: Vec<String>) -> (u16, thread::JoinHandle<()>) {
        use std::io::{Read as _, Write as _};
        use std::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0u8; 512];
                let _ = stream.read(&mut request);
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (port, handle)
    }

    fn http_json_response(status: u16, body: &str) -> String {
        let reason = match status {
            200 => "OK",
            404 => "Not Found",
            _ => "Status",
        };
        format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
    }

    fn remove_dir_all_with_retries(path: &Path) -> io::Result<()> {
        let mut last_error = None;
        for _ in 0..5 {
            match fs::remove_dir_all(path) {
                Ok(()) => return Ok(()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
                Err(error) => {
                    last_error = Some(error);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
        Err(last_error.unwrap_or_else(|| io::Error::other("remove_dir_all failed")))
    }
}
