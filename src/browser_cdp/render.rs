use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::error::AgetError;
use crate::session::PlaywrightState;

use super::chrome_process::ChromeProcess;
use super::client::{CdpClient, PageSession};
use super::page_scripts::shadow_dom_flatten_expression;
use super::CHROME_SHUTDOWN_WAIT;

pub(crate) struct BrowserRenderRequest<'a> {
    pub(crate) tmp_dir: &'a Path,
    pub(crate) url: &'a str,
    pub(crate) state: &'a PlaywrightState,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) wait_until: PageWaitUntil,
    pub(crate) wait_for_images: bool,
    pub(crate) scan_full_page: bool,
    pub(crate) scroll_delay: Duration,
    pub(crate) max_scroll_steps: usize,
    pub(crate) flatten_shadow_dom: bool,
    pub(crate) settle_delay: Duration,
    pub(crate) page_timeout: Duration,
    pub(crate) wait_for_timeout: Option<Duration>,
    pub(crate) timeout: Duration,
}

// Current-tab CLI/API wiring is still a follow-up; this owned renderer is the
// tested CDP primitive for extracting an already-open page without launching.
#[allow(dead_code)]
pub(crate) struct BrowserAttachedPageRenderRequest<'a> {
    pub(crate) ws_url: &'a str,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) wait_for_images: bool,
    pub(crate) scan_full_page: bool,
    pub(crate) scroll_delay: Duration,
    pub(crate) max_scroll_steps: usize,
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

#[derive(Debug)]
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
    let rendered = capture_attached_page(
        &mut client,
        &page,
        AttachedPageCaptureOptions {
            wait_for_selector: request.wait_for_selector,
            wait_for_images: request.wait_for_images,
            scan_full_page: request.scan_full_page,
            scroll_delay: request.scroll_delay,
            max_scroll_steps: request.max_scroll_steps,
            flatten_shadow_dom: request.flatten_shadow_dom,
            settle_delay: request.settle_delay,
            page_timeout: request.page_timeout,
            wait_for_timeout: request.wait_for_timeout,
        },
    )?;
    let _ = client.close_page(&page, Duration::from_secs(1));
    let _ = client.close_browser(Duration::from_secs(1));
    chrome.wait_or_kill(CHROME_SHUTDOWN_WAIT);
    Ok(rendered)
}

// Kept as a lower-level current-tab primitive until a public command chooses
// the consent and endpoint-discovery UX.
#[allow(dead_code)]
pub(crate) fn render_attached_page(
    request: BrowserAttachedPageRenderRequest<'_>,
) -> Result<RenderedPage, AgetError> {
    let mut client = CdpClient::connect(request.ws_url, request.timeout)?;
    let page = client
        .attach_existing_page(request.page_timeout)?
        .ok_or_else(|| AgetError::Stable {
            code: crate::error::ErrorCode::ExtractionFailed,
            message: "owned browser current-tab CDP attach found no page targets".to_string(),
        })?;
    client.enable_page_domains(&page.session_id, request.page_timeout)?;
    if request.flatten_shadow_dom {
        client.force_open_shadow_roots(&page.session_id, request.page_timeout)?;
    }
    capture_attached_page(
        &mut client,
        &page,
        AttachedPageCaptureOptions {
            wait_for_selector: request.wait_for_selector,
            wait_for_images: request.wait_for_images,
            scan_full_page: request.scan_full_page,
            scroll_delay: request.scroll_delay,
            max_scroll_steps: request.max_scroll_steps,
            flatten_shadow_dom: request.flatten_shadow_dom,
            settle_delay: request.settle_delay,
            page_timeout: request.page_timeout,
            wait_for_timeout: request.wait_for_timeout,
        },
    )
}

struct AttachedPageCaptureOptions<'a> {
    wait_for_selector: Option<&'a str>,
    wait_for_images: bool,
    scan_full_page: bool,
    scroll_delay: Duration,
    max_scroll_steps: usize,
    flatten_shadow_dom: bool,
    settle_delay: Duration,
    page_timeout: Duration,
    wait_for_timeout: Option<Duration>,
}

fn capture_attached_page(
    client: &mut CdpClient,
    page: &PageSession,
    options: AttachedPageCaptureOptions<'_>,
) -> Result<RenderedPage, AgetError> {
    let mut warnings = Vec::new();
    if options.scan_full_page {
        if let Err(error) = client.scan_full_page(
            &page.session_id,
            options.scroll_delay,
            options.max_scroll_steps,
            options.page_timeout,
        ) {
            warnings.push(format!(
                "crawl4ai.scan_full_page failed; continuing with partial scroll: {error}"
            ));
        }
    }
    if let Some(selector) = options.wait_for_selector {
        client.wait_for_selector(
            &page.session_id,
            selector,
            options.wait_for_timeout.unwrap_or(options.page_timeout),
        )?;
    }
    if options.wait_for_images && !client.wait_for_images_complete(&page.session_id)? {
        warnings.push(
            "some images did not finish loading before crawl4ai.wait_for_images timeout"
                .to_string(),
        );
    }
    if !options.settle_delay.is_zero() {
        thread::sleep(options.settle_delay);
    }
    if let Err(error) =
        client.remove_rendered_overlay_elements(&page.session_id, options.page_timeout)
    {
        warnings.push(format!("rendered overlay cleanup failed: {error}"));
    }
    let final_url =
        client.evaluate_string(&page.session_id, "location.href", options.page_timeout)?;
    let html = if options.flatten_shadow_dom {
        match client.evaluate_string(
            &page.session_id,
            shadow_dom_flatten_expression(),
            options.page_timeout,
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
                    options.page_timeout,
                )?
            }
            Err(error) => {
                warnings.push(format!(
                    "shadow DOM flattening failed; falling back to outerHTML: {error}"
                ));
                client.evaluate_string(
                    &page.session_id,
                    "document.documentElement.outerHTML || ''",
                    options.page_timeout,
                )?
            }
        }
    } else {
        client.evaluate_string(
            &page.session_id,
            "document.documentElement.outerHTML || ''",
            options.page_timeout,
        )?
    };
    Ok(RenderedPage {
        final_url,
        html,
        warnings,
    })
}
