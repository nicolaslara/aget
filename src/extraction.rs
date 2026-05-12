use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::cli::{ExtractorOption, OutputFormat};
use crate::error::{AgetError, ErrorCode};
use crate::session::{compose_playwright_state, PlaywrightState, SessionStore, TempStateFile};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const EXTRACTOR: &str = "crawl4ai";
const FALLBACK_EXTRACTOR: &str = "agent-browser-fallback";
const FALLBACK_WARNING: &str = "agent-browser fallback used after Crawl4AI failed";

#[derive(Debug, Clone)]
pub struct GetOptions {
    pub url: String,
    pub sessions: Vec<String>,
    pub out: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub format: OutputFormat,
    pub selector: Option<String>,
    pub exclude_selector: Option<String>,
    pub wait_for: Option<String>,
    pub max_chars: Option<usize>,
    pub extractor_options: Vec<ExtractorOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSuccess {
    pub ok: bool,
    pub url: String,
    pub final_url: String,
    pub format: String,
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
    pub markdown: String,
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
    pub format: OutputFormat,
    pub selector: Option<String>,
    pub exclude_selector: Option<String>,
    pub wait_for: Option<String>,
    pub extractor_options: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct BackendResult {
    ok: bool,
    #[serde(default)]
    final_url: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(default)]
    error: Option<String>,
}

pub fn get_url(options: GetOptions) -> Result<GetSuccess, AgetError> {
    let started = Instant::now();
    let store = SessionStore::from_env().map_err(io_aget_error)?;
    let sessions = load_selected_sessions(&store, &options.sessions)?;
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

    let markdown_path = options
        .out
        .clone()
        .unwrap_or_else(|| run_dir.join("content.md"));
    if let Some(parent) = markdown_path.parent() {
        fs::create_dir_all(parent).map_err(io_aget_error)?;
    }
    let metadata_path = run_dir.join("metadata.json");

    let backend = match run_backend(
        &options.url,
        temp_state.path(),
        &markdown_path,
        &metadata_path,
        &options,
        options.timeout.unwrap_or(DEFAULT_TIMEOUT),
    ) {
        Ok(backend) => backend,
        Err(error) => {
            if sensitive {
                sanitize_backend_artifacts(&metadata_path, &sensitive_values);
                if let Ok(fallback) = run_agent_browser_fallback(
                    &store.home().join("tmp"),
                    &options.url,
                    temp_state.path(),
                    &options,
                    options.timeout.unwrap_or(DEFAULT_TIMEOUT),
                ) {
                    return finalize_success(
                        &options,
                        &markdown_path,
                        &metadata_path,
                        selected_session_names,
                        sensitive,
                        output_options,
                        fallback,
                        started,
                    );
                }
            }
            let error = sanitize_backend_error(error, sensitive);
            let _ = write_error_metadata(
                &metadata_path,
                &options.url,
                &markdown_path,
                &selected_session_names,
                sensitive,
                &output_options,
                &options,
                &error,
                started,
            );
            return Err(error);
        }
    };

    if sensitive {
        sanitize_backend_artifacts(&metadata_path, &sensitive_values);
    }

    if !backend.ok {
        let error = sanitize_backend_error(
            AgetError::Stable {
                code: ErrorCode::ExtractionFailed,
                message: backend
                    .error
                    .unwrap_or_else(|| "Crawl4AI extraction failed".to_string()),
            },
            sensitive,
        );
        if sensitive {
            sanitize_backend_artifacts(&metadata_path, &sensitive_values);
            if let Ok(fallback) = run_agent_browser_fallback(
                &store.home().join("tmp"),
                &options.url,
                temp_state.path(),
                &options,
                options.timeout.unwrap_or(DEFAULT_TIMEOUT),
            ) {
                return finalize_success(
                    &options,
                    &markdown_path,
                    &metadata_path,
                    selected_session_names,
                    sensitive,
                    output_options,
                    fallback,
                    started,
                );
            }
        }
        let _ = write_error_metadata(
            &metadata_path,
            &options.url,
            &markdown_path,
            &selected_session_names,
            sensitive,
            &output_options,
            &options,
            &error,
            started,
        );
        return Err(error);
    }

    let content = match backend.content {
        Some(content) => content,
        None => fs::read_to_string(&markdown_path).map_err(|error| AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!(
                "backend did not return content and markdown artifact could not be read: {error}"
            ),
        })?,
    };
    finalize_success(
        &options,
        &markdown_path,
        &metadata_path,
        selected_session_names,
        sensitive,
        output_options,
        SuccessfulExtraction {
            final_url: backend.final_url.unwrap_or_else(|| options.url.clone()),
            content,
            warnings: backend.warnings,
            extractor: EXTRACTOR.to_string(),
        },
        started,
    )
}

struct SuccessfulExtraction {
    final_url: String,
    content: String,
    warnings: Vec<String>,
    extractor: String,
}

