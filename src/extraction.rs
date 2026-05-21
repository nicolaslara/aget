use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ego_tree::{NodeId, NodeRef};
use html5ever::tree_builder::TreeSink;
use scraper::{ElementRef, Html, HtmlTreeSink, Node, Selector};
use serde::{Deserialize, Serialize};
use ureq::ResponseExt;
use url::Url;

use crate::browser_cdp::PageWaitUntil;
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
const OWNED_BROWSER_FALLBACK: &str = "aget-owned-browser-fallback";
const FALLBACK_EXTRACTOR: &str = "agent-browser-fallback";
const FALLBACK_WARNING: &str = "agent-browser fallback used after Crawl4AI failed";
const OWNED_FALLBACK_WARNING: &str = "aget-owned fallback used after primary extractor failed";
const DEFAULT_RENDER_SETTLE_DELAY: Duration = Duration::from_millis(100);
const CRAWL4AI_IMPORTANT_ATTRS: &[&str] = &[
    "src", "href", "alt", "title", "width", "height", "class", "id",
];
const CRAWL4AI_EMPTY_ELEMENT_BYPASS_TAGS: &[&str] = &[
    "a", "img", "br", "hr", "input", "meta", "link", "source", "track", "wbr", "tr", "td", "th",
];

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
    let extractor = OwnedExtractorBackend;
    let browser_fallback = crate::aget::OwnedBrowserAutomationBackend;
    get_url_with_backends(options, &extractor, &browser_fallback)
}

pub fn get_url_with_backend(
    options: GetOptions,
    extractor_backend: &impl ExtractorBackend,
) -> Result<GetSuccess, AgetError> {
    let browser_fallback = crate::aget::OwnedBrowserAutomationBackend;
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
    let tmp_dir = request
        .state_path
        .parent()
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::IoError,
            message: format!(
                "owned extractor state path '{}' has no temp directory",
                request.state_path.display()
            ),
        })?;
    let extraction = extract_owned_static_or_rendered(
        tmp_dir,
        request.url,
        request.state,
        request.options,
        request.timeout,
        None,
    )?;

    let backend_response = ExtractorBackendResult {
        ok: true,
        final_url: Some(extraction.final_url),
        content: Some(extraction.content.clone()),
        warnings: extraction.warnings,
        error: None,
    };
    write_private_file(request.content_path, extraction.content.as_bytes())
        .map_err(io_aget_error)?;
    let metadata =
        serde_json::to_vec_pretty(&backend_response).map_err(|error| AgetError::Stable {
            code: ErrorCode::IoError,
            message: error.to_string(),
        })?;
    write_private_file(request.metadata_path, &metadata).map_err(io_aget_error)?;
    Ok(backend_response)
}

pub(crate) fn run_owned_browser_fallback(
    request: BrowserFallbackRequest<'_>,
) -> Result<BrowserFallbackResult, AgetError> {
    let extraction = extract_owned_static_or_rendered(
        request.tmp_dir,
        request.url,
        request.state,
        request.options,
        request.timeout,
        Some("body"),
    )?;
    let mut warnings = vec![OWNED_FALLBACK_WARNING.to_string()];
    warnings.extend(extraction.warnings);
    Ok(BrowserFallbackResult {
        final_url: extraction.final_url,
        content: extraction.content,
        warnings,
        extractor: OWNED_BROWSER_FALLBACK.to_string(),
    })
}

fn extract_owned_static_or_rendered(
    tmp_dir: &Path,
    url: &str,
    state: &PlaywrightState,
    options: &GetOptions,
    timeout: Duration,
    fallback_selector: Option<&str>,
) -> Result<OwnedPageExtraction, AgetError> {
    let owned_options = validate_owned_extraction_options(options)?;
    if owned_options.wait_for_images {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }
    if !state.origins.is_empty() {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }

    let response = owned_fetch(url, state, timeout)?;
    if should_render_scripted_response(&response.body) {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }

    match extract_owned_page_response(response, options, fallback_selector, &owned_options) {
        Ok(extraction) => Ok(extraction),
        Err(error) if should_retry_with_rendered_wait(&error, options) => {
            extract_owned_rendered_page(
                tmp_dir,
                url,
                state,
                options,
                timeout,
                fallback_selector,
                &owned_options,
            )
        }
        Err(error) => Err(error),
    }
}

fn extract_owned_rendered_page(
    tmp_dir: &Path,
    url: &str,
    state: &PlaywrightState,
    options: &GetOptions,
    timeout: Duration,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let rendered = crate::browser_cdp::render_page(crate::browser_cdp::BrowserRenderRequest {
        tmp_dir,
        url,
        state,
        wait_for_selector: options.wait_for_selector.as_deref(),
        wait_until: owned_options.wait_until,
        wait_for_images: owned_options.wait_for_images,
        settle_delay: owned_options.render_settle_delay,
        page_timeout: owned_options.page_timeout.unwrap_or(timeout),
        wait_for_timeout: owned_options.wait_for_timeout,
        timeout,
    })?;
    let mut extraction = extract_owned_html(
        rendered.final_url,
        rendered.html,
        options,
        fallback_selector,
        owned_options,
    )?;
    extraction.warnings.extend(rendered.warnings);
    Ok(extraction)
}

