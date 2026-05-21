use std::time::Duration;

use serde_json::json;

use super::{
    cdp_cookies, origin_storage_from_runtime_result, playwright_cookies_from_cdp,
    storage_candidate_origins, CdpClient,
};
use crate::browser_cdp::page_scripts::{
    local_storage_set_expression, origin_storage_expression, session_storage_set_expression,
};
use crate::browser_cdp::session_data::dedupe_playwright_cookies;
use crate::browser_cdp::PageWaitUntil;
use crate::error::AgetError;
use crate::session::{PlaywrightCookie, PlaywrightOrigin, PlaywrightState};

impl CdpClient {
    pub(in crate::browser_cdp) fn load_state(
        &mut self,
        session_id: &str,
        state: &PlaywrightState,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        if !state.cookies.is_empty() {
            self.send(
                "Network.setCookies",
                Some(json!({ "cookies": cdp_cookies(&state.cookies) })),
                Some(session_id),
                timeout,
            )?;
        }

        for origin in &state.origins {
            if origin.local_storage.is_empty() && origin.session_storage.is_empty() {
                continue;
            }
            let navigate_url = format!("{}/", origin.origin.trim_end_matches('/'));
            self.navigate_and_wait(session_id, &navigate_url, PageWaitUntil::Load, timeout)?;
            for entry in &origin.local_storage {
                let expression = local_storage_set_expression(&entry.name, &entry.value)?;
                self.send(
                    "Runtime.evaluate",
                    Some(json!({
                        "expression": expression,
                        "returnByValue": true,
                        "awaitPromise": false,
                    })),
                    Some(session_id),
                    timeout,
                )?;
            }
            for entry in &origin.session_storage {
                let expression = session_storage_set_expression(&entry.name, &entry.value)?;
                self.send(
                    "Runtime.evaluate",
                    Some(json!({
                        "expression": expression,
                        "returnByValue": true,
                        "awaitPromise": false,
                    })),
                    Some(session_id),
                    timeout,
                )?;
            }
        }
        Ok(())
    }

    pub(in crate::browser_cdp) fn export_state(
        &mut self,
        session_id: &str,
        allowed_domains: &[String],
        timeout: Duration,
    ) -> Result<PlaywrightState, AgetError> {
        let cookies = self.collect_cookies(session_id, allowed_domains, timeout)?;
        let origins = self.collect_storage_origins(session_id, allowed_domains, timeout)?;
        Ok(PlaywrightState { cookies, origins })
    }

    fn collect_cookies(
        &mut self,
        session_id: &str,
        allowed_domains: &[String],
        timeout: Duration,
    ) -> Result<Vec<PlaywrightCookie>, AgetError> {
        let result = self.send("Network.getAllCookies", None, Some(session_id), timeout)?;
        let mut cookies = playwright_cookies_from_cdp(&result);

        let urls = storage_candidate_origins(allowed_domains)
            .into_iter()
            .map(|origin| format!("{}/", origin.trim_end_matches('/')))
            .collect::<Vec<_>>();
        if !urls.is_empty() {
            let result = self.send(
                "Network.getCookies",
                Some(json!({ "urls": urls })),
                Some(session_id),
                timeout,
            )?;
            cookies.extend(playwright_cookies_from_cdp(&result));
        }

        if cookies.is_empty() {
            let result = self.send("Storage.getCookies", None, Some(session_id), timeout)?;
            cookies.extend(playwright_cookies_from_cdp(&result));
        }

        Ok(dedupe_playwright_cookies(cookies))
    }

    fn collect_storage_origins(
        &mut self,
        session_id: &str,
        allowed_domains: &[String],
        timeout: Duration,
    ) -> Result<Vec<PlaywrightOrigin>, AgetError> {
        let candidate_origins = storage_candidate_origins(allowed_domains);
        if candidate_origins.is_empty() {
            return Ok(Vec::new());
        }

        self.send(
            "Fetch.enable",
            Some(json!({ "patterns": [{ "urlPattern": "*" }] })),
            Some(session_id),
            timeout,
        )?;

        let mut origins = Vec::new();
        for origin in candidate_origins {
            let navigate_url = format!("{}/", origin.trim_end_matches('/'));
            self.navigate_with_blank_response(session_id, &navigate_url, timeout)?;
            let result = self.send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": origin_storage_expression(),
                    "returnByValue": true,
                    "awaitPromise": false,
                })),
                Some(session_id),
                timeout,
            )?;
            let Some(origin) = origin_storage_from_runtime_result(&result) else {
                continue;
            };
            if !origin.local_storage.is_empty() || !origin.session_storage.is_empty() {
                origins.push(origin);
            }
        }

        let _ = self.send(
            "Fetch.disable",
            None,
            Some(session_id),
            Duration::from_secs(1),
        );
        Ok(origins)
    }
}
