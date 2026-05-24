use html5ever::tree_builder::TreeSink;
use scraper::{Html, HtmlTreeSink};

use crate::error::AgetError;

use super::super::parse_css_selector;
use super::domains::{aget_like_base_domain, aget_like_url_base_domain, is_aget_like_external_url};

pub(super) fn remove_external_url_elements(
    document: Html,
    selector_list: &str,
    attribute: &str,
    base_url: &str,
) -> Result<Html, AgetError> {
    let selector = parse_css_selector(selector_list)?;
    let base_domain = aget_like_base_domain(base_url);
    let node_ids = document
        .select(&selector)
        .filter_map(|element| {
            let value = element.value().attr(attribute)?;
            is_aget_like_external_url(value, base_url, &base_domain).then_some(element.id())
        })
        .collect::<Vec<_>>();
    let tree = HtmlTreeSink::new(document);
    for id in node_ids {
        tree.remove_from_parent(&id);
    }
    Ok(tree.finish())
}

pub(super) fn remove_internal_url_elements(
    document: Html,
    selector_list: &str,
    attribute: &str,
    base_url: &str,
) -> Result<Html, AgetError> {
    let selector = parse_css_selector(selector_list)?;
    let base_domain = aget_like_base_domain(base_url);
    let node_ids = document
        .select(&selector)
        .filter_map(|element| {
            let value = element.value().attr(attribute)?;
            (!is_aget_like_external_url(value, base_url, &base_domain)).then_some(element.id())
        })
        .collect::<Vec<_>>();
    let tree = HtmlTreeSink::new(document);
    for id in node_ids {
        tree.remove_from_parent(&id);
    }
    Ok(tree.finish())
}

pub(super) fn remove_excluded_domain_url_elements(
    document: Html,
    selector_list: &str,
    attribute: &str,
    base_url: &str,
    excluded_domains: &[String],
) -> Result<Html, AgetError> {
    let selector = parse_css_selector(selector_list)?;
    let node_ids = document
        .select(&selector)
        .filter_map(|element| {
            let value = element.value().attr(attribute)?;
            let url_domain = aget_like_url_base_domain(value, base_url)?;
            excluded_domains
                .iter()
                .any(|excluded_domain| &url_domain == excluded_domain)
                .then_some(element.id())
        })
        .collect::<Vec<_>>();
    let tree = HtmlTreeSink::new(document);
    for id in node_ids {
        tree.remove_from_parent(&id);
    }
    Ok(tree.finish())
}
