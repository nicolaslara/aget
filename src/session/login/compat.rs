use std::fs;
use std::path::PathBuf;

use crate::error::{AgetError, ErrorCode};
use crate::session::agent_browser::{
    classify_agent_browser_failure, read_filtered_agent_browser_session, run_agent_browser,
    set_private_file_permissions, AgentBrowserSessionFilter, RawStateFile,
};
use crate::session::SessionSource;

use super::io_aget_error;
use super::pending::{
    allowed_domains_from_url, default_login_profile_path, prepare_login_profile_path,
    read_pending_login, remove_pending_login, remove_tool_owned_login_profile, validate_login_name,
    write_pending_login,
};
use super::types::{
    LoginCancelOptions, LoginCancelResult, LoginFinishOptions, LoginFinishResult,
    LoginStartOptions, LoginStartResult, PendingLogin,
};

pub fn start_login_session(options: LoginStartOptions) -> Result<LoginStartResult, AgetError> {
    if !options.injected_sessions.is_empty() {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: "session login start --session is supported only by the owned browser backend; unset AGET_AGENT_BROWSER_COMMAND to use provider-session injection".to_string(),
        });
    }
    validate_login_name(&options.name)?;
    let allowed_domains = allowed_domains_from_url(&options.url)?;
    let profile = options
        .profile
        .map(PathBuf::from)
        .unwrap_or_else(|| default_login_profile_path(&options.tmp_dir, &options.name));
    prepare_login_profile_path(&profile)?;
    let pending = PendingLogin {
        agent_session: format!("aget-login-{}", options.name),
        injected_sessions: Vec::new(),
        name: options.name,
        profile: profile.to_string_lossy().into_owned(),
        url: options.url,
        allowed_domains,
        browser_pid: None,
    };

    fs::create_dir_all(&options.tmp_dir).map_err(io_aget_error)?;
    write_pending_login(&options.tmp_dir, &pending)?;
    let open = run_agent_browser(
        &options.tmp_dir,
        &[
            "--profile",
            &pending.profile,
            "--session",
            &pending.agent_session,
            "open",
            &pending.url,
        ],
    )?;
    if !open.status.success() {
        let _ = remove_pending_login(&options.tmp_dir, &pending.name);
        return Err(classify_agent_browser_failure("open", &open));
    }
    Ok(LoginStartResult { pending })
}

pub fn finish_login_session(options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError> {
    validate_login_name(&options.name)?;
    let pending = read_pending_login(&options.tmp_dir, &options.name)?;
    let raw_state =
        RawStateFile::new(&options.tmp_dir, "login-raw-state").map_err(io_aget_error)?;
    let raw_state_path = raw_state.path().to_string_lossy().into_owned();
    let save = run_agent_browser(
        &options.tmp_dir,
        &[
            "--session",
            &pending.agent_session,
            "state",
            "save",
            &raw_state_path,
        ],
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
                "login flow exported no auth state for allowed domains: {}",
                pending.allowed_domains.join(", ")
            ),
        });
    }
    let close = run_agent_browser(
        &options.tmp_dir,
        &["--session", &pending.agent_session, "close"],
    )?;
    if !close.status.success() {
        return Err(classify_agent_browser_failure("close", &close));
    }
    Ok(LoginFinishResult { session, pending })
}

pub fn cancel_login_session(options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
    validate_login_name(&options.name)?;
    let pending = read_pending_login(&options.tmp_dir, &options.name)?;
    let close = run_agent_browser(
        &options.tmp_dir,
        &["--session", &pending.agent_session, "close"],
    );
    let cleanup_result = (|| {
        remove_tool_owned_login_profile(&options.tmp_dir, &pending)?;
        remove_pending_login(&options.tmp_dir, &pending.name)
    })();
    match close {
        Ok(output) if output.status.success() => cleanup_result.map_err(io_aget_error)?,
        Ok(output) => {
            let cleanup_error = cleanup_result.err();
            let mut error = classify_agent_browser_failure("close", &output);
            if let (AgetError::Stable { message, .. }, Some(cleanup_error)) =
                (&mut error, cleanup_error)
            {
                message.push_str(&format!("; cleanup also failed: {cleanup_error}"));
            }
            return Err(error);
        }
        Err(error) => {
            cleanup_result.map_err(io_aget_error)?;
            return Err(error);
        }
    }
    Ok(LoginCancelResult { pending })
}
