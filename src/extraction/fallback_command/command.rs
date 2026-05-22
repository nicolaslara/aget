use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::error::{AgetError, ErrorCode};
use crate::process::{configure_local_command, wait_for_child};

use super::super::{io_aget_error, read_output_file};
use super::temp::TempOutputFile;

#[derive(Debug)]
pub(super) struct AgentBrowserOutput {
    pub(super) status: std::process::ExitStatus,
    pub(super) stdout: String,
    pub(super) stderr: String,
}

pub(super) fn run_agent_browser(
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
    let stdout = stdout_file.create()?;
    let stderr = stderr_file.create()?;
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

pub(super) fn classify_agent_browser_failure(
    action: &str,
    output: &AgentBrowserOutput,
) -> AgetError {
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
