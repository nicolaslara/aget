use std::time::Duration;

use ureq::ResponseExt;

use crate::error::{AgetError, ErrorCode};
use crate::session::PlaywrightState;

use super::{domain_matches_host, extraction_failed};

#[derive(Debug, Clone)]
pub(super) struct OwnedHttpResponse {
    pub(super) final_url: String,
    pub(super) body: String,
}

pub(super) fn owned_fetch(
    url: &str,
    state: &PlaywrightState,
    timeout: Duration,
) -> Result<OwnedHttpResponse, AgetError> {
    let parsed = ParsedRequestUrl::parse(url)?;
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .max_redirects(5)
        .build();
    let agent: ureq::Agent = config.into();
    let mut request = agent.get(url).header("User-Agent", "aget/0.1");
    if let Some(cookie_header) = cookie_header_for_state(state, &parsed) {
        request = request.header("Cookie", cookie_header);
    }

    let mut response = request.call().map_err(map_ureq_error)?;
    let final_url = response.get_uri().to_string();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(map_ureq_error)?;
    Ok(OwnedHttpResponse { final_url, body })
}

#[derive(Debug, Clone)]
struct ParsedRequestUrl {
    scheme: String,
    host: String,
    path: String,
}

impl ParsedRequestUrl {
    fn parse(url: &str) -> Result<Self, AgetError> {
        let uri = url.parse::<ureq::http::Uri>().map_err(|error| {
            extraction_failed(format!(
                "owned extractor received invalid URL '{url}': {error}"
            ))
        })?;
        let scheme = uri
            .scheme_str()
            .ok_or_else(|| extraction_failed(format!("request URL '{url}' is missing a scheme")))?;
        if !matches!(scheme, "http" | "https") {
            return Err(extraction_failed(format!(
                "owned extractor supports only http and https URLs, got '{scheme}'"
            )));
        }
        let host = uri
            .host()
            .ok_or_else(|| extraction_failed(format!("request URL '{url}' is missing a host")))?;
        let path = uri.path().to_string();
        Ok(Self {
            scheme: scheme.to_string(),
            host: host.to_string(),
            path,
        })
    }
}

fn cookie_header_for_state(state: &PlaywrightState, parsed: &ParsedRequestUrl) -> Option<String> {
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

fn map_ureq_error(error: ureq::Error) -> AgetError {
    match error {
        ureq::Error::Timeout(_) => AgetError::Stable {
            code: ErrorCode::Timeout,
            message: error.to_string(),
        },
        other => extraction_failed(other.to_string()),
    }
}
