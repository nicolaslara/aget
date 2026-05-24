mod cleanup;
mod main_content;
mod render;
mod selection;

use scraper::Html;

use cleanup::remove_owned_line_through_elements;
use main_content::default_main_content_element_id;
use render::{extract_single_owned_element, extract_target_owned_elements};
use selection::{collect_target_owned_element_ids, element_by_id, is_body_or_document_root};

use super::options::OwnedExtractorOptions;
use crate::error::AgetError;

use crate::extraction::html_clean::{
    clean_owned_base64_image_sources, parse_css_selector, prune_owned_unwanted_attributes,
    remove_owned_empty_elements, remove_selected_elements, replace_owned_only_text_elements,
};
use crate::extraction::markdown::resolve_markdown_url;

pub(super) struct ExtractedOwnedContent {
    pub(super) html: String,
    pub(super) markdown: String,
    pub(super) text: String,
}

pub(super) fn extract_owned_content(
    mut document: Html,
    selector: Option<&str>,
    base_url: &str,
    prefer_main_content: bool,
    owned_options: &OwnedExtractorOptions,
) -> Result<ExtractedOwnedContent, AgetError> {
    let root_ids = if let Some(raw_selector) = selector {
        match parse_css_selector(raw_selector) {
            Ok(selector) => {
                let selected = document
                    .select(&selector)
                    .map(|element| element.id())
                    .collect::<Vec<_>>();
                if selected.is_empty() {
                    vec![document.root_element().id()]
                } else {
                    selected
                }
            }
            Err(_) => vec![document.root_element().id()],
        }
    } else if !owned_options.target_elements.is_empty() {
        vec![document.root_element().id()]
    } else if prefer_main_content {
        vec![default_main_content_element_id(
            &document,
            owned_options.word_count_threshold,
        )?]
    } else {
        vec![document.root_element().id()]
    };
    let should_remove_page_chrome_fallback = prefer_main_content
        && selector.is_none()
        && is_body_or_document_root(&document, &root_ids)?;
    let target_ids = if owned_options.target_elements.is_empty() {
        Vec::new()
    } else {
        collect_target_owned_element_ids(&document, &root_ids, &owned_options.target_elements)?
    };

    if owned_options.hide_strikethrough {
        document = remove_owned_line_through_elements(document, &root_ids, &target_ids);
    }

    // Selectors see original attributes, but serialized cleaned HTML keeps only
    // its small important-attribute allowlist.
    if owned_options.only_text {
        document = replace_owned_only_text_elements(document, &root_ids, &target_ids);
    }
    document = clean_owned_base64_image_sources(document);
    if should_remove_page_chrome_fallback {
        document =
            remove_selected_elements(document, "nav,footer,header,aside,form,iframe,noscript")?;
    }
    document = remove_owned_empty_elements(document, &root_ids, &target_ids);
    document = prune_owned_unwanted_attributes(
        document,
        owned_options.keep_data_attributes,
        &owned_options.keep_attrs,
        owned_options.google_doc,
    );

    if owned_options.target_elements.is_empty() {
        if let [root_id] = root_ids.as_slice() {
            let root = element_by_id(&document, *root_id)?;
            return Ok(extract_single_owned_element(root, base_url, owned_options));
        }
        return extract_target_owned_elements(&document, &root_ids, base_url, owned_options);
    }
    extract_target_owned_elements(&document, &target_ids, base_url, owned_options)
}

pub(super) fn markdown_base_url(document: &Html, final_url: &str) -> Result<String, AgetError> {
    let selector = parse_css_selector("base[href]")?;
    let Some(base_href) = document
        .select(&selector)
        .next()
        .and_then(|element| element.attr("href"))
    else {
        return Ok(final_url.to_string());
    };
    Ok(resolve_markdown_url(final_url, base_href))
}
