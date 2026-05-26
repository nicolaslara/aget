use std::path::Path;
use std::time::Duration;

use crate::error::AgetError;
use crate::session::PlaywrightState;

use super::super::http::owned_fetch;
use super::super::GetOptions;
use super::options::validate_owned_extraction_options;

mod html;
mod local_input;
mod readiness;
mod rendered;

use html::extract_owned_page_response;
pub(super) use html::{extract_owned_html, OwnedPageExtraction};
use local_input::{
    browser_context_options_require_render, local_browser_render_input,
    should_route_local_input_through_browser,
};
use readiness::should_render_scripted_response;
use rendered::{
    extract_owned_rendered_page, extract_owned_rendered_page_with_url,
    should_retry_with_rendered_wait,
};

pub(super) fn extract_owned_static_or_rendered(
    tmp_dir: &Path,
    url: &str,
    state: &PlaywrightState,
    options: &GetOptions,
    timeout: Duration,
    fallback_selector: Option<&str>,
) -> Result<OwnedPageExtraction, AgetError> {
    let owned_options = validate_owned_extraction_options(options)?;
    if should_route_local_input_through_browser(options, &owned_options) {
        if let Some(local_input) = local_browser_render_input(tmp_dir, url)? {
            return extract_owned_rendered_page_with_url(
                tmp_dir,
                &local_input.render_url,
                Some(local_input.final_url),
                state,
                options,
                timeout,
                fallback_selector,
                &owned_options,
            );
        }
    }
    if owned_options.wait_for_images || owned_options.process_iframes {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }
    if browser_context_options_require_render(&owned_options) {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }
    if !state.origins.is_empty() {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }

    let response = owned_fetch(url, state, timeout, owned_options.user_agent.as_deref())?;
    let can_auto_render = response.can_auto_render;
    if can_auto_render && should_render_scripted_response(&response.body) {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }

    match extract_owned_page_response(response, options, fallback_selector, &owned_options) {
        Ok(extraction) => Ok(extraction),
        Err(error) if can_auto_render && should_retry_with_rendered_wait(&error, options) => {
            extract_owned_rendered_page(
                tmp_dir,
                url,
                state,
                options,
                timeout,
                fallback_selector,
                &owned_options,
            )
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn extract_owned_rendered_html(
    final_url: String,
    html: String,
    options: &GetOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let owned_options = validate_owned_extraction_options(options)?;
    let source_bytes = html.len();
    extract_owned_html(
        final_url,
        html,
        options,
        None,
        &owned_options,
        Some(source_bytes),
    )
}
