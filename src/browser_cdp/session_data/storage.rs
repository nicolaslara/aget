use std::collections::BTreeSet;

use serde_json::Value;
use url::Url;

use crate::session::{PlaywrightOrigin, StorageEntry};

pub(in crate::browser_cdp) fn storage_candidate_origins(allowed_domains: &[String]) -> Vec<String> {
    let mut origins = BTreeSet::new();
    for domain in allowed_domains {
        let domain = domain
            .trim()
            .trim_start_matches('.')
            .trim_end_matches('/')
            .trim_end_matches('.');
        if domain.is_empty() {
            continue;
        }
        if domain.starts_with("http://") || domain.starts_with("https://") {
            origins.insert(domain.to_ascii_lowercase());
        } else {
            let domain = domain.to_ascii_lowercase();
            origins.insert(format!("https://{domain}"));
            origins.insert(format!("http://{domain}"));
        }
    }
    origins.into_iter().collect()
}

pub(in crate::browser_cdp) fn frame_storage_candidate_origins(
    frame_tree_result: &Value,
    allowed_domains: &[String],
) -> Vec<String> {
    let mut origins = BTreeSet::new();
    if let Some(frame_tree) = frame_tree_result.get("frameTree") {
        collect_allowed_frame_origins(frame_tree, allowed_domains, &mut origins);
    }
    origins.into_iter().collect()
}

fn collect_allowed_frame_origins(
    frame_tree: &Value,
    allowed_domains: &[String],
    origins: &mut BTreeSet<String>,
) {
    if let Some(frame) = frame_tree.get("frame") {
        if let Some(origin) = frame
            .get("url")
            .and_then(Value::as_str)
            .and_then(frame_url_origin)
            .filter(|origin| origin_allowed(origin, allowed_domains))
        {
            origins.insert(origin);
        }
    }
    if let Some(children) = frame_tree.get("childFrames").and_then(Value::as_array) {
        for child in children {
            collect_allowed_frame_origins(child, allowed_domains, origins);
        }
    }
}

fn frame_url_origin(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let origin = parsed.origin().ascii_serialization();
    if origin.is_empty() || origin == "null" {
        return None;
    }
    Some(origin)
}

fn origin_allowed(origin: &str, allowed_domains: &[String]) -> bool {
    let Some(host) = origin_host(origin) else {
        return false;
    };
    allowed_domains
        .iter()
        .any(|allowed| domain_matches_allowed(&host, allowed))
}

fn domain_matches_allowed(candidate_domain: &str, allowed_domain: &str) -> bool {
    let candidate = normalize_domain(candidate_domain);
    let allowed = allowed_domain_host(allowed_domain);
    if candidate.is_empty() || allowed.is_empty() {
        return false;
    }

    candidate == allowed || candidate.ends_with(&format!(".{allowed}"))
}

fn allowed_domain_host(allowed_domain: &str) -> String {
    if let Some(host) = origin_host(allowed_domain) {
        return host;
    }
    normalize_domain(allowed_domain)
}

fn origin_host(origin: &str) -> Option<String> {
    Url::parse(origin)
        .ok()
        .and_then(|url| url.host_str().map(normalize_domain))
        .filter(|host| !host.is_empty())
}

fn normalize_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

pub(in crate::browser_cdp) fn origin_storage_from_runtime_result(
    result: &Value,
) -> Option<PlaywrightOrigin> {
    let value = result
        .get("result")
        .and_then(|result| result.get("value"))?;
    let origin = value.get("origin")?.as_str()?;
    if origin.is_empty() || origin == "null" {
        return None;
    }
    let local_storage = value
        .get("localStorage")
        .and_then(|storage| serde_json::from_value::<Vec<StorageEntry>>(storage.clone()).ok())
        .unwrap_or_default();
    let session_storage = value
        .get("sessionStorage")
        .and_then(|storage| serde_json::from_value::<Vec<StorageEntry>>(storage.clone()).ok())
        .unwrap_or_default();
    Some(PlaywrightOrigin {
        origin: origin.to_string(),
        local_storage,
        session_storage,
    })
}
