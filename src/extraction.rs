use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use html5ever::tree_builder::TreeSink;
use scraper::{Html, HtmlTreeSink, Selector};
use serde::{Deserialize, Serialize};
use ureq::ResponseExt;

use crate::cli::{ExtractorOption, OutputFormat};
use crate::error::{AgetError, ErrorCode};
use crate::process::{configure_local_command, wait_for_child};
use crate::session::agent_browser::origin_host;
use crate::session::{
    compose_playwright_state, PlaywrightState, Session, SessionStore, TempStateFile,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const EXTRACTOR: &str = "crawl4ai";
const OWNED_EXTRACTOR: &str = "aget-owned-extractor";
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
    // default implementation is command-backed, but the rest of the pipeline
    // should not know whether the content came from Crawl4AI or in-process Rust.
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
pub struct OwnedExtractorBackend;

impl ExtractorBackend for OwnedExtractorBackend {
    fn name(&self) -> &'static str {
        OWNED_EXTRACTOR
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        run_owned_extractor_backend(request)
    }
}

pub fn get_url(options: GetOptions) -> Result<GetSuccess, AgetError> {
    let extractor = CommandExtractorBackend::new(None);
    let browser_fallback = CommandBrowserFallbackBackend;
    get_url_with_backends(options, &extractor, &browser_fallback)
}

