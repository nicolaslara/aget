use html5ever::tree_builder::TreeSink;
use scraper::{Html, HtmlTreeSink};
use url::Url;

use crate::error::AgetError;

use super::{parse_css_selector, CRAWL4AI_SOCIAL_MEDIA_DOMAINS};

pub(in crate::extraction) fn remove_owned_external_links(
    document: Html,
    base_url: &str,
) -> Result<Html, AgetError> {
    remove_external_url_elements(document, "a[href]", "href", base_url)
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
        .filter_map(|domain| crawl4ai_like_domain_from_option(domain))
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

fn remove_external_url_elements(
    document: Html,
    selector_list: &str,
    attribute: &str,
    base_url: &str,
) -> Result<Html, AgetError> {
    let selector = parse_css_selector(selector_list)?;
    let base_domain = crawl4ai_like_base_domain(base_url);
    let node_ids = document
        .select(&selector)
        .filter_map(|element| {
            let value = element.value().attr(attribute)?;
            is_crawl4ai_like_external_url(value, base_url, &base_domain).then_some(element.id())
        })
        .collect::<Vec<_>>();
    let tree = HtmlTreeSink::new(document);
    for id in node_ids {
        tree.remove_from_parent(&id);
    }
    Ok(tree.finish())
}

fn remove_excluded_domain_url_elements(
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
            let url_domain = crawl4ai_like_url_base_domain(value, base_url)?;
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

fn is_crawl4ai_like_external_url(raw_url: &str, base_url: &str, base_domain: &str) -> bool {
    let raw_url = raw_url.trim();
    if is_crawl4ai_special_url(raw_url) {
        return true;
    }
    let Ok(url) = Url::parse(raw_url)
        .or_else(|_| Url::parse(base_url).and_then(|base_url| base_url.join(raw_url)))
    else {
        return false;
    };
    let Some(url_host) = url.host_str() else {
        return false;
    };
    let url_domain = normalize_crawl4ai_domain(url_host);
    !url_domain.ends_with(base_domain)
}

fn is_crawl4ai_special_url(raw_url: &str) -> bool {
    let lower = raw_url.to_ascii_lowercase();
    ["mailto:", "tel:", "ftp:", "file:", "data:", "javascript:"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

fn crawl4ai_like_base_domain(raw_url: &str) -> String {
    Url::parse(raw_url)
        .ok()
        .and_then(|url| url.host_str().map(normalize_crawl4ai_domain))
        .map(|domain| {
            let parts = domain.split('.').collect::<Vec<_>>();
            if parts.len() > 2
                && matches!(
                    parts[parts.len() - 2],
                    "co" | "com"
                        | "org"
                        | "gov"
                        | "edu"
                        | "net"
                        | "mil"
                        | "int"
                        | "ac"
                        | "ad"
                        | "ae"
                        | "af"
                        | "ag"
                )
            {
                parts[parts.len() - 3..].join(".")
            } else if parts.len() >= 2 {
                parts[parts.len() - 2..].join(".")
            } else {
                domain
            }
        })
        .unwrap_or_default()
}

fn crawl4ai_like_url_base_domain(raw_url: &str, base_url: &str) -> Option<String> {
    Url::parse(raw_url)
        .or_else(|_| Url::parse(base_url).and_then(|base_url| base_url.join(raw_url)))
        .ok()
        .map(|url| crawl4ai_like_base_domain(url.as_str()))
        .filter(|domain| !domain.is_empty())
}

fn crawl4ai_like_domain_from_option(raw_domain: &str) -> Option<String> {
    let raw_domain = raw_domain.trim();
    if raw_domain.is_empty() {
        return None;
    }
    if let Ok(url) = Url::parse(raw_domain) {
        return Some(crawl4ai_like_base_domain(url.as_str())).filter(|domain| !domain.is_empty());
    }
    let domain = raw_domain
        .split('/')
        .next()
        .unwrap_or(raw_domain)
        .split(':')
        .next()
        .unwrap_or(raw_domain);
    Some(crawl4ai_like_base_domain(&format!("https://{domain}")))
        .filter(|domain| !domain.is_empty())
}

fn normalize_crawl4ai_domain(host: &str) -> String {
    let domain = host.trim_end_matches('.').to_ascii_lowercase();
    domain.strip_prefix("www.").unwrap_or(&domain).to_string()
}
