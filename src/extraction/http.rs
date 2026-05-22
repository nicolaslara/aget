use std::fs;
use std::time::Duration;

use ureq::ResponseExt;

use crate::error::{AgetError, ErrorCode};
use crate::session::PlaywrightState;

use super::{domain_matches_host, extraction_failed};

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
    Ok(OwnedHttpResponse {
        final_url,
        body,
        can_auto_render: true,
    })
}

#[derive(Debug, Clone)]
enum ParsedRequestSource<'a> {
    Http(ParsedRequestUrl),
    File { url: &'a str, path: &'a str },
    Raw { url: &'a str, html: &'a str },
}

impl<'a> ParsedRequestSource<'a> {
    fn parse(url: &'a str) -> Result<Self, AgetError> {
        if let Some(path) = url.strip_prefix("file://") {
            return Ok(Self::File { url, path });
        }
        if let Some(html) = url.strip_prefix("raw://") {
            return Ok(Self::Raw { url, html });
        }
        if let Some(html) = url.strip_prefix("raw:") {
            return Ok(Self::Raw { url, html });
        }
        Ok(Self::Http(ParsedRequestUrl::parse(url)?))
    }

    fn into_local_response(self) -> Result<OwnedHttpResponse, AgetError> {
        match self {
            Self::Http(_) => unreachable!("HTTP requests are handled by owned_fetch"),
            Self::File { url, path } => {
                let body = fs::read_to_string(path).map_err(|error| {
                    extraction_failed(format!("local file '{path}' could not be read: {error}"))
                })?;
                Ok(OwnedHttpResponse {
                    final_url: url.to_string(),
                    body,
                    can_auto_render: false,
                })
            }
            Self::Raw { url, html } => Ok(OwnedHttpResponse {
                final_url: url.to_string(),
                body: html.to_string(),
                can_auto_render: false,
            }),
        }
    }
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::session::PlaywrightState;

    use super::owned_fetch;

    fn empty_state() -> PlaywrightState {
        PlaywrightState {
            cookies: Vec::new(),
            origins: Vec::new(),
        }
    }

    #[test]
    fn owned_fetch_accepts_raw_colon_html_without_url_parsing() {
        let response = owned_fetch(
            "raw:<html><body><main>Raw</main></body></html>",
            &empty_state(),
            Duration::from_secs(1),
        )
        .unwrap();

        assert_eq!(
            response.final_url,
            "raw:<html><body><main>Raw</main></body></html>"
        );
        assert_eq!(response.body, "<html><body><main>Raw</main></body></html>");
        assert!(!response.can_auto_render);
    }

    #[test]
    fn owned_fetch_accepts_raw_slash_html_without_url_parsing() {
        let response = owned_fetch(
            "raw://<html><body><main>Raw slash</main></body></html>",
            &empty_state(),
            Duration::from_secs(1),
        )
        .unwrap();

        assert_eq!(
            response.body,
            "<html><body><main>Raw slash</main></body></html>"
        );
        assert!(!response.can_auto_render);
    }

    #[test]
    fn owned_fetch_reads_explicit_file_url() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("page.html");
        std::fs::write(&path, "<html><body><main>File</main></body></html>").unwrap();
        let url = format!("file://{}", path.display());

        let response = owned_fetch(&url, &empty_state(), Duration::from_secs(1)).unwrap();

        assert_eq!(response.final_url, url);
        assert_eq!(response.body, "<html><body><main>File</main></body></html>");
        assert!(!response.can_auto_render);
    }
}
