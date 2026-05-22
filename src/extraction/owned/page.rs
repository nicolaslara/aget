use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use scraper::Html;
use serde_json::Value;

use crate::cli::OutputFormat;
use crate::error::AgetError;
use crate::session::PlaywrightState;

use super::super::html_clean::{
    parse_css_selector, remove_owned_comments, remove_owned_excluded_domain_urls,
    remove_owned_excluded_tags, remove_owned_external_images, remove_owned_external_links,
    remove_owned_internal_links, remove_owned_overlay_elements, remove_owned_social_media_links,
    remove_selected_elements,
};
use super::super::http::{owned_fetch, OwnedHttpResponse};
use super::super::{extraction_failed, GetOptions};
use super::content::{extract_owned_content, markdown_base_url};
use super::metadata::extract_page_metadata;
use super::options::{validate_owned_extraction_options, OwnedExtractorOptions};

mod readiness;
mod rendered;

use readiness::should_render_scripted_response;
use rendered::{extract_owned_rendered_page, should_retry_with_rendered_wait};

pub(crate) struct OwnedPageExtraction {
    pub(crate) final_url: String,
    pub(crate) content: String,
    pub(crate) page_metadata: BTreeMap<String, Value>,
    pub(crate) warnings: Vec<String>,
}

pub(super) fn extract_owned_static_or_rendered(
    tmp_dir: &Path,
    url: &str,
    state: &PlaywrightState,
    options: &GetOptions,
    timeout: Duration,
    fallback_selector: Option<&str>,
) -> Result<OwnedPageExtraction, AgetError> {
    let owned_options = validate_owned_extraction_options(options)?;
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

    let response = owned_fetch(url, state, timeout)?;
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
    extract_owned_html(final_url, html, options, None, &owned_options)
}

fn extract_owned_page_response(
    response: OwnedHttpResponse,
    options: &GetOptions,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    extract_owned_html(
        response.final_url,
        response.body,
        options,
        fallback_selector,
        owned_options,
    )
}

fn extract_owned_html(
    final_url: String,
    body: String,
    options: &GetOptions,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let mut document = Html::parse_document(&body);
    let page_metadata = extract_page_metadata(&document)?;
    document = remove_owned_comments(document);
    document = remove_selected_elements(document, "script,style,link,meta,noscript")?;

    if let Some(wait_for) = &options.wait_for_selector {
        let selector = parse_css_selector(wait_for)?;
        if document.select(&selector).next().is_none() {
            return Err(extraction_failed(format!(
                "wait selector '{wait_for}' was not found by owned extractor"
            )));
        }
    }

    if owned_options.remove_overlay_elements {
        document = remove_owned_overlay_elements(document)?;
    }

    if owned_options.remove_forms {
        document = remove_selected_elements(document, "form")?;
    }

    if !owned_options.excluded_tags.is_empty() {
        document = remove_owned_excluded_tags(document, &owned_options.excluded_tags)?;
    }

    if let Some(exclude_selector) = &options.exclude_selector {
        document = remove_selected_elements(document, exclude_selector)?;
    }

    for excluded_selector in &owned_options.excluded_selectors {
        document = remove_selected_elements(document, excluded_selector)?;
    }

    if owned_options.exclude_all_images {
        document = remove_selected_elements(document, "img")?;
    }

    let selector = options.selector.as_deref().or(fallback_selector);
    let configured_base_url = owned_options.base_url.as_deref().unwrap_or(&final_url);
    let base_url = markdown_base_url(&document, configured_base_url)?;

    if !owned_options.exclude_domains.is_empty() {
        document =
            remove_owned_excluded_domain_urls(document, &base_url, &owned_options.exclude_domains)?;
    }

    if owned_options.exclude_external_links {
        document = remove_owned_external_links(document, &base_url)?;
    }

    if owned_options.exclude_internal_links {
        document = remove_owned_internal_links(document, &base_url)?;
    }

    if owned_options.exclude_social_media_links {
        document = remove_owned_social_media_links(
            document,
            &base_url,
            &owned_options.exclude_social_media_domains,
        )?;
    }

    if owned_options.exclude_external_images {
        document = remove_owned_external_images(document, &base_url)?;
    }

    let prefer_main_content = selector.is_none()
        && options.wait_for_selector.is_none()
        && options.content_format != OutputFormat::Html;
    let extracted = extract_owned_content(
        document,
        selector,
        &base_url,
        prefer_main_content,
        owned_options,
    )?;
    let content = match options.content_format {
        OutputFormat::Html => extracted.html,
        OutputFormat::Json => serde_json::json!({
            "url": final_url,
            "content": extracted.text,
        })
        .to_string(),
        OutputFormat::Markdown => extracted.markdown,
        OutputFormat::Text => extracted.text,
    };

    Ok(OwnedPageExtraction {
        final_url,
        content,
        page_metadata,
        warnings: Vec::new(),
    })
}
