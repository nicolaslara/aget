use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[path = "aget_mock_backend/args.rs"]
mod args;
#[path = "aget_mock_backend/behavior.rs"]
mod behavior;
#[path = "aget_mock_backend/config/mod.rs"]
mod config;
#[path = "aget_mock_backend/http.rs"]
mod http;
#[path = "aget_mock_backend/output.rs"]
mod output;

fn main() {
    if let Some(marker) = descendant_marker_arg() {
        std::thread::sleep(Duration::from_secs(2));
        let _ = fs::write(marker, "survived");
        return;
    }

    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32, String> {
    let args = args::parse_args(env::args().skip(1).collect())?;
    let config = config::read_config()?;

    config::assert_expected_args(&args, &config)?;
    config::assert_expected_environment(&config)?;
    config::assert_expected_state(&args, &config)?;

    behavior::run_behavior(&args, &config)
}

fn descendant_marker_arg() -> Option<PathBuf> {
    let mut args = env::args().skip(1);
    if args.next().as_deref() == Some("--descendant-marker") {
        args.next().map(PathBuf::from)
    } else {
        None
    }
}
