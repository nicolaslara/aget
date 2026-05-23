mod blank;
mod lifecycle;
mod network_idle;
mod readiness;

use serde_json::{json, Value};

use super::CdpClient;
use crate::browser_cdp::PageWaitUntil;
use crate::error::{AgetError, ErrorCode};

impl CdpClient {
    pub(in crate::browser_cdp) fn navigate_and_wait(
        &mut self,
        session_id: &str,
        url: &str,
        wait_until: PageWaitUntil,
        timeout: std::time::Duration,
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

fn navigation_wait_timeout(wait_until: &str) -> AgetError {
    let wait_until = match wait_until {
        "Page.domContentEventFired" => "domcontentloaded",
        "Page.loadEventFired" => "load",
        other => other,
    };
    AgetError::Stable {
        code: ErrorCode::Timeout,
        message: format!("owned browser fallback timed out waiting for {wait_until}"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavigationOutcome {
    NewDocument,
    SameDocument,
}
