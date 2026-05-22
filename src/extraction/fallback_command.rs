mod command;
mod temp;
mod text;

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cli::OutputFormat;
use crate::error::AgetError;

use super::{BrowserFallbackResult, GetOptions, FALLBACK_EXTRACTOR, FALLBACK_WARNING};

use command::{classify_agent_browser_failure, run_agent_browser};
use temp::TempAgentBrowserProfile;
use text::html_to_text;

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
        page_metadata: Default::default(),
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

fn unique_agent_browser_session_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("aget-fallback-{}-{nanos}", std::process::id())
}
