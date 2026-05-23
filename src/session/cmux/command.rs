use std::env;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{AgetError, ErrorCode};
use crate::process::{
    configure_local_command, create_private_file, wait_for_child, TempOutputFile,
    DEFAULT_SUBPROCESS_TIMEOUT,
};

use super::types::CmuxCookiesResponse;

pub(super) fn run_cmux_cookies_get(
    tmp_dir: &Path,
    surface: &str,
    domain: &str,
) -> Result<CmuxCookiesResponse, AgetError> {
    let command = env::var("AGET_CMUX_COMMAND").unwrap_or_else(|_| "cmux".to_string());
    let stdout_file = TempOutputFile::new(tmp_dir, "cmux-stdout").map_err(io_aget_error)?;
    let stderr_file = TempOutputFile::new(tmp_dir, "cmux-stderr").map_err(io_aget_error)?;
    let stdout = create_private_file(stdout_file.path()).map_err(io_aget_error)?;
    let stderr = create_private_file(stderr_file.path()).map_err(io_aget_error)?;
    let mut command_builder = Command::new(&command);
    command_builder
        .args([
            "--json",
            "browser",
            "--surface",
            surface,
            "cookies",
            "get",
            "--domain",
            domain,
        ])
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    configure_local_command(&mut command_builder);
    let mut child = command_builder
        .spawn()
        .map_err(|error| backend_unavailable(&command, error))?;
    let status = wait_for_child(&mut child, DEFAULT_SUBPROCESS_TIMEOUT, || {
        format!(
            "cmux cookies get timed out after {} seconds",
            DEFAULT_SUBPROCESS_TIMEOUT.as_secs()
        )
    })?;
    let stdout = stdout_file.read_to_string().map_err(io_aget_error)?;
    let stderr = stderr_file.read_to_string().map_err(io_aget_error)?;

    if !status.success() {
        let stderr = stderr.trim().to_string();
        let code = if matches!(status.code(), Some(126 | 127)) {
            ErrorCode::BackendUnavailable
        } else {
            ErrorCode::ExtractionFailed
        };
        return Err(AgetError::Stable {
            code,
            message: if stderr.is_empty() {
                format!("cmux exited with {status}")
            } else {
                stderr
            },
        });
    }

    serde_json::from_str(&stdout).map_err(|error| AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: format!("cmux returned malformed JSON: {error}"),
    })
}

fn backend_unavailable(command: &str, error: io::Error) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("cmux backend is unavailable for command '{command}': {error}"),
    }
}

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}