fn should_retry_with_rendered_wait(error: &AgetError, options: &GetOptions) -> bool {
    if options.wait_for_selector.is_none() {
        return false;
    }
    matches!(
        error,
        AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message,
        } if message.starts_with("wait selector ") && message.ends_with(" was not found by owned extractor")
    )
}

struct OwnedPageExtraction {
    final_url: String,
    content: String,
    warnings: Vec<String>,
}

#[derive(Debug)]
struct OwnedExtractorOptions {
    excluded_tags: Vec<String>,
    target_elements: Vec<String>,
    only_text: bool,
    wait_until: PageWaitUntil,
    wait_for_images: bool,
    render_settle_delay: Duration,
    page_timeout: Option<Duration>,
    wait_for_timeout: Option<Duration>,
}

impl Default for OwnedExtractorOptions {
    fn default() -> Self {
        Self {
            excluded_tags: Vec::new(),
            target_elements: Vec::new(),
            only_text: false,
            wait_until: PageWaitUntil::Load,
            wait_for_images: false,
            render_settle_delay: DEFAULT_RENDER_SETTLE_DELAY,
            page_timeout: None,
            wait_for_timeout: None,
        }
    }
}

fn extract_owned_page_response(
    response: OwnedHttpResponse,
    options: &GetOptions,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    extract_owned_html(
        response.final_url,
        response.body,
        options,
        fallback_selector,
        owned_options,
    )
}

fn should_render_scripted_response(body: &str) -> bool {
    let document = Html::parse_document(body);
    let Ok(selector) = Selector::parse(
        "script[src],script:not([type]),script[type=\"text/javascript\"],script[type=\"module\"]",
    ) else {
        return false;
    };
    document.select(&selector).next().is_some()
}

fn extract_owned_html(
    final_url: String,
    body: String,
    options: &GetOptions,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let mut document = Html::parse_document(&body);
    document = remove_selected_elements(document, "script,style,link,meta,noscript")?;

    if let Some(wait_for) = &options.wait_for_selector {
        let selector = parse_css_selector(wait_for)?;
        if document.select(&selector).next().is_none() {
            return Err(extraction_failed(format!(
                "wait selector '{wait_for}' was not found by owned extractor"
            )));
        }
    }

    if !owned_options.excluded_tags.is_empty() {
        document = remove_owned_excluded_tags(document, &owned_options.excluded_tags)?;
    }

    if let Some(exclude_selector) = &options.exclude_selector {
        document = remove_selected_elements(document, exclude_selector)?;
    }

    let selector = options.selector.as_deref().or(fallback_selector);
    let base_url = markdown_base_url(&document, &final_url)?;
    let prefer_main_content = selector.is_none()
        && options.wait_for_selector.is_none()
        && options.content_format != OutputFormat::Html;
    let extracted = extract_owned_content(
        document,
        selector,
        &base_url,
        prefer_main_content,
        owned_options,
    )?;
    let content = match options.content_format {
        OutputFormat::Html => extracted.html,
        OutputFormat::Json => serde_json::json!({
            "url": final_url,
            "content": extracted.text,
        })
        .to_string(),
        OutputFormat::Markdown => extracted.markdown,
        OutputFormat::Text => extracted.text,
    };

    Ok(OwnedPageExtraction {
        final_url,
        content,
        warnings: Vec::new(),
    })
}

fn validate_owned_extraction_options(
    options: &GetOptions,
) -> Result<OwnedExtractorOptions, AgetError> {
    if let Some(wait_for) = &options.wait_for_selector {
        validate_css_only_wait(wait_for)?;
    }
    let mut owned_options = OwnedExtractorOptions::default();
    for option in &options.backend_options {
        let option_name = option.key.strip_prefix("crawl4ai.").ok_or_else(|| {
            extraction_failed(format!(
                "owned extractor backend option '{}' must use the crawl4ai namespace",
                option.key
            ))
        })?;
        match option_name {
            "excluded_tags" => {
                owned_options
                    .excluded_tags
                    .extend(parse_owned_excluded_tags(&option.value)?);
            }
            "target_elements" => {
                owned_options
                    .target_elements
                    .extend(parse_owned_target_elements(&option.value)?);
            }
            "only_text" => {
                owned_options.only_text = parse_owned_bool("crawl4ai.only_text", &option.value)?;
            }
            "word_count_threshold" => {
                // Crawl4AI's default cleaned-content path accepts this config but
                // currently hardcodes empty-leaf pruning to threshold 1.
                parse_owned_integer("crawl4ai.word_count_threshold", &option.value)?;
            }
            "delay_before_return_html" => {
                owned_options.render_settle_delay = parse_owned_render_delay(&option.value)?;
            }
            "page_timeout" => {
                owned_options.page_timeout = Some(parse_owned_milliseconds(
                    "crawl4ai.page_timeout",
                    &option.value,
                )?);
            }
            "wait_for_timeout" => {
                owned_options.wait_for_timeout = Some(parse_owned_milliseconds(
                    "crawl4ai.wait_for_timeout",
                    &option.value,
                )?);
            }
            "wait_until" => {
                owned_options.wait_until = parse_owned_wait_until(&option.value)?;
            }
            "wait_for_images" => {
                owned_options.wait_for_images =
                    parse_owned_bool("crawl4ai.wait_for_images", &option.value)?;
            }
            _ => {
                return Err(extraction_failed(format!(
                    "owned extractor does not support backend option '{}'; supported options: crawl4ai.delay_before_return_html, crawl4ai.excluded_tags, crawl4ai.only_text, crawl4ai.page_timeout, crawl4ai.target_elements, crawl4ai.wait_for_images, crawl4ai.wait_for_timeout, crawl4ai.wait_until, crawl4ai.word_count_threshold",
                    option.key
                )));
            }
        }
    }
    Ok(owned_options)
}

