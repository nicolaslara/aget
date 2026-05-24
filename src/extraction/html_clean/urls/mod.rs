mod domains;
mod elements;

use scraper::Html;

use crate::error::AgetError;

use self::domains::aget_like_domain_from_option;
use self::elements::{
    remove_excluded_domain_url_elements, remove_external_url_elements, remove_internal_url_elements,
};
use super::CRAWL4AI_SOCIAL_MEDIA_DOMAINS;

pub(in crate::extraction) fn remove_owned_external_links(
    document: Html,
    base_url: &str,
) -> Result<Html, AgetError> {
    remove_external_url_elements(document, "a[href]", "href", base_url)
}

pub(in crate::extraction) fn remove_owned_internal_links(
    document: Html,
    base_url: &str,
) -> Result<Html, AgetError> {
    remove_internal_url_elements(document, "a[href]", "href", base_url)
}

pub(in crate::extraction) fn remove_owned_external_images(
    document: Html,
    base_url: &str,
) -> Result<Html, AgetError> {
    remove_external_url_elements(document, "img[src]", "src", base_url)
}

pub(in crate::extraction) fn remove_owned_excluded_domain_urls(
    document: Html,
    base_url: &str,
    excluded_domains: &[String],
) -> Result<Html, AgetError> {
    if excluded_domains.is_empty() {
        return Ok(document);
    }
    let excluded_domains = excluded_domains
        .iter()
        .filter_map(|domain| aget_like_domain_from_option(domain))
        .collect::<Vec<_>>();
    if excluded_domains.is_empty() {
        return Ok(document);
    }
    let document = remove_excluded_domain_url_elements(
        document,
        "a[href]",
        "href",
        base_url,
        &excluded_domains,
    )?;
    remove_excluded_domain_url_elements(document, "img[src]", "src", base_url, &excluded_domains)
}

pub(in crate::extraction) fn remove_owned_social_media_links(
    document: Html,
    base_url: &str,
    custom_social_domains: &[String],
) -> Result<Html, AgetError> {
    let mut excluded_domains = CRAWL4AI_SOCIAL_MEDIA_DOMAINS
        .iter()
        .map(|domain| (*domain).to_string())
        .collect::<Vec<_>>();
    excluded_domains.extend(custom_social_domains.iter().cloned());
    remove_excluded_domain_url_elements(document, "a[href]", "href", base_url, &excluded_domains)
}
