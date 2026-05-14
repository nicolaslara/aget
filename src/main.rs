use std::process::ExitCode;

use aget::session::SessionStore;
use aget::{
    cancel_login_session, complete_login_session, compose_session, finish_login_session, get_url,
    import_chrome_session, import_cmux_session, merge_login_session, start_login_session,
    ChromeImportOptions, Cli, CmuxImportOptions, Command, ErrorCode, ErrorResponse, GetOptions,
    ImportSessionSource, LoginCancelOptions, LoginCompleteOptions, LoginFinishOptions,
    LoginSessionSubcommand, LoginStartOptions, Session, SessionCookie, SessionSubcommand,
};

fn main() -> ExitCode {
    let cli =
        Cli::parse_from_aliasing_get(std::env::args_os()).unwrap_or_else(|error| error.exit());

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(response) => {
            eprintln!(
                "{}",
                serde_json::to_string(&response).unwrap_or_else(|_| "error".to_string())
            );
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), ErrorResponse> {
    let structured_output = cli.global.json || cli.global.envelope;
    match cli.command {
        Command::Get(get) => {
            let success = get_url(GetOptions {
                url: get.url,
                sessions: get.session,
                out: get.out,
                timeout: cli.global.timeout,
                format: get.format,
                selector: get.selector,
                exclude_selector: get.exclude_selector,
                wait_for: get.wait_for,
                max_chars: get.max_chars,
                extractor_options: get.extractor_options,
            })
            .map_err(error_response)?;
            if structured_output {
                println!("{}", serde_json::to_string(&success).map_err(io_error)?);
            } else if !cli.global.quiet {
                println!("{}", success.content);
            }
            Ok(())
        }
        Command::Session(session) => run_session(session.command, structured_output),
    }
}

fn run_session(command: SessionSubcommand, json: bool) -> Result<(), ErrorResponse> {
    let store = SessionStore::from_env().map_err(io_error)?;

    match command {
        SessionSubcommand::List => {
            let names = store.list().map_err(io_error)?;
            if json {
                println!("{}", serde_json::json!({ "ok": true, "sessions": names }));
            } else if names.is_empty() {
                println!("No sessions found");
            } else {
                for name in names {
                    println!("{name}");
                }
            }
            Ok(())
        }
        SessionSubcommand::Inspect(inspect) => {
            let session = store.load(&inspect.name).map_err(io_error)?;
            let view = session_view(&session, inspect.show_secrets);
            if json {
                println!("{}", serde_json::to_string_pretty(&view).map_err(io_error)?);
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
                }
            }
            Ok(())
        }
        SessionSubcommand::Delete(delete) => {
            let deleted = store.delete(&delete.name).map_err(io_error)?;
            if json {
                println!("{}", serde_json::json!({ "ok": true, "deleted": deleted }));
            } else if deleted {
                println!("Deleted session {}", delete.name);
            } else {
                println!("Session {} did not exist", delete.name);
            }
            Ok(())
        }
        SessionSubcommand::Import(import) => match import.source {
            ImportSessionSource::Cmux(cmux) => {
                let session = import_cmux_session(CmuxImportOptions {
                    surface: cmux.surface,
                    name: cmux.name,
                    domains: cmux.domain,
                })
                .map_err(error_response)?;
                store.save(&session).map_err(io_error)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "source": "cmux",
                            "name": session.name,
                            "cookie_count": session.cookies.len(),
                        })
                    );
                } else {
                    println!(
                        "Imported cmux session {} with {} cookies",
                        session.name,
                        session.cookies.len()
                    );
                }
                Ok(())
            }
            ImportSessionSource::Chrome(chrome) => {
                let session = import_chrome_session(ChromeImportOptions {
                    profile: chrome.profile,
                    name: chrome.name,
                    domains: chrome.domain,
                    tmp_dir: store.home().join("tmp"),
                })
                .map_err(error_response)?;
                store.save(&session).map_err(io_error)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "source": "chrome",
                            "name": session.name,
                            "cookie_count": session.cookies.len(),
                            "origin_count": session.origins.len(),
                        })
                    );
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
            validate_compose_target(&store, &compose.name, &compose.session)?;
            let source_sessions = compose
                .session
                .iter()
                .map(|name| store.load(name).map_err(io_error))
                .collect::<Result<Vec<_>, _>>()?;
            let session =
                compose_session(&compose.name, &source_sessions).map_err(error_response)?;
            store.save(&session).map_err(io_error)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "name": session.name,
                        "source_sessions": compose.session,
                        "cookie_count": session.cookies.len(),
                        "origin_count": session.origins.len(),
                    })
                );
            } else {
                println!(
                    "Composed session {} from {} source sessions with {} cookies and {} origins",
                    session.name,
                    compose.session.len(),
                    session.cookies.len(),
                    session.origins.len()
                );
            }
            Ok(())
        }
        SessionSubcommand::Login(login) => match login.command {
            LoginSessionSubcommand::Start(start) => {
                let result = start_login_session(LoginStartOptions {
                    name: start.name,
                    profile: start.profile,
                    url: start.url,
                    tmp_dir: store.home().join("tmp"),
                })
                .map_err(error_response)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
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
                        })
                    );
                } else {
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
                let result = finish_login_session(LoginFinishOptions {
                    name: finish.name,
                    tmp_dir: store.home().join("tmp"),
                })
                .map_err(error_response)?;
                let session = match store.load(&result.session.name) {
                    Ok(existing) => merge_login_session(existing, result.session),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => result.session,
                    Err(error) => return Err(io_error(error)),
                };
                store.save(&session).map_err(io_error)?;
                complete_login_session(LoginCompleteOptions {
                    pending: result.pending,
                    tmp_dir: store.home().join("tmp"),
                })
                .map_err(error_response)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "state": "login_finished",
                            "name": session.name,
                            "source": "agent_browser",
                            "cookie_count": session.cookies.len(),
                            "origin_count": session.origins.len(),
                        })
                    );
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
                let result = cancel_login_session(LoginCancelOptions {
                    name: cancel.name,
                    tmp_dir: store.home().join("tmp"),
                })
                .map_err(error_response)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "state": "login_cancelled",
                            "name": result.pending.name,
                            "agent_session": result.pending.agent_session,
                        })
                    );
                } else {
                    println!("Cancelled login flow {}", result.pending.name);
                }
                Ok(())
            }
        },
    }
}

