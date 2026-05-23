use std::time::Duration;

use serde_json::json;

use super::super::CdpClient;
use crate::error::AgetError;

impl CdpClient {
    pub(in crate::browser_cdp) fn set_user_agent_override(
        &mut self,
        session_id: &str,
        user_agent: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        self.send(
            "Network.setUserAgentOverride",
            Some(json!({ "userAgent": user_agent })),
            self.session_param(session_id),
            timeout,
        )?;
        Ok(())
    }

    pub(in crate::browser_cdp) fn set_locale_override(
        &mut self,
        session_id: &str,
        locale: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        self.send(
            "Emulation.setLocaleOverride",
            Some(json!({ "locale": locale })),
            self.session_param(session_id),
            timeout,
        )?;
        Ok(())
    }

    pub(in crate::browser_cdp) fn set_timezone_override(
        &mut self,
        session_id: &str,
        timezone_id: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        self.send(
            "Emulation.setTimezoneOverride",
            Some(json!({ "timezoneId": timezone_id })),
            self.session_param(session_id),
            timeout,
        )?;
        Ok(())
    }
}
