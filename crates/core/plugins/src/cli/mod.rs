// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! CLI plugin integration
//!
//! This module provides integration between plugins and CLI commands.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Plugin, Result};

/// CLI command type for plugin integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub parameters: HashMap<String, String>,
}

/// CLI command metadata
#[derive(Clone, Debug)]
pub struct CliCommandMetadata {
    /// Command name
    pub name: String,

    /// Command description
    pub description: String,

    /// Command usage
    pub usage: String,

    /// Command examples
    pub examples: Vec<String>,

    /// Required permissions
    pub permissions: Vec<String>,
}

/// CLI plugin trait
#[async_trait]
pub trait CliPlugin: Plugin {
    /// Get available commands
    fn get_commands(&self) -> Vec<CliCommandMetadata>;

    /// Execute a command
    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String>;

    /// Check if the plugin supports a command
    fn supports_command(&self, command: &str) -> bool {
        self.get_commands().iter().any(|cmd| cmd.name == command)
    }

    /// Get command help
    fn get_command_help(&self, command: &str) -> Option<String>;

    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
}
