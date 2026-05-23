use std::fs;

use crate::error::AgetError;

use super::super::extraction_failed;
use super::OwnedHttpResponse;

#[derive(Debug, Clone)]
pub(super) enum ParsedRequestSource<'a> {
    Http(ParsedRequestUrl),
    File { url: &'a str, path: &'a str },
    Raw { url: &'a str, html: &'a str },
}

impl<'a> ParsedRequestSource<'a> {
    pub(super) fn parse(url: &'a str) -> Result<Self, AgetError> {
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

    pub(super) fn into_local_response(self) -> Result<OwnedHttpResponse, AgetError> {
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
pub(super) struct ParsedRequestUrl {
    pub(super) scheme: String,
    pub(super) host: String,
    pub(super) path: String,
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