fn finalize_success(
    options: &GetOptions,
    markdown_path: &Path,
    metadata_path: &Path,
    selected_session_names: Vec<String>,
    sensitive: bool,
    output_options: OutputOptions,
    extraction: SuccessfulExtraction,
    started: Instant,
) -> Result<GetSuccess, AgetError> {
    let limits = apply_limits(extraction.content, options.max_chars);
    let content = limits.content;
    write_private_file(markdown_path, content.as_bytes()).map_err(io_aget_error)?;

    let success = GetSuccess {
        ok: true,
        url: options.url.clone(),
        final_url: extraction.final_url,
        format: options.format.to_string(),
        extractor: extraction.extractor,
        content,
        artifacts: Artifacts {
            markdown: markdown_path.to_string_lossy().into_owned(),
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
    store: &SessionStore,
    session_names: &[String],
) -> Result<Vec<crate::session::Session>, AgetError> {
    session_names
        .iter()
        .map(|name| store.load(name).map_err(io_aget_error))
        .collect()
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
    let extractor_options = options
        .extractor_options
        .iter()
        .map(|option| (option.key.clone(), option.value.clone()))
        .collect();

    OutputOptions {
        format: options.format,
        selector: options.selector.clone(),
        exclude_selector: options.exclude_selector.clone(),
        wait_for: options.wait_for.clone(),
        extractor_options,
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
    for value in sensitive_values {
        redacted = redacted.replace(value, "<redacted>");
    }
    redacted
}

fn run_backend(
    url: &str,
    state_path: &Path,
    markdown_path: &Path,
    metadata_path: &Path,
    options: &GetOptions,
    timeout: Duration,
) -> Result<BackendResult, AgetError> {
    let mut args = vec![
        "--url".to_string(),
        url.to_string(),
        "--state".to_string(),
        state_path.to_string_lossy().into_owned(),
        "--output".to_string(),
        markdown_path.to_string_lossy().into_owned(),
        "--metadata".to_string(),
        metadata_path.to_string_lossy().into_owned(),
        "--format".to_string(),
        options.format.to_string(),
    ];
    if let Some(selector) = &options.selector {
        args.push("--selector".to_string());
        args.push(selector.clone());
    }
    if let Some(exclude_selector) = &options.exclude_selector {
        args.push("--exclude-selector".to_string());
        args.push(exclude_selector.clone());
    }
    if let Some(wait_for) = &options.wait_for {
        args.push("--wait-for".to_string());
        args.push(wait_for.clone());
    }
    for extractor_option in &options.extractor_options {
        args.push("--extractor-option".to_string());
        args.push(format!(
            "{}={}",
            extractor_option.key, extractor_option.value
        ));
    }

    let command_string = env::var("AGET_CRAWL4AI_COMMAND").unwrap_or_else(|_| default_command());
    let backend_stdout_path = metadata_path.with_file_name("backend-stdout.json");
    let backend_stderr_path = metadata_path.with_file_name("backend-stderr.txt");
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
    configure_backend_command(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| backend_unavailable(&command_string, error))?;

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() >= deadline => {
                terminate_backend(&mut child);
                let _ = child.wait();
                return Err(AgetError::Stable {
                    code: ErrorCode::Timeout,
                    message: format!(
                        "Crawl4AI backend timed out after {} seconds",
                        timeout.as_secs()
                    ),
                });
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => return Err(io_aget_error(error)),
        }
    }

    let status = child.wait().map_err(io_aget_error)?;
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

fn parse_backend_stdout(stdout: &str) -> Result<BackendResult, AgetError> {
    let trimmed = stdout.trim();
    match serde_json::from_str(trimmed) {
        Ok(result) => return Ok(result),
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
) -> Result<SuccessfulExtraction, AgetError> {
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
    Ok(SuccessfulExtraction {
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
        return Ok(match options.format {
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

    Ok(match options.format {
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
    configure_backend_command(&mut command_builder);
    let mut child = command_builder.spawn().map_err(|error| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("agent-browser backend is unavailable for command '{command}': {error}"),
    })?;

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() >= deadline => {
                terminate_backend(&mut child);
                let _ = child.wait();
                return Err(AgetError::Stable {
                    code: ErrorCode::Timeout,
                    message: format!(
                        "agent-browser fallback timed out after {} seconds",
                        timeout.as_secs()
                    ),
                });
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => return Err(io_aget_error(error)),
        }
    }

    let status = child.wait().map_err(io_aget_error)?;

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

#[cfg(unix)]
fn configure_backend_command(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_backend_command(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_backend(child: &mut std::process::Child) {
    let process_group = format!("-{}", child.id());
    let _ = Command::new("kill")
        .args(["-TERM", &process_group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    thread::sleep(Duration::from_millis(50));
    let _ = Command::new("kill")
        .args(["-KILL", &process_group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(unix))]
fn terminate_backend(child: &mut std::process::Child) {
    let _ = child.kill();
}

fn write_error_metadata(
    path: &Path,
    url: &str,
    markdown_path: &Path,
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
        "format": options.format.to_string(),
        "extractor": EXTRACTOR,
        "artifacts": {
            "markdown": markdown_path.to_string_lossy(),
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
    metadata.insert("format", serde_json::json!(success.format));
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