pub fn get_url_with_backend(
    options: GetOptions,
    extractor_backend: &impl ExtractorBackend,
) -> Result<GetSuccess, AgetError> {
    let browser_fallback = CommandBrowserFallbackBackend;
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

fn sensitive_values(state: &PlaywrightState) -> Vec<String> {
    state
        .cookies
        .iter()
        .map(|cookie| cookie.value.clone())
        .chain(
            state
                .origins
                .iter()
                .flat_map(|origin| origin.local_storage.iter().map(|entry| entry.value.clone())),
        )
        .filter(|value| !value.is_empty())
        .collect()
}

fn sanitize_backend_error(error: AgetError, sensitive: bool) -> AgetError {
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

fn sanitize_backend_artifacts(metadata_path: &Path, sensitive_values: &[String]) {
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

fn redact_values(text: &str, sensitive_values: &[String]) -> String {
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

fn run_owned_extractor_backend(
    request: ExtractorRequest<'_>,
) -> Result<ExtractorBackendResult, AgetError> {
    validate_owned_extractor_request(&request)?;
    let response = owned_fetch(request.url, request.state, request.timeout)?;
    let mut document = Html::parse_document(&response.body);
    document = remove_selected_elements(document, "script,style,noscript")?;

    if let Some(wait_for) = &request.options.wait_for_selector {
        let selector = parse_css_selector(wait_for)?;
        if document.select(&selector).next().is_none() {
            return Err(extraction_failed(format!(
                "wait selector '{wait_for}' was not found by owned extractor"
            )));
        }
    }

    if let Some(exclude_selector) = &request.options.exclude_selector {
        document = remove_selected_elements(document, exclude_selector)?;
    }

    let extracted = extract_owned_content(&document, request.options.selector.as_deref())?;
    let content = match request.options.content_format {
        OutputFormat::Html => extracted.html,
        OutputFormat::Json => serde_json::json!({
            "url": response.final_url,
            "content": extracted.text,
        })
        .to_string(),
        OutputFormat::Markdown | OutputFormat::Text => extracted.text,
    };

    let backend_response = ExtractorBackendResult {
        ok: true,
        final_url: Some(response.final_url),
        content: Some(content.clone()),
        warnings: Vec::new(),
        error: None,
    };
    write_private_file(request.content_path, content.as_bytes()).map_err(io_aget_error)?;
    let metadata =
        serde_json::to_vec_pretty(&backend_response).map_err(|error| AgetError::Stable {
            code: ErrorCode::IoError,
            message: error.to_string(),
        })?;
    write_private_file(request.metadata_path, &metadata).map_err(io_aget_error)?;
    Ok(backend_response)
}

fn validate_owned_extractor_request(request: &ExtractorRequest<'_>) -> Result<(), AgetError> {
    if let Some(wait_for) = &request.options.wait_for_selector {
        validate_css_only_wait(wait_for)?;
    }
    if let Some(option) = request.options.backend_options.first() {
        return Err(extraction_failed(format!(
            "owned extractor does not support backend option '{}'; keep Crawl4AI compatibility options on the command adapter until an aget-owned namespace is designed",
            option.key
        )));
    }
    Ok(())
}

fn validate_css_only_wait(value: &str) -> Result<(), AgetError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.starts_with("js:")
        || ["=>", "function(", "return ", ";"]
            .iter()
            .any(|marker| normalized.contains(marker))
    {
        return Err(extraction_failed(
            "--wait-for-selector only supports CSS selectors in v1; JavaScript wait conditions are not allowed",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct OwnedHttpResponse {
    final_url: String,
    body: String,
}

fn owned_fetch(
    url: &str,
    state: &PlaywrightState,
    timeout: Duration,
) -> Result<OwnedHttpResponse, AgetError> {
    let parsed = ParsedRequestUrl::parse(url)?;
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .max_redirects(5)
        .build();
    let agent: ureq::Agent = config.into();
    let mut request = agent.get(url).header("User-Agent", "aget/0.1");
    if let Some(cookie_header) = cookie_header_for_state(state, &parsed) {
        request = request.header("Cookie", cookie_header);
    }

    let mut response = request.call().map_err(map_ureq_error)?;
    let final_url = response.get_uri().to_string();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(map_ureq_error)?;
    Ok(OwnedHttpResponse { final_url, body })
}

#[derive(Debug, Clone)]
struct ParsedRequestUrl {
    scheme: String,
    host: String,
    path: String,
}

impl ParsedRequestUrl {
    fn parse(url: &str) -> Result<Self, AgetError> {
        let uri = url.parse::<ureq::http::Uri>().map_err(|error| {
            extraction_failed(format!(
                "owned extractor received invalid URL '{url}': {error}"
            ))
        })?;
        let scheme = uri
            .scheme_str()
            .ok_or_else(|| extraction_failed(format!("request URL '{url}' is missing a scheme")))?;
        if !matches!(scheme, "http" | "https") {
            return Err(extraction_failed(format!(
                "owned extractor supports only http and https URLs, got '{scheme}'"
            )));
        }
        let host = uri
            .host()
            .ok_or_else(|| extraction_failed(format!("request URL '{url}' is missing a host")))?;
        let path = uri.path().to_string();
        Ok(Self {
            scheme: scheme.to_string(),
            host: host.to_string(),
            path,
        })
    }
}

fn cookie_header_for_state(state: &PlaywrightState, parsed: &ParsedRequestUrl) -> Option<String> {
    let cookies = state
        .cookies
        .iter()
        .filter(|cookie| !cookie.secure || parsed.scheme == "https")
        .filter(|cookie| domain_matches_host(&parsed.host, &cookie.domain))
        .filter(|cookie| request_path_matches_cookie_path(&parsed.path, &cookie.path))
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<_>>();
    (!cookies.is_empty()).then(|| cookies.join("; "))
}

fn request_path_matches_cookie_path(request_path: &str, cookie_path: &str) -> bool {
    let cookie_path = if cookie_path.is_empty() {
        "/"
    } else {
        cookie_path
    };
    request_path == cookie_path
        || (request_path.starts_with(cookie_path)
            && (cookie_path.ends_with('/')
                || request_path
                    .as_bytes()
                    .get(cookie_path.len())
                    .is_some_and(|byte| *byte == b'/')))
}

struct ExtractedOwnedContent {
    html: String,
    text: String,
}

fn extract_owned_content(
    document: &Html,
    selector: Option<&str>,
) -> Result<ExtractedOwnedContent, AgetError> {
    if let Some(raw_selector) = selector {
        let selector = parse_css_selector(raw_selector)?;
        let element = document.select(&selector).next().ok_or_else(|| {
            extraction_failed(format!(
                "selector '{raw_selector}' was not found by owned extractor"
            ))
        })?;
        return Ok(ExtractedOwnedContent {
            html: element.inner_html(),
            text: normalize_text_pieces(element.text()),
        });
    }

    let root = document.root_element();
    Ok(ExtractedOwnedContent {
        html: root.inner_html(),
        text: normalize_text_pieces(root.text()),
    })
}

fn remove_selected_elements(document: Html, selector_list: &str) -> Result<Html, AgetError> {
    let selector = parse_css_selector(selector_list)?;
    let node_ids = document
        .select(&selector)
        .map(|element| element.id())
        .collect::<Vec<_>>();
    let tree = HtmlTreeSink::new(document);
    for id in node_ids {
        tree.remove_from_parent(&id);
    }
    Ok(tree.finish())
}

fn parse_css_selector(raw: &str) -> Result<Selector, AgetError> {
    let selector = raw.trim().strip_prefix("css:").unwrap_or(raw.trim()).trim();
    if selector.is_empty() {
        return Err(extraction_failed(format!(
            "owned extractor received an empty CSS selector from '{raw}'"
        )));
    }
    Selector::parse(selector).map_err(|error| {
        extraction_failed(format!(
            "owned extractor could not parse CSS selector '{raw}': {error:?}"
        ))
    })
}

fn normalize_text_pieces<'a>(pieces: impl IntoIterator<Item = &'a str>) -> String {
    pieces
        .into_iter()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn map_ureq_error(error: ureq::Error) -> AgetError {
    match error {
        ureq::Error::Timeout(_) => AgetError::Stable {
            code: ErrorCode::Timeout,
            message: error.to_string(),
        },
        other => extraction_failed(other.to_string()),
    }
}

fn extraction_failed(message: impl Into<String>) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: message.into(),
    }
}

fn run_command_extractor_backend(
    command_override: Option<String>,
    request: ExtractorRequest<'_>,
) -> Result<ExtractorBackendResult, AgetError> {
    let mut args = vec![
        "--url".to_string(),
        request.url.to_string(),
        "--state".to_string(),
        request.state_path.to_string_lossy().into_owned(),
        "--output".to_string(),
        request.content_path.to_string_lossy().into_owned(),
        "--metadata".to_string(),
        request.metadata_path.to_string_lossy().into_owned(),
        "--format".to_string(),
        request.options.content_format.to_string(),
    ];
    if let Some(selector) = &request.options.selector {
        args.push("--selector".to_string());
        args.push(selector.clone());
    }
    if let Some(exclude_selector) = &request.options.exclude_selector {
        args.push("--exclude-selector".to_string());
        args.push(exclude_selector.clone());
    }
    if let Some(wait_for) = &request.options.wait_for_selector {
        args.push("--wait-for".to_string());
        args.push(wait_for.clone());
    }
    for extractor_option in &request.options.backend_options {
        args.push("--extractor-option".to_string());
        args.push(format!(
            "{}={}",
            extractor_option.key, extractor_option.value
        ));
    }

    // Current extractor backend adapter: spawn a Crawl4AI-compatible command.
    // A future in-process Rust extractor should satisfy the same contract without shelling out.
    let command_string = command_override
        .or_else(|| env::var("AGET_CRAWL4AI_COMMAND").ok())
        .unwrap_or_else(default_command);
    let backend_stdout_path = request.metadata_path.with_file_name("backend-stdout.json");
    let backend_stderr_path = request.metadata_path.with_file_name("backend-stderr.txt");
    let backend_stdout = create_private_file(&backend_stdout_path).map_err(io_aget_error)?;
    let backend_stderr = create_private_file(&backend_stderr_path).map_err(io_aget_error)?;
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("{} \"$@\"", command_string))
        .arg("aget-crawl4ai")
        .args(&args)
        .stdout(Stdio::from(backend_stdout))
        .stderr(Stdio::from(backend_stderr));
    configure_local_command(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| backend_unavailable(&command_string, error))?;

    let status = wait_for_child(&mut child, request.timeout, || {
        format!(
            "Crawl4AI backend timed out after {} seconds",
            request.timeout.as_secs()
        )
    })?;
    if !status.success() {
        if let Ok(stdout) = read_output_file(&backend_stdout_path) {
            if let Ok(result) = parse_backend_stdout(&stdout) {
                if !result.ok {
                    return Ok(result);
                }
            }
        }

        let stderr = read_output_file(&backend_stderr_path)
            .map(|text| text.trim().to_string())
            .unwrap_or_default();
        let code = if matches!(status.code(), Some(126 | 127)) {
            ErrorCode::BackendUnavailable
        } else {
            ErrorCode::ExtractionFailed
        };
        return Err(AgetError::Stable {
            code,
            message: if stderr.is_empty() {
                format!("Crawl4AI backend exited with {status}")
            } else {
                stderr
            },
        });
    }

    let stdout = read_output_file(&backend_stdout_path).map_err(io_aget_error)?;
    let result = parse_backend_stdout(&stdout)?;
    if result.ok {
        write_private_file(&backend_stdout_path, b"").map_err(io_aget_error)?;
    }
    Ok(result)
}

fn parse_backend_stdout(stdout: &str) -> Result<ExtractorBackendResult, AgetError> {
    let trimmed = stdout.trim();
    match serde_json::from_str(trimmed) {
        Ok(result) => Ok(result),
        Err(full_error) => {
            for line in stdout
                .lines()
                .rev()
                .map(str::trim)
                .filter(|line| !line.is_empty())
            {
                if !line.starts_with('{') {
                    continue;
                }
                if let Ok(result) = serde_json::from_str(line) {
                    return Ok(result);
                }
            }

            Err(AgetError::Stable {
                code: ErrorCode::ExtractionFailed,
                message: format!("Crawl4AI backend returned malformed JSON: {full_error}"),
            })
        }
    }
}

#[derive(Debug)]
struct AgentBrowserOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

fn run_agent_browser_fallback(
    tmp_dir: &Path,
    url: &str,
    state_path: &Path,
    options: &GetOptions,
    timeout: Duration,
) -> Result<BrowserFallbackResult, AgetError> {
    let profile = TempAgentBrowserProfile::new(tmp_dir)?;
    let session = unique_agent_browser_session_name();
    let profile_path = profile.path().to_string_lossy().into_owned();
    let state_path = state_path.to_string_lossy().into_owned();

    let load = run_agent_browser(
        tmp_dir,
        &[
            "--profile",
            &profile_path,
            "--session",
            &session,
            "state",
            "load",
            &state_path,
        ],
        timeout,
    )?;
    if !load.status.success() {
        return Err(classify_agent_browser_failure("state load", &load));
    }

    let open = run_agent_browser(
        tmp_dir,
        &[
            "--profile",
            &profile_path,
            "--session",
            &session,
            "open",
            url,
        ],
        timeout,
    )?;
    if !open.status.success() {
        let _ = run_agent_browser(
            tmp_dir,
            &["--profile", &profile_path, "--session", &session, "close"],
            timeout,
        );
        return Err(classify_agent_browser_failure("open", &open));
    }

    let content_result =
        extract_agent_browser_content(tmp_dir, &profile_path, &session, options, timeout);
    let close = run_agent_browser(
        tmp_dir,
        &["--profile", &profile_path, "--session", &session, "close"],
        timeout,
    );
    if let Ok(close) = &close {
        if !close.status.success() {
            return Err(classify_agent_browser_failure("close", close));
        }
    } else if content_result.is_ok() {
        return Err(close.unwrap_err());
    }

    let content = content_result?;
    Ok(BrowserFallbackResult {
        final_url: url.to_string(),
        content,
        warnings: vec![FALLBACK_WARNING.to_string()],
        extractor: FALLBACK_EXTRACTOR.to_string(),
    })
}

fn extract_agent_browser_content(
    tmp_dir: &Path,
    profile_path: &str,
    session: &str,
    options: &GetOptions,
    timeout: Duration,
) -> Result<String, AgetError> {
    let html = run_agent_browser(
        tmp_dir,
        &[
            "--profile",
            profile_path,
            "--session",
            session,
            "get",
            "html",
            "body",
        ],
        timeout,
    )?;
    if html.status.success() {
        let html = html.stdout;
        return Ok(match options.content_format {
            OutputFormat::Html => html,
            OutputFormat::Json => serde_json::json!({
                "url": options.url,
                "content": html_to_text(&html),
            })
            .to_string(),
            OutputFormat::Markdown | OutputFormat::Text => html_to_text(&html),
        });
    }

    let text = run_agent_browser(
        tmp_dir,
        &[
            "--profile",
            profile_path,
            "--session",
            session,
            "get",
            "text",
            "body",
        ],
        timeout,
    )?;
    if !text.status.success() {
        return Err(classify_agent_browser_failure("get", &text));
    }

    Ok(match options.content_format {
        OutputFormat::Json => serde_json::json!({
            "url": options.url,
            "content": text.stdout,
        })
        .to_string(),
        _ => text.stdout,
    })
}

fn run_agent_browser(
    tmp_dir: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<AgentBrowserOutput, AgetError> {
    let command =
        env::var("AGET_AGENT_BROWSER_COMMAND").unwrap_or_else(|_| "agent-browser".to_string());
    let stdout_file = TempOutputFile::new(tmp_dir, "agent-browser-stdout")?;
    let stderr_file = TempOutputFile::new(tmp_dir, "agent-browser-stderr")?;
    let stdout = create_private_file(stdout_file.path()).map_err(io_aget_error)?;
    let stderr = create_private_file(stderr_file.path()).map_err(io_aget_error)?;
    let mut command_builder = Command::new(&command);
    command_builder
        .args(args)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    configure_local_command(&mut command_builder);
    let mut child = command_builder.spawn().map_err(|error| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("agent-browser backend is unavailable for command '{command}': {error}"),
    })?;

    let status = wait_for_child(&mut child, timeout, || {
        format!(
            "agent-browser fallback timed out after {} seconds",
            timeout.as_secs()
        )
    })?;

    Ok(AgentBrowserOutput {
        status,
        stdout: read_output_file(stdout_file.path()).map_err(io_aget_error)?,
        stderr: read_output_file(stderr_file.path()).map_err(io_aget_error)?,
    })
}

fn classify_agent_browser_failure(action: &str, output: &AgentBrowserOutput) -> AgetError {
    let combined = format!("{}\n{}", output.stdout, output.stderr);
    let code = if matches!(output.status.code(), Some(126 | 127)) {
        ErrorCode::BackendUnavailable
    } else {
        ErrorCode::ExtractionFailed
    };
    AgetError::Stable {
        code,
        message: if combined.trim().is_empty() {
            format!(
                "agent-browser fallback {action} exited with {}",
                output.status
            )
        } else {
            combined.trim().to_string()
        },
    }
}

fn html_to_text(html: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    let mut skip_depth = 0usize;

    for character in html.chars() {
        match character {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                handle_html_tag(&tag, &mut output, &mut skip_depth);
            }
            _ if in_tag => tag.push(character),
            _ if skip_depth == 0 => output.push(character),
            _ => {}
        }
    }

    normalize_text(&decode_basic_entities(&output))
}

fn handle_html_tag(tag: &str, output: &mut String, skip_depth: &mut usize) {
    let tag_name = tag
        .trim()
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let closing = tag.trim_start().starts_with('/');

    if matches!(tag_name.as_str(), "script" | "style" | "noscript") {
        if closing {
            *skip_depth = skip_depth.saturating_sub(1);
        } else {
            *skip_depth += 1;
        }
        return;
    }

    if *skip_depth == 0
        && matches!(
            tag_name.as_str(),
            "br" | "p"
                | "div"
                | "section"
                | "article"
                | "main"
                | "li"
                | "tr"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
        )
    {
        output.push('\n');
    }
}

fn decode_basic_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn normalize_text(text: &str) -> String {
    let mut normalized = String::new();
    let mut blank_lines = 0usize;
    for line in text.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            blank_lines += 1;
            if blank_lines <= 1 && !normalized.is_empty() {
                normalized.push('\n');
            }
        } else {
            blank_lines = 0;
            normalized.push_str(&line);
            normalized.push('\n');
        }
    }
    normalized.trim().to_string()
}

fn unique_agent_browser_session_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("aget-fallback-{}-{nanos}", std::process::id())
}

