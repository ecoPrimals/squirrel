//! gRPC client for MCP services
//!
//! This module provides a client implementation for interacting with MCP gRPC services,
//! including task and sync services.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde_json::json;
use chrono::Utc;
use async_trait::async_trait;
use tonic::transport::Channel;
use tracing::{info, error};

// Import task-related types
use squirrel_mcp::generated::mcp_task::{
    task_service_client::TaskServiceClient,
    CreateTaskRequest, GetTaskRequest, UpdateTaskRequest,
    TaskStatus, AgentType, TaskPriority
};

// Import sync-related types
use squirrel_mcp::generated::mcp_sync::{
    sync_service_client::SyncServiceClient
};

use crate::mcp::{McpClient, McpCommandClient, McpClientConfig, McpError, ConnectionStatus};
use crate::api::commands::{
    CommandDefinition,
    CommandStatus,
    CommandStatusResponse,
};

/// gRPC-based MCP client implementation
#[derive(Debug)]
pub struct McpGrpcClient {
    /// Client configuration
    config: McpClientConfig,
    
    /// Task service client
    task_client: Arc<RwLock<Option<TaskServiceClient<Channel>>>>,
    
    /// Sync service client
    sync_client: Arc<RwLock<Option<SyncServiceClient<Channel>>>>,
    
    /// Connection status
    status: Arc<RwLock<ConnectionStatus>>,
}

impl McpGrpcClient {
    /// Create a new gRPC MCP client
    pub async fn new(config: McpClientConfig) -> Result<Self, McpError> {
        let client = Self {
            config: config.clone(),
            task_client: Arc::new(RwLock::new(None)),
            sync_client: Arc::new(RwLock::new(None)),
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
        };
        
        // Attempt initial connection
        if let Err(e) = client.connect().await {
            error!("Failed to connect to MCP server: {:?}", e);
        }
        
        Ok(client)
    }
    
    /// Connect to the MCP server
    async fn connect(&self) -> Result<(), McpError> {
        *self.status.write().await = ConnectionStatus::Connecting;
        
        // Set up connection timeout
        let timeout = Duration::from_millis(self.config.timeout_ms);
        
        // Create the gRPC endpoint
        let endpoint = format!("http://{}:{}", self.config.host, self.config.port);
        info!("Connecting to MCP server at {}", endpoint);
        
        // Create channel with timeout
        let channel = tonic::transport::Endpoint::from_shared(endpoint)
            .map_err(|e| McpError::ConnectionError(format!("Invalid endpoint: {}", e)))?
            .timeout(timeout)
            .connect_lazy();
            
        // Create the task service client
        let task_client = TaskServiceClient::new(channel.clone());
        *self.task_client.write().await = Some(task_client);
        
        // Create the sync service client
        let sync_client = SyncServiceClient::new(channel);
        *self.sync_client.write().await = Some(sync_client);
        
        // Update connection status
        *self.status.write().await = ConnectionStatus::Connected;
        
        Ok(())
    }
    
    /// Execute a command using the task service
    async fn execute_task_command(&self, command: &str, parameters: &serde_json::Value) -> Result<String, McpError> {
        let client = {
            let client_guard = self.task_client.read().await;
            match &*client_guard {
                Some(client) => client.clone(),
                None => return Err(McpError::ConnectionError("Not connected".to_string())),
            }
        };
        
        let mut client = client;
        
        // Create a new task for the command
        let create_request = tonic::Request::new(CreateTaskRequest {
            name: command.to_string(),
            description: "Command from web".to_string(),
            input_data: serde_json::to_string(parameters)
                .map_err(|e| McpError::CommandError(format!("Failed to serialize parameters: {}", e)))?.into_bytes(),
            context_id: "".to_string(),
            priority: TaskPriority::Medium as i32,
            metadata: format!("{{\"source\": \"web\", \"timestamp\": {}}}", chrono::Utc::now().timestamp()).into_bytes(),
            agent_id: "".to_string(),
            agent_type: AgentType::Ui as i32,
            prerequisite_task_ids: vec![],
        });
        
        let response = client.create_task(create_request).await
            .map_err(|e| McpError::CommandError(format!("Failed to create task: {}", e)))?;
        
        let task = response.into_inner();
        
        // Return the task ID as command ID
        Ok(task.task_id)
    }
    
