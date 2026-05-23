use std::path::Path;

use crate::error::AgetError;
use crate::session::PlaywrightState;

use super::private_files::{read_output_file, write_private_file};

pub(in crate::extraction) fn sensitive_values(state: &PlaywrightState) -> Vec<String> {
    state
        .cookies
        .iter()
        .map(|cookie| cookie.value.clone())
        .chain(state.origins.iter().flat_map(|origin| {
            origin
                .local_storage
                .iter()
                .chain(origin.session_storage.iter())
                .map(|entry| entry.value.clone())
        }))
        .filter(|value| !value.is_empty())
        .collect()
}

pub(in crate::extraction) fn sanitize_backend_error(
    error: AgetError,
    sensitive: bool,
) -> AgetError {
    if !sensitive {
        return error;
    }

    match error {
        AgetError::Stable { code, .. } => AgetError::Stable {
            code,
            message: "Crawl4AI extraction failed for session-backed request".to_string(),
        },
    }
}

pub(in crate::extraction) fn sanitize_backend_artifacts(
    metadata_path: &Path,
    sensitive_values: &[String],
) {
    for path in [
        metadata_path.with_file_name("backend-stdout.json"),
        metadata_path.with_file_name("backend-stderr.txt"),
    ] {
        let Ok(text) = read_output_file(&path) else {
            continue;
        };
        let redacted = redact_values(&text, sensitive_values);
        if redacted != text {
            let _ = write_private_file(&path, redacted.as_bytes());
        }
    }
}

pub(in crate::extraction) fn redact_values(text: &str, sensitive_values: &[String]) -> String {
    let mut redacted = text.to_string();
    let mut patterns = sensitive_values
        .iter()
        .flat_map(|value| redaction_patterns(value))
        .collect::<Vec<_>>();
    patterns.sort_by_key(|pattern| std::cmp::Reverse(pattern.len()));
    patterns.dedup();
    for pattern in patterns {
        redacted = redacted.replace(&pattern, "<redacted>");
    }
    redacted
}

fn redaction_patterns(value: &str) -> Vec<String> {
    let mut patterns = vec![
        value.to_string(),
        percent_encode(value, PercentEncoding::Upper),
        percent_encode(value, PercentEncoding::Lower),
        form_encode(value, PercentEncoding::Upper),
        form_encode(value, PercentEncoding::Lower),
    ];
    if let Ok(encoded) = serde_json::to_string(value) {
        patterns.push(encoded.trim_matches('"').to_string());
    }
    patterns.sort_by_key(|pattern| std::cmp::Reverse(pattern.len()));
    patterns.dedup();
    patterns.retain(|pattern| !pattern.is_empty());
    patterns
}

#[derive(Clone, Copy)]
enum PercentEncoding {
    Upper,
    Lower,
}

fn percent_encode(value: &str, case: PercentEncoding) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            match case {
                PercentEncoding::Upper => encoded.push_str(&format!("%{byte:02X}")),
                PercentEncoding::Lower => encoded.push_str(&format!("%{byte:02x}")),
            }
        }
    }
    encoded
}

fn form_encode(value: &str, case: PercentEncoding) -> String {
    percent_encode(value, case).replace("%20", "+")
}
