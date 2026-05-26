use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::error::AgetError;

use super::SuccessfulExtraction;
use crate::extraction::output::{apply_limits, OutputOptions};
use crate::extraction::{
    io_aget_error, sanitize_backend_error, write_error_metadata, write_metadata,
    write_private_bytes, write_private_file, Artifacts, CacheMetadata, CacheStatus, DebugArtifact,
    DebugArtifacts, GetOptions, GetSuccess, TimingMs, UsageMetrics,
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
    let debug_artifacts = write_error_debug_artifacts(
        options,
        metadata_path,
        selected_session_names,
        sensitive,
        extractor,
        output_options,
        &error,
        started,
    )?;
    let _ = write_error_metadata(
        metadata_path,
        &options.url,
        content_path,
        extractor,
        selected_session_names,
        sensitive,
        output_options,
        options,
        &debug_artifacts,
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
    let mut warnings = extraction.warnings;
    let debug_artifacts = write_success_debug_artifacts(
        options,
        metadata_path,
        &selected_session_names,
        sensitive,
        &extraction.final_url,
        &extraction.extractor,
        &mut warnings,
        &output_options,
        &cache,
        &usage,
        &limits.metadata,
        extraction.screenshot_png,
        started,
    )?;

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
            debug: debug_artifacts,
        },
        sessions: selected_session_names,
        sensitive,
        warnings,
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

#[allow(clippy::too_many_arguments)]
fn write_success_debug_artifacts(
    options: &GetOptions,
    metadata_path: &Path,
    selected_session_names: &[String],
    sensitive: bool,
    final_url: &str,
    extractor: &str,
    warnings: &mut Vec<String>,
    output_options: &OutputOptions,
    cache: &CacheMetadata,
    usage: &UsageMetrics,
    limits: &crate::extraction::Limits,
    screenshot_png: Option<Vec<u8>>,
    started: Instant,
) -> Result<DebugArtifacts, AgetError> {
    let run_dir = run_dir_from_metadata_path(metadata_path)?;
    let mut artifacts = DebugArtifacts::default();
    if options.debug.screenshot {
        if let Some(bytes) = screenshot_png {
            let path = run_dir.join("screenshot.png");
            write_private_bytes(&path, &bytes).map_err(io_aget_error)?;
            artifacts.screenshot = Some(debug_artifact(path, "image/png", sensitive));
        } else {
            warnings.push(
                "screenshot requested but no browser-rendered screenshot was captured".to_string(),
            );
        }
    }
    if options.debug.trace {
        let path = run_dir.join("debug-trace.json");
        let trace = serde_json::json!({
            "schema_version": "aget.debug_trace.v1",
            "ok": true,
            "url": redact_url_for_debug(&options.url, sensitive),
            "final_url": redact_url_for_debug(final_url, sensitive),
            "content_format": options.content_format.to_string(),
            "extractor": extractor,
            "sessions": selected_session_names,
            "sensitive": sensitive,
            "capture": {
                "screenshot_requested": options.debug.screenshot,
                "screenshot_captured": artifacts.screenshot.is_some(),
                "trace_requested": true,
            },
            "warnings": warnings,
            "timing_ms": {"total": started.elapsed().as_millis()},
            "limits": limits,
            "cache": cache,
            "usage": usage,
            "output_options": output_options,
        });
        let bytes = serde_json::to_vec_pretty(&trace).map_err(|error| AgetError::Stable {
            code: crate::error::ErrorCode::IoError,
            message: error.to_string(),
        })?;
        write_private_file(&path, &bytes).map_err(io_aget_error)?;
        artifacts.trace = Some(debug_artifact(path, "application/json", sensitive));
    }
    Ok(artifacts)
}

#[allow(clippy::too_many_arguments)]
fn write_error_debug_artifacts(
    options: &GetOptions,
    metadata_path: &Path,
    selected_session_names: &[String],
    sensitive: bool,
    extractor: &str,
    output_options: &OutputOptions,
    error: &AgetError,
    started: Instant,
) -> Result<DebugArtifacts, AgetError> {
    if !options.debug.trace {
        return Ok(DebugArtifacts::default());
    }
    let run_dir = run_dir_from_metadata_path(metadata_path)?;
    let path = run_dir.join("debug-trace.json");
    let (code, message) = match error {
        AgetError::Stable { code, message } => (code, message),
    };
    let trace = serde_json::json!({
        "schema_version": "aget.debug_trace.v1",
        "ok": false,
        "url": redact_url_for_debug(&options.url, sensitive),
        "content_format": options.content_format.to_string(),
        "extractor": extractor,
        "sessions": selected_session_names,
        "sensitive": sensitive,
        "capture": {
            "screenshot_requested": options.debug.screenshot,
            "screenshot_captured": false,
            "trace_requested": true,
        },
        "warnings": [],
        "timing_ms": {"total": started.elapsed().as_millis()},
        "cache": {
            "status": "disabled",
            "policy": options.cache_policy,
            "eligible": false,
            "key": null,
            "ttl_seconds": options.cache_ttl.as_secs(),
            "age_seconds": null,
            "reason": "request failed before cacheable content was available",
        },
        "usage": {
            "fetched_bytes": null,
            "content_bytes": 0,
            "estimated_tokens": 0,
            "estimated_tokens_saved": null,
        },
        "limits": {
            "max_chars": options.max_chars,
            "truncated": false,
            "truncated_by": null,
            "content_chars_before_truncation": 0,
            "content_chars_after_truncation": 0,
        },
        "output_options": output_options,
        "error": {"code": code, "message": message},
    });
    let bytes = serde_json::to_vec_pretty(&trace).map_err(|error| AgetError::Stable {
        code: crate::error::ErrorCode::IoError,
        message: error.to_string(),
    })?;
    write_private_file(&path, &bytes).map_err(io_aget_error)?;
    let mut artifacts = DebugArtifacts::default();
    artifacts.trace = Some(debug_artifact(path, "application/json", sensitive));
    Ok(artifacts)
}

fn run_dir_from_metadata_path(metadata_path: &Path) -> Result<PathBuf, AgetError> {
    metadata_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| AgetError::Stable {
            code: crate::error::ErrorCode::IoError,
            message: format!(
                "debug artifact metadata path '{}' has no run directory",
                metadata_path.display()
            ),
        })
}

fn debug_artifact(path: PathBuf, media_type: &str, sensitive: bool) -> DebugArtifact {
    DebugArtifact {
        path: path.to_string_lossy().into_owned(),
        media_type: media_type.to_string(),
        sensitive,
    }
}

fn redact_url_for_debug(value: &str, sensitive: bool) -> String {
    if !sensitive {
        return value.to_string();
    }
    let Ok(mut url) = url::Url::parse(value) else {
        return "<redacted>".to_string();
    };
    if url.query().is_some() {
        url.set_query(Some("<redacted>"));
    }
    if url.fragment().is_some() {
        url.set_fragment(Some("<redacted>"));
    }
    url.to_string()
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