    /// Get the status of a task-based command
    async fn get_task_status(&self, task_id: &str) -> Result<CommandStatusResponse, McpError> {
        let client = {
            let client_guard = self.task_client.read().await;
            match &*client_guard {
                Some(client) => client.clone(),
                None => return Err(McpError::ConnectionError("Not connected".to_string())),
            }
        };
        
        let mut client = client;
        
        // Get the task status
        let get_request = tonic::Request::new(GetTaskRequest {
            task_id: task_id.to_string(),
        });
        
        let response = client.get_task(get_request).await
            .map_err(|e| McpError::CommandError(format!("Failed to get task status: {}", e)))?;
        
        let response_inner = response.into_inner();
        
        // Ensure we have a task and the request was successful
        if !response_inner.success {
            return Err(McpError::CommandError(format!("Failed to get task: {}", 
                response_inner.error_message)));
        }
        
        let task = response_inner.task.ok_or_else(|| 
            McpError::CommandError("No task in response".to_string()))?;
        
        // Convert task status to command status
        let status = match task.status {
            // Convert i32 to TaskStatus enum and then to CommandStatus
            status if status == TaskStatus::Created as i32 => CommandStatus::Queued,
            status if status == TaskStatus::Assigned as i32 => CommandStatus::Queued,
            status if status == TaskStatus::Running as i32 => CommandStatus::Running,
            status if status == TaskStatus::Completed as i32 => CommandStatus::Completed,
            status if status == TaskStatus::Failed as i32 => CommandStatus::Failed,
            status if status == TaskStatus::Cancelled as i32 => CommandStatus::Cancelled,
            status if status == TaskStatus::Pending as i32 => CommandStatus::Queued,
            _ => CommandStatus::Failed,
        };
        
        // Convert result if present
        let result = if !task.output_data.is_empty() {
            let result_str = String::from_utf8_lossy(&task.output_data);
            serde_json::from_str(&result_str)
                .map_err(|e| McpError::InvalidResponse(format!("Failed to parse task result: {}", e)))?
        } else {
            json!(null)
        };
        
        // Try to extract timestamps from metadata if available
        let mut started_at = None;
        let mut completed_at = None;
        
        if !task.metadata.is_empty() {
            if let Ok(metadata_str) = String::from_utf8(task.metadata.clone()) {
                if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&metadata_str) {
                    if let Some(started) = metadata.get("started_at").and_then(|v| v.as_i64()) {
                        started_at = Some(chrono::DateTime::from_timestamp(started, 0)
                            .unwrap_or_else(|| Utc::now()));
                    }
                    
                    if let Some(completed) = metadata.get("completed_at").and_then(|v| v.as_i64()) {
                        completed_at = Some(chrono::DateTime::from_timestamp(completed, 0)
                            .unwrap_or_else(|| Utc::now()));
                    }
                }
            }
        }
        
        // Get error message if task failed
        let error = if task.status == TaskStatus::Failed as i32 && !task.error_message.is_empty() {
            Some(task.error_message)
        } else {
            None
        };
        
        Ok(CommandStatusResponse {
            id: task_id.to_string(),
            command: task.name,
            status,
            progress: task.progress_percent as f32,
            result: Some(result),
            error,
            started_at,
            completed_at,
            created_at: task.created_at
                .as_ref()
                .and_then(|ts| chrono::DateTime::from_timestamp(ts.seconds, 0))
                .unwrap_or_else(|| Utc::now()),
            updated_at: task.updated_at
                .as_ref()
                .and_then(|ts| chrono::DateTime::from_timestamp(ts.seconds, 0))
                .unwrap_or_else(|| Utc::now()),
        })
    }
}