fn parse_owned_excluded_tags(value: &str) -> Result<Vec<String>, AgetError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(|tag| {
            if is_html_tag_name(tag) {
                Ok(tag.to_string())
            } else {
                Err(extraction_failed(format!(
                    "crawl4ai.excluded_tags entry '{tag}' is not a plain HTML tag name"
                )))
            }
        })
        .collect()
}

fn is_html_tag_name(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

fn parse_owned_target_elements(value: &str) -> Result<Vec<String>, AgetError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|selector| !selector.is_empty())
        .map(|selector| {
            parse_css_selector(selector)?;
            Ok(selector.to_string())
        })
        .collect()
}

fn parse_owned_render_delay(value: &str) -> Result<Duration, AgetError> {
    let seconds = value.trim().parse::<f64>().map_err(|_| {
        extraction_failed(format!(
            "crawl4ai.delay_before_return_html expects a non-negative number of seconds, got '{value}'"
        ))
    })?;
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(extraction_failed(format!(
            "crawl4ai.delay_before_return_html expects a non-negative finite number of seconds, got '{value}'"
        )));
    }
    Ok(Duration::from_secs_f64(seconds))
}

fn parse_owned_bool(name: &str, value: &str) -> Result<bool, AgetError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(extraction_failed(format!(
            "{name} expects a boolean value, got '{value}'"
        ))),
    }
}

fn parse_owned_milliseconds(name: &str, value: &str) -> Result<Duration, AgetError> {
    let milliseconds = value.trim().parse::<u64>().map_err(|_| {
        extraction_failed(format!(
            "{name} expects a non-negative integer number of milliseconds, got '{value}'"
        ))
    })?;
    Ok(Duration::from_millis(milliseconds))
}

fn parse_owned_integer(name: &str, value: &str) -> Result<i64, AgetError> {
    value
        .trim()
        .parse::<i64>()
        .map_err(|_| extraction_failed(format!("{name} expects an integer value, got '{value}'")))
}

fn parse_owned_wait_until(value: &str) -> Result<PageWaitUntil, AgetError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "domcontentloaded" => Ok(PageWaitUntil::DomContentLoaded),
        "load" => Ok(PageWaitUntil::Load),
        "networkidle" => Ok(PageWaitUntil::NetworkIdle),
        _ => Err(extraction_failed(format!(
            "crawl4ai.wait_until supports only 'domcontentloaded', 'load', or 'networkidle' in the owned extractor, got '{value}'"
        ))),
    }
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
    markdown: String,
    text: String,
}

fn extract_owned_content(
    mut document: Html,
    selector: Option<&str>,
    base_url: &str,
    prefer_main_content: bool,
    owned_options: &OwnedExtractorOptions,
) -> Result<ExtractedOwnedContent, AgetError> {
    let root_ids = if let Some(raw_selector) = selector {
        match parse_css_selector(raw_selector) {
            Ok(selector) => {
                let selected = document
                    .select(&selector)
                    .map(|element| element.id())
                    .collect::<Vec<_>>();
                if selected.is_empty() {
                    vec![document.root_element().id()]
                } else {
                    selected
                }
            }
            Err(_) => vec![document.root_element().id()],
        }
    } else if !owned_options.target_elements.is_empty() {
        vec![document.root_element().id()]
    } else if prefer_main_content {
        vec![default_main_content_element(&document)?.id()]
    } else {
        vec![document.root_element().id()]
    };
    let target_ids = if owned_options.target_elements.is_empty() {
        Vec::new()
    } else {
        collect_target_owned_element_ids(&document, &root_ids, &owned_options.target_elements)?
    };

    // Match Crawl4AI's cleanup order: selectors see original attributes, but
    // serialized cleaned HTML keeps only its small important-attribute allowlist.
    document = clean_owned_base64_image_sources(document);
    document = remove_owned_empty_elements(document, &root_ids, &target_ids);
    document = prune_owned_unwanted_attributes(document);

    if owned_options.target_elements.is_empty() {
        if let [root_id] = root_ids.as_slice() {
            let root = element_by_id(&document, *root_id)?;
            return Ok(extract_single_owned_element(root, base_url, owned_options));
        }
        return extract_target_owned_elements(&document, &root_ids, base_url, owned_options);
    }
    extract_target_owned_elements(&document, &target_ids, base_url, owned_options)
}

