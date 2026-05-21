use std::path::PathBuf;

use aget::{BrowserChoice, ErrorCode, ErrorResponse};

pub(super) fn resolve_chrome_authorize_profile(
    browser: BrowserChoice,
    browser_profile: Option<String>,
    chrome_profile: Option<String>,
) -> Result<String, ErrorResponse> {
    match browser {
        BrowserChoice::Chrome => match (browser_profile, chrome_profile) {
            (Some(browser_profile), Some(chrome_profile)) if browser_profile != chrome_profile => {
                Err(ErrorResponse::new(
                    ErrorCode::UsageError,
                    "--browser-profile and --chrome-profile must match when both are supplied",
                ))
            }
            (Some(profile), _) | (_, Some(profile)) => Ok(profile),
            (None, None) => Err(ErrorResponse::new(
                ErrorCode::UsageError,
                "session authorize requires --browser-profile <profile> or --chrome-profile <profile>",
            )),
        },
        unsupported => Err(ErrorResponse::new(
            ErrorCode::UsageError,
            format!(
                "session authorize does not support '{}' yet; use --browser chrome for the verified local import path",
                unsupported.as_str()
            ),
        )),
    }
}

pub(super) fn resolve_browser_import_profile(
    browser: BrowserChoice,
    browser_profile: Option<String>,
    profile_path: Option<PathBuf>,
) -> Result<String, ErrorResponse> {
    let profile_path = profile_path.map(|path| path.to_string_lossy().into_owned());
    match (browser_profile, profile_path) {
        (Some(browser_profile), Some(profile_path)) if browser_profile != profile_path => {
            Err(ErrorResponse::new(
                ErrorCode::UsageError,
                "--browser-profile and --profile-path must match when both are supplied",
            ))
        }
        (Some(profile), _) | (_, Some(profile)) => Ok(profile),
        (None, None) => Err(ErrorResponse::new(
            ErrorCode::UsageError,
            format!(
                "session import browser --browser {} requires --browser-profile <profile> or --profile-path <path>",
                browser.as_str()
            ),
        )),
    }
}
