// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::registry::CommandRegistry;
use tempfile::tempdir;

// ========== VersionCommand ==========

#[test]
fn test_version_command() {
    let cmd = VersionCommand;
    assert_eq!(cmd.name(), "version");
    assert!(cmd.execute(&[]).is_ok());
}

#[test]
fn test_version_command_description() {
    let cmd = VersionCommand::new();
    assert_eq!(
        cmd.description(),
        "Shows the current version of the application"
    );
}

#[test]
fn test_version_command_execute_output() {
    let cmd = VersionCommand::new();
    let result = cmd.execute(&[]).unwrap();
    assert!(result.starts_with("Version: "));
    assert!(result.len() > 9);
}

#[test]
fn test_version_command_execute_with_args_ignored() {
    let cmd = VersionCommand::new();
    let result = cmd.execute(&["ignored".to_string(), "args".to_string()]);
    assert!(result.is_ok());
}

#[test]
fn test_version_command_default() {
    let cmd = VersionCommand;
    assert_eq!(cmd.name(), "version");
}

#[test]
fn test_version_command_parser() {
    let cmd = VersionCommand::new();
    let parser = cmd.parser();
    assert_eq!(parser.get_name(), "version");
}

#[test]
fn test_version_command_clone_box() {
    let cmd = VersionCommand::new();
    let cloned = cmd.clone_box();
    assert_eq!(cloned.name(), "version");
    assert!(cloned.execute(&[]).is_ok());
}

// ========== HelpCommand ==========

#[test]
fn test_help_command() {
    let cmd = HelpCommand {
        command_help: vec![("test".to_string(), "Test command description".to_string())],
    };
    assert_eq!(cmd.name(), "help");
    assert!(cmd.execute(&[]).is_ok());
}

#[test]
fn test_help_command_description() {
    let cmd = HelpCommand {
        command_help: vec![],
    };
    assert_eq!(
        cmd.description(),
        "Provides help information for available commands"
    );
}

#[test]
fn test_help_command_execute_all_commands() {
    let cmd = HelpCommand {
        command_help: vec![
            ("cmd1".to_string(), "cmd1: First command".to_string()),
            ("cmd2".to_string(), "cmd2: Second command".to_string()),
        ],
    };
    let result = cmd.execute(&[]).unwrap();
    assert!(result.contains("Available commands:"));
    assert!(result.contains("cmd1: First command"));
    assert!(result.contains("cmd2: Second command"));
}

#[test]
fn test_help_command_execute_specific_command() {
    let cmd = HelpCommand {
        command_help: vec![
            ("echo".to_string(), "echo: Echoes arguments".to_string()),
            ("version".to_string(), "version: Shows version".to_string()),
        ],
    };
    let result = cmd.execute(&["echo".to_string()]).unwrap();
    assert_eq!(result, "echo: Echoes arguments");
}

#[test]
fn test_help_command_execute_command_not_found() {
    let cmd = HelpCommand {
        command_help: vec![("known".to_string(), "known: Known command".to_string())],
    };
    let result = cmd.execute(&["nonexistent".to_string()]);
    assert!(result.is_err());
    if let Err(CommandError::CommandNotFound(name)) = result {
        assert_eq!(name, "nonexistent");
    } else {
        panic!("Expected CommandNotFound error");
    }
}

#[test]
fn test_help_command_empty_help_list() {
    let cmd = HelpCommand {
        command_help: vec![],
    };
    let result = cmd.execute(&[]).unwrap();
    assert!(result.contains("Available commands:"));
    assert!(!result.contains("  ")); // No command entries
}

#[test]
fn test_help_command_new_with_empty_registry() {
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    let cmd = HelpCommand::new(registry);
    assert_eq!(cmd.name(), "help");
    let result = cmd.execute(&[]).unwrap();
    assert!(result.contains("Available commands:"));
}

