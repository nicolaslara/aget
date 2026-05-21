use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AgetError, ErrorCode};
use crate::session::agent_browser::{
    classify_agent_browser_failure, read_filtered_agent_browser_session, run_agent_browser,
    set_private_file_permissions, AgentBrowserSessionFilter, RawStateFile,
};
use crate::session::{Session, SessionSource};

use super::{io_aget_error, ChromeImportOptions};

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

fn unique_agent_browser_session_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("aget-import-{}-{nanos}", std::process::id())
}
