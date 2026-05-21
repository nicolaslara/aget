use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AgetError, ErrorCode};

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

fn find_chrome_user_data_dir() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("AGET_CHROME_USER_DATA_DIR") {
        let path = PathBuf::from(path);
        if path.join("Local State").is_file() {
            return Some(path);
        }
    }

    chrome_user_data_dirs()
        .into_iter()
        .find(|dir| dir.join("Local State").is_file())
}

fn chrome_user_data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let base = PathBuf::from(home).join("Library/Application Support");
            dirs.extend([
                base.join("Google/Chrome"),
                base.join("Google/Chrome Canary"),
                base.join("Chromium"),
                base.join("BraveSoftware/Brave-Browser"),
            ]);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let base = PathBuf::from(home).join(".config");
            dirs.extend([
                base.join("google-chrome"),
                base.join("google-chrome-unstable"),
                base.join("chromium"),
                base.join("BraveSoftware/Brave-Browser"),
            ]);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let base = PathBuf::from(local);
            dirs.extend([
                base.join(r"Google\Chrome\User Data"),
                base.join(r"Google\Chrome SxS\User Data"),
                base.join(r"Chromium\User Data"),
                base.join(r"BraveSoftware\Brave-Browser\User Data"),
            ]);
        }
    }

    dirs
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ChromeProfile {
    directory: String,
    name: String,
}

fn list_chrome_profiles(user_data_dir: &Path) -> Vec<ChromeProfile> {
    let content = match fs::read_to_string(user_data_dir.join("Local State")) {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(_) => return Vec::new(),
    };
    let Some(info_cache) = json
        .get("profile")
        .and_then(|profile| profile.get("info_cache"))
        .and_then(serde_json::Value::as_object)
    else {
        return Vec::new();
    };

    let mut profiles = info_cache
        .iter()
        .map(|(directory, info)| ChromeProfile {
            directory: directory.clone(),
            name: info
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(directory)
                .to_string(),
        })
        .collect::<Vec<_>>();
    profiles.sort_by(|left, right| left.directory.cmp(&right.directory));
    profiles
}

pub(super) fn resolve_chrome_profile(
    user_data_dir: &Path,
    input: &str,
) -> Result<String, AgetError> {
    let profiles = list_chrome_profiles(user_data_dir);
    if profiles.is_empty() {
        return Err(AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!(
                "owned Chrome import found no Chrome profiles in {}",
                user_data_dir.display()
            ),
        });
    }

    if let Some(profile) = profiles.iter().find(|profile| profile.directory == input) {
        return Ok(profile.directory.clone());
    }

    let input_lower = input.to_ascii_lowercase();
    let display_matches = profiles
        .iter()
        .filter(|profile| profile.name.to_ascii_lowercase() == input_lower)
        .collect::<Vec<_>>();
    if display_matches.len() == 1 {
        return Ok(display_matches[0].directory.clone());
    }
    if display_matches.len() > 1 {
        return Err(AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!(
                "owned Chrome import found ambiguous profile name '{}'; available profiles:\n{}",
                input,
                format_profile_list(&display_matches)
            ),
        });
    }

    if let Some(profile) = profiles
        .iter()
        .find(|profile| profile.directory.to_ascii_lowercase() == input_lower)
    {
        return Ok(profile.directory.clone());
    }

    let profile_refs = profiles.iter().collect::<Vec<_>>();
    Err(AgetError::Stable {
        code: ErrorCode::RequiresUserAction,
        message: format!(
            "owned Chrome import could not find profile '{}'; available profiles:\n{}",
            input,
            format_profile_list(&profile_refs)
        ),
    })
}

fn format_profile_list(profiles: &[&ChromeProfile]) -> String {
    profiles
        .iter()
        .map(|profile| format!("  {} ({})", profile.directory, profile.name))
        .collect::<Vec<_>>()
        .join("\n")
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
