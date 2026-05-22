use std::net::TcpStream;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

use crate::error::{AgetError, ErrorCode};

use super::page_scripts::shadow_dom_attach_override_expression;
pub(super) use super::session_data::{
    cdp_cookies, frame_storage_candidate_origins, origin_storage_from_runtime_result,
    playwright_cookies_from_cdp, preferred_page_target_id, storage_candidate_origins,
};

mod navigation;
mod state;
mod transport;
#[cfg(test)]
pub(in crate::browser_cdp) use transport::cdp_websocket_config;

pub(super) struct PageSession {
    /// Empty for direct page WebSocket connections, where CDP commands already
    /// target the page and Chrome does not return a flattened target session.
    pub(super) target_id: String,
    pub(super) session_id: String,
}

pub(super) struct CdpClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_id: u64,
    direct_page_connection: bool,
    keepalive_interval: Duration,
    last_keepalive: Instant,
}

impl CdpClient {
    pub(super) fn create_page(&mut self, timeout: Duration) -> Result<PageSession, AgetError> {
        if self.direct_page_connection {
            return Ok(direct_page_session());
        }
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
        if self.direct_page_connection {
            return Ok(Some(direct_page_session()));
        }
        self.send(
            "Target.setDiscoverTargets",
            Some(json!({ "discover": true })),
            None,
            timeout,
        )?;
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
        self.send("Page.enable", None, self.session_param(session_id), timeout)?;
        self.send(
            "Runtime.enable",
            None,
            self.session_param(session_id),
            timeout,
        )?;
        let _ = self.send(
            "Runtime.runIfWaitingForDebugger",
            None,
            self.session_param(session_id),
            Duration::from_secs(1),
        );
        self.send(
            "Network.enable",
            None,
            self.session_param(session_id),
            timeout,
        )?;
        if !self.direct_page_connection {
            let _ = self.send(
                "Target.setAutoAttach",
                Some(json!({
                    "autoAttach": true,
                    "waitForDebuggerOnStart": false,
                    "flatten": true,
                })),
                self.session_param(session_id),
                Duration::from_secs(1),
            );
        }
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
            self.session_param(session_id),
            timeout,
        )?;
        Ok(())
    }

    pub(super) fn close_browser(&mut self, timeout: Duration) -> Result<(), AgetError> {
        if self.direct_page_connection {
            return Ok(());
        }
        self.send("Browser.close", None, None, timeout)?;
        Ok(())
    }

    pub(super) fn close_page(
        &mut self,
        page: &PageSession,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        if page.target_id.is_empty() {
            return Ok(());
        }
        self.send(
            "Target.closeTarget",
            Some(json!({ "targetId": page.target_id })),
            None,
            timeout,
        )?;
        Ok(())
    }

    pub(super) fn session_param<'a>(&self, session_id: &'a str) -> Option<&'a str> {
        if session_id.is_empty() {
            None
        } else {
            Some(session_id)
        }
    }
}

fn direct_page_session() -> PageSession {
    PageSession {
        target_id: String::new(),
        session_id: String::new(),
    }
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

pub(super) fn fail_on_runtime_evaluation_exception(result: &Value) -> Result<(), AgetError> {
    if let Some(message) = runtime_evaluation_exception(result) {
        return Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!("owned browser fallback Runtime.evaluate failed: {message}"),
        });
    }
    Ok(())
}

fn runtime_evaluation_exception(result: &Value) -> Option<String> {
    let details = result.get("exceptionDetails")?;
    let message = details
        .get("exception")
        .and_then(|exception| exception.get("description"))
        .and_then(Value::as_str)
        .or_else(|| details.get("text").and_then(Value::as_str))
        .unwrap_or("unknown Runtime.evaluate exception");
    Some(message.to_string())
}