fn extract_single_owned_element(
    element: ElementRef<'_>,
    base_url: &str,
    owned_options: &OwnedExtractorOptions,
) -> ExtractedOwnedContent {
    ExtractedOwnedContent {
        html: element.inner_html(),
        markdown: element_to_markdown(element, base_url, owned_options.only_text),
        text: normalize_text_pieces(element.text()),
    }
}

fn collect_target_owned_element_ids(
    document: &Html,
    source_ids: &[NodeId],
    raw_selectors: &[String],
) -> Result<Vec<NodeId>, AgetError> {
    let mut ids = Vec::new();
    for source_id in source_ids {
        let source = element_by_id(document, *source_id)?;
        for raw_selector in raw_selectors {
            let selector = parse_css_selector(raw_selector)?;
            ids.extend(source.select(&selector).map(|element| element.id()));
        }
    }
    Ok(ids)
}

fn extract_target_owned_elements(
    document: &Html,
    element_ids: &[NodeId],
    base_url: &str,
    owned_options: &OwnedExtractorOptions,
) -> Result<ExtractedOwnedContent, AgetError> {
    let elements = element_ids
        .iter()
        .map(|id| element_by_id(document, *id))
        .collect::<Result<Vec<_>, _>>()?;

    let html = elements
        .iter()
        .map(|element| element.html())
        .collect::<Vec<_>>()
        .join("\n");
    let markdown = normalize_markdown(
        &elements
            .iter()
            .map(|element| element_to_markdown(*element, base_url, owned_options.only_text))
            .filter(|markdown| !markdown.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
    );
    let text = normalize_text_pieces(elements.iter().flat_map(|element| element.text()));

    Ok(ExtractedOwnedContent {
        html,
        markdown,
        text,
    })
}

fn default_main_content_element(document: &Html) -> Result<ElementRef<'_>, AgetError> {
    for selector in ["main", r#"[role="main"]"#, "article"] {
        if let Some(element) = unique_selected_element(document, selector)? {
            return Ok(element);
        }
    }
    if let Some(body) = first_selected_element(document, "body")? {
        return Ok(body);
    }
    Ok(document.root_element())
}

fn unique_selected_element<'a>(
    document: &'a Html,
    raw_selector: &str,
) -> Result<Option<ElementRef<'a>>, AgetError> {
    let selector = parse_css_selector(raw_selector)?;
    let mut matches = document.select(&selector);
    let Some(first) = matches.next() else {
        return Ok(None);
    };
    Ok(matches.next().is_none().then_some(first))
}

fn first_selected_element<'a>(
    document: &'a Html,
    raw_selector: &str,
) -> Result<Option<ElementRef<'a>>, AgetError> {
    let selector = parse_css_selector(raw_selector)?;
    Ok(document.select(&selector).next())
}

fn element_by_id(document: &Html, id: NodeId) -> Result<ElementRef<'_>, AgetError> {
    document
        .tree
        .get(id)
        .and_then(ElementRef::wrap)
        .ok_or_else(|| extraction_failed("owned extractor lost a selected HTML element"))
}

fn prune_owned_unwanted_attributes(mut document: Html) -> Html {
    for node in document.tree.values_mut() {
        if let Node::Element(element) = node {
            element
                .attrs
                .retain(|(name, _)| is_crawl4ai_important_attr(name.local.as_ref()));
        }
    }
    document
}

fn clean_owned_base64_image_sources(mut document: Html) -> Html {
    for node in document.tree.values_mut() {
        let Node::Element(element) = node else {
            continue;
        };
        if element.name.local.as_ref() != "img" {
            continue;
        }
        for (name, value) in &mut element.attrs {
            if name.local.as_ref() == "src" && is_base64_image_src(value.as_ref()) {
                value.clear();
            }
        }
    }
    document
}

fn remove_owned_empty_elements(document: Html, root_ids: &[NodeId], target_ids: &[NodeId]) -> Html {
    let node_ids = document
        .tree
        .nodes()
        .filter_map(|node| ElementRef::wrap(node).map(|element| element.id()))
        .collect::<Vec<_>>();
    let sink = HtmlTreeSink::new(document);
    for id in node_ids.into_iter().rev() {
        let should_remove = {
            let document = sink.0.borrow();
            should_remove_owned_empty_element(&document, id, root_ids, target_ids)
        };
        if should_remove {
            sink.remove_from_parent(&id);
        }
    }
    sink.finish()
}

fn should_remove_owned_empty_element(
    document: &Html,
    id: NodeId,
    root_ids: &[NodeId],
    target_ids: &[NodeId],
) -> bool {
    if root_ids.contains(&id) || target_ids.contains(&id) {
        return false;
    }
    let Some(element) = document.tree.get(id).and_then(ElementRef::wrap) else {
        return false;
    };
    if element.parent().is_none() {
        return false;
    }
    let tag = element.value().name();
    if CRAWL4AI_EMPTY_ELEMENT_BYPASS_TAGS.contains(&tag) || is_descendant_of_code_block(element) {
        return false;
    }
    if element.child_elements().next().is_some() {
        return false;
    }
    element
        .text()
        .all(|text| text.split_whitespace().next().is_none())
}

