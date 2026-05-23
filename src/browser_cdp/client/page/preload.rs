use std::time::Duration;

use serde_json::json;

use super::super::CdpClient;
use crate::browser_cdp::page_scripts::shadow_dom_attach_override_expression;
use crate::error::AgetError;

impl CdpClient {
    pub(in crate::browser_cdp) fn force_open_shadow_roots(
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
}
