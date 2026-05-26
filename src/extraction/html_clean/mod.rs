mod attributes;
#[cfg(test)]
mod tests;
mod urls;

use html5ever::tree_builder::TreeSink;
use scraper::{Html, HtmlTreeSink, Selector};

use crate::error::AgetError;

use super::extraction_failed;

pub(super) use self::attributes::{
    clean_owned_base64_image_sources, prune_owned_unwanted_attributes, remove_owned_comments,
    remove_owned_empty_elements, replace_owned_only_text_elements,
};
pub(super) use self::urls::{
    remove_owned_excluded_domain_urls, remove_owned_external_images, remove_owned_external_links,
    remove_owned_internal_links, remove_owned_social_media_links,
};

const AGET_OVERLAY_SELECTORS: &[&str] = &[
    r#"button[class*="close" i]"#,
    r#"button[class*="dismiss" i]"#,
    r#"button[aria-label*="close" i]"#,
    r#"button[title*="close" i]"#,
    r#"a[class*="close" i]"#,
    r#"span[class*="close" i]"#,
    r#"[class*="cookie-banner" i]"#,
    r#"[id*="cookie-banner" i]"#,
    r#"[class*="cookie-consent" i]"#,
    r#"[id*="cookie-consent" i]"#,
    r#"[class*="newsletter" i]"#,
    r#"[class*="subscribe" i]"#,
    r#"[class*="popup" i]"#,
    r#"[class*="modal" i]"#,
    r#"[class*="overlay" i]"#,
    r#"[class*="dialog" i]"#,
    r#"[role="dialog"]"#,
    r#"[role="alertdialog"]"#,
];
const AGET_CONSENT_SELECTORS: &[&str] = &[
    r#"[class*="cookie-consent" i]"#,
    r#"[id*="cookie-consent" i]"#,
    r#"[class*="cookie-banner" i]"#,
    r#"[id*="cookie-banner" i]"#,
    r#"[class*="cookie-notice" i]"#,
    r#"[id*="cookie-notice" i]"#,
    r#"[class*="cookie-law" i]"#,
    r#"[id*="cookie-law" i]"#,
    r#"[class*="cookie-popup" i]"#,
    r#"[id*="cookie-popup" i]"#,
    r#"[class*="cookie-overlay" i]"#,
    r#"[id*="cookie-overlay" i]"#,
    r#"[class*="gdpr" i]"#,
    r#"[id*="gdpr" i]"#,
    r#"iframe[title*="cookie" i]"#,
    r#"iframe[src*="cookie" i]"#,
];
const AGET_SOCIAL_MEDIA_DOMAINS: &[&str] = &[
    "facebook.com",
    "twitter.com",
    "x.com",
    "linkedin.com",
    "instagram.com",
    "pinterest.com",
    "tiktok.com",
    "snapchat.com",
    "reddit.com",
];

pub(super) fn remove_owned_excluded_tags(
    document: Html,
    tags: &[String],
) -> Result<Html, AgetError> {
    if tags.is_empty() {
        return Ok(document);
    }
    remove_selected_elements(document, &tags.join(","))
}

pub(super) fn remove_owned_overlay_elements(document: Html) -> Result<Html, AgetError> {
    remove_selected_elements(document, &AGET_OVERLAY_SELECTORS.join(","))
}

pub(super) fn remove_owned_consent_popups(document: Html) -> Result<Html, AgetError> {
    remove_selected_elements(document, &AGET_CONSENT_SELECTORS.join(","))
}

pub(super) fn remove_selected_elements(
    document: Html,
    selector_list: &str,
) -> Result<Html, AgetError> {
    let Ok(selector) = parse_css_selector(selector_list) else {
        return Ok(document);
    };
    let node_ids = document
        .select(&selector)
        .map(|element| element.id())
        .collect::<Vec<_>>();
    let tree = HtmlTreeSink::new(document);
    for id in node_ids {
        tree.remove_from_parent(&id);
    }
    Ok(tree.finish())
}

pub(super) fn parse_css_selector(raw: &str) -> Result<Selector, AgetError> {
    let selector = raw.trim().strip_prefix("css:").unwrap_or(raw.trim()).trim();
    if selector.is_empty() {
        return Err(extraction_failed(format!(
            "owned extractor received an empty CSS selector from '{raw}'"
        )));
    }
    Selector::parse(selector).map_err(|error| {
        extraction_failed(format!(
            "owned extractor could not parse CSS selector '{raw}': {error:?}"
        ))
    })
}
