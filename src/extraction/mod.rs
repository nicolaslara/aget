mod artifacts;
mod backends;
mod cache;
mod html_clean;
mod http;
mod markdown;
mod output;
mod owned;
mod pipeline;
mod replay_scope;
mod types;

use std::fs;
use std::time::{Duration, Instant};

#[cfg(test)]
use self::artifacts::redact_values;
use self::artifacts::{
    create_private_dir, run_id, sanitize_backend_artifacts, sanitize_backend_error,
    sensitive_values, write_error_metadata, write_metadata, write_private_file,
};
pub use self::backends::AgetExtractorBackend;
use self::cache::{CacheContext, CacheLookup};
use self::output::output_options;
pub use self::output::OutputOptions;
pub(crate) use self::owned::{
    extract_owned_rendered_html, run_owned_browser_fallback, run_owned_extractor_backend,
    validate_owned_extraction_options, OWNED_EXTRACTOR,
};
pub(crate) use self::pipeline::finish_direct_extraction;
use self::pipeline::{
    finalize_error, finalize_success, load_selected_sessions, run_primary_extractor,
    try_session_fallback,
};
use self::replay_scope::domain_matches_host;
use self::replay_scope::enforce_replay_scope;
pub use self::types::{
    Artifacts, BrowserFallbackBackend, BrowserFallbackRequest, BrowserFallbackResult,
    CacheMetadata, CacheStatus, ExtractionSessionStore, ExtractorBackend, ExtractorBackendResult,
    ExtractorRequest, GetOptions, GetSuccess, Limits, TimingMs, UsageMetrics,
};

use crate::aget::AgetBrowserBackend;
use crate::error::{AgetError, ErrorCode};
use crate::session::{compose_playwright_state, SessionStore, TempStateFile};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
pub fn get_url(options: GetOptions) -> Result<GetSuccess, AgetError> {
    let extractor = AgetExtractorBackend::default();
    let browser_fallback = AgetBrowserBackend::default();
    get_url_with_backends(options, &extractor, &browser_fallback)
}

pub fn get_url_with_backend(
    options: GetOptions,
    extractor_backend: &impl ExtractorBackend,
) -> Result<GetSuccess, AgetError> {
    let browser_fallback = AgetBrowserBackend::default();
    get_url_with_backends(options, extractor_backend, &browser_fallback)
}

pub fn get_url_with_backends(
    options: GetOptions,
    extractor_backend: &impl ExtractorBackend,
    browser_fallback_backend: &impl BrowserFallbackBackend,
) -> Result<GetSuccess, AgetError> {
    let store = match &options.home {
        Some(home) => SessionStore::new(home).map_err(io_aget_error)?,
        None => SessionStore::from_env().map_err(io_aget_error)?,
    };
    get_url_with_session_store(options, &store, extractor_backend, browser_fallback_backend)
}

pub fn get_url_with_session_store(
    options: GetOptions,
    store: &impl ExtractionSessionStore,
    extractor_backend: &impl ExtractorBackend,
    browser_fallback_backend: &impl BrowserFallbackBackend,
) -> Result<GetSuccess, AgetError> {
    let started = Instant::now();
    let sessions = load_selected_sessions(store, &options.sessions)?;
    enforce_replay_scope(&options.url, &sessions)?;
    let selected_session_names = sessions
        .iter()
        .map(|session| session.name.clone())
        .collect::<Vec<_>>();
    let sensitive = !sessions.is_empty();
    let output_options = output_options(&options);
    let state = compose_playwright_state(&sessions)?;
    let sensitive_values = sensitive_values(&state);
    let temp_state =
        TempStateFile::write(&store.home().join("tmp"), &state).map_err(io_aget_error)?;
    let run_dir = store.home().join("runs").join(run_id());
    create_private_dir(&run_dir).map_err(io_aget_error)?;

    let content_path = options
        .output
        .clone()
        .unwrap_or_else(|| run_dir.join("content.md"));
    if let Some(parent) = content_path.parent() {
        fs::create_dir_all(parent).map_err(io_aget_error)?;
    }
    let metadata_path = run_dir.join("metadata.json");

    let cache_context = match CacheContext::lookup(store.home(), &options, sensitive)? {
        CacheLookup::Hit {
            extraction,
            metadata,
        } => {
            return finalize_success(
                &options,
                &content_path,
                &metadata_path,
                selected_session_names,
                sensitive,
                output_options,
                metadata,
                extraction,
                started,
            );
        }
        CacheLookup::Fetch(context) => context,
    };

    let extraction = match run_primary_extractor(
        &options,
        &state,
        temp_state.path(),
        &content_path,
        &metadata_path,
        extractor_backend,
    ) {
        Ok(extraction) => extraction,
        Err(error) => {
            if let Some(fallback) = try_session_fallback(
                browser_fallback_backend,
                &store.home().join("tmp"),
                &options,
                &state,
                temp_state.path(),
                sensitive,
                &metadata_path,
                &sensitive_values,
            ) {
                fallback
            } else {
                return finalize_error(
                    &options,
                    &content_path,
                    &metadata_path,
                    extractor_backend.name(),
                    &selected_session_names,
                    sensitive,
                    &output_options,
                    error,
                    started,
                );
            }
        }
    };

    if sensitive {
        sanitize_backend_artifacts(&metadata_path, &sensitive_values);
    }

    cache_context.store(&options, &extraction)?;
    finalize_success(
        &options,
        &content_path,
        &metadata_path,
        selected_session_names,
        sensitive,
        output_options,
        cache_context.metadata(),
        extraction,
        started,
    )
}

fn extraction_failed(message: impl Into<String>) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: message.into(),
    }
}

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_literal_percent_encoded_and_json_escaped_values() {
        let value = "secret value \"quoted\"";
        let text = r#"literal=secret value "quoted" encoded=secret%20value%20%22quoted%22 lower=secret%20value%20%22quoted%22 form=secret+value+%22quoted%22 json=secret value \"quoted\""#;

        let redacted = redact_values(text, &[value.to_string()]);

        assert!(!redacted.contains("secret value \"quoted\""));
        assert!(!redacted.contains("secret%20value%20%22quoted%22"));
        assert!(!redacted.contains("secret+value+%22quoted%22"));
        assert!(!redacted.contains(r#"secret value \"quoted\""#));

        let newline_redacted = redact_values(
            "upper=line+one%0Abreak lower=line+one%0abreak",
            &["line one\nbreak".to_string()],
        );
        assert!(!newline_redacted.contains("line+one%0Abreak"));
        assert!(!newline_redacted.contains("line+one%0abreak"));
    }

    #[test]
    fn redacts_overlapping_values_longest_first() {
        let redacted = redact_values(
            "token=abcdef short=abc",
            &["abc".to_string(), "abcdef".to_string()],
        );

        assert!(!redacted.contains("abcdef"));
        assert!(!redacted.contains("abc"));
        assert!(!redacted.contains("<redacted>def"));
    }
}
