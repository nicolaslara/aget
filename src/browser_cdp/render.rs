use std::path::Path;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::error::AgetError;
use crate::session::PlaywrightState;

use super::chrome_process::ChromeProcess;
use super::client::CdpClient;
use super::page_scripts::shadow_dom_flatten_expression;
use super::CHROME_SHUTDOWN_WAIT;

pub(crate) struct BrowserRenderRequest<'a> {
    pub(crate) tmp_dir: &'a Path,
    pub(crate) url: &'a str,
    pub(crate) state: &'a PlaywrightState,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) wait_until: PageWaitUntil,
    pub(crate) wait_for_images: bool,
    pub(crate) flatten_shadow_dom: bool,
    pub(crate) settle_delay: Duration,
    pub(crate) page_timeout: Duration,
    pub(crate) wait_for_timeout: Option<Duration>,
    pub(crate) timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PageWaitUntil {
    DomContentLoaded,
    Load,
    NetworkIdle,
}

impl PageWaitUntil {
    pub(super) fn lifecycle_event_name(self) -> Option<&'static str> {
        match self {
            Self::DomContentLoaded => Some("Page.domContentEventFired"),
            Self::Load => Some("Page.loadEventFired"),
            Self::NetworkIdle => None,
        }
    }
}

pub(crate) struct RenderedPage {
    pub(crate) final_url: String,
    pub(crate) html: String,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn render_page(request: BrowserRenderRequest<'_>) -> Result<RenderedPage, AgetError> {
    let mut chrome =
        ChromeProcess::launch_temp(request.tmp_dir, request.timeout, "owned browser fallback")?;
    let mut client = CdpClient::connect(&chrome.ws_url, request.timeout)?;
    let page = client.create_page(request.page_timeout)?;
    client.enable_page_domains(&page.session_id, request.page_timeout)?;
    if request.flatten_shadow_dom {
        client.force_open_shadow_roots(&page.session_id, request.page_timeout)?;
    }
    client.load_state(&page.session_id, request.state, request.page_timeout)?;
    client.navigate_and_wait(
        &page.session_id,
        request.url,
        request.wait_until,
        request.page_timeout,
    )?;
    if let Some(selector) = request.wait_for_selector {
        client.wait_for_selector(
            &page.session_id,
            selector,
            request.wait_for_timeout.unwrap_or(request.page_timeout),
        )?;
    }
    let mut warnings = Vec::new();
    if request.wait_for_images && !client.wait_for_images_complete(&page.session_id)? {
        warnings.push(
            "some images did not finish loading before crawl4ai.wait_for_images timeout"
                .to_string(),
        );
    }
    if !request.settle_delay.is_zero() {
        thread::sleep(request.settle_delay);
    }
    if let Err(error) =
        client.remove_rendered_overlay_elements(&page.session_id, request.page_timeout)
    {
        warnings.push(format!("rendered overlay cleanup failed: {error}"));
    }
    let final_url =
        client.evaluate_string(&page.session_id, "location.href", request.page_timeout)?;
    let html = if request.flatten_shadow_dom {
        match client.evaluate_string(
            &page.session_id,
            shadow_dom_flatten_expression(),
            request.page_timeout,
        ) {
            Ok(html) if !html.trim().is_empty() => html,
            Ok(_) => {
                warnings.push(
                    "shadow DOM flattening returned no content; falling back to outerHTML"
                        .to_string(),
                );
                client.evaluate_string(
                    &page.session_id,
                    "document.documentElement.outerHTML || ''",
                    request.page_timeout,
                )?
            }
            Err(error) => {
                warnings.push(format!(
                    "shadow DOM flattening failed; falling back to outerHTML: {error}"
                ));
                client.evaluate_string(
                    &page.session_id,
                    "document.documentElement.outerHTML || ''",
                    request.page_timeout,
                )?
            }
        }
    } else {
        client.evaluate_string(
            &page.session_id,
            "document.documentElement.outerHTML || ''",
            request.page_timeout,
        )?
    };
    let _ = client.send(
        "Target.closeTarget",
        Some(json!({ "targetId": page.target_id })),
        None,
        Duration::from_secs(1),
    );
    let _ = client.send("Browser.close", None, None, Duration::from_secs(1));
    chrome.wait_or_kill(CHROME_SHUTDOWN_WAIT);
    Ok(RenderedPage {
        final_url,
        html,
        warnings,
    })
}
