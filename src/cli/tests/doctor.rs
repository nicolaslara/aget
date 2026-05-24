use clap::Parser;

use super::super::*;

#[test]
fn parses_doctor_defaults() {
    let cli = Cli::try_parse_from(["aget", "doctor"]).unwrap();

    assert_eq!(
        cli.command,
        Command::Doctor(DoctorCommand {
            quick: false,
            checks: Vec::new(),
        })
    );
}

#[test]
fn parses_doctor_check_filters() {
    let cli = Cli::try_parse_from([
        "aget", "doctor", "--quick", "--check", "store", "--check", "cmux",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Doctor(DoctorCommand {
            quick: true,
            checks: vec![DoctorCheck::Store, DoctorCheck::Cmux],
        })
    );
}