fn is_descendant_of_code_block(element: ElementRef<'_>) -> bool {
    element.ancestors().any(|ancestor| {
        ElementRef::wrap(ancestor)
            .map(|ancestor| matches!(ancestor.value().name(), "pre" | "code"))
            .unwrap_or(false)
    })
}

fn is_base64_image_src(src: &str) -> bool {
    let Some(after_prefix) = src.strip_prefix("data:image/") else {
        return false;
    };
    let Some((mime_type, after_mime_type)) = after_prefix.split_once(';') else {
        return false;
    };
    !mime_type.is_empty() && after_mime_type.starts_with("base64,")
}

fn is_crawl4ai_important_attr(name: &str) -> bool {
    CRAWL4AI_IMPORTANT_ATTRS.contains(&name)
}

fn markdown_base_url(document: &Html, final_url: &str) -> Result<String, AgetError> {
    let selector = parse_css_selector("base[href]")?;
    let Some(base_href) = document
        .select(&selector)
        .next()
        .and_then(|element| element.attr("href"))
    else {
        return Ok(final_url.to_string());
    };
    Ok(resolve_markdown_url(final_url, base_href))
}

fn remove_owned_excluded_tags(document: Html, tags: &[String]) -> Result<Html, AgetError> {
    if tags.is_empty() {
        return Ok(document);
    }
    remove_selected_elements(document, &tags.join(","))
}

fn remove_selected_elements(document: Html, selector_list: &str) -> Result<Html, AgetError> {
    let Ok(selector) = parse_css_selector(selector_list) else {
        return Ok(document);
    };
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

fn element_to_markdown(element: ElementRef<'_>, base_url: &str, only_text: bool) -> String {
    let mut writer = MarkdownWriter::new(base_url, only_text);
    render_node(*element, &mut writer);
    writer.append_abbreviation_definitions();
    normalize_markdown(&writer.output)
}

struct MarkdownWriter {
    output: String,
    base_url: Option<Url>,
    only_text: bool,
    list_depth: usize,
    abbreviations: Rc<RefCell<Vec<(String, String)>>>,
}

impl MarkdownWriter {
    fn new(base_url: &str, only_text: bool) -> Self {
        Self {
            output: String::new(),
            base_url: Url::parse(base_url).ok(),
            only_text,
            list_depth: 0,
            abbreviations: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn child(&self) -> Self {
        Self {
            output: String::new(),
            base_url: self.base_url.clone(),
            only_text: self.only_text,
            list_depth: self.list_depth,
            abbreviations: Rc::clone(&self.abbreviations),
        }
    }

    fn resolve_url(&self, raw: &str) -> String {
        self.base_url
            .as_ref()
            .map(|base| resolve_markdown_url(base.as_str(), raw))
            .unwrap_or_else(|| raw.to_string())
    }

    fn push_text(&mut self, text: &str) {
        let mut text = normalize_inline_markdown(text);
        if is_markdown_line_start(&self.output) {
            text = escape_markdown_line_start(&text);
        }
        self.push_inline(&text);
    }

    fn push_inline(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if needs_space_before_inline(&self.output) && !starts_with_closing_punctuation(text) {
            self.output.push(' ');
        }
        self.output.push_str(text);
    }

    fn ensure_blank_line(&mut self) {
        trim_trailing_horizontal_space(&mut self.output);
        if self.output.is_empty() {
            return;
        }
        match trailing_newline_count(&self.output) {
            0 => self.output.push_str("\n\n"),
            1 => self.output.push('\n'),
            _ => {}
        }
    }

    fn record_abbreviation(&mut self, text: String, title: String) {
        if text.is_empty() || title.is_empty() {
            return;
        }
        let mut abbreviations = self.abbreviations.borrow_mut();
        if let Some((_, existing_title)) = abbreviations
            .iter_mut()
            .find(|(existing_text, _)| existing_text == &text)
        {
            *existing_title = title;
        } else {
            abbreviations.push((text, title));
        }
    }

    fn append_abbreviation_definitions(&mut self) {
        let abbreviations = self.abbreviations.borrow().clone();
        if abbreviations.is_empty() {
            return;
        }
        self.ensure_blank_line();
        for (text, title) in abbreviations {
            self.output.push_str("  *[");
            self.output.push_str(&text);
            self.output.push_str("]: ");
            self.output.push_str(&title);
            self.output.push('\n');
        }
    }
}

fn render_node(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    match node.value() {
        Node::Text(text) => writer.push_text(text),
        Node::Element(element) => render_element(node, element.name(), writer),
        _ => render_children(node, writer),
    }
}

fn render_element(node: NodeRef<'_, Node>, tag: &str, writer: &mut MarkdownWriter) {
    if writer.only_text && is_only_text_eligible_tag(tag) {
        writer.push_text(&raw_text_from_node(node));
        return;
    }

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => render_heading(node, tag, writer),
        "p" => render_block(node, writer),
        "br" => writer.output.push_str("  \n"),
        "hr" => render_horizontal_rule(writer),
        "ul" => render_list(node, false, writer),
        "ol" => render_list(node, true, writer),
        "li" => render_block(node, writer),
        "dl" => render_definition_list(node, writer),
        "table" => render_table(node, writer),
        "pre" => render_code_block(node, writer),
        "code" | "kbd" | "tt" => writer.push_inline(&format!("`{}`", inline_text_from_node(node))),
        "strong" | "b" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("**{inner}**"));
        }
        "em" | "i" | "u" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("_{inner}_"));
        }
        "del" | "strike" | "s" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("~~{inner}~~"));
        }
        "q" => {
            let inner = inline_markdown_from_children(node, writer);
            writer.push_inline(&format!("\"{inner}\""));
        }
        "a" => render_link(node, writer),
        "abbr" => render_abbreviation(node, writer),
        "img" => render_image(node, writer),
        "blockquote" => render_blockquote(node, writer),
        "article" | "aside" | "body" | "div" | "footer" | "header" | "html" | "main" | "nav"
        | "section" => {
            render_children(node, writer);
            if is_structural_block(tag) {
                writer.ensure_blank_line();
            }
        }
        _ => render_children(node, writer),
    }
}

