use std::time::Duration;

use serde_json::{json, Value};

use super::super::{preferred_page_target_id, CdpClient};
use super::PageSession;
use crate::error::{AgetError, ErrorCode};

impl CdpClient {
    pub(in crate::browser_cdp) fn create_page(
        &mut self,
        timeout: Duration,
    ) -> Result<PageSession, AgetError> {
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

    pub(in crate::browser_cdp) fn attach_existing_page(
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