struct TempAgentBrowserProfile {
    path: PathBuf,
}

impl TempAgentBrowserProfile {
    fn new(tmp_dir: &Path) -> Result<Self, AgetError> {
        fs::create_dir_all(tmp_dir).map_err(io_aget_error)?;
        let path = tmp_dir
            .join("agent-browser")
            .join(unique_agent_browser_session_name());
        create_private_dir(&path).map_err(io_aget_error)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempAgentBrowserProfile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct TempOutputFile {
    path: PathBuf,
}

impl TempOutputFile {
    fn new(dir: &Path, prefix: &str) -> Result<Self, AgetError> {
        fs::create_dir_all(dir).map_err(io_aget_error)?;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        Ok(Self {
            path: dir.join(format!("{prefix}-{}-{nanos}.txt", std::process::id())),
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempOutputFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn read_output_file(path: &Path) -> io::Result<String> {
    let mut output = String::new();
    File::open(path)?.read_to_string(&mut output)?;
    Ok(output)
}

#[allow(clippy::too_many_arguments)]
fn write_error_metadata(
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

fn write_metadata(path: &Path, success: &GetSuccess) -> Result<(), AgetError> {
    let mut metadata = BTreeMap::new();
    metadata.insert("ok", serde_json::json!(success.ok));
    metadata.insert("url", serde_json::json!(success.url));
    metadata.insert("final_url", serde_json::json!(success.final_url));
    metadata.insert("content_format", serde_json::json!(success.content_format));
    metadata.insert("extractor", serde_json::json!(success.extractor));
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

fn default_command() -> String {
    let helper = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/crawl4ai_extract.py");
    format!(
        "uv run --with crawl4ai {}",
        shell_quote(&helper.to_string_lossy())
    )
}

fn run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("run-{}-{nanos}", std::process::id())
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

fn write_private_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = create_private_file(path)?;
    file.write_all(bytes)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    set_private_dir_permissions(path)
}

fn create_private_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    set_private_file_mode(&mut options);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file_mode(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_private_file_mode(_options: &mut OpenOptions) {}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
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
