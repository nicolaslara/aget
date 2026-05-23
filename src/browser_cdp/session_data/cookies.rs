use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::session::PlaywrightCookie;

pub(in crate::browser_cdp) fn cdp_cookies(cookies: &[PlaywrightCookie]) -> Vec<Value> {
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

pub(in crate::browser_cdp) fn playwright_cookies_from_cdp(result: &Value) -> Vec<PlaywrightCookie> {
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

pub(in crate::browser_cdp) fn dedupe_playwright_cookies(
    cookies: Vec<PlaywrightCookie>,
) -> Vec<PlaywrightCookie> {
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
