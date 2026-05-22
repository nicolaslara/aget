use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use serde_json::Value;

use crate::error::{AgetError, ErrorCode};
use crate::session::{PlaywrightState, Session, SessionStore};

use super::output::{apply_limits, output_options, OutputOptions};
use super::{
    create_private_dir, io_aget_error, run_id, sanitize_backend_artifacts, sanitize_backend_error,
    write_error_metadata, write_metadata, write_private_file, Artifacts, BrowserFallbackBackend,
    BrowserFallbackRequest, ExtractionSessionStore, ExtractorBackend, ExtractorRequest, GetOptions,
    GetSuccess, TimingMs, DEFAULT_TIMEOUT,
};

pub(super) struct SuccessfulExtraction {
    final_url: String,
    content: String,
    page_metadata: BTreeMap<String, Value>,
    warnings: Vec<String>,
    extractor: String,
}

pub(crate) fn finish_direct_extraction(
    mut options: GetOptions,
    final_url: String,
    content: String,
    warnings: Vec<String>,
    extractor: impl Into<String>,
    sensitive: bool,
    started: Instant,
) -> Result<GetSuccess, AgetError> {
    if options.home.is_none() {
        let store = SessionStore::from_env().map_err(io_aget_error)?;
        options.home = Some(store.home().to_path_buf());
    }
    let home = options.home.clone().ok_or_else(|| AgetError::Stable {
        code: ErrorCode::IoError,
        message: "direct extraction requires an aget home directory".to_string(),
    })?;
    let run_dir = home.join("runs").join(run_id());
    create_private_dir(&run_dir).map_err(io_aget_error)?;
    let content_path = options
        .output
        .clone()
        .unwrap_or_else(|| run_dir.join("content.md"));
    if let Some(parent) = content_path.parent() {
        fs::create_dir_all(parent).map_err(io_aget_error)?;
    }
    let metadata_path = run_dir.join("metadata.json");
    let output_options = output_options(&options);
    finalize_success(
        &options,
        &content_path,
        &metadata_path,
        Vec::new(),
        sensitive,
        output_options,
        SuccessfulExtraction {
            final_url,
            content,
            page_metadata: BTreeMap::new(),
            warnings,
            extractor: extractor.into(),
        },
        started,
    )
}

pub(super) fn run_primary_extractor(
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
                .unwrap_or_else(|| "Crawl4AI extraction failed".to_string()),
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
    })
}

pub(super) fn try_session_fallback(
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
        })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finalize_error(
    options: &GetOptions,
    content_path: &Path,
    metadata_path: &Path,
    extractor: &str,
    selected_session_names: &[String],
    sensitive: bool,
    output_options: &OutputOptions,
    error: AgetError,
    started: Instant,
) -> Result<GetSuccess, AgetError> {
    let error = sanitize_backend_error(error, sensitive);
    let _ = write_error_metadata(
        metadata_path,
        &options.url,
        content_path,
        extractor,
        selected_session_names,
        sensitive,
        output_options,
        options,
        &error,
        started,
    );
    Err(error)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finalize_success(
    options: &GetOptions,
    content_path: &Path,
    metadata_path: &Path,
    selected_session_names: Vec<String>,
    sensitive: bool,
    output_options: OutputOptions,
    extraction: SuccessfulExtraction,
    started: Instant,
) -> Result<GetSuccess, AgetError> {
    let limits = apply_limits(extraction.content, options.max_chars);
    let content = limits.content;
    write_private_file(content_path, content.as_bytes()).map_err(io_aget_error)?;

    let success = GetSuccess {
        ok: true,
        url: options.url.clone(),
        final_url: extraction.final_url,
        content_format: options.content_format.to_string(),
        extractor: extraction.extractor,
        content,
        page_metadata: extraction.page_metadata,
        artifacts: Artifacts {
            content: content_path.to_string_lossy().into_owned(),
            metadata: metadata_path.to_string_lossy().into_owned(),
        },
        sessions: selected_session_names,
        sensitive,
        warnings: extraction.warnings,
        timing_ms: TimingMs {
            total: started.elapsed().as_millis(),
        },
        limits: limits.metadata,
        output_options,
    };

    write_metadata(metadata_path, &success)?;
    Ok(success)
}

pub(super) fn load_selected_sessions(
    store: &impl ExtractionSessionStore,
    session_names: &[String],
) -> Result<Vec<Session>, AgetError> {
    session_names
        .iter()
        .map(|name| store.load(name).map_err(io_aget_error))
        .collect()
}
