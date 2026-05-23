use std::time::Duration;

use serde_json::{json, Value};

use super::{preferred_page_target_id, CdpClient};
use crate::browser_cdp::page_scripts::shadow_dom_attach_override_expression;
use crate::error::{AgetError, ErrorCode};

pub(in crate::browser_cdp) struct PageSession {
    /// Empty for direct page WebSocket connections, where CDP commands already
    /// target the page and Chrome does not return a flattened target session.
    pub(in crate::browser_cdp) target_id: String,
    pub(in crate::browser_cdp) session_id: String,
}

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
