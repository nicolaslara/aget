use crate::error::{AgetError, ErrorCode};

use super::artifacts::write_private_file;
use super::{
    io_aget_error, BrowserFallbackRequest, BrowserFallbackResult, ExtractorBackendResult,
    ExtractorRequest,
};

mod content;
mod metadata;
mod options;
mod page;
mod text;

pub(crate) use self::options::validate_owned_extraction_options;
pub(crate) use self::page::extract_owned_rendered_html;
use self::page::extract_owned_static_or_rendered;

pub(crate) const OWNED_EXTRACTOR: &str = "aget-owned-extractor";

const OWNED_BROWSER_FALLBACK: &str = "aget-owned-browser-fallback";
const OWNED_FALLBACK_WARNING: &str = "aget-owned fallback used after primary extractor failed";

pub(crate) fn run_owned_extractor_backend(
    request: ExtractorRequest<'_>,
) -> Result<ExtractorBackendResult, AgetError> {
    let tmp_dir = request
        .state_path
        .parent()
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::IoError,
            message: format!(
                "owned extractor state path '{}' has no temp directory",
                request.state_path.display()
            ),
        })?;
    let extraction = extract_owned_static_or_rendered(
        tmp_dir,
        request.url,
        request.state,
        request.options,
        request.timeout,
        None,
    )?;

    let backend_response = ExtractorBackendResult {
        ok: true,
        final_url: Some(extraction.final_url),
        content: Some(extraction.content.clone()),
        page_metadata: extraction.page_metadata,
        warnings: extraction.warnings,
        error: None,
    };
    write_private_file(request.content_path, extraction.content.as_bytes())
        .map_err(io_aget_error)?;
    let metadata =
        serde_json::to_vec_pretty(&backend_response).map_err(|error| AgetError::Stable {
            code: ErrorCode::IoError,
            message: error.to_string(),
        })?;
    write_private_file(request.metadata_path, &metadata).map_err(io_aget_error)?;
    Ok(backend_response)
}

pub(crate) fn run_owned_browser_fallback(
    request: BrowserFallbackRequest<'_>,
) -> Result<BrowserFallbackResult, AgetError> {
    let extraction = extract_owned_static_or_rendered(
        request.tmp_dir,
        request.url,
        request.state,
        request.options,
        request.timeout,
        Some("body"),
    )?;
    let mut warnings = vec![OWNED_FALLBACK_WARNING.to_string()];
    warnings.extend(extraction.warnings);
    Ok(BrowserFallbackResult {
        final_url: extraction.final_url,
        content: extraction.content,
        warnings,
        extractor: OWNED_BROWSER_FALLBACK.to_string(),
    })
}
