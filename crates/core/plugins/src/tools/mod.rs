// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tool plugin module
//!
//! This module provides functionality for tool plugins.

use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use serde::{Serialize, Deserialize};

use crate::plugin::Plugin;

/// Command metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandMetadata {
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
    
    /// Command arguments
    pub arguments: Vec<CommandArgument>,
    
    /// Command flags
    pub flags: Vec<CommandFlag>,
}

/// Command argument
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandArgument {
    /// Argument name
    pub name: String,
    
    /// Argument description
    pub description: String,
    
    /// Whether the argument is required
    pub required: bool,
    
    /// Argument type
    pub arg_type: String,
    
    /// Default value
    pub default_value: Option<String>,
}

/// Command flag
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandFlag {
    /// Flag name
    pub name: String,
    
    /// Flag description
    pub description: String,
    
    /// Flag alias
    pub alias: Option<String>,
    
    /// Whether the flag takes a value
    pub takes_value: bool,
    
    /// Flag type
    pub flag_type: Option<String>,
    
    /// Default value
    pub default_value: Option<String>,
}

/// Command result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandResult {
    /// Success flag
    pub success: bool,
    
    /// Result data
    pub data: Option<Value>,
    
    /// Error message
    pub error: Option<String>,
}

impl CommandResult {
    /// Create a successful result
    pub fn success(data: impl Into<Value>) -> Self {
        Self {
            success: true,
            data: Some(data.into()),
            error: None,
        }
    }
    
    /// Create an error result
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }
}

/// Tool plugin trait
#[async_trait]
pub trait ToolPlugin: Plugin {
    /// Get supported commands
    fn get_commands(&self) -> Vec<CommandMetadata>;
    
    /// Check if the plugin supports a specific command
    fn supports_command(&self, command: &str) -> bool {
        self.get_commands().iter().any(|cmd| cmd.name == command)
    }
    
    /// Execute a command
    async fn execute_command(&self, command: &str, args: Value) -> Result<CommandResult>;
    
    /// Get command metadata
    fn get_command_metadata(&self, command: &str) -> Option<CommandMetadata> {
        self.get_commands().into_iter().find(|cmd| cmd.name == command)
    }
} 