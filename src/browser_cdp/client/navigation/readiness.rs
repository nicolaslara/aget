use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::super::fail_on_runtime_evaluation_exception;
use super::super::transport::remaining;
use super::super::CdpClient;
use crate::browser_cdp::page_scripts::{
    full_page_scan_expression, iframe_process_expression, rendered_overlay_cleanup_expression,
    selector_exists_expression,
};
use crate::error::{AgetError, ErrorCode};

impl CdpClient {
    pub(in crate::browser_cdp) fn wait_for_selector(
        &mut self,
        session_id: &str,
        selector: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let expression = selector_exists_expression(selector)?;
        let deadline = Instant::now() + timeout;
        loop {
            let result = self.send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": false,
                })),
                self.session_param(session_id),
                remaining(deadline),
            )?;
            fail_on_runtime_evaluation_exception(&result)?;
            if result
                .get("result")
                .and_then(|result| result.get("value"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(AgetError::Stable {
                    code: ErrorCode::Timeout,
                    message: format!(
                        "owned browser fallback timed out waiting for selector '{selector}'"
                    ),
                });
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub(in crate::browser_cdp) fn wait_for_images_complete(
        &mut self,
        session_id: &str,
    ) -> Result<bool, AgetError> {
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if Instant::now() >= deadline {
                return Ok(false);
            }
            let result = self.send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": "Array.from(document.images).every((img) => img.complete)",
                    "returnByValue": true,
                    "awaitPromise": false,
                })),
                self.session_param(session_id),
                remaining(deadline),
            )?;
            fail_on_runtime_evaluation_exception(&result)?;
            if result
                .get("result")
                .and_then(|result| result.get("value"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub(in crate::browser_cdp) fn scan_full_page(
        &mut self,
        session_id: &str,
        scroll_delay: Duration,
        max_scroll_steps: usize,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let result = self.send(
            "Runtime.evaluate",
            Some(json!({
                "expression": full_page_scan_expression(scroll_delay, max_scroll_steps),
                "returnByValue": true,
                "awaitPromise": true,
            })),
            self.session_param(session_id),
            timeout,
        )?;
        fail_on_runtime_evaluation_exception(&result)?;
        Ok(())
    }

    pub(in crate::browser_cdp) fn remove_rendered_overlay_elements(
        &mut self,
        session_id: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let result = self.send(
            "Runtime.evaluate",
            Some(json!({
                "expression": rendered_overlay_cleanup_expression(),
                "returnByValue": true,
                "awaitPromise": true,
            })),
            self.session_param(session_id),
            timeout,
        )?;
        fail_on_runtime_evaluation_exception(&result)?;
        Ok(())
    }

    pub(in crate::browser_cdp) fn process_iframes(
        &mut self,
        session_id: &str,
        timeout: Duration,
    ) -> Result<(usize, usize, usize), AgetError> {
        let result = self.send(
            "Runtime.evaluate",
            Some(json!({
                "expression": iframe_process_expression(),
                "returnByValue": true,
                "awaitPromise": true,
            })),
            self.session_param(session_id),
            timeout,
        )?;
        fail_on_runtime_evaluation_exception(&result)?;
        let raw = result
            .get("result")
            .and_then(|result| result.get("value"))
            .and_then(Value::as_str)
            .ok_or_else(|| AgetError::Stable {
                code: ErrorCode::ExtractionFailed,
                message: "crawl4ai.process_iframes returned no iframe processing result"
                    .to_string(),
            })?;
        let summary: Value = serde_json::from_str(raw).map_err(|error| AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!("crawl4ai.process_iframes returned invalid summary JSON: {error}"),
        })?;
        Ok((
            summary
                .get("total")
                .and_then(Value::as_u64)
                .unwrap_or_default() as usize,
            summary
                .get("replaced")
                .and_then(Value::as_u64)
                .unwrap_or_default() as usize,
            summary
                .get("inaccessible")
                .and_then(Value::as_u64)
                .unwrap_or_default() as usize,
        ))
    }

    pub(in crate::browser_cdp) fn evaluate_string(
        &mut self,
        session_id: &str,
        expression: &str,
        timeout: Duration,
    ) -> Result<String, AgetError> {
        let result = self.send(
            "Runtime.evaluate",
            Some(json!({
                "expression": expression,
                "returnByValue": true,
                "awaitPromise": false,
            })),
            self.session_param(session_id),
            timeout,
        )?;
        fail_on_runtime_evaluation_exception(&result)?;
        Ok(result
            .get("result")
            .and_then(|result| result.get("value"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string())
    }
}
