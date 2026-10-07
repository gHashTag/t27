use std::process::Command;
use std::process::ExitCode;

#[test]
fn test_help_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_tri"))
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    // --help should exit with code 0
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_unknown_flag_exits_one() {
    let output = Command::new(env!("CARGO_BIN_EXE_tri"))
        .arg("mutate")
        .arg("spec")
        .arg("--no-such-flag")
        .output()
        .expect("Failed to execute command");

    // Unknown flag should exit with code 1 (not 2)
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_unknown_subcommand_exits_one() {
    let output = Command::new(env!("CARGO_BIN_EXE_tri"))
        .arg("mutate")
        .arg("no-such-subcommand")
        .output()
        .expect("Failed to execute command");

    // Unknown subcommand should exit with code 1 (not 2)
    assert_eq!(output.status.code(), Some(1));
}