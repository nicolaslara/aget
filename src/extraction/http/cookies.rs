use crate::session::PlaywrightState;

use super::super::domain_matches_host;
use super::source::ParsedRequestUrl;

pub(super) fn cookie_header_for_state(
    state: &PlaywrightState,
    parsed: &ParsedRequestUrl,
) -> Option<String> {
    let cookies = state
        .cookies
        .iter()
        .filter(|cookie| !cookie.secure || parsed.scheme == "https")
        .filter(|cookie| domain_matches_host(&parsed.host, &cookie.domain))
        .filter(|cookie| request_path_matches_cookie_path(&parsed.path, &cookie.path))
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<_>>();
    (!cookies.is_empty()).then(|| cookies.join("; "))
}

fn request_path_matches_cookie_path(request_path: &str, cookie_path: &str) -> bool {
    let cookie_path = if cookie_path.is_empty() {
        "/"
    } else {
        cookie_path
    };
    request_path == cookie_path
        || (request_path.starts_with(cookie_path)
            && (cookie_path.ends_with('/')
                || request_path
                    .as_bytes()
                    .get(cookie_path.len())
                    .is_some_and(|byte| *byte == b'/')))
}
