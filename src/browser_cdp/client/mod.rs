use std::net::TcpStream;
use std::time::Duration;

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

use crate::error::{AgetError, ErrorCode};

use super::page_scripts::shadow_dom_attach_override_expression;
pub(super) use super::session_data::{
    cdp_cookies, origin_storage_from_runtime_result, playwright_cookies_from_cdp,
    preferred_page_target_id, storage_candidate_origins,
};

mod navigation;
mod state;
mod transport;

pub(super) struct PageSession {
    pub(super) target_id: String,
    pub(super) session_id: String,
}

pub(super) struct CdpClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_id: u64,
}

impl CdpClient {
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

    pub(super) fn close_browser(&mut self, timeout: Duration) -> Result<(), AgetError> {
        self.send("Browser.close", None, None, timeout)?;
        Ok(())
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
