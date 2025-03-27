// Command Adapter Pattern implementation
//
// This module provides a set of adapter implementations that facilitate integration
// between the command system and external protocols or interfaces, following the 
// Command Adapter Pattern described in specs/patterns/command-adapter-pattern.md.

use async_trait::async_trait;

pub mod error;
pub mod helper;
pub mod registry;
mod mcp;
mod plugins;
pub mod tests;
pub mod isolated_tests;
pub mod completely_isolated_tests;
#[cfg(test)]
mod isolated_adapters_test;

pub use self::error::{AdapterError, AdapterResult};
pub use self::registry::CommandRegistryAdapter;
pub use self::mcp::{McpCommandAdapter, AuthProvider, BasicAuthProvider, TokenAuthProvider, ApiKeyAuthProvider, Auth, UserRole};
pub use self::plugins::CommandsPluginAdapter;

// Export adapter type
pub type CommandAdapter = dyn CommandAdapterTrait + Send + Sync;

/// Trait for command adapters, providing a consistent interface for command execution
/// across different contexts such as CLI, MCP, and plugin systems.
#[async_trait]
pub trait CommandAdapterTrait: Send + Sync {
    /// Execute a command with the given arguments
    async fn execute_command(&self, command: &str, args: Vec<String>) -> AdapterResult<String>;
    
    /// Get help text for a command
    async fn get_help(&self, command: &str) -> AdapterResult<String>;
    
    /// List all available commands
    async fn list_commands(&self) -> AdapterResult<Vec<String>>;
}

/// Enum for adapter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterType {
    /// Registry adapter for direct command execution
    Registry,
    
    /// MCP adapter for authenticated command execution
    Mcp,
    
    /// Plugin adapter for plugin-based command execution
    Plugin
} 