fn is_only_text_eligible_tag(tag: &str) -> bool {
    matches!(
        tag,
        "abbr"
            | "b"
            | "cite"
            | "code"
            | "del"
            | "dfn"
            | "em"
            | "i"
            | "ins"
            | "kbd"
            | "mark"
            | "q"
            | "s"
            | "small"
            | "span"
            | "strike"
            | "strong"
            | "sub"
            | "sup"
            | "tt"
            | "time"
            | "u"
            | "var"
    )
}

fn render_children(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        render_node(current, writer);
        child = next;
    }
}

fn render_heading(node: NodeRef<'_, Node>, tag: &str, writer: &mut MarkdownWriter) {
    let level = tag
        .strip_prefix('h')
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1)
        .clamp(1, 6);
    let text = inline_markdown_from_children(node, writer);
    if text.is_empty() {
        return;
    }
    writer.ensure_blank_line();
    writer.output.push_str(&"#".repeat(level));
    writer.output.push(' ');
    writer.output.push_str(&text);
    writer.ensure_blank_line();
}

fn render_block(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    render_children(node, writer);
    writer.ensure_blank_line();
}

fn render_horizontal_rule(writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    writer.output.push_str("* * *");
    writer.ensure_blank_line();
}

fn render_list(node: NodeRef<'_, Node>, ordered: bool, writer: &mut MarkdownWriter) {
    let is_nested = writer.list_depth > 0;
    if is_nested {
        trim_trailing_horizontal_space(&mut writer.output);
        if !writer.output.is_empty() && !writer.output.ends_with('\n') {
            writer.output.push('\n');
        }
    } else {
        writer.ensure_blank_line();
    }
    writer.list_depth += 1;
    let item_depth = writer.list_depth;
    let mut child = node.first_child();
    let mut index = 1usize;
    while let Some(current) = child {
        let next = current.next_sibling();
        if let Some(element) = ElementRef::wrap(current) {
            if element.value().name() == "li" {
                trim_trailing_horizontal_space(&mut writer.output);
                if !writer.output.is_empty() && !writer.output.ends_with('\n') {
                    writer.output.push('\n');
                }
                writer.output.push_str(&list_item_indent(item_depth));
                if ordered {
                    writer.output.push_str(&format!("{index}. "));
                } else {
                    writer.output.push_str("* ");
                }
                render_children(current, writer);
                trim_trailing_horizontal_space(&mut writer.output);
                if !writer.output.ends_with('\n') {
                    writer.output.push('\n');
                }
                index += 1;
            } else {
                render_node(current, writer);
            }
        } else {
            render_node(current, writer);
        }
        child = next;
    }
    writer.list_depth = writer.list_depth.saturating_sub(1);
    if is_nested {
        trim_trailing_horizontal_space(&mut writer.output);
        if !writer.output.ends_with('\n') {
            writer.output.push('\n');
        }
    } else {
        writer.ensure_blank_line();
    }
}

fn list_item_indent(depth: usize) -> String {
    "  ".repeat(depth.saturating_sub(1))
}

fn render_definition_list(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    writer.ensure_blank_line();
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        if let Some(element) = ElementRef::wrap(current) {
            match element.value().name() {
                "dt" => {
                    let term = inline_markdown_from_children(current, writer);
                    if !term.is_empty() {
                        if !writer.output.ends_with('\n') {
                            writer.output.push('\n');
                        }
                        writer.output.push_str(&term);
                        writer.output.push('\n');
                    }
                }
                "dd" => {
                    let definition = inline_markdown_from_children(current, writer);
                    if !definition.is_empty() {
                        writer.output.push_str("    ");
                        writer.output.push_str(&definition);
                        writer.output.push('\n');
                    }
                }
                _ => render_node(current, writer),
            }
        } else {
            render_node(current, writer);
        }
        child = next;
    }
    writer.ensure_blank_line();
}

