//! Commands plugin module
//!
//! This module provides functionality for command plugins.

use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::plugin::Plugin;

/// Command metadata
#[derive(Clone, Debug)]
pub struct CommandMetadata {
    /// Command ID
    pub id: String,
    
    /// Command name
    pub name: String,
    
    /// Command description
    pub description: String,
    
    /// Command input schema
    pub input_schema: Value,
    
    /// Command output schema
    pub output_schema: Value,
    
    /// Required permissions
    pub permissions: Vec<String>,
}

/// Commands plugin trait
#[async_trait]
pub trait CommandsPlugin: Plugin {
    /// Get available commands
    fn get_available_commands(&self) -> Vec<CommandMetadata>;
    
    /// Execute a command
    async fn execute_command(&self, command_id: &str, input: Value) -> Result<Value>;
    
    /// Get command help
    fn get_command_help(&self, command_id: &str) -> Option<String>;
    
    /// Check if plugin supports a command
    fn supports_command(&self, command_id: &str) -> bool {
        self.get_available_commands().iter().any(|cmd| cmd.id == command_id)
    }
    
    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
} 