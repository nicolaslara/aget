use std::process::ExitCode;

use aget::{Cli, Command};

fn main() -> ExitCode {
    let cli =
        Cli::parse_from_aliasing_get(std::env::args_os()).unwrap_or_else(|error| error.exit());

    match cli.command {
        Command::Get(get) => {
            println!("aget get {}", get.url);
            ExitCode::SUCCESS
        }
        Command::Session(_) => {
            eprintln!("session commands are not implemented yet");
            ExitCode::from(2)
        }
    }
}
