use std::time::Instant;

use aget::{
    Aget, AuthorizeSessionOptions, BrowserChoice, ErrorCode, ErrorResponse, ImportSessionSource,
    LoginSessionSubcommand, SessionSubcommand, TimingMs,
};

mod command_name;
mod envelope;
mod inspect;
mod profile;

use command_name::session_command_name;
use envelope::authorize_envelope_data;
use inspect::{session_view, source_suffix};
use profile::{resolve_browser_import_profile, resolve_chrome_authorize_profile};

const OAUTH_LOGIN_WARNING: &str = "OAuth providers may reject automation-controlled login browsers. If this site uses OAuth, prefer signing in with your real browser and importing a scoped session, for example: aget session import browser --browser chrome --browser-profile <profile> --name <name> --allow-domain <domain>.";

pub(super) fn run_session(
    command: SessionSubcommand,
    json: bool,
    timeout: Option<std::time::Duration>,
) -> Result<(), ErrorResponse> {
    let command_name = session_command_name(&command);
    let started = Instant::now();

    (|| {
        let aget = Aget::from_env()
            .map_err(super::io_error)?
            .with_timeout_opt(timeout);

        match command {
            SessionSubcommand::List => {
                let names = aget.list_sessions().map_err(super::error_response)?;
                if json {
                    super::print_success_envelope(
                        command_name,
                        serde_json::json!({ "sessions": names }),
                        Vec::<String>::new(),
                        elapsed_timing(started),
                    )?;
                } else if names.is_empty() {
                    println!("No sessions found");
                } else {
                    for name in names {
                        println!("{name}");
                    }
                }
                Ok(())
            }
            SessionSubcommand::Authorize(authorize) => {
                let chrome_profile = resolve_chrome_authorize_profile(
                    authorize.browser,
                    authorize.browser_profile,
                    authorize.chrome_profile,
                )?;
                let result = aget
                    .authorize_chrome_session(AuthorizeSessionOptions {
                        name: authorize.name,
                        url: authorize.url,
                        chrome_profile,
                        allow_domains: authorize.allow_domain,
                        must_contain: authorize.must_contain,
                        must_not_contain: authorize.must_not_contain,
                        output: authorize.output,
                    })
                    .map_err(super::error_response)?;
                if json {
                    super::print_success_envelope(
                        command_name,
                        authorize_envelope_data(&result)?,
                        result.warnings,
                        elapsed_timing(started),
                    )?;
                } else {
                    match result.state {
                        aget::AuthorizationState::Verified => {
                            println!("Authorized session {}; verification passed", result.name);
                        }
                        aget::AuthorizationState::VerificationFailed => {
                            println!(
                                "Imported session {}, but verification predicates failed",
                                result.name
                            );
                            println!(
                                "Complete login in the selected browser/profile, then rerun session authorize to re-import and verify"
                            );
                        }
                    }
                }
                Ok(())
            }
            SessionSubcommand::Inspect(inspect) => {
                let session = aget
                    .load_session(&inspect.name)
                    .map_err(super::error_response)?;
                let view = session_view(&session, inspect.show_secrets);
                if json {
                    super::print_success_envelope(
                        command_name,
                        super::envelope_data(&view, &["ok"])?,
                        Vec::<String>::new(),
                        elapsed_timing(started),
                    )?;
                } else {
                    println!("Session: {}", session.name);
                    println!("Sensitive: {}", session.sensitive);
                    println!(
                        "Cookie domains: {}",
                        session.allowed_cookie_domains.join(", ")
                    );
                    println!(
                        "Storage origins: {}",
                        session.allowed_storage_origins.join(", ")
                    );
                    println!("Cookies: {}", session.cookies.len());
                    for cookie in view.cookies {
                        println!(
                            "- {} {}={}{}",
                            cookie.domain,
                            cookie.name,
                            cookie.value,
                            source_suffix(cookie.source_session)
                        );
                    }
                    println!("Origins: {}", session.origins.len());
                    for origin in view.origins {
                        println!(
                            "- {}{}",
                            origin.origin,
                            source_suffix(origin.source_session)
                        );
                        for entry in origin.local_storage {
                            println!("  - localStorage {}={}", entry.name, entry.value);
                        }
                        for entry in origin.session_storage {
                            println!("  - sessionStorage {}={}", entry.name, entry.value);
                        }
                    }
                }
                Ok(())
            }
            SessionSubcommand::Delete(delete) => {
                let deleted = aget
                    .delete_session(&delete.name)
                    .map_err(super::error_response)?;
                if json {
                    super::print_success_envelope(
                        command_name,
                        serde_json::json!({ "deleted": deleted }),
                        Vec::<String>::new(),
                        elapsed_timing(started),
                    )?;
                } else if deleted {
                    println!("Deleted session {}", delete.name);
                } else {
                    println!("Session {} did not exist", delete.name);
                }
                Ok(())
            }
            SessionSubcommand::Import(import) => match import.source {
                ImportSessionSource::Cmux(cmux) => {
                    let session = aget
                        .import_cmux_session(cmux.surface, cmux.name, cmux.allow_domain)
                        .map_err(super::error_response)?;
                    if json {
                        super::print_success_envelope(
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
                            .map_err(super::error_response)?,
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
                        super::print_success_envelope(
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
                        .import_chrome_session(
                            chrome.chrome_profile,
                            chrome.name,
                            chrome.allow_domain,
                        )
                        .map_err(super::error_response)?;
                    if json {
                        super::print_success_envelope(
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
            },
            SessionSubcommand::Compose(compose) => {
                let source_count = compose.session.len();
                let source_sessions = compose.session.clone();
                let session = aget
                    .compose_sessions(compose.name, compose.session)
                    .map_err(super::error_response)?;
                if json {
                    super::print_success_envelope(
                        command_name,
                        serde_json::json!({
                            "name": session.name,
                            "source_sessions": source_sessions,
                            "cookie_count": session.cookies.len(),
                            "origin_count": session.origins.len(),
                        }),
                        Vec::<String>::new(),
                        elapsed_timing(started),
                    )?;
                } else {
                    println!(
                        "Composed session {} from {} source sessions with {} cookies and {} origins",
                        session.name,
                        source_count,
                        session.cookies.len(),
                        session.origins.len()
                    );
                }
                Ok(())
            }
            SessionSubcommand::Login(login) => match login.command {
                LoginSessionSubcommand::Start(start) => {
                    let result = aget
                        .start_login_session(start.name, start.profile, start.url)
                        .map_err(super::error_response)?;
                    let warnings = vec![OAUTH_LOGIN_WARNING.to_string()];
                    if json {
                        super::print_success_envelope(
                            command_name,
                            serde_json::json!({
                                "state": "login_started",
                                "name": result.pending.name,
                                "profile": result.pending.profile,
                                "agent_session": result.pending.agent_session,
                                "url": result.pending.url,
                                "allowed_domains": result.pending.allowed_domains,
                                "next_command": [
                                    "aget",
                                    "session",
                                    "login",
                                    "finish",
                                    result.pending.name,
                                ],
                            }),
                            warnings,
                            elapsed_timing(started),
                        )?;
                    } else {
                        println!("Warning: {OAUTH_LOGIN_WARNING}");
                        println!(
                            "Opened login bucket {} in profile {}. After completing login, run: aget session login finish {}",
                            result.pending.name,
                            result.pending.profile,
                            result.pending.name,
                        );
                    }
                    Ok(())
                }
                LoginSessionSubcommand::Finish(finish) => {
                    let session = aget
                        .finish_login_session(finish.name)
                        .map_err(super::error_response)?;
                    if json {
                        super::print_success_envelope(
                            command_name,
                            serde_json::json!({
                                "state": "login_finished",
                                "name": session.name,
                                "source": "agent_browser",
                                "cookie_count": session.cookies.len(),
                                "origin_count": session.origins.len(),
                            }),
                            Vec::<String>::new(),
                            elapsed_timing(started),
                        )?;
                    } else {
                        println!(
                            "Saved session {} with {} cookies and {} origins",
                            session.name,
                            session.cookies.len(),
                            session.origins.len(),
                        );
                    }
                    Ok(())
                }
                LoginSessionSubcommand::Cancel(cancel) => {
                    let result = aget
                        .cancel_login_session(cancel.name)
                        .map_err(super::error_response)?;
                    if json {
                        super::print_success_envelope(
                            command_name,
                            serde_json::json!({
                                "state": "login_cancelled",
                                "name": result.pending.name,
                                "agent_session": result.pending.agent_session,
                            }),
                            Vec::<String>::new(),
                            elapsed_timing(started),
                        )?;
                    } else {
                        println!("Cancelled login flow {}", result.pending.name);
                    }
                    Ok(())
                }
            },
        }
    })()
    .map_err(|error: ErrorResponse| error.with_command(command_name))
}

fn elapsed_timing(started: Instant) -> TimingMs {
    TimingMs {
        total: started.elapsed().as_millis(),
    }
}
