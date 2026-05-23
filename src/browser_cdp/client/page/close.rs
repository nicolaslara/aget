use std::time::Duration;

use serde_json::json;

use super::super::CdpClient;
use super::PageSession;
use crate::error::AgetError;

impl CdpClient {
    pub(in crate::browser_cdp) fn close_browser(
        &mut self,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        if self.direct_page_connection {
            return Ok(());
        }
        self.send("Browser.close", None, None, timeout)?;
        Ok(())
    }

    pub(in crate::browser_cdp) fn close_page(
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
}
