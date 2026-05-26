use std::collections::BTreeMap;

use scraper::Html;
use serde_json::Value;

use crate::cli::OutputFormat;
use crate::error::AgetError;

use super::super::super::html_clean::{
    parse_css_selector, remove_owned_comments, remove_owned_consent_popups,
    remove_owned_excluded_domain_urls, remove_owned_excluded_tags, remove_owned_external_images,
    remove_owned_external_links, remove_owned_internal_links, remove_owned_overlay_elements,
    remove_owned_social_media_links, remove_selected_elements,
};
use super::super::super::http::OwnedHttpResponse;
use super::super::super::{extraction_failed, GetOptions};
use super::super::content::{extract_owned_content, markdown_base_url};
use super::super::metadata::extract_page_metadata;
use super::super::options::OwnedExtractorOptions;

pub(crate) struct OwnedPageExtraction {
    pub(crate) final_url: String,
    pub(crate) content: String,
    pub(crate) page_metadata: BTreeMap<String, Value>,
    pub(crate) warnings: Vec<String>,
    pub(crate) source_bytes: Option<usize>,
    pub(crate) screenshot_png: Option<Vec<u8>>,
}

pub(super) fn extract_owned_page_response(
    response: OwnedHttpResponse,
    options: &GetOptions,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let source_bytes = response.body.len();
    extract_owned_html(
        response.final_url,
        response.body,
        options,
        fallback_selector,
        owned_options,
        Some(source_bytes),
    )
}

pub(crate) fn extract_owned_html(
    final_url: String,
    body: String,
    options: &GetOptions,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
    source_bytes: Option<usize>,
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

    if owned_options.remove_consent_popups {
        document = remove_owned_consent_popups(document)?;
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

    let selector = options
        .selector
        .as_deref()
        .or(owned_options.css_selector.as_deref())
        .or(fallback_selector);
    let configured_base_url = owned_options.base_url.as_deref().unwrap_or_else(|| {
        if final_url.starts_with("raw:") {
            ""
        } else {
            &final_url
        }
    });
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
        OutputFormat::Html if owned_options.prettiify => fast_format_owned_html(&extracted.html),
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
        source_bytes,
        screenshot_png: None,
    })
}

fn fast_format_owned_html(html: &str) -> String {
    let mut indent = 0usize;
    let mut formatted = Vec::new();
    for part in html.replace('>', ">\n").replace('<', "\n<").split('\n') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part.starts_with("</") {
            indent = indent.saturating_sub(1);
            formatted.push(format!("{}{}", "  ".repeat(indent), part));
        } else if part.starts_with('<') && part.ends_with("/>") {
            formatted.push(format!("{}{}", "  ".repeat(indent), part));
        } else if part.starts_with('<') {
            formatted.push(format!("{}{}", "  ".repeat(indent), part));
            indent += 1;
        } else {
            formatted.push(format!("{}{}", "  ".repeat(indent), part));
        }
    }
    formatted.join("\n")
}
