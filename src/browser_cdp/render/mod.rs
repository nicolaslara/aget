mod capture;

use std::path::Path;
use std::time::Duration;

use crate::error::{AgetError, ErrorCode};
use crate::session::PlaywrightState;

use super::chrome_process::ChromeProcess;
use super::client::CdpClient;
use super::CHROME_SHUTDOWN_WAIT;
use capture::{capture_attached_page, AttachedPageCaptureOptions};

pub(crate) struct BrowserRenderRequest<'a> {
    pub(crate) tmp_dir: &'a Path,
    pub(crate) url: &'a str,
    pub(crate) state: &'a PlaywrightState,
    pub(crate) user_agent: Option<&'a str>,
    pub(crate) locale: Option<&'a str>,
    pub(crate) timezone_id: Option<&'a str>,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) wait_until: PageWaitUntil,
    pub(crate) wait_for_images: bool,
    pub(crate) scan_full_page: bool,
    pub(crate) scroll_delay: Duration,
    pub(crate) max_scroll_steps: usize,
    pub(crate) flatten_shadow_dom: bool,
    pub(crate) process_iframes: bool,
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
    pub(crate) locale: Option<&'a str>,
    pub(crate) timezone_id: Option<&'a str>,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) wait_for_images: bool,
    pub(crate) scan_full_page: bool,
    pub(crate) scroll_delay: Duration,
    pub(crate) max_scroll_steps: usize,
    pub(crate) flatten_shadow_dom: bool,
    pub(crate) process_iframes: bool,
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
    if let Some(user_agent) = request.user_agent {
        client.set_user_agent_override(&page.session_id, user_agent, request.page_timeout)?;
    }
    apply_browser_context_overrides(
        &mut client,
        &page.session_id,
        request.locale,
        request.timezone_id,
        request.page_timeout,
    )?;
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
            process_iframes: request.process_iframes,
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
            code: ErrorCode::ExtractionFailed,
            message: "owned browser current-tab CDP attach found no page targets".to_string(),
        })?;
    client.enable_page_domains(&page.session_id, request.page_timeout)?;
    apply_browser_context_overrides(
        &mut client,
        &page.session_id,
        request.locale,
        request.timezone_id,
        request.page_timeout,
    )?;
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
            process_iframes: request.process_iframes,
            settle_delay: request.settle_delay,
            page_timeout: request.page_timeout,
            wait_for_timeout: request.wait_for_timeout,
        },
    )
}

fn apply_browser_context_overrides(
    client: &mut CdpClient,
    session_id: &str,
    locale: Option<&str>,
    timezone_id: Option<&str>,
    timeout: Duration,
) -> Result<(), AgetError> {
    if let Some(locale) = locale {
        client.set_locale_override(session_id, locale, timeout)?;
    }
    if let Some(timezone_id) = timezone_id {
        client.set_timezone_override(session_id, timezone_id, timeout)?;
    }
    Ok(())
}
