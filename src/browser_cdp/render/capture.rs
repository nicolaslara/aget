use std::thread;
use std::time::Duration;

use crate::error::AgetError;

use super::RenderedPage;
use crate::browser_cdp::client::{CdpClient, PageSession};
use crate::browser_cdp::page_scripts::shadow_dom_flatten_expression;

pub(super) struct AttachedPageCaptureOptions<'a> {
    pub(super) wait_for_selector: Option<&'a str>,
    pub(super) wait_for_images: bool,
    pub(super) scan_full_page: bool,
    pub(super) scroll_delay: Duration,
    pub(super) max_scroll_steps: usize,
    pub(super) flatten_shadow_dom: bool,
    pub(super) process_iframes: bool,
    pub(super) settle_delay: Duration,
    pub(super) page_timeout: Duration,
    pub(super) wait_for_timeout: Option<Duration>,
}

pub(super) fn capture_attached_page(
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
                "aget.scan_full_page failed; continuing with partial scroll: {error}"
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
            "some images did not finish loading before aget.wait_for_images timeout".to_string(),
        );
    }
    if !options.settle_delay.is_zero() {
        thread::sleep(options.settle_delay);
    }
    if options.process_iframes {
        match client.process_iframes(&page.session_id, options.page_timeout) {
            Ok((_total, _replaced, inaccessible)) if inaccessible > 0 => warnings.push(format!(
                "aget.process_iframes could not access {inaccessible} iframe(s); continuing with accessible content only"
            )),
            Ok(_) => {}
            Err(error) => warnings.push(format!(
                "aget.process_iframes failed; continuing with unprocessed iframes: {error}"
            )),
        }
    }
    if let Err(error) =
        client.remove_rendered_overlay_elements(&page.session_id, options.page_timeout)
    {
        warnings.push(format!("rendered overlay cleanup failed: {error}"));
    }
    let final_url =
        client.evaluate_string(&page.session_id, "location.href", options.page_timeout)?;
    let html = capture_page_html(client, &page.session_id, &mut warnings, &options)?;
    Ok(RenderedPage {
        final_url,
        html,
        warnings,
    })
}

fn capture_page_html(
    client: &mut CdpClient,
    session_id: &str,
    warnings: &mut Vec<String>,
    options: &AttachedPageCaptureOptions<'_>,
) -> Result<String, AgetError> {
    if options.flatten_shadow_dom {
        match client.evaluate_string(
            session_id,
            shadow_dom_flatten_expression(),
            options.page_timeout,
        ) {
            Ok(html) if !html.trim().is_empty() => return Ok(html),
            Ok(_) => warnings.push(
                "shadow DOM flattening returned no content; falling back to outerHTML".to_string(),
            ),
            Err(error) => warnings.push(format!(
                "shadow DOM flattening failed; falling back to outerHTML: {error}"
            )),
        }
    }
    client.evaluate_string(
        session_id,
        "document.documentElement.outerHTML || ''",
        options.page_timeout,
    )
}
