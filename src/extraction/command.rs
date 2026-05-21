use std::env;
use std::process::{Command, Stdio};

use crate::error::{AgetError, ErrorCode};
use crate::process::{configure_local_command, wait_for_child};

use super::{
    backend_unavailable, create_private_file, io_aget_error, read_output_file, write_private_file,
    ExtractorBackendResult, ExtractorRequest,
};

pub(super) fn run_command_extractor_backend(
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
