// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Type Definitions for Task Server
//!
//! This module contains trait definitions, type aliases, and core types
//! used throughout the task server implementation.

use clap;
use tokio::sync::mpsc;

// use crate::generated::mcp_task::WatchTaskResponse;

/// Alternative type for task updates that doesn't depend on protobuf
pub type TaskUpdateSender = mpsc::Sender<String>;

/// Stream type for watch task responses
// pub type WatchTaskStream = Pin<Box<dyn Stream<Item = std::result::Result<WatchTaskResponse, Status>> + Send + 'static>>;

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
