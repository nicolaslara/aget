use std::ffi::OsString;
use std::process::ExitCode;

use aget::{
    Aget, Cli, Command, CurrentTabOptions, EnvelopeFormat, ErrorCode, ErrorResponse, GetSuccess,
    InlineContent, TimingMs, ENVELOPE_SCHEMA_VERSION,
};
use clap::error::ErrorKind;
use serde::Serialize;
use serde_json::Value;

mod main_session;

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
    if tokens.contains(&"current-tab") {
        return "current-tab";
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
        Command::CurrentTab(current_tab) => (|| {
            let aget = Aget::from_env()
                .map_err(io_error)?
                .with_timeout_opt(cli.global.timeout);
            let inline_content = current_tab.inline_content;
            let success = aget
                .current_tab(CurrentTabOptions {
                    port: current_tab.cdp_port,
                    allow_private_content: current_tab.allow_private_content,
                    output: current_tab.output,
                    timeout: cli.global.timeout,
                    content_format: current_tab.content_format,
                    selector: current_tab.selector,
                    exclude_selector: current_tab.exclude_selector,
                    wait_for_selector: current_tab.wait_for_selector,
                    max_chars: current_tab.max_chars,
                    backend_options: current_tab.backend_options,
                })
                .map_err(error_response)?;
            if structured_output {
                print_success_envelope(
                    "current-tab",
                    get_envelope_data(&success, inline_content)?,
                    success.warnings,
                    success.timing_ms,
                )?;
            } else if !cli.global.quiet {
                println!("{}", success.content);
            }
            Ok::<(), ErrorResponse>(())
        })()
        .map_err(|error| error.with_command("current-tab")),
        Command::Session(session) => {
            main_session::run_session(session.command, structured_output, cli.global.timeout)
        }
    }
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

fn io_error(error: impl std::fmt::Display) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::IoError, error.to_string())
}

fn error_response(error: aget::AgetError) -> ErrorResponse {
    match error {
        aget::AgetError::Stable { code, message } => ErrorResponse::new(code, message),
    }
}
