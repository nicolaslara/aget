mod discovery;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AgetError, ErrorCode};

use discovery::find_chrome_user_data_dir;
pub(super) use discovery::resolve_chrome_profile;

use super::io_aget_error;

pub(super) struct PreparedOwnedChromeProfile {
    pub(super) user_data_dir: PathBuf,
    pub(super) profile_directory: Option<String>,
    pub(super) use_real_keychain: bool,
    pub(super) source_session: String,
    _copy: Option<CopiedChromeProfile>,
}

pub(super) struct CopiedChromeProfile {
    pub(super) path: PathBuf,
}

impl Drop for CopiedChromeProfile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(super) fn prepare_owned_chrome_profile(
    profile: &str,
    tmp_dir: &Path,
) -> Result<PreparedOwnedChromeProfile, AgetError> {
    if looks_like_profile_path(profile) {
        let path = explicit_profile_dir(profile)?;
        return Ok(PreparedOwnedChromeProfile {
            user_data_dir: path,
            profile_directory: None,
            use_real_keychain: false,
            source_session: format!("owned-chrome:{profile}"),
            _copy: None,
        });
    }

    let user_data_dir = find_chrome_user_data_dir().ok_or_else(|| AgetError::Stable {
        code: ErrorCode::RequiresUserAction,
        message: "owned Chrome import could not find a Chrome user-data directory with Local State; use an explicit profile path or import after Chrome has created a profile".to_string(),
    })?;
    let profile_directory = resolve_chrome_profile(&user_data_dir, profile)?;
    let copied = copy_chrome_profile(&user_data_dir, &profile_directory, tmp_dir)?;
    let copied_path = copied.path.clone();
    Ok(PreparedOwnedChromeProfile {
        user_data_dir: copied_path,
        profile_directory: Some(profile_directory),
        use_real_keychain: true,
        source_session: format!("owned-chrome:{profile}"),
        _copy: Some(copied),
    })
}

pub(super) fn explicit_profile_dir(profile: &str) -> Result<PathBuf, AgetError> {
    let path = expand_tilde(profile);
    if !path.is_dir() {
        return Err(AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!(
                "owned Chrome import profile directory does not exist: {}",
                path.display()
            ),
        });
    }
    Ok(path)
}

const PROFILE_COPY_EXCLUDE_DIRS: &[&str] = &[
    "Cache",
    "Code Cache",
    "GPUCache",
    "Service Worker",
    "blob_storage",
    "File System",
    "GCM Store",
    "optimization_guide",
    "ShaderCache",
    "component_crx_cache",
];

const PROFILE_COPY_EXCLUDE_FILES: &[&str] = &[
    "DevToolsActivePort",
    "SingletonCookie",
    "SingletonLock",
    "SingletonSocket",
];

pub(super) fn copy_chrome_profile(
    user_data_dir: &Path,
    profile_directory: &str,
    tmp_dir: &Path,
) -> Result<CopiedChromeProfile, AgetError> {
    let root = tmp_dir.join("owned-chrome-import");
    create_private_dir(&root).map_err(io_aget_error)?;
    let temp_dir = root.join(format!(
        "aget-profile-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    ));
    create_private_dir(&temp_dir).map_err(io_aget_error)?;

    let result = (|| {
        let local_state = user_data_dir.join("Local State");
        if local_state.is_file() {
            fs::copy(&local_state, temp_dir.join("Local State")).map_err(io_aget_error)?;
        }

        let source_profile = user_data_dir.join(profile_directory);
        if !source_profile.is_dir() {
            return Err(AgetError::Stable {
                code: ErrorCode::RequiresUserAction,
                message: format!(
                    "owned Chrome import profile directory does not exist: {}",
                    source_profile.display()
                ),
            });
        }

        copy_dir_recursive(&source_profile, &temp_dir.join(profile_directory))?;
        Ok(())
    })();

    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temp_dir);
        return Err(error);
    }

    Ok(CopiedChromeProfile { path: temp_dir })
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), AgetError> {
    create_private_dir(destination).map_err(io_aget_error)?;
    for entry in fs::read_dir(source).map_err(io_aget_error)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if PROFILE_COPY_EXCLUDE_FILES.contains(&name.as_ref()) {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(name.as_ref());
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            if PROFILE_COPY_EXCLUDE_DIRS.contains(&name.as_ref()) {
                continue;
            }
            copy_dir_recursive(&source_path, &destination_path)?;
        } else {
            let _ = fs::copy(&source_path, &destination_path);
        }
    }
    Ok(())
}

fn looks_like_profile_path(profile: &str) -> bool {
    let profile = profile.trim();
    profile.starts_with('/')
        || profile.starts_with('~')
        || profile.contains(std::path::MAIN_SEPARATOR)
        || profile.contains('\\')
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

fn create_private_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    set_private_dir_permissions(path)
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(())
}
