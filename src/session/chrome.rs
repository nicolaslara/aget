use std::fs;
use std::path::PathBuf;
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
    let profile_dir = explicit_profile_dir(&options.profile)?;
    let state =
        crate::browser_cdp::export_browser_state(crate::browser_cdp::BrowserStateExportRequest {
            profile_dir: &profile_dir,
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
            source_session: format!("owned-chrome:{}", options.profile),
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

fn explicit_profile_dir(profile: &str) -> Result<PathBuf, AgetError> {
    if !looks_like_profile_path(profile) {
        return Err(AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: "owned Chrome import currently supports explicit user-data-dir paths; use the command-backed agent-browser adapter for named Chrome profiles".to_string(),
        });
    }

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
    fn owned_import_rejects_named_chrome_profiles_for_now() {
        let error = explicit_profile_dir("Default").unwrap_err();

        assert_eq!(error.code(), ErrorCode::BackendUnavailable);
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
}
