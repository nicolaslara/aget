use std::collections::BTreeSet;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::transport::remaining;
use super::CdpClient;
use crate::browser_cdp::page_scripts::{
    rendered_overlay_cleanup_expression, selector_exists_expression,
};
use crate::browser_cdp::PageWaitUntil;
use crate::error::{AgetError, ErrorCode};

const NETWORK_IDLE_DURATION: Duration = Duration::from_millis(500);

impl CdpClient {
    pub(in crate::browser_cdp) fn navigate_with_blank_response(
        &mut self,
        session_id: &str,
        url: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let navigate_id = self.send_no_wait(
            "Page.navigate",
            Some(json!({ "url": url })),
            self.session_param(session_id),
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
            if !self.message_matches_session(&message, session_id) {
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
                            self.session_param(session_id),
                        )?;
                    }
                }
                Some("Page.loadEventFired") => return Ok(()),
                _ => {}
            }
        }
    }

    pub(in crate::browser_cdp) fn navigate_and_wait(
        &mut self,
        session_id: &str,
        url: &str,
        wait_until: PageWaitUntil,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let navigate_id = self.send_no_wait(
            "Page.navigate",
            Some(json!({ "url": url })),
            self.session_param(session_id),
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
                if self.handle_navigate_response(&message)? == NavigationOutcome::SameDocument {
                    return Ok(());
                }
                continue;
            }
            if message.get("method").and_then(Value::as_str) != Some(event_name) {
                continue;
            }
            if self.message_matches_session(&message, session_id) {
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
                if self.handle_navigate_response(&message)? == NavigationOutcome::SameDocument {
                    return Ok(());
                }
                navigate_response_seen = true;
                if inflight_requests.is_empty() {
                    idle_since = Instant::now();
                }
                continue;
            }
            if !self.message_matches_session(&message, session_id) {
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

    fn handle_navigate_response(&self, message: &Value) -> Result<NavigationOutcome, AgetError> {
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
        let result = message.get("result").unwrap_or(&Value::Null);
        if let Some(error_text) = result.get("errorText").and_then(Value::as_str) {
            return Err(AgetError::Stable {
                code: ErrorCode::ExtractionFailed,
                message: format!("owned browser fallback CDP Page.navigate failed: {error_text}"),
            });
        }
        if result.get("loaderId").and_then(Value::as_str).is_none() {
            return Ok(NavigationOutcome::SameDocument);
        }
        Ok(NavigationOutcome::NewDocument)
    }

    pub(in crate::browser_cdp) fn wait_for_selector(
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
                self.session_param(session_id),
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

    pub(in crate::browser_cdp) fn wait_for_images_complete(
        &mut self,
        session_id: &str,
    ) -> Result<bool, AgetError> {
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
                self.session_param(session_id),
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

    pub(in crate::browser_cdp) fn remove_rendered_overlay_elements(
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
            self.session_param(session_id),
            timeout,
        )?;
        Ok(())
    }

    pub(in crate::browser_cdp) fn evaluate_string(
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
            self.session_param(session_id),
            timeout,
        )?;
        Ok(result
            .get("result")
            .and_then(|result| result.get("value"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string())
    }
}

impl CdpClient {
    fn message_matches_session(&self, message: &Value, session_id: &str) -> bool {
        match self.session_param(session_id) {
            Some(session_id) => {
                message.get("sessionId").and_then(Value::as_str) == Some(session_id)
            }
            None => message.get("sessionId").is_none(),
        }
    }
}

fn network_request_id(message: &Value) -> Option<&str> {
    message
        .get("params")
        .and_then(|params| params.get("requestId"))
        .and_then(Value::as_str)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavigationOutcome {
    NewDocument,
    SameDocument,
}
