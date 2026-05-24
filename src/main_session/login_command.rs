use std::time::Instant;

use aget::{Aget, ErrorResponse, LoginSessionCommand, LoginSessionSubcommand};

use super::elapsed_timing;

const OAUTH_LOGIN_WARNING: &str = "OAuth providers may reject automation-controlled login browsers. If this site uses OAuth and you already have a provider session, pass it with --session <provider>; otherwise prefer signing in with your real browser and importing a scoped session, for example: aget session import browser --browser chrome --browser-profile <profile> --name <name> --allow-domain <domain>.";

pub(super) fn run_login(
    aget: &Aget,
    login: LoginSessionCommand,
    json: bool,
    command_name: &str,
    started: Instant,
) -> Result<(), ErrorResponse> {
    match login.command {
        LoginSessionSubcommand::Start(start) => {
            let requested_sessions = start.session;
            let result = aget
                .start_login_session(start.name, start.profile, start.url, requested_sessions)
                .map_err(super::super::error_response)?;
            let mut warnings = vec![OAUTH_LOGIN_WARNING.to_string()];
            if !result.pending.injected_sessions.is_empty() {
                warnings.push(format!(
                    "Injected {} existing local session(s) into the login browser profile. Credentials remain user-controlled; login finish saves only the target session.",
                    result.pending.injected_sessions.len()
                ));
            }
            if json {
                super::super::print_success_envelope(
                    command_name,
                    serde_json::json!({
                        "state": "login_started",
                        "name": result.pending.name,
                        "profile": result.pending.profile,
                        "agent_session": result.pending.agent_session,
                        "url": result.pending.url,
                        "allowed_domains": result.pending.allowed_domains,
                        "injected_sessions": result.pending.injected_sessions,
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
                if !result.pending.injected_sessions.is_empty() {
                    println!(
                        "Injected existing local sessions: {}. Finish saves only the target session unless you later compose sessions explicitly.",
                        result.pending.injected_sessions.join(", ")
                    );
                }
            }
            Ok(())
        }
        LoginSessionSubcommand::Finish(finish) => {
            let session = aget
                .finish_login_session(finish.name)
                .map_err(super::super::error_response)?;
            if json {
                super::super::print_success_envelope(
                    command_name,
                    serde_json::json!({
                        "state": "login_finished",
                        "name": session.name,
                        "source": "browser_login",
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
                .map_err(super::super::error_response)?;
            if json {
                super::super::print_success_envelope(
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
    }
}
