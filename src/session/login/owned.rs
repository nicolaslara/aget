use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AgetError, ErrorCode};
use crate::process::DEFAULT_SUBPROCESS_TIMEOUT;
use crate::session::agent_browser::{filter_playwright_state, AgentBrowserSessionFilter};
use crate::session::SessionSource;

use super::io_aget_error;
use super::pending::{
    allowed_domains_from_url, default_owned_login_profile_path, prepare_login_profile_path,
    read_pending_login, remove_pending_login, remove_tool_owned_login_profile,
    rewrite_pending_login, validate_login_name, write_pending_login,
};
use super::types::{
    LoginCancelOptions, LoginCancelResult, LoginFinishOptions, LoginFinishResult,
    LoginStartOptions, LoginStartResult, PendingLogin,
};

pub(crate) fn start_owned_login_session(
    options: LoginStartOptions,
) -> Result<LoginStartResult, AgetError> {
    validate_login_name(&options.name)?;
    let allowed_domains = allowed_domains_from_url(&options.url)?;
    let profile = options
        .profile
        .map(PathBuf::from)
        .unwrap_or_else(|| default_owned_login_profile_path(&options.tmp_dir, &options.name));
    prepare_login_profile_path(&profile)?;
    let mut pending = PendingLogin {
        agent_session: format!("aget-login-{}", options.name),
        name: options.name,
        profile: profile.to_string_lossy().into_owned(),
        url: options.url,
        allowed_domains,
        browser_pid: None,
    };

    fs::create_dir_all(&options.tmp_dir).map_err(io_aget_error)?;
    write_pending_login(&options.tmp_dir, &pending)?;
    let started =
        crate::browser_cdp::start_login_browser(crate::browser_cdp::BrowserLoginStartRequest {
            profile_dir: &profile,
            url: &pending.url,
            timeout: DEFAULT_SUBPROCESS_TIMEOUT,
        });
    let started = match started {
        Ok(started) => started,
        Err(error) => {
            let _ = remove_pending_login(&options.tmp_dir, &pending.name);
            let _ = remove_tool_owned_login_profile(&options.tmp_dir, &pending);
            return Err(error);
        }
    };
    pending.browser_pid = Some(started.pid);
    if let Err(error) = rewrite_pending_login(&options.tmp_dir, &pending) {
        let _ = remove_pending_login(&options.tmp_dir, &pending.name);
        let _ = remove_tool_owned_login_profile(&options.tmp_dir, &pending);
        return Err(error);
    }
    Ok(LoginStartResult { pending })
}

pub(crate) fn finish_owned_login_session(
    options: LoginFinishOptions,
) -> Result<LoginFinishResult, AgetError> {
    validate_login_name(&options.name)?;
    let pending = read_pending_login(&options.tmp_dir, &options.name)?;
    let state = crate::browser_cdp::export_login_browser_state(
        crate::browser_cdp::BrowserLoginStateExportRequest {
            profile_dir: Path::new(&pending.profile),
            allowed_domains: &pending.allowed_domains,
            pid: pending.browser_pid,
            timeout: DEFAULT_SUBPROCESS_TIMEOUT,
        },
    )?;
    let session = filter_playwright_state(
        state,
        AgentBrowserSessionFilter {
            name: pending.name.clone(),
            source: SessionSource::AgentBrowser {
                session: pending.agent_session.clone(),
            },
            allowed_domains: pending.allowed_domains.clone(),
            source_session: pending.agent_session.clone(),
        },
    )?;
    if session.cookies.is_empty() && session.origins.is_empty() {
        return Err(AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!(
                "owned login flow exported no auth state for allowed domains: {}",
                pending.allowed_domains.join(", ")
            ),
        });
    }
    Ok(LoginFinishResult { session, pending })
}

pub(crate) fn cancel_owned_login_session(
    options: LoginCancelOptions,
) -> Result<LoginCancelResult, AgetError> {
    validate_login_name(&options.name)?;
    let pending = read_pending_login(&options.tmp_dir, &options.name)?;
    let close =
        crate::browser_cdp::close_login_browser(crate::browser_cdp::BrowserLoginCloseRequest {
            profile_dir: Path::new(&pending.profile),
            pid: pending.browser_pid,
            timeout: DEFAULT_SUBPROCESS_TIMEOUT,
        });
    let cleanup_result = (|| {
        remove_tool_owned_login_profile(&options.tmp_dir, &pending)?;
        remove_pending_login(&options.tmp_dir, &pending.name)
    })();

    if let Err(error) = close {
        cleanup_result.map_err(io_aget_error)?;
        return Err(error);
    }
    cleanup_result.map_err(io_aget_error)?;
    Ok(LoginCancelResult { pending })
}
