mod artifacts;
mod command;
mod fallback_command;
mod html_clean;
mod http;
mod markdown;
mod owned;

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[cfg(test)]
use self::artifacts::redact_values;
use self::artifacts::{
    create_private_dir, create_private_file, read_output_file, run_id, sanitize_backend_artifacts,
    sanitize_backend_error, sensitive_values, write_error_metadata, write_metadata,
    write_private_file,
};
use self::command::run_command_extractor_backend;
use self::fallback_command::run_agent_browser_fallback;
pub(crate) use self::owned::{
    run_owned_browser_fallback, run_owned_extractor_backend, OWNED_EXTRACTOR,
};

use crate::aget::AgetBrowserBackend;
use crate::aget_extractor::AgetExtractor;
use crate::cli::{ExtractorOption, OutputFormat};
use crate::error::{AgetError, ErrorCode};
use crate::session::agent_browser::origin_host;
use crate::session::{
    compose_playwright_state, PlaywrightState, Session, SessionStore, TempStateFile,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const EXTRACTOR: &str = "crawl4ai";
const FALLBACK_EXTRACTOR: &str = "agent-browser-fallback";
const FALLBACK_WARNING: &str = "agent-browser fallback used after Crawl4AI failed";

#[derive(Clone)]
pub struct GetOptions {
    pub url: String,
    pub sessions: Vec<String>,
    pub output: Option<PathBuf>,
    pub home: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub content_format: OutputFormat,
    pub selector: Option<String>,
    pub exclude_selector: Option<String>,
    pub wait_for_selector: Option<String>,
    pub max_chars: Option<usize>,
    pub backend_options: Vec<ExtractorOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSuccess {
    pub ok: bool,
    pub url: String,
    pub final_url: String,
    pub content_format: String,
    pub extractor: String,
    pub content: String,
    pub artifacts: Artifacts,
    pub sessions: Vec<String>,
    pub sensitive: bool,
    pub warnings: Vec<String>,
    pub timing_ms: TimingMs,
    pub limits: Limits,
    pub output_options: OutputOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifacts {
    pub content: String,
    pub metadata: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMs {
    pub total: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub max_chars: Option<usize>,
    pub truncated: bool,
    pub truncated_by: Option<String>,
    pub content_chars_before_truncation: usize,
    pub content_chars_after_truncation: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputOptions {
    pub content_format: OutputFormat,
    pub selector: Option<String>,
    pub exclude_selector: Option<String>,
    pub wait_for_selector: Option<String>,
    pub backend_options: BTreeMap<String, String>,
}

pub struct ExtractorRequest<'a> {
    pub url: &'a str,
    // Structured session state lets in-process extractors avoid re-reading the
    // temp Playwright file that exists for command-backed compatibility.
    pub state: &'a PlaywrightState,
    pub state_path: &'a Path,
    pub content_path: &'a Path,
    pub metadata_path: &'a Path,
    pub options: &'a GetOptions,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorBackendResult {
    pub ok: bool,
    #[serde(default)]
    pub final_url: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub error: Option<String>,
}

pub trait ExtractorBackend {
    // Extraction is the "URL plus session state to content" capability. The
    // default implementation is owned Rust extraction, but the rest of the
    // pipeline should not know whether content came from an owned or compatibility
    // backend.
    fn name(&self) -> &'static str;
    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError>;
}

pub struct BrowserFallbackRequest<'a> {
    pub tmp_dir: &'a Path,
    pub url: &'a str,
    // Keep the structured state alongside the temp file path so future browser
    // backends can load cookies/storage without coupling to command adapter I/O.
    pub state: &'a PlaywrightState,
    pub state_path: &'a Path,
    pub options: &'a GetOptions,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct BrowserFallbackResult {
    pub final_url: String,
    pub content: String,
    pub warnings: Vec<String>,
    pub extractor: String,
}

pub trait BrowserFallbackBackend {
    // Fallback browser extraction handles authenticated pages when the primary
    // extractor cannot consume the composed session state directly.
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError>;
}

pub trait ExtractionSessionStore {
    // `aget get` only needs scoped session lookup plus a local home for private
    // run artifacts. Keeping this separate lets `AgetWith` use non-filesystem
    // stores without changing the extraction pipeline.
    fn home(&self) -> &Path;
    fn load(&self, name: &str) -> io::Result<Session>;
}

impl ExtractionSessionStore for SessionStore {
    fn home(&self) -> &Path {
        self.home()
    }

    fn load(&self, name: &str) -> io::Result<Session> {
        self.load(name)
    }
}

#[derive(Debug, Clone)]
pub struct CommandBrowserFallbackBackend;

impl BrowserFallbackBackend for CommandBrowserFallbackBackend {
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        run_agent_browser_fallback(
            request.tmp_dir,
            request.url,
            request.state_path,
            request.options,
            request.timeout,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CommandExtractorBackend {
    command: Option<String>,
}

impl CommandExtractorBackend {
    pub fn new(command: Option<String>) -> Self {
        Self { command }
    }
}

impl ExtractorBackend for CommandExtractorBackend {
    fn name(&self) -> &'static str {
        EXTRACTOR
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        run_command_extractor_backend(self.command.clone(), request)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AgetExtractorBackend {
    extractor: AgetExtractor,
}

impl AgetExtractorBackend {
    pub fn new(extractor: AgetExtractor) -> Self {
        Self { extractor }
    }
}

impl ExtractorBackend for AgetExtractorBackend {
    fn name(&self) -> &'static str {
        self.extractor.name()
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        self.extractor.extract(request)
    }
}

#[derive(Debug, Clone, Default)]
pub struct OwnedExtractorBackend;

impl ExtractorBackend for OwnedExtractorBackend {
    fn name(&self) -> &'static str {
        AgetExtractorBackend::default().name()
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        AgetExtractorBackend::default().extract(request)
    }
}

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

    finalize_success(
        &options,
        &content_path,
        &metadata_path,
        selected_session_names,
        sensitive,
        output_options,
        extraction,
        started,
    )
}

struct SuccessfulExtraction {
    final_url: String,
    content: String,
    warnings: Vec<String>,
    extractor: String,
}

fn run_primary_extractor(
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
        warnings: backend.warnings,
        extractor: extractor_backend.name().to_string(),
    })
}

fn try_session_fallback(
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
            warnings: fallback.warnings,
            extractor: fallback.extractor,
        })
}

