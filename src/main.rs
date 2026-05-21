use std::ffi::OsString;
use std::process::ExitCode;
use std::time::Instant;

use aget::{
    Aget, AuthorizeSessionOptions, AuthorizeSessionResult, BrowserChoice, Cli, Command,
    EnvelopeFormat, ErrorCode, ErrorResponse, GetSuccess, ImportSessionSource, InlineContent,
    LoginSessionSubcommand, Session, SessionCookie, SessionSubcommand, TimingMs,
    ENVELOPE_SCHEMA_VERSION,
};
use clap::error::ErrorKind;
use serde::Serialize;
use serde_json::Value;

const OAUTH_LOGIN_WARNING: &str = "OAuth providers may reject automation-controlled login browsers. If this site uses OAuth, prefer signing in with your real browser and importing a scoped session, for example: aget session import browser --browser chrome --browser-profile <profile> --name <name> --allow-domain <domain>.";

fn main() -> ExitCode {
    let args = std::env::args_os().collect::<Vec<_>>();
    let structured_output = args_request_json_envelope(&args);
    let cli = match Cli::parse_from_aliasing_get(args.clone()) {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            error.exit()
        }
        Err(error) if structured_output => {
            let command = command_name_from_args(&args);
            let response =
                ErrorResponse::new(ErrorCode::UsageError, error.to_string()).with_command(command);
            eprintln!(
                "{}",
                serde_json::to_string(&response).unwrap_or_else(|_| "error".to_string())
            );
            return ExitCode::from(error.exit_code() as u8);
        }
        Err(error) => error.exit(),
    };

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

fn command_name_from_args(args: &[OsString]) -> &'static str {
    let tokens = args
        .iter()
        .skip(1)
        .filter_map(|arg| arg.to_str())
        .collect::<Vec<_>>();

    if let Some(index) = tokens.iter().position(|token| *token == "session") {
        return match tokens.get(index + 1).copied() {
            Some("list") => "session.list",
            Some("authorize") => "session.authorize",
            Some("inspect") => "session.inspect",
            Some("delete") => "session.delete",
            Some("compose") => "session.compose",
            Some("import") => match tokens.get(index + 2).copied() {
                Some("cmux") => "session.import.cmux",
                Some("browser") => "session.import.browser",
                Some("chrome") => "session.import.chrome",
                _ => "session.import",
            },
            Some("login") => match tokens.get(index + 2).copied() {
                Some("start") => "session.login.start",
                Some("finish") => "session.login.finish",
                Some("cancel") => "session.login.cancel",
                _ => "session.login",
            },
            _ => "session",
        };
    }

    if tokens.contains(&"get")
        || tokens
            .iter()
            .any(|token| token.starts_with("http://") || token.starts_with("https://"))
    {
        return "get";
    }

    "cli"
}

fn args_request_json_envelope(args: &[OsString]) -> bool {
    let mut iter = args.iter().filter_map(|arg| arg.to_str());
    while let Some(arg) = iter.next() {
        if arg == "--envelope" {
            return matches!(iter.next(), Some("json"));
        }
        if arg == "--envelope=json" {
            return true;
        }
    }
    false
}

fn run(cli: Cli) -> Result<(), ErrorResponse> {
    let structured_output = matches!(cli.global.envelope, EnvelopeFormat::Json);
    match cli.command {
        Command::Get(get) => (|| {
            let aget = Aget::from_env()
                .map_err(io_error)?
                .with_timeout_opt(cli.global.timeout);
            let inline_content = get.inline_content;
            let mut request = aget.get(get.url).content_format(get.content_format);
            for session in get.session {
                request = request.session(session);
            }
            if let Some(output) = get.output {
                request = request.output(output);
            }
            if let Some(selector) = get.selector {
                request = request.selector(selector);
            }
            if let Some(exclude_selector) = get.exclude_selector {
                request = request.exclude_selector(exclude_selector);
            }
            if let Some(wait_for_selector) = get.wait_for_selector {
                request = request.wait_for_selector(wait_for_selector);
            }
            if let Some(max_chars) = get.max_chars {
                request = request.max_chars(max_chars);
            }
            for backend_option in get.backend_options {
                request = request.backend_option(backend_option.key, backend_option.value);
            }
            let success = request.run().map_err(error_response)?;
            if structured_output {
                print_success_envelope(
                    "get",
                    get_envelope_data(&success, inline_content)?,
                    success.warnings,
                    success.timing_ms,
                )?;
            } else if !cli.global.quiet {
                println!("{}", success.content);
            }
            Ok::<(), ErrorResponse>(())
        })()
        .map_err(|error| error.with_command("get")),
        Command::Session(session) => {
            run_session(session.command, structured_output, cli.global.timeout)
        }
    }
}

