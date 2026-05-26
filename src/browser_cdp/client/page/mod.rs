mod close;
mod domains;
mod emulation;
mod preload;
mod targets;

use std::time::Duration;

use base64::Engine;
use serde_json::json;

use crate::error::{AgetError, ErrorCode};

pub(in crate::browser_cdp) struct PageSession {
    /// Empty for direct page WebSocket connections, where CDP commands already
    /// target the page and Chrome does not return a flattened target session.
    pub(in crate::browser_cdp) target_id: String,
    pub(in crate::browser_cdp) session_id: String,
}

impl super::CdpClient {
    pub(super) fn session_param<'a>(&self, session_id: &'a str) -> Option<&'a str> {
        if session_id.is_empty() {
            None
        } else {
            Some(session_id)
        }
    }

    pub(in crate::browser_cdp) fn capture_screenshot_png(
        &mut self,
        session_id: &str,
        timeout: Duration,
    ) -> Result<Vec<u8>, AgetError> {
        let result = self.send(
            "Page.captureScreenshot",
            Some(json!({
                "format": "png",
                "fromSurface": true,
            })),
            self.session_param(session_id),
            timeout,
        )?;
        let data = result
            .get("data")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| AgetError::Stable {
                code: ErrorCode::ExtractionFailed,
                message: "owned browser fallback screenshot response did not include image data"
                    .to_string(),
            })?;
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|error| AgetError::Stable {
                code: ErrorCode::ExtractionFailed,
                message: format!("owned browser fallback screenshot decode failed: {error}"),
            })
    }
}
