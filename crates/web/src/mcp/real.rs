use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::api::commands::{CommandDefinition, CommandStatusResponse, CommandStatus};
use crate::mcp::client::{McpClient, McpCommandClient};
use crate::mcp::context::{ContextManager, McpContext};
use crate::mcp::error::McpError;
use crate::mcp::types::{ConnectionStatus, McpClientConfig, McpMessage};

/// Real MCP client implementation that integrates with the MCP crate
pub struct RealMcpClient {
    /// MCP client configuration
    config: McpClientConfig,
    
    /// Connection status
    status: Arc<RwLock<ConnectionStatus>>,
    
    /// Command status cache
    command_status_cache: Arc<RwLock<HashMap<String, CommandStatusResponse>>>,
    
    /// The host and port for connection
    addr: SocketAddr,

    /// Context manager for tracking MCP contexts
    context_manager: Arc<ContextManager>,
}

impl RealMcpClient {
    /// Create a new real MCP client with the given configuration
    pub async fn new(config: McpClientConfig) -> Result<Self, McpError> {
        // Parse address
        let addr_str = format!("{}:{}", config.host, config.port);
        let addr = addr_str.parse::<SocketAddr>()
            .map_err(|e| McpError::ConnectionError(format!("Invalid address: {}", e)))?;
        
        // Create context manager
        let context_manager = Arc::new(ContextManager::new());
        
        // Create client
        let client = Self {
            config,
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            command_status_cache: Arc::new(RwLock::new(HashMap::new())),
            addr,
            context_manager,
        };
        
        // Connect
        client.connect().await?;
        
        Ok(client)
    }
    
    /// Connect to the MCP server
    async fn connect(&self) -> Result<(), McpError> {
        // Set status to connecting
        {
            let mut status = self.status.write().await;
            *status = ConnectionStatus::Connecting;
        }
        
        // We don't actually store the client connection - just update the status
        // based on whether we can make a connection
        let mut connected = false;
        
        // Attempt to connect
        for attempt in 0..self.config.retry_attempts {
            // Try to establish a TCP connection to validate the server is available
            match tokio::net::TcpStream::connect(self.addr).await {
                Ok(_) => {
                    connected = true;
                    info!("Connected to MCP server at {}", self.addr);
                    break;
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to MCP server (attempt {}/{}): {}", 
                        attempt + 1, self.config.retry_attempts, e);
                    
                    if attempt < self.config.retry_attempts - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            self.config.retry_delay_ms
                        )).await;
                    }
                }
            }
        }
        
        // Update connection status
        {
            let mut status = self.status.write().await;
            if connected {
                *status = ConnectionStatus::Connected;
                Ok(())
            } else {
                *status = ConnectionStatus::Error;
                Err(McpError::ConnectionError(format!(
                    "Failed to connect to MCP server after {} attempts", 
                    self.config.retry_attempts
                )))
            }
        }
    }
    
    /// Create a command message with enhanced context
    fn create_command_message(
        &self,
        command: &str,
        parameters: &serde_json::Value,
        context: Option<McpContext>,
    ) -> String {
        let context = match context {
            Some(ctx) => ctx,
            None => McpContext {
                user_id: "anonymous".to_string(),
                request_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                session_id: None,
                source: Some("web".to_string()),
                correlation_id: None,
                metadata: None,
            },
        };
        
        let message = McpMessage {
            type_: "command.execute".to_string(),
            payload: json!({
                "command": command,
                "parameters": parameters,
            }),
            context,
        };
        
        match serde_json::to_string(&message) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to serialize MCP message: {}", e);
                "{}".to_string()
            }
        }
    }
    
    /// Convert MCP command status to API command status
    fn convert_command_status(&self, status: &str) -> CommandStatus {
        match status.to_lowercase().as_str() {
            "pending" => CommandStatus::Queued, // Map pending to Queued
            "running" => CommandStatus::Running,
            "completed" => CommandStatus::Completed,
            "failed" => CommandStatus::Failed,
            "cancelled" => CommandStatus::Cancelled,
            _ => CommandStatus::Queued, // Default to Queued
        }
    }
    
    /// Send a request to the MCP server
    async fn send_request(&self, message: &str) -> Result<String, McpError> {
        // In a real implementation, we'd connect to the server and send the message
        // For now, we'll just log the message and return a dummy response
        
        // Check if we're connected
        let status = self.status.read().await;
        if *status != ConnectionStatus::Connected {
            return Err(McpError::ConnectionError("Not connected to MCP server".to_string()));
        }
        
        // Parse the message
        let msg_value: serde_json::Value = serde_json::from_str(message)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse message: {}", e)))?;
        
        // In real implementation, send to server
        // For now, create a mock response based on the request
        let response = if let Some(command) = msg_value.get("command").and_then(|c| c.as_str()) {
            // This is a command message
            json!({
                "id": Uuid::new_v4().to_string(),
                "type": "response",
                "status": "success",
                "command_id": Uuid::new_v4().to_string(),
                "data": {
                    "message": format!("Received command: {}", command)
                }
            })
        } else {
            // Generic response
            json!({
                "id": Uuid::new_v4().to_string(),
                "type": "response",
                "status": "success",
                "data": {
                    "message": "Message received"
                }
            })
        };
        
        // Serialize response
        serde_json::to_string(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to serialize response: {}", e)))
    }
}