fn render_code_block(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let text = raw_text_from_node(node);
    if text.trim().is_empty() {
        return;
    }
    writer.ensure_blank_line();
    writer.output.push_str("```\n");
    writer.output.push_str(text.trim_matches('\n'));
    writer.output.push_str("\n```");
    writer.ensure_blank_line();
}

struct MarkdownTableRow {
    cells: Vec<String>,
    has_header_cells: bool,
}

fn render_table(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let caption = table_caption_text(node, writer);
    let mut rows = Vec::new();
    collect_table_rows(node, writer, &mut rows);
    let Some(max_columns) = rows.iter().map(|row| row.cells.len()).max() else {
        if !caption.is_empty() {
            writer.ensure_blank_line();
            writer.output.push_str(&caption);
            writer.ensure_blank_line();
        }
        return;
    };
    if max_columns == 0 {
        if !caption.is_empty() {
            writer.ensure_blank_line();
            writer.output.push_str(&caption);
            writer.ensure_blank_line();
        }
        return;
    }

    let header_index = rows
        .iter()
        .position(|row| row.has_header_cells)
        .unwrap_or(0);
    let header = normalize_table_row(&rows[header_index].cells, max_columns);
    let body_rows = rows
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != header_index)
        .map(|(_, row)| normalize_table_row(&row.cells, max_columns))
        .collect::<Vec<_>>();

    writer.ensure_blank_line();
    if !caption.is_empty() {
        writer.output.push_str(&caption);
        writer.ensure_blank_line();
    }
    writer.output.push_str(&markdown_table_line(&header));
    writer.output.push('\n');
    writer
        .output
        .push_str(&markdown_table_line(&vec!["---".to_string(); max_columns]));
    writer.output.push('\n');
    for row in body_rows {
        writer.output.push_str(&markdown_table_line(&row));
        writer.output.push('\n');
    }
    writer.ensure_blank_line();
}

fn table_caption_text(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> String {
    if let Some(element) = ElementRef::wrap(node) {
        if element.value().name() == "caption" {
            return inline_markdown_from_children(node, writer);
        }
    }

    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        let caption = table_caption_text(current, writer);
        if !caption.is_empty() {
            return caption;
        }
        child = next;
    }
    String::new()
}

fn collect_table_rows(
    node: NodeRef<'_, Node>,
    writer: &MarkdownWriter,
    rows: &mut Vec<MarkdownTableRow>,
) {
    if let Some(element) = ElementRef::wrap(node) {
        if element.value().name() == "tr" {
            rows.push(markdown_table_row(node, writer));
            return;
        }
    }

    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        collect_table_rows(current, writer, rows);
        child = next;
    }
}

fn markdown_table_row(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> MarkdownTableRow {
    let mut cells = Vec::new();
    let mut has_header_cells = false;
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        if let Some(element) = ElementRef::wrap(current) {
            match element.value().name() {
                "th" => {
                    has_header_cells = true;
                    cells.push(table_cell_text(current, writer));
                }
                "td" => cells.push(table_cell_text(current, writer)),
                _ => {}
            }
        }
        child = next;
    }
    MarkdownTableRow {
        cells,
        has_header_cells,
    }
}

fn table_cell_text(node: NodeRef<'_, Node>, writer: &MarkdownWriter) -> String {
    escape_table_cell(&inline_markdown_from_children(node, writer))
}

fn normalize_table_row(cells: &[String], columns: usize) -> Vec<String> {
    let mut row = cells.to_vec();
    row.resize(columns, String::new());
    row
}

fn markdown_table_line(cells: &[String]) -> String {
    format!("| {} |", cells.join(" | "))
}

fn render_link(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let Some(element) = ElementRef::wrap(node) else {
        render_children(node, writer);
        return;
    };
    let Some(href) = element.attr("href") else {
        render_children(node, writer);
        return;
    };
    if href.starts_with("mailto:") || href.starts_with('#') {
        render_children(node, writer);
        return;
    }
    let label = inline_markdown_from_children(node, writer);
    let title = element.attr("title").unwrap_or("").trim();
    if title.is_empty() && !label.is_empty() && label == href && is_absolute_http_url(href) {
        writer.push_inline(&format!("<{}>", href));
        return;
    }
    let title = if title.is_empty() {
        String::new()
    } else {
        format!(" \"{}\"", escape_link_title(title))
    };
    writer.push_inline(&format!(
        "[{}]({}{})",
        escape_link_text(&label),
        escape_markdown_link_target(&writer.resolve_url(href)),
        title
    ));
}

fn render_abbreviation(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let text = inline_markdown_from_children(node, writer);
    if text.is_empty() {
        return;
    }
    if let Some(title) = ElementRef::wrap(node)
        .and_then(|element| element.attr("title"))
        .map(normalize_inline_markdown)
    {
        writer.record_abbreviation(text.clone(), title);
    }
    writer.push_inline(&text);
}