#[derive(serde::Serialize)]
struct SessionView<'a> {
    ok: bool,
    version: u32,
    name: &'a str,
    sensitive: bool,
    allowed_cookie_domains: &'a [String],
    allowed_storage_origins: &'a [String],
    cookies: Vec<CookieView<'a>>,
    origins: Vec<OriginView<'a>>,
}

#[derive(serde::Serialize)]
struct CookieView<'a> {
    name: &'a str,
    value: String,
    domain: &'a str,
    path: &'a str,
    secure: bool,
    http_only: bool,
    source_session: &'a Option<String>,
}

#[derive(serde::Serialize)]
struct OriginView<'a> {
    origin: &'a str,
    local_storage: Vec<StorageEntryView<'a>>,
    source_session: &'a Option<String>,
}

#[derive(serde::Serialize)]
struct StorageEntryView<'a> {
    name: &'a str,
    value: String,
}

fn session_view(session: &Session, show_secrets: bool) -> SessionView<'_> {
    SessionView {
        ok: true,
        version: session.version,
        name: &session.name,
        sensitive: session.sensitive,
        allowed_cookie_domains: &session.allowed_cookie_domains,
        allowed_storage_origins: &session.allowed_storage_origins,
        cookies: session
            .cookies
            .iter()
            .map(|cookie| cookie_view(cookie, show_secrets))
            .collect(),
        origins: session
            .origins
            .iter()
            .map(|origin| origin_view(origin, show_secrets))
            .collect(),
    }
}

fn validate_compose_target(
    store: &SessionStore,
    target: &str,
    sources: &[String],
) -> Result<(), ErrorResponse> {
    if sources.iter().any(|source| source == target) {
        return Err(ErrorResponse::new(
            ErrorCode::UsageError,
            format!("compose target '{target}' must not match a source session"),
        ));
    }
    if store.exists(target).map_err(io_error)? {
        return Err(ErrorResponse::new(
            ErrorCode::UsageError,
            format!("session '{target}' already exists"),
        ));
    }
    Ok(())
}

fn cookie_view(cookie: &SessionCookie, show_secrets: bool) -> CookieView<'_> {
    CookieView {
        name: &cookie.name,
        value: if show_secrets {
            cookie.value.clone()
        } else {
            "<redacted>".to_string()
        },
        domain: &cookie.domain,
        path: &cookie.path,
        secure: cookie.secure,
        http_only: cookie.http_only,
        source_session: &cookie.source_session,
    }
}

fn origin_view(origin: &aget::SessionOrigin, show_secrets: bool) -> OriginView<'_> {
    OriginView {
        origin: &origin.origin,
        local_storage: origin
            .local_storage
            .iter()
            .map(|entry| storage_entry_view(entry, show_secrets))
            .collect(),
        source_session: &origin.source_session,
    }
}

fn storage_entry_view(entry: &aget::StorageEntry, show_secrets: bool) -> StorageEntryView<'_> {
    StorageEntryView {
        name: &entry.name,
        value: if show_secrets {
            entry.value.clone()
        } else {
            "<redacted>".to_string()
        },
    }
}

fn source_suffix(source_session: &Option<String>) -> String {
    match source_session {
        Some(source) => format!(" source={source}"),
        None => String::new(),
    }
}

fn io_error(error: impl std::fmt::Display) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::IoError, error.to_string())
}

fn error_response(error: aget::AgetError) -> ErrorResponse {
    match error {
        aget::AgetError::Stable { code, message } => ErrorResponse::new(code, message),
    }
}
