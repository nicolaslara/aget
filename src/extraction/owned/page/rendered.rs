use std::path::Path;
use std::time::Duration;

use crate::browser_cdp::BrowserRenderRequest;
use crate::error::{AgetError, ErrorCode};
use crate::session::PlaywrightState;

use super::super::super::GetOptions;
use super::super::options::OwnedExtractorOptions;
use super::{extract_owned_html, OwnedPageExtraction};

pub(super) fn extract_owned_rendered_page(
    tmp_dir: &Path,
    url: &str,
    state: &PlaywrightState,
    options: &GetOptions,
    timeout: Duration,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    extract_owned_rendered_page_with_url(
        tmp_dir,
        url,
        None,
        state,
        options,
        timeout,
        fallback_selector,
        owned_options,
    )
}

pub(super) fn extract_owned_rendered_page_with_url(
    tmp_dir: &Path,
    render_url: &str,
    final_url_override: Option<String>,
    state: &PlaywrightState,
    options: &GetOptions,
    timeout: Duration,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let rendered = crate::browser_cdp::render_page(BrowserRenderRequest {
        tmp_dir,
        url: render_url,
        state,
        wait_for_selector: options.wait_for_selector.as_deref(),
        wait_until: owned_options.wait_until,
        wait_for_images: owned_options.wait_for_images,
        scan_full_page: owned_options.scan_full_page,
        scroll_delay: owned_options.scroll_delay,
        max_scroll_steps: owned_options.max_scroll_steps,
        flatten_shadow_dom: owned_options.flatten_shadow_dom,
        process_iframes: owned_options.process_iframes,
        settle_delay: owned_options.render_settle_delay,
        page_timeout: owned_options.page_timeout.unwrap_or(timeout),
        wait_for_timeout: owned_options.wait_for_timeout,
        timeout,
    })?;
    let mut extraction = extract_owned_html(
        final_url_override.unwrap_or(rendered.final_url),
        rendered.html,
        options,
        fallback_selector,
        owned_options,
    )?;
    extraction.warnings.extend(rendered.warnings);
    Ok(extraction)
}

pub(super) fn should_retry_with_rendered_wait(error: &AgetError, options: &GetOptions) -> bool {
    if options.wait_for_selector.is_none() {
        return false;
    }
    matches!(
        error,
        AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message,
        } if message.starts_with("wait selector ") && message.ends_with(" was not found by owned extractor")
    )
}
