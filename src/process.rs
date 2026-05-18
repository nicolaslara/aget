use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{AgetError, ErrorCode};

pub(crate) const DEFAULT_SUBPROCESS_TIMEOUT: Duration = Duration::from_secs(60);
const TERMINATION_WAIT: Duration = Duration::from_secs(1);
const WAIT_POLL: Duration = Duration::from_millis(10);

pub(crate) fn configure_local_command(command: &mut Command) {
    command.env_clear();
    for key in [
        "PATH",
        "HOME",
        "USER",
        "TMPDIR",
        "TEMP",
        "TMP",
        "SYSTEMROOT",
        "WINDIR",
        // Test-only hooks used by integration fake backends. Real aget configuration
        // such as AGET_HOME and backend command overrides stays in the parent.
        "AGET_FAKE_AGENT_BROWSER_LOG",
        "AGENT_BROWSER_LOG",
    ] {
        if let Some(value) = env::var_os(key) {
            command.env(key, value);
        }
    }
    configure_process_group(command);
}

pub(crate) fn wait_for_child(
    child: &mut Child,
    timeout: Duration,
    timeout_message: impl FnOnce() -> String,
) -> Result<ExitStatus, AgetError> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() >= deadline => {
                terminate_child(child);
                let _ = wait_after_terminate(child);
                return Err(AgetError::Stable {
                    code: ErrorCode::Timeout,
                    message: timeout_message(),
                });
            }
            Ok(None) => thread::sleep(WAIT_POLL),
            Err(error) => {
                return Err(AgetError::Stable {
                    code: ErrorCode::IoError,
                    message: error.to_string(),
                });
            }
        }
    }
}

pub(crate) struct TempOutputFile {
    path: PathBuf,
}

impl TempOutputFile {
    pub(crate) fn new(dir: &Path, prefix: &str) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = dir.join(format!("{prefix}-{}-{nanos}.txt", std::process::id()));
        drop(create_private_file(&path)?);
        Ok(Self { path })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn read_to_string(&self) -> io::Result<String> {
        let mut text = String::new();
        File::open(&self.path)?.read_to_string(&mut text)?;
        Ok(text)
    }
}

impl Drop for TempOutputFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub(crate) fn create_private_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    set_private_file_mode(&mut options);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
}

fn wait_after_terminate(child: &mut Child) -> Option<ExitStatus> {
    let deadline = Instant::now() + TERMINATION_WAIT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) if Instant::now() >= deadline => return None,
            Ok(None) => thread::sleep(WAIT_POLL),
            Err(_) => return None,
        }
    }
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

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

#[cfg(unix)]
fn terminate_child(child: &mut Child) {
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
fn terminate_child(child: &mut Child) {
    let _ = child.kill();
}
