use crate::session::SessionCookie;

use super::super::PlaywrightCookie;

pub(super) fn same_cookie_for_composition(left: &SessionCookie, right: &SessionCookie) -> bool {
    normalize_cookie_name(&left.name) == normalize_cookie_name(&right.name)
        && left.value == right.value
        && normalize_cookie_domain(&left.domain) == normalize_cookie_domain(&right.domain)
        && normalize_cookie_path(&left.path) == normalize_cookie_path(&right.path)
        && left.expires == right.expires
        && left.http_only == right.http_only
        && left.secure == right.secure
        && left.same_site == right.same_site
}

pub(super) fn same_playwright_cookie(left: &PlaywrightCookie, right: &PlaywrightCookie) -> bool {
    normalize_cookie_name(&left.name) == normalize_cookie_name(&right.name)
        && left.value == right.value
        && normalize_cookie_domain(&left.domain) == normalize_cookie_domain(&right.domain)
        && normalize_cookie_path(&left.path) == normalize_cookie_path(&right.path)
        && left.expires == right.expires
        && left.http_only == right.http_only
        && left.secure == right.secure
        && left.same_site == right.same_site
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CookieKey {
    pub(super) name: String,
    pub(super) domain: String,
    pub(super) path: String,
}

impl From<&PlaywrightCookie> for CookieKey {
    fn from(cookie: &PlaywrightCookie) -> Self {
        Self {
            name: normalize_cookie_name(&cookie.name),
            domain: normalize_cookie_domain(&cookie.domain),
            path: normalize_cookie_path(&cookie.path),
        }
    }
}

impl From<&SessionCookie> for CookieKey {
    fn from(cookie: &SessionCookie) -> Self {
        Self {
            name: normalize_cookie_name(&cookie.name),
            domain: normalize_cookie_domain(&cookie.domain),
            path: normalize_cookie_path(&cookie.path),
        }
    }
}

fn normalize_cookie_name(name: &str) -> String {
    name.trim().to_string()
}

fn normalize_cookie_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

fn normalize_cookie_path(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    }
}

pub(super) fn normalize_playwright_cookie(mut cookie: PlaywrightCookie) -> PlaywrightCookie {
    cookie.name = normalize_cookie_name(&cookie.name);
    cookie.domain = normalize_cookie_domain(&cookie.domain);
    cookie.path = normalize_cookie_path(&cookie.path);
    cookie
}

pub(super) fn normalize_session_cookie(mut cookie: SessionCookie) -> SessionCookie {
    cookie.name = normalize_cookie_name(&cookie.name);
    cookie.domain = normalize_cookie_domain(&cookie.domain);
    cookie.path = normalize_cookie_path(&cookie.path);
    cookie
}
