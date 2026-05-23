use std::time::Duration;

use serde_json::json;

use super::super::CdpClient;
use crate::error::AgetError;

impl CdpClient {
    pub(in crate::browser_cdp) fn enable_page_domains(
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
}