#[async_trait]
impl McpClient for McpGrpcClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        // Parse the message as a JSON object to determine the type of operation
        let json: serde_json::Value = serde_json::from_str(message)
            .map_err(|e| McpError::CommandError(format!("Invalid message format: {}", e)))?;
        
        // Check the message type using the correct field name "type"
        match json["type"].as_str() {
            Some("execute_command") => {
                if let Some(payload) = json["payload"].as_object() {
                    let command = payload["command"].as_str()
                        .ok_or_else(|| McpError::CommandError("Missing command name".to_string()))?;
                    
                    let parameters = payload["parameters"].clone();
                    
                    self.execute_task_command(command, &parameters).await
                } else {
                    Err(McpError::CommandError("Missing payload".to_string()))
                }
            },
            Some("get_command_status") => {
                if let Some(payload) = json["payload"].as_object() {
                    let command_id = payload["command_id"].as_str()
                        .ok_or_else(|| McpError::CommandError("Missing command ID".to_string()))?;
                    
                    let status = self.get_task_status(command_id).await?;
                    
                    serde_json::to_string(&status)
                        .map_err(|e| McpError::CommandError(format!("Failed to serialize status: {}", e)))
                } else {
                    Err(McpError::CommandError("Missing payload".to_string()))
                }
            },
            // Placeholder for subscription handling
            Some("subscribe") | Some("unsubscribe") => {
                // In a real gRPC implementation, this would involve starting/stopping a stream
                // or making a specific RPC call for event subscription management.
                // For now, just return a simple success message.
                info!(message_type = json["type"].as_str().unwrap_or("unknown"), "Received message, returning placeholder success.");
                Ok(json!({
                    "success": true,
                    "message": "Subscription/Unsubscription placeholder handled.",
                    "subscription_id": "dummy_subscription_id" // Provide a dummy ID for compatibility
                }).to_string())
            },
            // Add other message types as needed
            Some(unsupported_type) => Err(McpError::CommandError(format!("Unsupported message type: {}", unsupported_type))),
            None => Err(McpError::CommandError("Message type is missing or not a string".to_string())),
        }
    }
    
    async fn get_status(&self) -> Result<ConnectionStatus, McpError> {
        let status = self.status.read().await;
        Ok(*status)
    }
}

#[async_trait]
impl McpCommandClient for McpGrpcClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        <Self as McpClient>::send_message(self, message).await
    }
    
    async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, McpError> {
        self.execute_task_command(command, parameters).await
    }
    
    async fn get_command_status(
        &self,
        command_id: &str,
    ) -> Result<CommandStatusResponse, McpError> {
        self.get_task_status(command_id).await
    }
    
    async fn cancel_command(
        &self,
        command_id: &str,
    ) -> Result<(), McpError> {
        let client = {
            let client_guard = self.task_client.read().await;
            match &*client_guard {
                Some(client) => client.clone(),
                None => return Err(McpError::ConnectionError("Not connected".to_string())),
            }
        };
        
        let mut client = client;
        
        // Cancel the task - we need to set the status to cancelled
        // Since we can't use Task struct directly with optional fields,
        // we'll set all required fields with reasonable defaults
        let update_request = tonic::Request::new(UpdateTaskRequest {
            task_id: command_id.to_string(),
            name: "".to_string(),
            description: "Cancelled from web".to_string(),
            priority: TaskPriority::Medium as i32,
            input_data: vec![],
            metadata: format!("{{\"cancelled_at\": {}}}", chrono::Utc::now().timestamp()).into_bytes(),
        });
        
        client.update_task(update_request).await
            .map_err(|e| McpError::CommandError(format!("Failed to cancel task: {}", e)))?;
        
        Ok(())
    }
    
    async fn list_available_commands(
        &self,
    ) -> Result<Vec<CommandDefinition>, McpError> {
        // For now, return a fixed set of commands
        // In a real implementation, we might query the MCP service for available commands
        Ok(vec![
            CommandDefinition {
                id: "echo".to_string(),
                name: "echo".to_string(),
                description: "Echo the input parameters".to_string(),
                parameter_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo"
                        }
                    },
                    "required": ["message"]
                }),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            CommandDefinition {
                id: "sleep".to_string(),
                name: "sleep".to_string(),
                description: "Sleep for a specified number of seconds".to_string(),
                parameter_schema: json!({
                    "type": "object",
                    "properties": {
                        "seconds": {
                            "type": "number",
                            "description": "Number of seconds to sleep"
                        }
                    },
                    "required": ["seconds"]
                }),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        ])
    }
} 