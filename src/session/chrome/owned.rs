use std::fs;

use crate::error::{AgetError, ErrorCode};
use crate::process::DEFAULT_SUBPROCESS_TIMEOUT;
use crate::session::browser_state::{filter_playwright_state, BrowserSessionFilter};
use crate::session::{Session, SessionSource};

use super::profile::prepare_owned_chrome_profile;
use super::{io_aget_error, ChromeImportOptions};

pub fn import_owned_chrome_session(options: ChromeImportOptions) -> Result<Session, AgetError> {
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
        BrowserSessionFilter {
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
