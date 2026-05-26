use std::path::Path;
use std::time::Instant;

use crate::error::AgetError;

use super::SuccessfulExtraction;
use crate::extraction::output::{apply_limits, OutputOptions};
use crate::extraction::{
    io_aget_error, sanitize_backend_error, write_error_metadata, write_metadata,
    write_private_file, Artifacts, CacheMetadata, CacheStatus, GetOptions, GetSuccess, TimingMs,
    UsageMetrics,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::extraction) fn finalize_error(
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
pub(in crate::extraction) fn finalize_success(
    options: &GetOptions,
    content_path: &Path,
    metadata_path: &Path,
    selected_session_names: Vec<String>,
    sensitive: bool,
    output_options: OutputOptions,
    cache: CacheMetadata,
    extraction: SuccessfulExtraction,
    started: Instant,
) -> Result<GetSuccess, AgetError> {
    let limits = apply_limits(extraction.content, options.max_chars);
    let content = limits.content;
    write_private_file(content_path, content.as_bytes()).map_err(io_aget_error)?;
    let usage = usage_metrics(&content, extraction.source_bytes);

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
        cache,
        usage,
        output_options,
    };

    write_metadata(metadata_path, &success)?;
    Ok(success)
}

pub(in crate::extraction) fn disabled_cache_metadata(options: &GetOptions) -> CacheMetadata {
    CacheMetadata {
        status: CacheStatus::Disabled,
        policy: options.cache_policy,
        eligible: false,
        key: None,
        ttl_seconds: options.cache_ttl.as_secs(),
        age_seconds: None,
        reason: Some("cache unavailable for this extraction path".to_string()),
    }
}

fn usage_metrics(content: &str, fetched_bytes: Option<usize>) -> UsageMetrics {
    let estimated_tokens = estimate_tokens_from_chars(content.chars().count());
    let estimated_tokens_saved = fetched_bytes
        .map(|bytes| estimate_tokens_from_bytes(bytes).saturating_sub(estimated_tokens));
    UsageMetrics {
        fetched_bytes,
        content_bytes: content.len(),
        estimated_tokens,
        estimated_tokens_saved,
    }
}

fn estimate_tokens_from_bytes(bytes: usize) -> usize {
    bytes.div_ceil(4)
}

fn estimate_tokens_from_chars(chars: usize) -> usize {
    chars.div_ceil(4)
}
