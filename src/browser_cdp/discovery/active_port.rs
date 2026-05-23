use std::fs;
use std::path::Path;
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{AgetError, ErrorCode};
use crate::process::TempOutputFile;

use super::devtools_ws_url_from_chrome_stderr;

pub(in crate::browser_cdp) fn wait_for_devtools_active_port(
    child: &mut Child,
    user_data_dir: &Path,
    stderr_capture: &TempOutputFile,
    timeout: Duration,
) -> Result<String, AgetError> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(Some(status)) = child.try_wait() {
            let exit_code = status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            return Err(AgetError::Stable {
                code: ErrorCode::BackendUnavailable,
                message: format!(
                    "owned browser fallback Chrome exited early (exit code: {exit_code}) without writing DevToolsActivePort"
                ),
            });
        }
        if let Some((port, path)) = read_devtools_active_port(user_data_dir) {
            return Ok(format!("ws://127.0.0.1:{port}{path}"));
        }
        if let Some(ws_url) = devtools_ws_url_from_chrome_stderr(stderr_capture) {
            return Ok(ws_url);
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(AgetError::Stable {
        code: ErrorCode::Timeout,
        message: "owned browser fallback timed out waiting for Chrome CDP startup".to_string(),
    })
}

pub(in crate::browser_cdp) fn read_devtools_active_port(
    user_data_dir: &Path,
) -> Option<(u16, String)> {
    let content = fs::read_to_string(user_data_dir.join("DevToolsActivePort")).ok()?;
    let mut lines = content.lines();
    let port = lines.next()?.trim().parse::<u16>().ok()?;
    let path = lines
        .next()
        .unwrap_or("/devtools/browser")
        .trim()
        .to_string();
    Some((port, path))
}