fn render_image(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let Some(element) = ElementRef::wrap(node) else {
        return;
    };
    let Some(src) = element.attr("src") else {
        return;
    };
    if src.trim().is_empty() {
        return;
    }
    let alt = element.attr("alt").unwrap_or("");
    writer.push_inline(&format!(
        "![{}]({})",
        escape_markdown_link_target(alt),
        escape_markdown_link_target(&writer.resolve_url(src))
    ));
}

fn render_blockquote(node: NodeRef<'_, Node>, writer: &mut MarkdownWriter) {
    let mut child_writer = writer.child();
    render_children(node, &mut child_writer);
    let quote = normalize_markdown(&child_writer.output);
    if quote.is_empty() {
        return;
    }
    writer.ensure_blank_line();
    for line in quote.lines() {
        writer.output.push('>');
        let line = line.trim();
        if !line.is_empty() {
            writer.output.push(' ');
            writer.output.push_str(line);
        }
        writer.output.push('\n');
    }
    writer.ensure_blank_line();
}

fn inline_markdown_from_children(node: NodeRef<'_, Node>, parent: &MarkdownWriter) -> String {
    let mut writer = parent.child();
    render_children(node, &mut writer);
    normalize_inline_markdown(&writer.output)
}

fn inline_text_from_node(node: NodeRef<'_, Node>) -> String {
    normalize_inline_markdown(&raw_text_from_node(node))
}

fn raw_text_from_node(node: NodeRef<'_, Node>) -> String {
    let mut output = String::new();
    collect_raw_text(node, &mut output);
    output
}

fn collect_raw_text(node: NodeRef<'_, Node>, output: &mut String) {
    if let Node::Text(text) = node.value() {
        output.push_str(text);
    }
    let mut child = node.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        collect_raw_text(current, output);
        child = next;
    }
}

fn normalize_markdown(markdown: &str) -> String {
    let mut output = String::new();
    let mut blank_lines = 0usize;
    for line in markdown.lines().map(normalize_markdown_line_end) {
        if line.trim().is_empty() {
            blank_lines += 1;
            if blank_lines <= 1 && !output.is_empty() {
                output.push('\n');
            }
            continue;
        }
        blank_lines = 0;
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&line);
        output.push('\n');
    }
    output.trim().to_string()
}

fn normalize_markdown_line_end(line: &str) -> String {
    let without_tabs = line.trim_end_matches('\t');
    if without_tabs.ends_with("  ") {
        let without_spaces = without_tabs.trim_end_matches(' ');
        if !without_spaces.is_empty() {
            return format!("{without_spaces}  ");
        }
    }
    line.trim_end().to_string()
}

fn normalize_inline_markdown(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn needs_space_before_inline(output: &str) -> bool {
    output
        .chars()
        .last()
        .is_some_and(|character| !character.is_whitespace())
}

fn starts_with_closing_punctuation(text: &str) -> bool {
    if text.starts_with("![") {
        return false;
    }
    text.chars()
        .next()
        .is_some_and(|character| matches!(character, '.' | ',' | ':' | ';' | '!' | '?' | ')' | ']'))
}

fn is_markdown_line_start(output: &str) -> bool {
    output.is_empty() || output.ends_with('\n')
}

fn escape_markdown_line_start(text: &str) -> String {
    if starts_with_ordered_list_marker(text) {
        return text.replacen('.', "\\.", 1);
    }
    if text
        .strip_prefix(['-', '+'])
        .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    {
        return format!("\\{text}");
    }
    text.to_string()
}

fn starts_with_ordered_list_marker(text: &str) -> bool {
    let Some((prefix, rest)) = text.split_once('.') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|character| character.is_ascii_digit())
        && rest.starts_with(char::is_whitespace)
}

fn trim_trailing_horizontal_space(output: &mut String) {
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
}

fn trailing_newline_count(output: &str) -> usize {
    output.chars().rev().take_while(|&c| c == '\n').count()
}

fn escape_link_text(text: &str) -> String {
    text.replace('[', "\\[").replace(']', "\\]")
}

fn escape_markdown_link_target(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn escape_link_title(text: &str) -> String {
    escape_markdown_link_target(text).replace('"', "\\\"")
}

fn is_absolute_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn escape_table_cell(text: &str) -> String {
    text.replace('\n', " ").replace('|', "\\|")
}

fn resolve_markdown_url(base: &str, raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("mailto:") {
        return raw.to_string();
    }
    Url::parse(base)
        .and_then(|base| base.join(raw))
        .map(|url| url.to_string())
        .unwrap_or_else(|_| raw.to_string())
}

fn is_structural_block(tag: &str) -> bool {
    matches!(
        tag,
        "article" | "aside" | "footer" | "header" | "main" | "nav" | "section"
    )
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

    let command_string = command_override
        .or_else(|| env::var("AGET_CRAWL4AI_COMMAND").ok())
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message:
                "Crawl4AI compatibility backend requires AGET_CRAWL4AI_COMMAND or an explicit command"
                    .to_string(),
        })?;
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
    let command = env::var("AGET_AGENT_BROWSER_COMMAND").map_err(|_| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: "agent-browser compatibility backend requires AGET_AGENT_BROWSER_COMMAND"
            .to_string(),
    })?;
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
