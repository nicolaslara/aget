mod cookies;
mod source;
#[cfg(test)]
mod tests;

use std::time::Duration;

use ureq::ResponseExt;

use crate::error::{AgetError, ErrorCode};
use crate::session::PlaywrightState;

use self::cookies::cookie_header_for_state;
use self::source::ParsedRequestSource;
use super::extraction_failed;

#[derive(Debug, Clone)]
pub(super) struct OwnedHttpResponse {
    pub(super) final_url: String,
    pub(super) body: String,
    pub(super) can_auto_render: bool,
}

pub(super) fn owned_fetch(
    url: &str,
    state: &PlaywrightState,
    timeout: Duration,
    user_agent: Option<&str>,
) -> Result<OwnedHttpResponse, AgetError> {
    let source = ParsedRequestSource::parse(url)?;
    let ParsedRequestSource::Http(parsed) = source else {
        return source.into_local_response();
    };
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .max_redirects(5)
        .build();
    let agent: ureq::Agent = config.into();
    let mut request = agent
        .get(url)
        .header("User-Agent", user_agent.unwrap_or("aget/0.1"));
    if let Some(cookie_header) = cookie_header_for_state(state, &parsed) {
        request = request.header("Cookie", cookie_header);
    }

    let mut response = request.call().map_err(map_ureq_error)?;
    let final_url = response.get_uri().to_string();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(map_ureq_error)?;
    Ok(OwnedHttpResponse {
        final_url,
        body,
        can_auto_render: true,
    })
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