#[async_trait]
impl McpClient for RealMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        self.send_request(message).await
    }
    
    async fn get_status(&self) -> Result<ConnectionStatus, McpError> {
        let status = self.status.read().await;
        Ok(*status)
    }
}

#[async_trait]
impl McpCommandClient for RealMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        // Delegate to McpClient implementation
        <Self as McpClient>::send_message(self, message).await
    }
    
    async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, McpError> {
        info!("Executing command: {}", command);
        
        // Create context with enhanced fields
        let context = self.context_manager.create_context(
            "web-user".to_string(),  // This should be replaced with actual user ID in production
            Some("web-session".to_string()),  // This should be replaced with actual session ID in production
            Some("web".to_string()),
            None,
            Some(json!({
                "source_ip": "127.0.0.1",  // This should be replaced with actual client IP in production
                "command_source": "web_api"
            })),
        ).await;
        
        // Create command message
        let message = self.create_command_message(command, parameters, Some(context));
        
        // Send request
        let response = self.send_request(&message).await?;
        
        // Parse response
        let response_json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse response: {}", e)))?;
        
        // Check for command ID
        let command_id = response_json.get("command_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidResponse("Missing command_id".to_string()))?;
        
        Ok(command_id.to_string())
    }
    
    async fn get_command_status(&self, command_id: &str) -> Result<CommandStatusResponse, McpError> {
        // Check the cache first
        {
            let cache = self.command_status_cache.read().await;
            if let Some(status) = cache.get(command_id) {
                return Ok(status.clone());
            }
        }
        
        // Create status request message
        let status_message = serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "type": "command_status",
            "command_id": command_id,
        });
        
        // Send status request
        let status_request = serde_json::to_string(&status_message)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to serialize status request: {}", e)))?;
        
        let response = <Self as McpClient>::send_message(self, &status_request).await?;
        
        // Parse response
        let response_value: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse response: {}", e)))?;
        
        // Extract status
        let status = response_value.get("status").and_then(|s| s.as_str()).unwrap_or("queued");
        let progress = response_value.get("progress").and_then(|p| p.as_f64()).unwrap_or(0.0) as f32;
        let result = response_value.get("result").cloned();
        let error = response_value.get("error").and_then(|e| e.as_str()).map(|e| e.to_string());
        
        // Create command status
        let command_status = CommandStatusResponse {
            id: command_id.to_string(),
            command: response_value.get("command").and_then(|c| c.as_str()).unwrap_or("unknown").to_string(),
            status: self.convert_command_status(status),
            progress,
            result,
            error,
            started_at: response_value.get("started_at").and_then(|t| t.as_str()).and_then(|t| DateTime::parse_from_rfc3339(t).ok()).map(|t| t.with_timezone(&Utc)),
            completed_at: response_value.get("completed_at").and_then(|t| t.as_str()).and_then(|t| DateTime::parse_from_rfc3339(t).ok()).map(|t| t.with_timezone(&Utc)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Update the cache
        {
            let mut cache = self.command_status_cache.write().await;
            cache.insert(command_id.to_string(), command_status.clone());
        }
        
        Ok(command_status)
    }
    
    async fn cancel_command(&self, command_id: &str) -> Result<(), McpError> {
        // Create cancel request message
        let cancel_message = serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "type": "command_cancel",
            "command_id": command_id,
        });
        
        // Send cancel request
        let cancel_request = serde_json::to_string(&cancel_message)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to serialize cancel request: {}", e)))?;
        
        let _ = <Self as McpClient>::send_message(self, &cancel_request).await?;
        
        // Update the cache status to cancelled
        {
            let mut cache = self.command_status_cache.write().await;
            if let Some(mut status) = cache.get(command_id).cloned() {
                status.status = CommandStatus::Cancelled;
                status.updated_at = Utc::now();
                cache.insert(command_id.to_string(), status);
            }
        }
        
        Ok(())
    }
    
    async fn list_available_commands(&self) -> Result<Vec<CommandDefinition>, McpError> {
        // Create list request message
        let list_message = serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "type": "list_commands",
        });
        
        // Send list request
        let list_request = serde_json::to_string(&list_message)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to serialize list request: {}", e)))?;
        
        let response = <Self as McpClient>::send_message(self, &list_request).await?;
        
        // Parse response
        let response_value: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse response: {}", e)))?;
        
        // Extract commands
        let commands = response_value.get("commands").and_then(|c| c.as_array());
        if let Some(commands) = commands {
            let mut result = Vec::new();
            for cmd in commands {
                let id = cmd.get("id").and_then(|i| i.as_str()).unwrap_or_default().to_string();
                let name = cmd.get("name").and_then(|n| n.as_str()).unwrap_or_default().to_string();
                let description = cmd.get("description").and_then(|d| d.as_str()).unwrap_or_default().to_string();
                let parameter_schema = cmd.get("parameter_schema").cloned().unwrap_or(serde_json::json!({}));
                
                result.push(CommandDefinition {
                    id,
                    name,
                    description,
                    parameter_schema,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
} 