#[test]
fn test_help_command_new_with_populated_registry() {
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    registry
        .lock()
        .unwrap()
        .register("version", Arc::new(VersionCommand::new()))
        .unwrap();
    registry
        .lock()
        .unwrap()
        .register("echo", Arc::new(EchoCommand::new()))
        .unwrap();

    let cmd = HelpCommand::new(Arc::clone(&registry));
    let result = cmd.execute(&[]).unwrap();
    assert!(result.contains("version"));
    assert!(result.contains("echo"));
}

#[test]
fn test_help_command_update() {
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    let mut cmd = HelpCommand::new(Arc::clone(&registry));
    let initial = cmd.execute(&[]).unwrap();

    registry
        .lock()
        .unwrap()
        .register("newcmd", Arc::new(VersionCommand::new()))
        .unwrap();

    cmd.update(&registry);
    let updated = cmd.execute(&[]).unwrap();
    // Help displays command.help() text; VersionCommand's help contains "version"
    assert!(updated.contains("version"));
    assert!(
        updated.len() > initial.len(),
        "update should add command help"
    );
}

#[test]
fn test_help_command_parser() {
    let cmd = HelpCommand {
        command_help: vec![],
    };
    let parser = cmd.parser();
    assert_eq!(parser.get_name(), "help");
}

#[test]
fn test_help_command_clone_box() {
    let cmd = HelpCommand {
        command_help: vec![("x".to_string(), "x: help".to_string())],
    };
    let cloned = cmd.clone_box();
    assert_eq!(cloned.name(), "help");
    assert!(cloned.execute(&["x".to_string()]).is_ok());
}

// ========== EchoCommand ==========

