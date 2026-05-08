use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{AgetError, ErrorCode};
use crate::session::{compose_playwright_state, SessionStore, TempStateFile};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const EXTRACTOR: &str = "crawl4ai";

#[derive(Debug, Clone)]
pub struct GetOptions {
    pub url: String,
    pub session: Option<String>,
    pub out: Option<PathBuf>,
    pub timeout: Option<Duration>,
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
    pub max_tokens: Option<usize>,
    pub truncated: bool,
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
    let sessions = load_selected_sessions(&store, options.session.as_deref())?;
    let selected_session_names = sessions
        .iter()
        .map(|session| session.name.clone())
        .collect::<Vec<_>>();
    let sensitive = !sessions.is_empty();
    let state = compose_playwright_state(&sessions)?;
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
        options.timeout.unwrap_or(DEFAULT_TIMEOUT),
    ) {
        Ok(backend) => backend,
        Err(error) => {
            let _ = write_error_metadata(
                &metadata_path,
                &options.url,
                &markdown_path,
                &selected_session_names,
                sensitive,
                &error,
                started,
            );
            return Err(error);
        }
    };

    if !backend.ok {
        let error = AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: backend
                .error
                .unwrap_or_else(|| "Crawl4AI extraction failed".to_string()),
        };
        let _ = write_error_metadata(
            &metadata_path,
            &options.url,
            &markdown_path,
            &selected_session_names,
            sensitive,
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
    write_private_file(&markdown_path, content.as_bytes()).map_err(io_aget_error)?;

    let success = GetSuccess {
        ok: true,
        url: options.url.clone(),
        final_url: backend.final_url.unwrap_or_else(|| options.url.clone()),
        format: "markdown".to_string(),
        extractor: EXTRACTOR.to_string(),
        content,
        artifacts: Artifacts {
            markdown: markdown_path.to_string_lossy().into_owned(),
            metadata: metadata_path.to_string_lossy().into_owned(),
        },
        sessions: selected_session_names,
        sensitive,
        warnings: backend.warnings,
        timing_ms: TimingMs {
            total: started.elapsed().as_millis(),
        },
        limits: Limits {
            max_chars: None,
            max_tokens: None,
            truncated: false,
        },
    };

    write_metadata(&metadata_path, &success)?;
    Ok(success)
}

fn load_selected_sessions(
    store: &SessionStore,
    session_name: Option<&str>,
) -> Result<Vec<crate::session::Session>, AgetError> {
    match session_name {
        Some(name) => store
            .load(name)
            .map(|session| vec![session])
            .map_err(io_aget_error),
        None => Ok(Vec::new()),
    }
}

fn run_backend(
    url: &str,
    state_path: &Path,
    markdown_path: &Path,
    metadata_path: &Path,
    timeout: Duration,
) -> Result<BackendResult, AgetError> {
    let args = vec![
        "--url".to_string(),
        url.to_string(),
        "--state".to_string(),
        state_path.to_string_lossy().into_owned(),
        "--output".to_string(),
        markdown_path.to_string_lossy().into_owned(),
        "--metadata".to_string(),
        metadata_path.to_string_lossy().into_owned(),
    ];

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
    parse_backend_stdout(&stdout)
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
    error: &AgetError,
    started: Instant,
) -> Result<(), AgetError> {
    let (code, message) = match error {
        AgetError::Stable { code, message } => (code, message),
    };
    let metadata = serde_json::json!({
        "ok": false,
        "url": url,
        "format": "markdown",
        "extractor": EXTRACTOR,
        "artifacts": {
            "markdown": markdown_path.to_string_lossy(),
            "metadata": path.to_string_lossy(),
        },
        "sessions": sessions,
        "sensitive": sensitive,
        "warnings": [],
        "timing_ms": {"total": started.elapsed().as_millis()},
        "limits": {"max_chars": null, "max_tokens": null, "truncated": false},
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
