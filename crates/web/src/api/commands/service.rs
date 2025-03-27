use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use thiserror::Error;
use async_trait::async_trait;
use serde_json::json;
use tracing::warn;

use crate::api::commands::models::{
    CommandExecution, CommandStatus, 
    CommandSummary, CreateCommandRequest, CreateCommandResponse,
    CommandStatusResponse, AvailableCommand, CommandDefinition
};
use crate::api::commands::repository::CommandRepository;
use crate::mcp::McpCommandClient;
use crate::websocket::{ConnectionManager, ChannelCategory, WebSocketEvent};

#[derive(Debug, Error)]
pub enum CommandServiceError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    
    #[error("Invalid command parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Repository error: {0}")]
    RepositoryError(#[from] anyhow::Error),
    
    #[error("MCP error: {0}")]
    McpError(String),
    
    #[error("Invalid command execution ID: {0}")]
    InvalidExecutionId(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

#[async_trait]
pub trait CommandService: Send + Sync {
    /// Execute a command
    async fn execute_command(&self, request: CreateCommandRequest, user_id: &str) -> Result<CreateCommandResponse, CommandServiceError>;
    
    /// Get available commands
    async fn get_available_commands(&self) -> Result<Vec<AvailableCommand>, CommandServiceError>;
    
    /// Get command status
    async fn get_command_status(&self, command_id: &str, user_id: &str) -> Result<CommandStatusResponse, CommandServiceError>;
    
    /// List user commands
    async fn list_user_commands(
        &self, 
        user_id: &str, 
        page: i64, 
        limit: i64
    ) -> Result<(Vec<CommandSummary>, i64), CommandServiceError>;
    
    /// Update command status
    async fn update_command_status(
        &self,
        command_id: &str,
        status: CommandStatus,
        progress: f32,
        result: Option<serde_json::Value>,
        error: Option<String>
    ) -> Result<(), CommandServiceError>;
    
    /// Cancel a command
    async fn cancel_command(&self, command_id: &str, user_id: &str) -> Result<(), CommandServiceError>;
    
    /// Get command definition
    async fn get_command_definition(&self, command_name: &str) -> Result<Option<CommandDefinition>, CommandServiceError>;
}

pub struct CommandServiceImpl {
    repository: Arc<dyn CommandRepository>,
    mcp_client: Arc<dyn McpCommandClient>,
    ws_manager: Arc<ConnectionManager>,
}

impl CommandServiceImpl {
    pub fn new(
        repository: Arc<dyn CommandRepository>, 
        mcp_client: Arc<dyn McpCommandClient>,
        ws_manager: Arc<ConnectionManager>
    ) -> Self {
        Self { repository, mcp_client, ws_manager }
    }
}

#[async_trait]
impl CommandService for CommandServiceImpl {
    async fn execute_command(
        &self,
        request: CreateCommandRequest, 
        user_id: &str
    ) -> Result<CreateCommandResponse, CommandServiceError> {
        // Validate command exists
        let _command_def = self.repository
            .get_command_definition(&request.command)
            .await?
            .ok_or_else(|| CommandServiceError::CommandNotFound(request.command.clone()))?;
            
        // TODO: Validate parameters against schema
        
        // Create command execution record
        let command_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let execution = CommandExecution {
            id: command_id.clone(),
            command_name: request.command.clone(),
            user_id: user_id.to_string(),
            parameters: request.parameters.clone(),
            status: CommandStatus::Queued,
            progress: 0.0,
            result: None,
            error: None,
            started_at: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        };
        
        // Save to repository
        self.repository.create_command_execution(execution).await?;
        
        // Execute command via MCP
        let _mcp_command_id = self.mcp_client
            .execute_command(&request.command, &request.parameters)
            .await
            .map_err(|e| CommandServiceError::McpError(e.to_string()))?;
        
        // Return response
        Ok(CreateCommandResponse {
            id: command_id.clone(),
            command: request.command,
            status: CommandStatus::Queued,
            status_url: format!("/api/commands/{}", command_id),
        })
    }
    
    async fn get_available_commands(&self) -> Result<Vec<AvailableCommand>, CommandServiceError> {
        // First try to get commands from MCP
        match self.mcp_client.list_available_commands().await {
            Ok(definitions) => {
                // Save/update command definitions in repository
                for def in &definitions {
                    self.repository.upsert_command_definition(def.clone()).await?;
                }
                
                // Convert to available commands
                Ok(definitions.into_iter().map(|def| AvailableCommand {
                    name: def.name,
                    description: def.description,
                    parameter_schema: def.parameter_schema,
                }).collect())
            },
            Err(e) => {
                tracing::warn!("Failed to get commands from MCP: {}", e);
                
                // Fall back to repository
                let definitions = self.repository.list_command_definitions().await?;
                
                Ok(definitions.into_iter().map(|def| AvailableCommand {
                    name: def.name,
                    description: def.description,
                    parameter_schema: def.parameter_schema,
                }).collect())
            }
        }
    }
    
    async fn get_command_status(
        &self, 
        command_id: &str, 
        user_id: &str
    ) -> Result<CommandStatusResponse, CommandServiceError> {
        // First check local repository
        let local_execution = self.repository
            .get_command_execution(command_id)
            .await?;
            
        // If we have a local record, check if the user has access
        if let Some(ref execution) = local_execution {
            if execution.user_id != user_id {
                return Err(CommandServiceError::Unauthorized(
                    "You do not have access to this command execution".to_string()
                ));
            }
        }
        
        // Try to get status from MCP
        match self.mcp_client.get_command_status(command_id).await {
            Ok(status) => {
                // If we have a local record, update it with the MCP status
                if let Some(mut execution) = local_execution {
                    // Update local execution with MCP status
                    execution.status = status.status;
                    execution.progress = status.progress;
                    execution.result = status.result.clone();
                    execution.error = status.error.clone();
                    execution.started_at = status.started_at;
                    execution.completed_at = status.completed_at;
                    execution.updated_at = Utc::now();
                    
                    // Save updates
                    self.repository.update_command_execution(execution).await?;
                }
                
                Ok(status)
            },
            Err(e) => {
                tracing::warn!("Failed to get status from MCP: {}", e);
                
                // Fall back to local repository
                if let Some(execution) = local_execution {
                    Ok(CommandStatusResponse {
                        id: execution.id,
                        command: execution.command_name,
                        status: execution.status,
                        progress: execution.progress,
                        result: execution.result,
                        error: execution.error,
                        started_at: execution.started_at,
                        completed_at: execution.completed_at,
                        created_at: execution.created_at,
                        updated_at: execution.updated_at,
                    })
                } else {
                    Err(CommandServiceError::InvalidExecutionId(command_id.to_string()))
                }
            }
        }
    }
    
    async fn list_user_commands(
        &self, 
        user_id: &str, 
        page: i64, 
        limit: i64
    ) -> Result<(Vec<CommandSummary>, i64), CommandServiceError> {
        let offset = (page - 1) * limit;
        let commands = self.repository.list_command_executions(user_id, limit, offset).await?;
        let total = self.repository.count_command_executions(user_id).await?;
        
        Ok((commands, total))
    }
    
    async fn update_command_status(
        &self,
        command_id: &str,
        status: CommandStatus,
        progress: f32,
        result: Option<serde_json::Value>,
        error: Option<String>
    ) -> Result<(), CommandServiceError> {
        // Get the existing execution
        let mut execution = self.repository
            .get_command_execution(command_id)
            .await?
            .ok_or_else(|| CommandServiceError::InvalidExecutionId(command_id.to_string()))?;
        
        // Update status
        execution.status = status;
        execution.progress = progress;
        execution.updated_at = Utc::now();
        
        // Update timestamps based on status
        match status {
            CommandStatus::Running if execution.started_at.is_none() => {
                execution.started_at = Some(Utc::now());
            },
            CommandStatus::Completed | CommandStatus::Failed | CommandStatus::Cancelled 
                if execution.completed_at.is_none() => {
                execution.completed_at = Some(Utc::now());
            },
            _ => {}
        }
        
        // Update result if provided
        if let Some(result_value) = result {
            execution.result = Some(result_value);
        }
        
        // Update error if provided
        if let Some(error_message) = error {
            execution.error = Some(error_message);
        }
        
        // Save updates
        self.repository.update_command_execution(execution.clone()).await?;
        
        // Broadcast update via WebSocket
        let event = WebSocketEvent {
            event: "command-status".to_string(),
            category: ChannelCategory::Command,
            channel: command_id.to_string(),
            data: json!({
                "id": command_id,
                "status": status.as_str(),
                "progress": progress,
                "command": execution.command_name,
                "result": execution.result,
                "error": execution.error,
                "started_at": execution.started_at.map(|t| t.to_rfc3339()),
                "completed_at": execution.completed_at.map(|t| t.to_rfc3339()),
            }),
            timestamp: Utc::now().to_rfc3339(),
        };
        
        if let Err(e) = self.ws_manager.broadcast(event).await {
            warn!("Failed to broadcast command status update: {}", e);
        }
        
        Ok(())
    }
    
    async fn cancel_command(&self, command_id: &str, user_id: &str) -> Result<(), CommandServiceError> {
        // Get the execution
        let execution = self.repository
            .get_command_execution(command_id)
            .await?
            .ok_or_else(|| CommandServiceError::InvalidExecutionId(command_id.to_string()))?;
        
        // Check if user has access
        if execution.user_id != user_id {
            return Err(CommandServiceError::Unauthorized(
                "You do not have access to this command execution".to_string()
            ));
        }
        
        // Try to cancel via MCP
        match self.mcp_client.cancel_command(command_id).await {
            Ok(_) => {
                // Update local status
                self.update_command_status(
                    command_id,
                    CommandStatus::Cancelled,
                    execution.progress,
                    None,
                    None
                ).await?;
                
                Ok(())
            },
            Err(e) => {
                tracing::warn!("Failed to cancel command via MCP: {}", e);
                
                // Still update local status
                self.update_command_status(
                    command_id,
                    CommandStatus::Cancelled,
                    execution.progress,
                    None,
                    Some(format!("Failed to cancel command via MCP: {}", e))
                ).await?;
                
                Err(CommandServiceError::McpError(e.to_string()))
            }
        }
    }
    
    async fn get_command_definition(&self, command_name: &str) -> Result<Option<CommandDefinition>, CommandServiceError> {
        // Try to get from repository first
        if let Some(definition) = self.repository.get_command_definition(command_name).await? {
            return Ok(Some(definition));
        }
        
        // If not found, try to get from MCP
        match self.mcp_client.list_available_commands().await {
            Ok(definitions) => {
                // Save all definitions
                for def in &definitions {
                    self.repository.upsert_command_definition(def.clone()).await?;
                }
                
                // Find the requested definition
                Ok(definitions.into_iter().find(|d| d.name == command_name))
            },
            Err(e) => {
                tracing::warn!("Failed to get commands from MCP: {}", e);
                Ok(None)
            }
        }
    }
} 