fn run_session(
    command: SessionSubcommand,
    json: bool,
    timeout: Option<std::time::Duration>,
) -> Result<(), ErrorResponse> {
    let command_name = session_command_name(&command);
    let started = Instant::now();

    (|| {
        let aget = Aget::from_env().map_err(io_error)?.with_timeout_opt(timeout);

        match command {
        SessionSubcommand::List => {
            let names = aget.list_sessions().map_err(error_response)?;
            if json {
                print_success_envelope(
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
                .map_err(error_response)?;
            if json {
                print_success_envelope(
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
                .map_err(error_response)?;
            let view = session_view(&session, inspect.show_secrets);
            if json {
                print_success_envelope(
                    command_name,
                    envelope_data(&view, &["ok"])?,
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
                .map_err(error_response)?;
            if json {
                print_success_envelope(
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
                    .map_err(error_response)?;
                if json {
                    print_success_envelope(
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
                        .map_err(error_response)?,
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
                    print_success_envelope(
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
                    .map_err(error_response)?;
                if json {
                    print_success_envelope(
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
                .map_err(error_response)?;
            if json {
                print_success_envelope(
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
                    .map_err(error_response)?;
                let warnings = vec![OAUTH_LOGIN_WARNING.to_string()];
                if json {
                    print_success_envelope(
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
                    .map_err(error_response)?;
                if json {
                    print_success_envelope(
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
                    .map_err(error_response)?;
                if json {
                    print_success_envelope(
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

fn resolve_chrome_authorize_profile(
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

fn resolve_browser_import_profile(
    browser: BrowserChoice,
    browser_profile: Option<String>,
    profile_path: Option<std::path::PathBuf>,
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

fn authorize_envelope_data(result: &AuthorizeSessionResult) -> Result<Value, ErrorResponse> {
    Ok(serde_json::json!({
        "state": result.state,
        "name": result.name,
        "source": result.source,
        "allowed_domains": result.allowed_domains,
        "baseline": get_envelope_data(&result.baseline, InlineContent::Never)?,
        "verification": get_envelope_data(&result.verification, InlineContent::Never)?,
        "verification_sensitive": result.verification.sensitive,
        "verification_content_inlined": false,
        "predicates": result.predicates,
        "next_command": if result.state == aget::AuthorizationState::VerificationFailed {
            Some(serde_json::json!([
                "aget",
                "session",
                "authorize",
                result.name,
                "--url",
                result.verification.url,
                "--browser-profile",
                "<profile>",
                "--allow-domain",
                "<domain>",
            ]))
        } else {
            None
        },
    }))
}

fn print_success_envelope(
    command: &str,
    data: Value,
    warnings: Vec<String>,
    timing_ms: TimingMs,
) -> Result<(), ErrorResponse> {
    let envelope = serde_json::json!({
        "ok": true,
        "schema_version": ENVELOPE_SCHEMA_VERSION,
        "command": command,
        "data": data,
        "warnings": warnings,
        "timing_ms": timing_ms,
    });
    println!("{}", serde_json::to_string(&envelope).map_err(io_error)?);
    Ok(())
}

fn get_envelope_data(
    success: &GetSuccess,
    inline_content: InlineContent,
) -> Result<Value, ErrorResponse> {
    let mut data = envelope_data(success, &["ok", "warnings", "timing_ms"])?;
    if let Value::Object(object) = &mut data {
        let include_content = match inline_content {
            InlineContent::Always => true,
            InlineContent::Never => false,
            InlineContent::Auto => !success.sensitive,
        };
        if !include_content {
            object.remove("content");
        }
    }
    Ok(data)
}

fn envelope_data<T: Serialize>(value: &T, remove_keys: &[&str]) -> Result<Value, ErrorResponse> {
    let mut data = serde_json::to_value(value).map_err(io_error)?;
    if let Value::Object(object) = &mut data {
        for key in remove_keys {
            object.remove(*key);
        }
    }
    Ok(data)
}

fn elapsed_timing(started: Instant) -> TimingMs {
    TimingMs {
        total: started.elapsed().as_millis(),
    }
}

fn session_command_name(command: &SessionSubcommand) -> &'static str {
    match command {
        SessionSubcommand::List => "session.list",
        SessionSubcommand::Authorize(_) => "session.authorize",
        SessionSubcommand::Inspect(_) => "session.inspect",
        SessionSubcommand::Delete(_) => "session.delete",
        SessionSubcommand::Import(import) => match &import.source {
            ImportSessionSource::Cmux(_) => "session.import.cmux",
            ImportSessionSource::Browser(_) => "session.import.browser",
            ImportSessionSource::Chrome(_) => "session.import.chrome",
        },
        SessionSubcommand::Compose(_) => "session.compose",
        SessionSubcommand::Login(login) => match &login.command {
            LoginSessionSubcommand::Start(_) => "session.login.start",
            LoginSessionSubcommand::Finish(_) => "session.login.finish",
            LoginSessionSubcommand::Cancel(_) => "session.login.cancel",
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
    session_storage: Vec<StorageEntryView<'a>>,
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
        session_storage: origin
            .session_storage
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
