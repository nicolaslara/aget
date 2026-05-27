use std::process::ExitCode;

use aget::{Aget, Cli, Command, CurrentTabOptions, EnvelopeFormat, ErrorCode, ErrorResponse};
use clap::error::ErrorKind;

mod main_args;
mod main_artifacts;
mod main_batch;
mod main_crawl;
mod main_doctor;
mod main_envelope;
mod main_extract;
mod main_interact;
mod main_map;
mod main_search_page;
mod main_session;
mod main_setup_skills;

use main_args::{args_request_json_envelope, command_name_from_args};
use main_envelope::{
    envelope_data, error_response, get_envelope_data, io_error, print_success_envelope,
};

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
        Ok(exit_code) => exit_code,
        Err(response) => {
            eprintln!(
                "{}",
                serde_json::to_string(&response).unwrap_or_else(|_| "error".to_string())
            );
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, ErrorResponse> {
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
            request = request
                .cache_policy(get.cache.policy())
                .cache_ttl(get.cache.cache_ttl);
            if get.debug.screenshot {
                request = request.capture_screenshot();
            }
            if get.debug.trace {
                request = request.capture_trace();
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
        .map(|()| ExitCode::SUCCESS)
        .map_err(|error| error.with_command("get")),
        Command::Batch(batch) => main_batch::run_batch(batch, structured_output, cli.global.quiet)
            .map_err(|error| error.with_command("batch")),
        Command::Map(map) => main_map::run_map(map, structured_output, cli.global.quiet)
            .map_err(|error| error.with_command("map")),
        Command::Crawl(crawl) => main_crawl::run_crawl(crawl, structured_output, cli.global.quiet)
            .map_err(|error| error.with_command("crawl")),
        Command::SearchPage(search_page) => {
            main_search_page::run_search_page(search_page, structured_output, cli.global.quiet)
                .map_err(|error| error.with_command("search-page"))
        }
        Command::Extract(extract) => {
            main_extract::run_extract(extract, structured_output, cli.global.quiet)
                .map_err(|error| error.with_command("extract"))
        }
        Command::Interact(interact) => main_interact::run_interact(
            interact,
            structured_output,
            cli.global.quiet,
            cli.global.timeout,
            std::time::Instant::now(),
        )
        .map_err(|error| error.with_command("interact")),
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
                    debug: current_tab.debug.into(),
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
        .map(|()| ExitCode::SUCCESS)
        .map_err(|error| error.with_command("current-tab")),
        Command::Session(session) => {
            main_session::run_session(session.command, structured_output, cli.global.timeout)
                .map(|()| ExitCode::SUCCESS)
        }
        Command::Artifacts(artifacts) => {
            main_artifacts::run_artifacts(artifacts.command, structured_output, cli.global.quiet)
                .map(|()| ExitCode::SUCCESS)
        }
        Command::Doctor(doctor) => main_doctor::run_doctor(
            doctor,
            structured_output,
            cli.global.quiet,
            std::time::Instant::now(),
        )
        .map_err(|error| error.with_command("doctor")),
        Command::SetupSkills(setup_skills) => main_setup_skills::run_setup_skills(
            setup_skills,
            structured_output,
            cli.global.quiet,
            std::time::Instant::now(),
        )
        .map(|()| ExitCode::SUCCESS)
        .map_err(|error| error.with_command("setup-skills")),
    }
}