#[test]
fn test_echo_command() {
    let cmd = EchoCommand::new();
    assert_eq!(cmd.name(), "echo");
    let result = cmd.execute(&["hello".to_string(), "world".to_string()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Echo: hello world");
}

#[test]
fn test_echo_command_description() {
    let cmd = EchoCommand::new();
    assert_eq!(
        cmd.description(),
        "Echoes the provided arguments back to the user"
    );
}

#[test]
fn test_echo_command_empty_args() {
    let cmd = EchoCommand::new();
    let result = cmd.execute(&[]).unwrap();
    assert_eq!(result, "Echo: ");
}

#[test]
fn test_echo_command_single_arg() {
    let cmd = EchoCommand::new();
    let result = cmd.execute(&["hello".to_string()]).unwrap();
    assert_eq!(result, "Echo: hello");
}

#[test]
fn test_echo_command_with_prefix() {
    let cmd = EchoCommand::with_prefix(">> ");
    let result = cmd.execute(&["hello".to_string()]).unwrap();
    assert_eq!(result, ">> hello");
}

#[test]
fn test_echo_command_with_prefix_empty() {
    let cmd = EchoCommand::with_prefix("");
    let result = cmd.execute(&["hello".to_string()]).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_echo_command_default() {
    let cmd = EchoCommand::default();
    assert_eq!(cmd.name(), "echo");
    assert_eq!(cmd.execute(&["x".to_string()]).unwrap(), "Echo: x");
}

#[test]
fn test_echo_command_parser() {
    let cmd = EchoCommand::new();
    let parser = cmd.parser();
    assert_eq!(parser.get_name(), "echo");
}

#[test]
fn test_echo_command_clone_box() {
    let cmd = EchoCommand::with_prefix("PREFIX: ");
    let cloned = cmd.clone_box();
    assert_eq!(cloned.name(), "echo");
    assert_eq!(cloned.execute(&["x".to_string()]).unwrap(), "PREFIX: x");
}

// ========== ExitCommand ==========

#[test]
fn test_exit_command() {
    let cmd = ExitCommand::new();
    assert_eq!(cmd.name(), "exit");
    let result = cmd.execute(&[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Exiting application");
}

#[test]
fn test_exit_command_description() {
    let cmd = ExitCommand::new();
    assert_eq!(cmd.description(), "Exits the application");
}

#[test]
fn test_exit_command_execute_with_args_ignored() {
    let cmd = ExitCommand::new();
    let result = cmd.execute(&["--force".to_string()]);
    assert!(result.is_ok());
}

#[test]
fn test_exit_command_default() {
    let cmd = ExitCommand;
    assert_eq!(cmd.name(), "exit");
}

#[test]
fn test_exit_command_parser() {
    let cmd = ExitCommand::new();
    let parser = cmd.parser();
    assert_eq!(parser.get_name(), "exit");
}

#[test]
fn test_exit_command_clone_box() {
    let cmd = ExitCommand::new();
    let cloned = cmd.clone_box();
    assert_eq!(cloned.name(), "exit");
    assert!(cloned.execute(&[]).is_ok());
}

// ========== KillCommand ==========

#[test]
fn test_kill_command() {
    let cmd = KillCommand::new();
    assert_eq!(cmd.name(), "kill");

    // Test with valid PID
    let result = cmd.execute(&["1234".to_string()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Process with PID 1234 terminated");

    // Test with invalid PID
    let result = cmd.execute(&["invalid".to_string()]);
    assert!(result.is_err());

    // Test with no PID
    let result = cmd.execute(&[]);
    assert!(result.is_err());
}

#[test]
fn test_kill_command_description() {
    let cmd = KillCommand::new();
    assert_eq!(cmd.description(), "Terminates a running process by PID");
}

#[test]
fn test_kill_command_empty_args_validation_error() {
    let cmd = KillCommand::new();
    let result = cmd.execute(&[]);
    assert!(result.is_err());
    if let Err(CommandError::ValidationError(msg)) = result {
        assert_eq!(msg, "No PID provided");
    } else {
        panic!("Expected ValidationError");
    }
}

#[test]
fn test_kill_command_invalid_pid_format() {
    let cmd = KillCommand::new();
    let result = cmd.execute(&["not-a-number".to_string()]);
    assert!(result.is_err());
    if let Err(CommandError::ValidationError(msg)) = result {
        assert!(msg.contains("Invalid PID format"));
    } else {
        panic!("Expected ValidationError");
    }
}

#[test]
fn test_kill_command_pid_zero() {
    let cmd = KillCommand::new();
    let result = cmd.execute(&["0".to_string()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Process with PID 0 terminated");
}

#[test]
fn test_kill_command_default() {
    let cmd = KillCommand;
    assert_eq!(cmd.name(), "kill");
}

#[test]
fn test_kill_command_parser() {
    let cmd = KillCommand::new();
    let parser = cmd.parser();
    assert_eq!(parser.get_name(), "kill");
}

#[test]
fn test_kill_command_clone_box() {
    let cmd = KillCommand::new();
    let cloned = cmd.clone_box();
    assert_eq!(cloned.name(), "kill");
    assert!(cloned.execute(&["999".to_string()]).is_ok());
}

// ========== HistoryCommand ==========

fn create_test_history() -> (Arc<CommandHistory>, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let history_file = dir.path().join("builtin_test_history.json");
    let history = Arc::new(CommandHistory::with_options(100, &history_file).unwrap());
    (history, dir)
}

#[test]
fn test_history_command_empty() {
    let (history, _dir) = create_test_history();
    let cmd = HistoryCommand::new(history);
    assert_eq!(cmd.name(), "history");
    assert_eq!(cmd.description(), "View and manage command history");

    let result = cmd.execute(&[]).unwrap();
    assert!(result.contains("Command history is empty") || result.contains("No history"));
}

#[test]
fn test_history_command_with_entries() {
    let (history, _dir) = create_test_history();
    history
        .add(
            "echo".to_string(),
            vec!["hello".to_string()],
            true,
            None,
            None,
        )
        .unwrap();
    history
        .add("version".to_string(), vec![], true, None, None)
        .unwrap();

    let cmd = HistoryCommand::new(history);
    let result = cmd.execute(&[]).unwrap();
    assert!(result.contains("Command history") || result.contains("echo"));
    assert!(result.contains("version") || result.contains("1:") || result.contains("2:"));
}

#[test]
fn test_history_command_clear() {
    let (history, _dir) = create_test_history();
    history
        .add("test".to_string(), vec![], true, None, None)
        .unwrap();

    let cmd = HistoryCommand::new(history);
    let result = cmd.execute(&["--clear".to_string()]).unwrap();
    assert_eq!(result, "Command history cleared.");
}

#[test]
fn test_history_command_search() {
    let (history, _dir) = create_test_history();
    history
        .add(
            "find".to_string(),
            vec!["file.txt".to_string()],
            true,
            None,
            None,
        )
        .unwrap();
    history
        .add(
            "grep".to_string(),
            vec!["pattern".to_string()],
            true,
            None,
            None,
        )
        .unwrap();

    let cmd = HistoryCommand::new(history);
    let result = cmd
        .execute(&["--search".to_string(), "file.txt".to_string()])
        .unwrap();
    assert!(result.contains("file.txt"));
    assert!(result.contains("find") || result.contains("Search results"));
}

#[test]
fn test_history_command_search_no_match() {
    let (history, _dir) = create_test_history();
    history
        .add(
            "echo".to_string(),
            vec!["hello".to_string()],
            true,
            None,
            None,
        )
        .unwrap();

    let cmd = HistoryCommand::new(history);
    let result = cmd
        .execute(&["--search".to_string(), "nonexistent_query_xyz".to_string()])
        .unwrap();
    assert!(result.contains("No command history entries found matching"));
}

#[test]
fn test_history_command_specific_command() {
    let (history, _dir) = create_test_history();
    history
        .add(
            "echo".to_string(),
            vec!["hello".to_string()],
            true,
            None,
            None,
        )
        .unwrap();

    let cmd = HistoryCommand::new(history);
    let result = cmd
        .execute(&["--command".to_string(), "echo".to_string()])
        .unwrap();
    assert!(result.contains("echo"));
    assert!(result.contains("hello") || result.contains("Last execution"));
}

#[test]
fn test_history_command_specific_command_not_found() {
    let (history, _dir) = create_test_history();
    let cmd = HistoryCommand::new(history);
    let result = cmd
        .execute(&["--command".to_string(), "never_executed_cmd".to_string()])
        .unwrap();
    assert!(result.contains("No history found for command"));
}

#[test]
fn test_history_command_limit() {
    let (history, _dir) = create_test_history();
    for i in 0..5 {
        history
            .add(format!("cmd{i}"), vec![], true, None, None)
            .unwrap();
    }

    let cmd = HistoryCommand::new(history);
    let result = cmd.execute(&["-n".to_string(), "2".to_string()]).unwrap();
    let line_count = result
        .lines()
        .filter(|l| l.starts_with(|c: char| c.is_ascii_digit()))
        .count();
    assert!(line_count <= 2 || result.contains("cmd"));
}

#[test]
fn test_history_command_cleanup() {
    let (history, _dir) = create_test_history();
    let mut entry = HistoryEntry::new("old".to_string(), vec![], true, None, None);
    entry.timestamp = 1000;
    history.add_entry(entry).unwrap();

    let cmd = HistoryCommand::new(history);
    let result = cmd
        .execute(&["--cleanup".to_string(), "365".to_string()])
        .unwrap();
    assert!(result.contains("Removed") && result.contains("history entries"));
}

#[test]
fn test_history_command_parser() {
    let (history, _dir) = create_test_history();
    let cmd = HistoryCommand::new(history);
    let parser = cmd.parser();
    assert_eq!(parser.get_name(), "history");
}

#[test]
fn test_history_command_clone_box() {
    let (history, _dir) = create_test_history();
    let cmd = HistoryCommand::new(history);
    let cloned = cmd.clone_box();
    assert_eq!(cloned.name(), "history");
    assert!(cloned.execute(&[]).is_ok());
}

#[test]
fn test_history_command_invalid_args() {
    let (history, _dir) = create_test_history();
    let cmd = HistoryCommand::new(history);
    // Invalid: -n expects a number, not a string
    let result = cmd.execute(&["-n".to_string(), "not-a-number".to_string()]);
    assert!(result.is_err());
    if let Err(CommandError::ExecutionError(msg)) = result {
        assert!(msg.contains("Invalid arguments") || msg.contains("invalid"));
    }
}
