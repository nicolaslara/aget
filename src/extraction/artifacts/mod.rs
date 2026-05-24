mod metadata;
mod private_files;
mod redaction;

use std::time::{SystemTime, UNIX_EPOCH};

pub(super) use metadata::{write_error_metadata, write_metadata};
pub(super) use private_files::{create_private_dir, write_private_file};
#[cfg(test)]
pub(super) use redaction::redact_values;
pub(super) use redaction::{sanitize_backend_artifacts, sanitize_backend_error, sensitive_values};

pub(super) fn run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("run-{}-{nanos}", std::process::id())
}
