use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::error::{AgetError, ErrorCode};
use crate::session::agent_browser::origin_host;

use super::io_aget_error;
use super::types::{LoginCompleteOptions, PendingLogin};

pub fn complete_login_session(options: LoginCompleteOptions) -> Result<(), AgetError> {
    validate_login_name(&options.pending.name)?;
    remove_tool_owned_login_profile(&options.tmp_dir, &options.pending).map_err(io_aget_error)?;
    remove_pending_login(&options.tmp_dir, &options.pending.name).map_err(io_aget_error)?;
    Ok(())
}

pub(super) fn default_login_profile_path(tmp_dir: &Path, name: &str) -> PathBuf {
    tmp_dir.join("agent-browser").join(format!("aget-{name}"))
}

pub(super) fn default_owned_login_profile_path(tmp_dir: &Path, name: &str) -> PathBuf {
    tmp_dir.join("owned-login").join(format!("aget-{name}"))
}

pub(super) fn prepare_login_profile_path(profile: &Path) -> Result<(), AgetError> {
    if let Some(parent) = profile.parent() {
        fs::create_dir_all(parent).map_err(io_aget_error)?;
    }
    Ok(())
}

pub(super) fn pending_login_path(tmp_dir: &Path, name: &str) -> PathBuf {
    tmp_dir.join(format!("login-{name}.json"))
}

pub(super) fn validate_login_name(name: &str) -> Result<(), AgetError> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
    {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("invalid login session name '{name}'"),
        });
    }
    Ok(())
}

pub(super) fn allowed_domains_from_url(url: &str) -> Result<Vec<String>, AgetError> {
    let Some((scheme, _)) = url.split_once("://") else {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("invalid login URL '{url}'"),
        });
    };
    if !scheme.eq_ignore_ascii_case("https") {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("login URL must use https, got '{url}'"),
        });
    }
    let Some(host) = origin_host(url) else {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("invalid login URL '{url}'"),
        });
    };

    if let Some(bare_host) = host.strip_prefix("www.") {
        Ok(vec![bare_host.to_string(), host])
    } else {
        Ok(vec![host])
    }
}

pub(super) fn write_pending_login(tmp_dir: &Path, pending: &PendingLogin) -> Result<(), AgetError> {
    let path = pending_login_path(tmp_dir, &pending.name);
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    set_private_file_mode(&mut options);
    let mut file = match options.open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(AgetError::Stable {
                code: ErrorCode::UsageError,
                message: format!("pending login flow named '{}' already exists", pending.name),
            });
        }
        Err(error) => return Err(io_aget_error(error)),
    };
    let write_result = (|| {
        serde_json::to_writer_pretty(&mut file, pending).map_err(|error| AgetError::Stable {
            code: ErrorCode::IoError,
            message: error.to_string(),
        })?;
        file.write_all(b"\n").map_err(io_aget_error)
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&path);
        return Err(error);
    }
    Ok(())
}

pub(super) fn rewrite_pending_login(
    tmp_dir: &Path,
    pending: &PendingLogin,
) -> Result<(), AgetError> {
    let path = pending_login_path(tmp_dir, &pending.name);
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    set_private_file_mode(&mut options);
    let mut file = options.open(&path).map_err(io_aget_error)?;
    serde_json::to_writer_pretty(&mut file, pending).map_err(|error| AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    })?;
    file.write_all(b"\n").map_err(io_aget_error)
}

pub(super) fn read_pending_login(tmp_dir: &Path, name: &str) -> Result<PendingLogin, AgetError> {
    let file =
        File::open(pending_login_path(tmp_dir, name)).map_err(|error| AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!("no pending login flow named '{name}': {error}"),
        })?;
    serde_json::from_reader(file).map_err(|error| AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: format!("pending login metadata is malformed: {error}"),
    })
}

pub(super) fn remove_pending_login(tmp_dir: &Path, name: &str) -> io::Result<()> {
    match fs::remove_file(pending_login_path(tmp_dir, name)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(super) fn remove_tool_owned_login_profile(
    tmp_dir: &Path,
    pending: &PendingLogin,
) -> io::Result<()> {
    let profile = PathBuf::from(&pending.profile);
    if profile != default_login_profile_path(tmp_dir, &pending.name)
        && profile != default_owned_login_profile_path(tmp_dir, &pending.name)
    {
        return Ok(());
    }

    remove_dir_all_with_retries(&profile)
}

fn remove_dir_all_with_retries(path: &Path) -> io::Result<()> {
    let mut last_error = None;
    for _ in 0..5 {
        match fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    Err(last_error.unwrap_or_else(|| io::Error::other("remove_dir_all failed")))
}

#[cfg(unix)]
fn set_private_file_mode(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_private_file_mode(_options: &mut OpenOptions) {}
