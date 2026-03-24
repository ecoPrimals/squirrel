// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Type Definitions for Task Server
//!
//! This module contains trait definitions, type aliases, and core types
//! used throughout the task server implementation.

use clap;
use tokio::sync::mpsc;

/// Alternative type for task updates that doesn't depend on protobuf
pub type TaskUpdateSender = mpsc::Sender<String>;

/// Simplified Command trait for command execution
pub trait SimpleCommand: Send + Sync + std::fmt::Debug {
    /// Get the command name
    fn name(&self) -> &str;

    /// Get the command description
    fn description(&self) -> &str;

    /// Execute the command with given arguments
    fn execute(&self, args: &[String]) -> Result<String, String>;

    /// Get help text for the command
    fn help(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }

    /// Get the command parser
    fn parser(&self) -> clap::Command;

    /// Clone the command into a boxed trait object
    fn clone_box(&self) -> Box<dyn SimpleCommand>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestSimpleCommand;

    impl SimpleCommand for TestSimpleCommand {
        fn name(&self) -> &'static str {
            "test_cmd"
        }

        fn description(&self) -> &'static str {
            "test description"
        }

        fn execute(&self, args: &[String]) -> Result<String, String> {
            Ok(format!("args_len={}", args.len()))
        }

        fn parser(&self) -> clap::Command {
            clap::Command::new("test_cmd").about("test description")
        }

        fn clone_box(&self) -> Box<dyn SimpleCommand> {
            Box::new(self.clone())
        }
    }

    #[tokio::test]
    async fn task_update_sender_round_trip_message() {
        let (tx, mut rx): (TaskUpdateSender, _) = mpsc::channel(4);
        tx.send("ping".to_string()).await.expect("should succeed");
        assert_eq!(rx.recv().await, Some("ping".to_string()));
    }

    #[test]
    fn simple_command_help_default_matches_name_and_description() {
        let cmd = TestSimpleCommand;
        assert_eq!(cmd.help(), "test_cmd: test description");
        let dbg = format!("{cmd:?}");
        assert!(dbg.contains("TestSimpleCommand"));
    }

    #[test]
    fn simple_command_execute_and_clone_box() {
        let boxed: Box<dyn SimpleCommand> = TestSimpleCommand.clone_box();
        assert_eq!(boxed.name(), "test_cmd");
        assert_eq!(boxed.execute(&[]).expect("should succeed"), "args_len=0");
        let boxed2 = boxed.clone_box();
        assert_eq!(
            boxed2
                .execute(&[String::from("a")])
                .expect("should succeed"),
            "args_len=1"
        );
    }

    #[test]
    fn simple_command_parser_has_expected_name() {
        let cmd = TestSimpleCommand;
        assert_eq!(cmd.parser().get_name(), "test_cmd");
    }
}
