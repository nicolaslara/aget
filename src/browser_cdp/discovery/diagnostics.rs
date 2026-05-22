use crate::error::{AgetError, ErrorCode};
use crate::process::TempOutputFile;

use super::super::{CHROME_SANDBOX_STARTUP_HINT, CHROME_SILENT_STARTUP_HINT};

pub(super) fn devtools_ws_url_from_chrome_stderr(
    stderr_capture: &TempOutputFile,
) -> Option<String> {
    let stderr = stderr_capture.read_to_string().ok()?;
    devtools_ws_url_from_stderr(&stderr)
}

pub(in crate::browser_cdp) fn devtools_ws_url_from_stderr(stderr: &str) -> Option<String> {
    stderr.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("DevTools listening on ")?;
        let url = rest.split_whitespace().next().unwrap_or(rest);
        if url.starts_with("ws://") || url.starts_with("wss://") {
            Some(url.to_string())
        } else {
            None
        }
    })
}

pub(in crate::browser_cdp) fn classify_chrome_startup_error(
    operation: &str,
    error: AgetError,
    stderr_capture: &TempOutputFile,
) -> AgetError {
    let stderr = stderr_capture.read_to_string().unwrap_or_default();
    let detail = relevant_chrome_stderr(&stderr);
    let combined = format!("{}\n{}", error, detail);
    if chrome_startup_requires_user_action(&combined) {
        return AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: if detail.is_empty() {
                format!("{operation} requires user action: {error}")
            } else {
                format!("{operation} requires user action: {error}; Chrome stderr: {detail}")
            },
        };
    }

    if detail.is_empty() {
        if stderr.trim().is_empty() {
            return append_silent_chrome_startup_hint(error);
        }
        return error;
    }

    match error {
        AgetError::Stable { code, message } => AgetError::Stable {
            code,
            message: format!("{message}; Chrome stderr: {detail}"),
        },
    }
}

fn chrome_startup_requires_user_action(output: &str) -> bool {
    let output = output.to_ascii_lowercase();
    [
        "opening in existing browser session",
        "profile lock",
        "profile is locked",
        "profile in use",
        "profile appears to be in use",
        "already running",
        "another chrome process",
        "another google chrome process",
        "processsingleton",
        "process singleton",
        "singletonlock",
        "singleton lock",
        "user data directory is already in use",
        "chrome must be quit",
        "quit chrome",
        "close chrome",
    ]
    .iter()
    .any(|needle| output.contains(needle))
}

pub(in crate::browser_cdp) fn relevant_chrome_stderr(stderr: &str) -> String {
    let relevant = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            [
                "error",
                "fatal",
                "sandbox",
                "namespace",
                "permission",
                "cannot",
                "failed",
                "abort",
                "profile",
                "singleton",
                "user data directory",
                "already running",
                "opening in existing browser session",
            ]
            .iter()
            .any(|needle| lower.contains(needle))
        })
        .take(5)
        .collect::<Vec<_>>();
    if relevant.is_empty() {
        let lines = stderr
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .rev()
            .take(5)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n  ");
        if lines.is_empty() {
            String::new()
        } else {
            let count = lines.lines().count();
            format!("Chrome stderr (last {count} lines):\n  {lines}")
        }
    } else {
        append_chrome_startup_hint(relevant.join("\n  "))
    }
}

fn append_chrome_startup_hint(detail: String) -> String {
    let lower = detail.to_ascii_lowercase();
    if lower.contains("sandbox") || lower.contains("namespace") {
        format!("{detail}\n  {CHROME_SANDBOX_STARTUP_HINT}")
    } else {
        detail
    }
}

fn append_silent_chrome_startup_hint(error: AgetError) -> AgetError {
    match error {
        AgetError::Stable { code, message } => AgetError::Stable {
            code,
            message: format!("{message}; {CHROME_SILENT_STARTUP_HINT}"),
        },
    }
}
