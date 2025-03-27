use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn test_cli_help_command() {
    // Create a new command that runs the CLI binary with the help argument
    let mut cmd = Command::cargo_bin("squirrel").unwrap();
    
    // Run help command and verify output
    cmd.arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available commands"));
}

#[test]
fn test_cli_version_command() {
    // Create a new command that runs the CLI binary with the version argument
    let mut cmd = Command::cargo_bin("squirrel").unwrap();
    
    // Run version command and verify output
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("Squirrel CLI"));
}

#[test]
fn test_cli_status_command() {
    // Create a new command that runs the CLI binary with the status argument
    let mut cmd = Command::cargo_bin("squirrel").unwrap();
    
    // Run status command and verify output
    cmd.arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("System Status"));
}

#[test]
fn test_cli_invalid_command() {
    // Create a new command that runs the CLI binary with an invalid argument
    let mut cmd = Command::cargo_bin("squirrel").unwrap();
    
    // Run invalid command and verify error output
    cmd.arg("invalid_command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_cli_config_list_command() {
    // Create a new command that runs the CLI binary with the config list command
    let mut cmd = Command::cargo_bin("squirrel").unwrap();
    
    // Run config list command and verify output
    cmd.arg("config")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration settings"));
}

#[test]
fn test_cli_verbose_output() {
    // Create a new command that runs the CLI binary with verbose flag
    let mut cmd = Command::cargo_bin("squirrel").unwrap();
    
    // Run a command with verbose flag and verify detailed output
    cmd.arg("--verbose")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Detailed information"));
}

#[test]
fn test_cli_json_output() {
    // Create a new command that runs the CLI binary with JSON output format
    let mut cmd = Command::cargo_bin("squirrel").unwrap();
    
    // Run a command with JSON output format and verify JSON structure
    cmd.arg("status")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("}"));
}

#[test]
fn test_cli_help_for_specific_command() {
    // Create a new command that runs the CLI binary with help for specific command
    let mut cmd = Command::cargo_bin("squirrel").unwrap();
    
    // Run help for specific command and verify output
    cmd.arg("help")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("Usage"));
} 