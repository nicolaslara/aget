use std::time::Instant;

use aget::{Aget, AuthorizeSessionOptions, ErrorResponse, SessionSubcommand, TimingMs};

mod command_name;
mod envelope;
mod import_command;
mod inspect;
mod login_command;
mod profile;

use command_name::session_command_name;
use envelope::authorize_envelope_data;
use import_command::run_import;
use inspect::{session_view, source_suffix};
use login_command::run_login;
use profile::resolve_chrome_authorize_profile;

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
            SessionSubcommand::Import(import) => run_import(&aget, import, json, command_name, started),
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
            SessionSubcommand::Login(login) => run_login(&aget, login, json, command_name, started),
        }
    })()
    .map_err(|error: ErrorResponse| error.with_command(command_name))
}

pub(super) fn elapsed_timing(started: Instant) -> TimingMs {
    TimingMs {
        total: started.elapsed().as_millis(),
    }
}
