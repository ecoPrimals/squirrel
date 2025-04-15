use async_trait::async_trait;
use crate::api::commands::{CommandDefinition, CommandStatusResponse};
use crate::mcp::error::McpError;
use crate::mcp::types::ConnectionStatus;

/// MCP client interface for all MCP communications
#[async_trait]
pub trait McpClient: Send + Sync + 'static {
    /// Send a message to the MCP
    async fn send_message(&self, message: &str) -> Result<String, McpError>;
    
    /// Get the client's connection status
    async fn get_status(&self) -> Result<ConnectionStatus, McpError>;
}

/// MCP command client interface for command execution
#[async_trait]
pub trait McpCommandClient: Send + Sync + 'static {
    /// Send a message to the MCP
    async fn send_message(&self, message: &str) -> Result<String, McpError>;
    
    /// Execute a command via MCP
    async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, McpError>;
    
    /// Get command status
    async fn get_command_status(
        &self,
        command_id: &str,
    ) -> Result<CommandStatusResponse, McpError>;
    
    /// Cancel command
    async fn cancel_command(
        &self,
        command_id: &str,
    ) -> Result<(), McpError>;
    
    /// List available commands
    async fn list_available_commands(
        &self,
    ) -> Result<Vec<CommandDefinition>, McpError>;
} 