use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AgetError, ErrorCode};

pub(super) fn find_chrome_user_data_dir() -> Option<PathBuf> {
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
struct ChromeProfile {
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

pub(in crate::session::chrome) fn resolve_chrome_profile(
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
