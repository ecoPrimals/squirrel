use async_trait::async_trait;
use chrono::Utc;
use tracing::info;
use crate::api::commands::{CommandDefinition, CommandStatus, CommandStatusResponse};
use crate::mcp::client::{McpClient, McpCommandClient};
use crate::mcp::error::McpError;
use crate::mcp::types::ConnectionStatus;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Mock MCP client implementation for testing and development
pub struct MockMcpClient {
    host: String,
    port: u16,
    status: ConnectionStatus,
    // Add a command registry to keep track of executed commands
    command_registry: RwLock<HashMap<String, String>>, // Map from command_id to command_name
    // Add command status registry
    command_status: RwLock<HashMap<String, CommandStatus>>, // Map from command_id to status
}

impl MockMcpClient {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            status: ConnectionStatus::Connected,
            command_registry: RwLock::new(HashMap::new()),
            command_status: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl McpClient for MockMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        info!("Mock MCP client sending message: {}", message);
        
        // Parse the incoming message to determine its type
        if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(message) {
            // Check if this is a subscription message
            if let Some(msg_type) = msg_json.get("type").and_then(|t| t.as_str()) {
                if msg_type == "subscribe" {
                    // Return a proper subscription response
                    return Ok(r#"{"success":true,"subscription_id":"mock-subscription-123"}"#.to_string());
                } else if msg_type == "unsubscribe" {
                    // Return a proper unsubscription response
                    return Ok(r#"{"success":true}"#.to_string());
                }
            }
        }
        
        // Default response for other message types
        Ok(r#"{"success":true,"data":null}"#.to_string())
    }
    
    async fn get_status(&self) -> Result<ConnectionStatus, McpError> {
        // Always return connected for mock client
        Ok(ConnectionStatus::Connected)
    }
}

#[async_trait]
impl McpCommandClient for MockMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        <Self as McpClient>::send_message(self, message).await
    }
    
    async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, McpError> {
        info!(
            "Mock MCP client executing command: {} with parameters: {}", 
            command, 
            parameters
        );
        
        // Generate a fake command ID
        let command_id = uuid::Uuid::new_v4().to_string();
        
        // Store the command name and initial status in our registry
        {
            let mut registry = self.command_registry.write().await;
            registry.insert(command_id.clone(), command.to_string());
            
            let mut status_map = self.command_status.write().await;
            status_map.insert(command_id.clone(), CommandStatus::Queued);
        }
        
        Ok(command_id)
    }
    
    async fn get_command_status(
        &self,
        command_id: &str,
    ) -> Result<CommandStatusResponse, McpError> {
        info!("Mock MCP client getting status for command: {}", command_id);
        
        // Look up the command name in our registry
        let (command_name, status) = {
            let registry = self.command_registry.read().await;
            let name = registry.get(command_id).cloned().unwrap_or_else(|| "mock-command".to_string());
            
            let status_map = self.command_status.read().await;
            let status = status_map.get(command_id).cloned().unwrap_or(CommandStatus::Running);
            
            (name, status)
        };
        
        // Set appropriate properties based on status
        let (progress, completed_at) = match status {
            CommandStatus::Completed => (1.0, Some(Utc::now())),
            CommandStatus::Failed => (0.0, Some(Utc::now())),
            CommandStatus::Cancelled => (0.0, Some(Utc::now())),
            CommandStatus::Running => (0.5, None),
            CommandStatus::Queued => (0.0, None),
        };
        
        // Return a command status with the correct status
        Ok(CommandStatusResponse {
            id: command_id.to_string(),
            command: command_name,
            status,
            progress,
            result: None,
            error: None,
            started_at: Some(Utc::now()),
            completed_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    async fn cancel_command(
        &self,
        command_id: &str,
    ) -> Result<(), McpError> {
        info!("Mock MCP client cancelling command: {}", command_id);
        
        // Actually update the command status to cancelled
        {
            let mut status_map = self.command_status.write().await;
            status_map.insert(command_id.to_string(), CommandStatus::Cancelled);
        }
        
        // Pretend to cancel the command
        Ok(())
    }
    
    async fn list_available_commands(
        &self,
    ) -> Result<Vec<CommandDefinition>, McpError> {
        info!("Mock MCP client listing available commands");
        
        // Return a list of fake commands
        let now = Utc::now();
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