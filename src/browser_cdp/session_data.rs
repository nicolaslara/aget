use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};
use url::Url;

use crate::session::{PlaywrightCookie, PlaywrightOrigin, StorageEntry};

pub(super) fn cdp_cookies(cookies: &[PlaywrightCookie]) -> Vec<Value> {
    cookies
        .iter()
        .map(|cookie| {
            let mut value = json!({
                "name": cookie.name,
                "value": cookie.value,
                "domain": cookie.domain,
                "path": cookie.path,
                "httpOnly": cookie.http_only,
                "secure": cookie.secure,
            });
            if let Some(expires) = cookie.expires {
                value["expires"] = json!(expires);
            }
            if let Some(same_site) = &cookie.same_site {
                value["sameSite"] = json!(same_site);
            }
            value
        })
        .collect()
}

pub(super) fn playwright_cookies_from_cdp(result: &Value) -> Vec<PlaywrightCookie> {
    result
        .get("cookies")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(playwright_cookie_from_cdp)
        .collect()
}

fn playwright_cookie_from_cdp(cookie: &Value) -> Option<PlaywrightCookie> {
    Some(PlaywrightCookie {
        name: cookie.get("name")?.as_str()?.to_string(),
        value: cookie.get("value")?.as_str()?.to_string(),
        domain: cookie.get("domain")?.as_str()?.to_string(),
        path: cookie
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("/")
            .to_string(),
        expires: cdp_cookie_expires(cookie.get("expires")),
        http_only: cookie
            .get("httpOnly")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        secure: cookie
            .get("secure")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        same_site: cookie
            .get("sameSite")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

pub(super) fn dedupe_playwright_cookies(cookies: Vec<PlaywrightCookie>) -> Vec<PlaywrightCookie> {
    let mut by_key = BTreeMap::new();
    for cookie in cookies {
        let key = (
            cookie.name.trim().to_string(),
            cookie
                .domain
                .trim()
                .trim_start_matches('.')
                .trim_end_matches('.')
                .to_ascii_lowercase(),
            if cookie.path.trim().is_empty() {
                "/".to_string()
            } else {
                cookie.path.trim().to_string()
            },
        );
        by_key.entry(key).or_insert(cookie);
    }
    by_key.into_values().collect()
}

fn cdp_cookie_expires(value: Option<&Value>) -> Option<i64> {
    let expires = value.and_then(Value::as_i64).or_else(|| {
        value
            .and_then(Value::as_f64)
            .map(|value| value.trunc() as i64)
    })?;
    (expires > 0).then_some(expires)
}

pub(super) fn storage_candidate_origins(allowed_domains: &[String]) -> Vec<String> {
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

pub(super) fn frame_storage_candidate_origins(
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

pub(super) fn origin_storage_from_runtime_result(result: &Value) -> Option<PlaywrightOrigin> {
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

pub(super) fn preferred_page_target_id(target_infos: &[Value]) -> Option<String> {
    let mut fallback = None;
    for target in target_infos {
        let Some((target_id, url)) = trackable_page_target(target) else {
            continue;
        };
        fallback.get_or_insert_with(|| target_id.to_string());
        if !url.is_empty() && url != "about:blank" {
            return Some(target_id.to_string());
        }
    }
    fallback
}

fn trackable_page_target(target: &Value) -> Option<(&str, &str)> {
    let target_type = target.get("type").and_then(Value::as_str)?;
    if target_type != "page" && target_type != "webview" {
        return None;
    }
    let url = target
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if is_internal_chrome_target(url) {
        return None;
    }
    let target_id = target.get("targetId").and_then(Value::as_str)?;
    Some((target_id, url))
}

fn is_internal_chrome_target(url: &str) -> bool {
    url.starts_with("chrome://")
        || url.starts_with("chrome-extension://")
        || url.starts_with("devtools://")
}