#[allow(clippy::too_many_arguments)]
fn finalize_error(
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
fn finalize_success(
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

fn load_selected_sessions(
    store: &impl ExtractionSessionStore,
    session_names: &[String],
) -> Result<Vec<Session>, AgetError> {
    session_names
        .iter()
        .map(|name| store.load(name).map_err(io_aget_error))
        .collect()
}

fn enforce_replay_scope(url: &str, sessions: &[Session]) -> Result<(), AgetError> {
    if sessions.is_empty() {
        return Ok(());
    }

    let target_host = origin_host(url).ok_or_else(|| AgetError::Stable {
        code: ErrorCode::UsageError,
        message: format!("invalid request URL '{url}'"),
    })?;

    for session in sessions {
        if let Some(scope_error) = session_replay_scope_error(session, &target_host) {
            return Err(AgetError::Stable {
                code: ErrorCode::PrivacyPolicyBlocked,
                message: scope_error,
            });
        }
    }

    Ok(())
}

fn session_replay_scope_error(session: &Session, target_host: &str) -> Option<String> {
    let mut has_matching_scope = session
        .allowed_cookie_domains
        .iter()
        .any(|domain| domain_matches_host(target_host, domain));

    for cookie in &session.cookies {
        if domain_matches_host(target_host, &cookie.domain) {
            has_matching_scope = true;
        } else {
            return Some(format!(
                "session '{}' contains cookie state for '{}' outside request host '{}'",
                session.name,
                normalize_domain(&cookie.domain),
                target_host
            ));
        }
    }

    for origin in &session.allowed_storage_origins {
        if let Some(host) = origin_host(origin) {
            has_matching_scope |= domain_matches_host(target_host, &host);
        }
    }

    for origin in &session.origins {
        let host = origin_host(&origin.origin).unwrap_or_else(|| origin.origin.clone());
        if domain_matches_host(target_host, &host) {
            has_matching_scope = true;
        } else {
            return Some(format!(
                "session '{}' contains storage state for '{}' outside request host '{}'",
                session.name, origin.origin, target_host
            ));
        }
    }

    if has_matching_scope {
        None
    } else {
        Some(format!(
            "session '{}' is not scoped for request host '{}'",
            session.name, target_host
        ))
    }
}

fn domain_matches_host(host: &str, allowed_domain: &str) -> bool {
    let host = normalize_domain(host);
    let allowed = normalize_domain(allowed_domain);
    if host.is_empty() || allowed.is_empty() {
        return false;
    }

    host == allowed || host.ends_with(&format!(".{allowed}"))
}

fn normalize_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

struct LimitApplication {
    content: String,
    metadata: Limits,
}

fn apply_limits(content: String, max_chars: Option<usize>) -> LimitApplication {
    let before = content.chars().count();
    let (content, truncated) = match max_chars {
        Some(max_chars) if before > max_chars => (content.chars().take(max_chars).collect(), true),
        _ => (content, false),
    };
    let after = content.chars().count();

    LimitApplication {
        content,
        metadata: Limits {
            max_chars,
            truncated,
            truncated_by: truncated.then(|| "max_chars".to_string()),
            content_chars_before_truncation: before,
            content_chars_after_truncation: after,
        },
    }
}

fn output_options(options: &GetOptions) -> OutputOptions {
    let backend_options = options
        .backend_options
        .iter()
        .map(|option| (option.key.clone(), option.value.clone()))
        .collect();

    OutputOptions {
        content_format: options.content_format,
        selector: options.selector.clone(),
        exclude_selector: options.exclude_selector.clone(),
        wait_for_selector: options.wait_for_selector.clone(),
        backend_options,
    }
}

fn extraction_failed(message: impl Into<String>) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: message.into(),
    }
}

fn backend_unavailable(command: &str, error: io::Error) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("Crawl4AI backend is unavailable for command '{command}': {error}"),
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

    #[test]
    fn replay_scope_allows_subdomains_and_rejects_unrelated_hosts() {
        let mut session = Session::new("docs");
        session
            .allowed_cookie_domains
            .push("example.com".to_string());

        assert!(session_replay_scope_error(&session, "docs.example.com").is_none());
        assert!(session_replay_scope_error(&session, "example.com.evil").is_some());
    }

    #[test]
    fn replay_scope_rejects_mixed_domain_session_state() {
        let mut session = Session::new("mixed");
        session.cookies.push(crate::session::SessionCookie {
            name: "app".to_string(),
            value: "app-secret".to_string(),
            domain: "app.example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: true,
            secure: true,
            same_site: None,
            source_session: None,
        });
        session.cookies.push(crate::session::SessionCookie {
            name: "provider".to_string(),
            value: "provider-secret".to_string(),
            domain: "provider.example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: true,
            secure: true,
            same_site: None,
            source_session: None,
        });

        let error = session_replay_scope_error(&session, "app.example.com").unwrap();

        assert!(error.contains("provider.example.com"));
    }
}
