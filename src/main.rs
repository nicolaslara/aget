use std::process::ExitCode;

use aget::session::SessionStore;
use aget::{
    get_url, import_cmux_session, Cli, CmuxImportOptions, Command, ErrorCode, ErrorResponse,
    GetOptions, ImportSessionSource, Session, SessionCookie, SessionSubcommand,
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
    match cli.command {
        Command::Get(get) => {
            let success = get_url(GetOptions {
                url: get.url,
                session: get.session,
                out: get.out,
                timeout: cli.global.timeout,
                format: get.format,
                selector: get.selector,
                exclude_selector: get.exclude_selector,
                only_main: get.only_main,
                wait_for: get.wait_for,
                max_chars: get.max_chars,
                max_tokens: get.max_tokens,
                extractor_options: get.extractor_options,
            })
            .map_err(error_response)?;
            if cli.global.json {
                println!("{}", serde_json::to_string(&success).map_err(io_error)?);
            } else if !cli.global.quiet {
                println!("{}", success.content);
            }
            Ok(())
        }
        Command::Session(session) => run_session(session.command, cli.global.json),
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
                    println!("- {} {}={}", cookie.domain, cookie.name, cookie.value);
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
    origins: &'a [aget::SessionOrigin],
}

#[derive(serde::Serialize)]
struct CookieView<'a> {
    name: &'a str,
    value: String,
    domain: &'a str,
    path: &'a str,
    secure: bool,
    http_only: bool,
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
        origins: &session.origins,
    }
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
