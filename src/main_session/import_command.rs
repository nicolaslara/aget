use std::time::Instant;

use aget::{
    Aget, BrowserChoice, ErrorCode, ErrorResponse, ImportSessionCommand, ImportSessionSource,
};

use super::elapsed_timing;
use super::profile::resolve_browser_import_profile;

pub(super) fn run_import(
    aget: &Aget,
    import: ImportSessionCommand,
    json: bool,
    command_name: &str,
    started: Instant,
) -> Result<(), ErrorResponse> {
    match import.source {
        ImportSessionSource::Cmux(cmux) => {
            let session = aget
                .import_cmux_session(cmux.surface, cmux.name, cmux.allow_domain)
                .map_err(super::super::error_response)?;
            if json {
                super::super::print_success_envelope(
                    command_name,
                    serde_json::json!({
                        "source": "cmux",
                        "name": session.name,
                        "cookie_count": session.cookies.len(),
                    }),
                    Vec::<String>::new(),
                    elapsed_timing(started),
                )?;
            } else {
                println!(
                    "Imported cmux session {} with {} cookies",
                    session.name,
                    session.cookies.len()
                );
            }
            Ok(())
        }
        ImportSessionSource::Browser(browser) => {
            let profile = resolve_browser_import_profile(
                browser.browser,
                browser.browser_profile,
                browser.profile_path,
            )?;
            let session = match browser.browser {
                BrowserChoice::Chrome => aget
                    .import_chrome_session(profile, browser.name, browser.allow_domain)
                    .map_err(super::super::error_response)?,
                unsupported => {
                    return Err(ErrorResponse::new(
                        ErrorCode::UsageError,
                        format!(
                            "session import browser does not support '{}' yet; use --browser chrome for the verified local import path, or use session login start as an explicit fallback for controlled non-OAuth flows",
                            unsupported.as_str()
                        ),
                    ));
                }
            };
            if json {
                super::super::print_success_envelope(
                    command_name,
                    serde_json::json!({
                        "source": "browser_profile",
                        "browser": browser.browser.as_str(),
                        "name": session.name,
                        "cookie_count": session.cookies.len(),
                        "origin_count": session.origins.len(),
                    }),
                    Vec::<String>::new(),
                    elapsed_timing(started),
                )?;
            } else {
                println!(
                    "Imported {} browser session {} with {} cookies and {} origins",
                    browser.browser.as_str(),
                    session.name,
                    session.cookies.len(),
                    session.origins.len()
                );
            }
            Ok(())
        }
        ImportSessionSource::Chrome(chrome) => {
            let session = aget
                .import_chrome_session(chrome.chrome_profile, chrome.name, chrome.allow_domain)
                .map_err(super::super::error_response)?;
            if json {
                super::super::print_success_envelope(
                    command_name,
                    serde_json::json!({
                        "source": "chrome",
                        "name": session.name,
                        "cookie_count": session.cookies.len(),
                        "origin_count": session.origins.len(),
                    }),
                    Vec::<String>::new(),
                    elapsed_timing(started),
                )?;
            } else {
                println!(
                    "Imported chrome session {} with {} cookies and {} origins",
                    session.name,
                    session.cookies.len(),
                    session.origins.len()
                );
            }
            Ok(())
        }
    }
}
