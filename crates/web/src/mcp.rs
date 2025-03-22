use anyhow::Result;
use std::fmt;
use async_trait::async_trait;
use crate::api::commands::{
    CommandDefinition, 
    CommandStatusResponse,
};

/// MCP connection errors
#[derive(Debug)]
pub enum McpError {
    /// Connection error
    ConnectionError(String),
    /// Command execution error
    CommandError(String),
    /// Invalid response
    InvalidResponse(String),
    /// Command not found
    CommandNotFound(String),
    /// Timeout
    Timeout(String),
    /// Internal error
    Internal(String),
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpError::ConnectionError(msg) => write!(f, "MCP connection error: {}", msg),
            McpError::CommandError(msg) => write!(f, "MCP command error: {}", msg),
            McpError::InvalidResponse(msg) => write!(f, "MCP invalid response: {}", msg),
            McpError::CommandNotFound(msg) => write!(f, "Command not found: {}", msg),
            McpError::Timeout(msg) => write!(f, "MCP timeout: {}", msg),
            McpError::Internal(msg) => write!(f, "MCP internal error: {}", msg),
        }
    }
}

/// MCP client interface for command execution
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

/// Mock MCP client implementation for testing and development
pub struct MockMcpClient {
    host: String,
    port: u16,
}

impl MockMcpClient {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

#[async_trait]
impl McpCommandClient for MockMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        tracing::info!("Mock MCP client sending message: {}", message);
        // In a real implementation, this would send to the MCP server
        Ok(format!("Acknowledged: {}", message))
    }
    
    async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, McpError> {
        tracing::info!(
            "Mock MCP client executing command: {} with parameters: {}", 
            command, 
            parameters
        );
        
        // Generate a fake command ID
        let command_id = uuid::Uuid::new_v4().to_string();
        
        Ok(command_id)
    }
    
    async fn get_command_status(
        &self,
        command_id: &str,
    ) -> Result<CommandStatusResponse, McpError> {
        tracing::info!("Mock MCP client getting status for command: {}", command_id);
        
        // Return a fake command status
        Ok(CommandStatusResponse {
            id: command_id.to_string(),
            command: "mock-command".to_string(),
            status: crate::api::commands::CommandStatus::Running,
            progress: 0.5,
            result: None,
            error: None,
            started_at: Some(chrono::Utc::now().to_rfc3339()),
            completed_at: None,
            elapsed: "00:01:30".to_string(),
        })
    }
    
    async fn cancel_command(
        &self,
        command_id: &str,
    ) -> Result<(), McpError> {
        tracing::info!("Mock MCP client cancelling command: {}", command_id);
        
        // Pretend to cancel the command
        Ok(())
    }
    
    async fn list_available_commands(
        &self,
    ) -> Result<Vec<CommandDefinition>, McpError> {
        tracing::info!("Mock MCP client listing available commands");
        
        // Return a list of fake commands
        let now = chrono::Utc::now();
        let commands = vec![
            CommandDefinition {
                id: "cmd-1".to_string(),
                name: "test-command".to_string(),
                description: "A test command".to_string(),
                parameter_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "param1": {
                            "type": "string",
                            "description": "First parameter"
                        },
                        "param2": {
                            "type": "number",
                            "description": "Second parameter"
                        }
                    },
                    "required": ["param1"]
                }),
                created_at: now,
                updated_at: now,
            },
            CommandDefinition {
                id: "cmd-2".to_string(),
                name: "another-command".to_string(),
                description: "Another test command".to_string(),
                parameter_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "option": {
                            "type": "string",
                            "description": "Command option"
                        }
                    }
                }),
                created_at: now,
                updated_at: now,
            },
        ];
        
        Ok(commands)
    }
} 