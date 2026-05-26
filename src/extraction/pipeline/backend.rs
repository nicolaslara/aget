use std::fs;
use std::path::Path;

use crate::error::{AgetError, ErrorCode};
use crate::session::PlaywrightState;

use super::SuccessfulExtraction;
use crate::extraction::{
    sanitize_backend_artifacts, BrowserFallbackBackend, BrowserFallbackRequest, ExtractorBackend,
    ExtractorRequest, GetOptions, DEFAULT_TIMEOUT,
};

pub(in crate::extraction) fn run_primary_extractor(
    options: &GetOptions,
    state: &PlaywrightState,
    state_path: &Path,
    content_path: &Path,
    metadata_path: &Path,
    extractor_backend: &impl ExtractorBackend,
) -> Result<SuccessfulExtraction, AgetError> {
    let backend = extractor_backend.extract(ExtractorRequest {
        url: &options.url,
        state,
        state_path,
        content_path,
        metadata_path,
        options,
        timeout: options.timeout.unwrap_or(DEFAULT_TIMEOUT),
    })?;

    if !backend.ok {
        return Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: backend
                .error
                .unwrap_or_else(|| "primary extraction failed".to_string()),
        });
    }

    let content = match backend.content {
        Some(content) => content,
        None => fs::read_to_string(content_path).map_err(|error| AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!(
                "backend did not return content and content artifact could not be read: {error}"
            ),
        })?,
    };

    Ok(SuccessfulExtraction {
        final_url: backend.final_url.unwrap_or_else(|| options.url.clone()),
        content,
        page_metadata: backend.page_metadata,
        warnings: backend.warnings,
        extractor: extractor_backend.name().to_string(),
        source_bytes: backend.source_bytes,
        screenshot_png: backend.screenshot_png,
    })
}

pub(in crate::extraction) fn try_session_fallback(
    browser_fallback_backend: &impl BrowserFallbackBackend,
    tmp_dir: &Path,
    options: &GetOptions,
    state: &PlaywrightState,
    state_path: &Path,
    sensitive: bool,
    metadata_path: &Path,
    sensitive_values: &[String],
) -> Option<SuccessfulExtraction> {
    if !sensitive {
        return None;
    }

    sanitize_backend_artifacts(metadata_path, sensitive_values);
    browser_fallback_backend
        .extract_with_state(BrowserFallbackRequest {
            tmp_dir,
            url: &options.url,
            state,
            state_path,
            options,
            timeout: options.timeout.unwrap_or(DEFAULT_TIMEOUT),
        })
        .ok()
        .map(|fallback| SuccessfulExtraction {
            final_url: fallback.final_url,
            content: fallback.content,
            page_metadata: fallback.page_metadata,
            warnings: fallback.warnings,
            extractor: fallback.extractor,
            source_bytes: fallback.source_bytes,
            screenshot_png: fallback.screenshot_png,
        })
}
