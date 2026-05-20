use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AgetError, ErrorCode};
use crate::process::DEFAULT_SUBPROCESS_TIMEOUT;
use crate::session::agent_browser::{
    classify_agent_browser_failure, filter_playwright_state, read_filtered_agent_browser_session,
    run_agent_browser, set_private_file_permissions, AgentBrowserSessionFilter, RawStateFile,
};
use crate::session::{Session, SessionSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromeImportOptions {
    pub profile: String,
    pub name: String,
    pub domains: Vec<String>,
    pub tmp_dir: PathBuf,
}

pub fn import_chrome_session(options: ChromeImportOptions) -> Result<Session, AgetError> {
    fs::create_dir_all(&options.tmp_dir).map_err(io_aget_error)?;
    let raw_state =
        RawStateFile::new(&options.tmp_dir, "agent-browser-raw-state").map_err(io_aget_error)?;
    let temp_session = unique_agent_browser_session_name();
    let mut opened = false;

    let result = (|| {
        let open = run_agent_browser(
            &options.tmp_dir,
            &[
                "--profile",
                &options.profile,
                "--session",
                &temp_session,
                "open",
                "about:blank",
            ],
        )?;
        if !open.status.success() {
            return Err(classify_agent_browser_failure("open", &open));
        }
        opened = true;

        let raw_state_path = raw_state.path().to_string_lossy().into_owned();
        let save = run_agent_browser(
            &options.tmp_dir,
            &["--session", &temp_session, "state", "save", &raw_state_path],
        )?;
        if !save.status.success() {
            return Err(classify_agent_browser_failure("state save", &save));
        }
        if raw_state.path().exists() {
            set_private_file_permissions(raw_state.path()).map_err(io_aget_error)?;
        }

        let session = read_filtered_agent_browser_session(
            raw_state.path(),
            AgentBrowserSessionFilter {
                name: options.name.clone(),
                source: SessionSource::ChromeProfile {
                    profile: options.profile.clone(),
                },
                allowed_domains: options.domains.clone(),
                source_session: temp_session.clone(),
            },
        )?;
        if session.cookies.is_empty() && session.origins.is_empty() {
            return Err(AgetError::Stable {
                code: ErrorCode::RequiresUserAction,
                message: format!(
                    "agent-browser exported no auth state for allowed domains: {}",
                    options.domains.join(", ")
                ),
            });
        }

        Ok(session)
    })();

    let close_result = if opened {
        let close = run_agent_browser(&options.tmp_dir, &["--session", &temp_session, "close"]);
        match close {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(classify_agent_browser_failure("close", &output)),
            Err(error) => Err(error),
        }
    } else {
        Ok(())
    };

    match (result, close_result) {
        (Ok(session), Ok(())) => Ok(session),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

pub(crate) fn import_owned_chrome_session(
    options: ChromeImportOptions,
) -> Result<Session, AgetError> {
    fs::create_dir_all(&options.tmp_dir).map_err(io_aget_error)?;
    let prepared = prepare_owned_chrome_profile(&options.profile, &options.tmp_dir)?;
    let state =
        crate::browser_cdp::export_browser_state(crate::browser_cdp::BrowserStateExportRequest {
            profile_dir: &prepared.user_data_dir,
            profile_directory: prepared.profile_directory.as_deref(),
            use_real_keychain: prepared.use_real_keychain,
            allowed_domains: &options.domains,
            timeout: DEFAULT_SUBPROCESS_TIMEOUT,
        })?;

    let session = filter_playwright_state(
        state,
        AgentBrowserSessionFilter {
            name: options.name.clone(),
            source: SessionSource::ChromeProfile {
                profile: options.profile.clone(),
            },
            allowed_domains: options.domains.clone(),
            source_session: prepared.source_session.clone(),
        },
    )?;
    if session.cookies.is_empty() && session.origins.is_empty() {
        return Err(AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!(
                "owned Chrome import exported no auth state for allowed domains: {}",
                options.domains.join(", ")
            ),
        });
    }

    Ok(session)
}

struct PreparedOwnedChromeProfile {
    user_data_dir: PathBuf,
    profile_directory: Option<String>,
    use_real_keychain: bool,
    source_session: String,
    _copy: Option<CopiedChromeProfile>,
}

struct CopiedChromeProfile {
    path: PathBuf,
}

impl Drop for CopiedChromeProfile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn prepare_owned_chrome_profile(
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

fn explicit_profile_dir(profile: &str) -> Result<PathBuf, AgetError> {
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

fn resolve_chrome_profile(user_data_dir: &Path, input: &str) -> Result<String, AgetError> {
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

fn copy_chrome_profile(
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

fn unique_agent_browser_session_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("aget-import-{}-{nanos}", std::process::id())
}

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_chrome_profile_by_directory_display_and_case_insensitive_directory() {
        let temp = tempfile::tempdir().unwrap();
        write_local_state(
            temp.path(),
            &[("Default", "Person 1"), ("Profile 1", "Work")],
        );

        assert_eq!(
            resolve_chrome_profile(temp.path(), "Default").unwrap(),
            "Default"
        );
        assert_eq!(
            resolve_chrome_profile(temp.path(), "work").unwrap(),
            "Profile 1"
        );
        assert_eq!(
            resolve_chrome_profile(temp.path(), "profile 1").unwrap(),
            "Profile 1"
        );
    }

    #[test]
    fn resolving_chrome_profile_reports_ambiguous_display_name() {
        let temp = tempfile::tempdir().unwrap();
        write_local_state(temp.path(), &[("Default", "Work"), ("Profile 1", "Work")]);

        let error = resolve_chrome_profile(temp.path(), "work").unwrap_err();

        assert_eq!(error.code(), ErrorCode::RequiresUserAction);
        assert!(error.to_string().contains("ambiguous profile name"));
        assert!(error.to_string().contains("Default"));
        assert!(error.to_string().contains("Profile 1"));
    }

    #[test]
    fn resolving_chrome_profile_reports_available_profiles_when_missing() {
        let temp = tempfile::tempdir().unwrap();
        write_local_state(temp.path(), &[("Default", "Person 1")]);

        let error = resolve_chrome_profile(temp.path(), "Missing").unwrap_err();

        assert_eq!(error.code(), ErrorCode::RequiresUserAction);
        assert!(error.to_string().contains("could not find profile"));
        assert!(error.to_string().contains("Default"));
    }

    #[test]
    fn owned_import_requires_existing_profile_path() {
        let missing =
            std::env::temp_dir().join(format!("aget-missing-profile-{}", std::process::id()));
        let error = explicit_profile_dir(&missing.to_string_lossy()).unwrap_err();

        assert_eq!(error.code(), ErrorCode::RequiresUserAction);
    }

    #[test]
    fn owned_import_accepts_existing_profile_path() {
        let temp = tempfile::tempdir().unwrap();
        let profile = explicit_profile_dir(&temp.path().to_string_lossy()).unwrap();

        assert_eq!(profile, temp.path());
    }

    #[test]
    fn copy_chrome_profile_copies_local_state_and_excludes_cache_and_locks() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let tmp = temp.path().join("tmp");
        write_local_state(&source, &[("Default", "Person 1")]);
        fs::create_dir_all(source.join("Default/Local Storage/leveldb")).unwrap();
        fs::write(source.join("Default/Cookies"), "cookies").unwrap();
        fs::write(source.join("Default/SingletonLock"), "lock").unwrap();
        fs::create_dir_all(source.join("Default/Cache")).unwrap();
        fs::write(source.join("Default/Cache/data_0"), "cache").unwrap();
        fs::write(
            source.join("Default/Local Storage/leveldb/CURRENT"),
            "storage",
        )
        .unwrap();

        let copied = copy_chrome_profile(&source, "Default", &tmp).unwrap();
        let copied_path = copied.path.clone();

        assert!(copied_path.join("Local State").is_file());
        assert_eq!(
            fs::read_to_string(copied_path.join("Default/Cookies")).unwrap(),
            "cookies"
        );
        assert!(copied_path
            .join("Default/Local Storage/leveldb/CURRENT")
            .is_file());
        assert!(!copied_path.join("Default/SingletonLock").exists());
        assert!(!copied_path.join("Default/Cache").exists());
        drop(copied);
        assert!(!copied_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn copied_chrome_profile_parent_uses_private_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let tmp = temp.path().join("tmp");
        write_local_state(&source, &[("Default", "Person 1")]);
        fs::create_dir_all(source.join("Default")).unwrap();

        let copied = copy_chrome_profile(&source, "Default", &tmp).unwrap();
        let mode = fs::metadata(&copied.path).unwrap().permissions().mode() & 0o777;

        assert_eq!(mode, 0o700);
    }

    fn write_local_state(user_data_dir: &Path, profiles: &[(&str, &str)]) {
        fs::create_dir_all(user_data_dir).unwrap();
        let info_cache = profiles
            .iter()
            .map(|(directory, name)| {
                (
                    directory.to_string(),
                    serde_json::json!({
                        "name": name,
                    }),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        let local_state = serde_json::json!({
            "profile": {
                "info_cache": info_cache,
            }
        });
        fs::write(
            user_data_dir.join("Local State"),
            serde_json::to_string(&local_state).unwrap(),
        )
        .unwrap();
    }
}
