use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cli::OutputFormat;
use crate::error::{AgetError, ErrorCode};
use crate::process::{configure_local_command, wait_for_child};

use super::{
    create_private_dir, create_private_file, io_aget_error, read_output_file,
    BrowserFallbackResult, GetOptions, FALLBACK_EXTRACTOR, FALLBACK_WARNING,
};

#[derive(Debug)]
struct AgentBrowserOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

pub(super) fn run_agent_browser_fallback(
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
