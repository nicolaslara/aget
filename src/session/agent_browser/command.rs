use std::env;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{AgetError, ErrorCode};
use crate::process::{
    configure_local_command, create_private_file as create_private_output_file, wait_for_child,
    TempOutputFile, DEFAULT_SUBPROCESS_TIMEOUT,
};

#[derive(Debug)]
pub(crate) struct AgentBrowserOutput {
    pub(crate) status: std::process::ExitStatus,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn run_agent_browser(
    tmp_dir: &Path,
    args: &[&str],
) -> Result<AgentBrowserOutput, AgetError> {
    let command = env::var("AGET_AGENT_BROWSER_COMMAND").map_err(|_| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: "agent-browser compatibility backend requires AGET_AGENT_BROWSER_COMMAND"
            .to_string(),
    })?;
    let stdout_file =
        TempOutputFile::new(tmp_dir, "agent-browser-stdout").map_err(io_aget_error)?;
    let stderr_file =
        TempOutputFile::new(tmp_dir, "agent-browser-stderr").map_err(io_aget_error)?;
    let stdout = create_private_output_file(stdout_file.path()).map_err(io_aget_error)?;
    let stderr = create_private_output_file(stderr_file.path()).map_err(io_aget_error)?;
    let mut command_builder = Command::new(&command);
    command_builder
        .args(args)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    configure_local_command(&mut command_builder);
    let mut child = command_builder
        .spawn()
        .map_err(|error| backend_unavailable(&command, error))?;
    let status = wait_for_child(&mut child, DEFAULT_SUBPROCESS_TIMEOUT, || {
        format!(
            "agent-browser timed out after {} seconds",
            DEFAULT_SUBPROCESS_TIMEOUT.as_secs()
        )
    })?;
    Ok(AgentBrowserOutput {
        status,
        stdout: stdout_file.read_to_string().map_err(io_aget_error)?,
        stderr: stderr_file.read_to_string().map_err(io_aget_error)?,
    })
}

pub(crate) fn classify_agent_browser_failure(
    action: &str,
    output: &AgentBrowserOutput,
) -> AgetError {
    let combined = format!("{}\n{}", output.stdout, output.stderr);
    if indicates_user_action(&combined) {
        return AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: user_action_message(action, combined.trim()),
        };
    }

    let code = if matches!(output.status.code(), Some(126 | 127)) {
        ErrorCode::BackendUnavailable
    } else {
        ErrorCode::ExtractionFailed
    };
    AgetError::Stable {
        code,
        message: if combined.trim().is_empty() {
            format!("agent-browser {action} exited with {}", output.status)
        } else {
            combined.trim().to_string()
        },
    }
}

pub(super) fn indicates_user_action(output: &str) -> bool {
    let output = output.to_ascii_lowercase();
    [
        "quit chrome",
        "close chrome",
        "chrome must be quit",
        "profile lock",
        "profile is locked",
        "profile in use",
        "already running",
        "login needed",
        "please log in",
        "login required",
        "not logged in",
        "please sign in",
        "sign in required",
        "no auth state",
        "no authentication state",
    ]
    .iter()
    .any(|needle| output.contains(needle))
}

fn user_action_message(action: &str, details: &str) -> String {
    if details.is_empty() {
        format!("agent-browser {action} requires user action")
    } else {
        format!("agent-browser {action} requires user action: {details}")
    }
}

fn backend_unavailable(command: &str, error: io::Error) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("agent-browser backend is unavailable for command '{command}': {error}"),
    }
}

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}
