use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use crate::error::{AgetError, ErrorCode};

use super::super::{io_aget_error, GetOptions, GetSuccess, OutputOptions};
use super::private_files::write_private_file;

#[allow(clippy::too_many_arguments)]
pub(in crate::extraction) fn write_error_metadata(
    path: &Path,
    url: &str,
    content_path: &Path,
    extractor: &str,
    sessions: &[String],
    sensitive: bool,
    output_options: &OutputOptions,
    options: &GetOptions,
    error: &AgetError,
    started: Instant,
) -> Result<(), AgetError> {
    let (code, message) = match error {
        AgetError::Stable { code, message } => (code, message),
    };
    let metadata = serde_json::json!({
        "ok": false,
        "url": url,
        "content_format": options.content_format.to_string(),
        "extractor": extractor,
        "artifacts": {
            "content": content_path.to_string_lossy(),
            "metadata": path.to_string_lossy(),
        },
        "sessions": sessions,
        "sensitive": sensitive,
        "warnings": [],
        "timing_ms": {"total": started.elapsed().as_millis()},
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
    let bytes = serde_json::to_vec_pretty(&metadata).map_err(|error| AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    })?;
    write_private_file(path, &bytes).map_err(io_aget_error)
}

pub(in crate::extraction) fn write_metadata(
    path: &Path,
    success: &GetSuccess,
) -> Result<(), AgetError> {
    let mut metadata = BTreeMap::new();
    metadata.insert("ok", serde_json::json!(success.ok));
    metadata.insert("url", serde_json::json!(success.url));
    metadata.insert("final_url", serde_json::json!(success.final_url));
    metadata.insert("content_format", serde_json::json!(success.content_format));
    metadata.insert("extractor", serde_json::json!(success.extractor));
    metadata.insert("page_metadata", serde_json::json!(success.page_metadata));
    metadata.insert("artifacts", serde_json::json!(success.artifacts));
    metadata.insert("sessions", serde_json::json!(success.sessions));
    metadata.insert("sensitive", serde_json::json!(success.sensitive));
    metadata.insert("warnings", serde_json::json!(success.warnings));
    metadata.insert("timing_ms", serde_json::json!(success.timing_ms));
    metadata.insert("limits", serde_json::json!(success.limits));
    metadata.insert("output_options", serde_json::json!(success.output_options));
    let bytes = serde_json::to_vec_pretty(&metadata).map_err(|error| AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    })?;
    write_private_file(path, &bytes).map_err(io_aget_error)
}
