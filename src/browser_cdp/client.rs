use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

use crate::error::{AgetError, ErrorCode};
use crate::session::{PlaywrightCookie, PlaywrightOrigin, PlaywrightState, StorageEntry};

use super::page_scripts::{
    local_storage_set_expression, origin_storage_expression, rendered_overlay_cleanup_expression,
    selector_exists_expression, session_storage_set_expression,
    shadow_dom_attach_override_expression,
};
use super::{io_aget_error, PageWaitUntil};

const CDP_READ_POLL: Duration = Duration::from_millis(100);
const NETWORK_IDLE_DURATION: Duration = Duration::from_millis(500);

pub(super) struct PageSession {
    pub(super) target_id: String,
    pub(super) session_id: String,
}

pub(super) struct CdpClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl CdpClient {
    pub(super) fn connect(ws_url: &str, timeout: Duration) -> Result<Self, AgetError> {
        let (mut socket, _) = connect(ws_url).map_err(|error| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!("owned browser fallback could not connect to Chrome CDP: {error}"),
        })?;
        configure_socket_timeout(&mut socket, timeout)?;
        Ok(Self { socket, next_id: 1 })
    }

    pub(super) fn create_page(&mut self, timeout: Duration) -> Result<PageSession, AgetError> {
        let created = self.send(
            "Target.createTarget",
            Some(json!({ "url": "about:blank" })),
            None,
            timeout,
        )?;
        let target_id = string_field(&created, "targetId", "Target.createTarget")?;
        self.attach_to_target(target_id, timeout)
    }

    pub(super) fn attach_existing_page(
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

    pub(super) fn enable_page_domains(
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

    pub(super) fn force_open_shadow_roots(
        &mut self,
        session_id: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        self.send(
            "Page.addScriptToEvaluateOnNewDocument",
            Some(json!({
                "source": shadow_dom_attach_override_expression(),
            })),
            Some(session_id),
            timeout,
        )?;
        Ok(())
    }

    pub(super) fn load_state(
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

    pub(super) fn export_state(
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

    pub(super) fn navigate_with_blank_response(
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

    pub(super) fn navigate_and_wait(
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

    pub(super) fn wait_for_selector(
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

    pub(super) fn wait_for_images_complete(&mut self, session_id: &str) -> Result<bool, AgetError> {
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

    pub(super) fn remove_rendered_overlay_elements(
        &mut self,
        session_id: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        self.send(
            "Runtime.evaluate",
            Some(json!({
                "expression": rendered_overlay_cleanup_expression(),
                "returnByValue": true,
                "awaitPromise": true,
            })),
            Some(session_id),
            timeout,
        )?;
        Ok(())
    }

    pub(super) fn evaluate_string(
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

    pub(super) fn close_browser(&mut self, timeout: Duration) -> Result<(), AgetError> {
        self.send("Browser.close", None, None, timeout)?;
        Ok(())
    }

    pub(super) fn send(
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

pub(super) fn cdp_cookies(cookies: &[PlaywrightCookie]) -> Vec<Value> {
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

pub(super) fn playwright_cookies_from_cdp(result: &Value) -> Vec<PlaywrightCookie> {
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

pub(super) fn storage_candidate_origins(allowed_domains: &[String]) -> Vec<String> {
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

pub(super) fn origin_storage_from_runtime_result(result: &Value) -> Option<PlaywrightOrigin> {
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

pub(super) fn preferred_page_target_id(target_infos: &[Value]) -> Option<String> {
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
