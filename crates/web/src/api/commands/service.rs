use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use thiserror::Error;
use async_trait::async_trait;

use crate::api::commands::models::{
    CommandDefinition, CommandExecution, CommandStatus, 
    CommandSummary, CreateCommandRequest, CreateCommandResponse,
    CommandStatusResponse, AvailableCommand
};
use crate::api::commands::repository::CommandRepository;
use crate::mcp::{McpClient, McpMessage, McpContext};

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
}

pub struct CommandServiceImpl {
    repository: Arc<dyn CommandRepository>,
    mcp_client: Arc<dyn McpClient>,
}

impl CommandServiceImpl {
    pub fn new(repository: Arc<dyn CommandRepository>, mcp_client: Arc<dyn McpClient>) -> Self {
        Self { repository, mcp_client }
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
        let command_def = self.repository
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
        
        // Submit to MCP for execution
        let mcp_message = McpMessage {
            type_: "command.execute".to_string(),
            payload: serde_json::json!({
                "command": request.command,
                "parameters": request.parameters,
                "execution_id": command_id,
            }),
            context: McpContext {
                user_id: user_id.to_string(),
                request_id: command_id.clone(),
                timestamp: now,
            },
        };
        
        let _response = self.mcp_client
            .send_message(&serde_json::to_string(&mcp_message).unwrap())
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
        let definitions = self.repository.list_command_definitions().await?;
        
        Ok(definitions.into_iter().map(|def| AvailableCommand {
            name: def.name,
            description: def.description,
            parameter_schema: def.parameter_schema,
        }).collect())
    }
    
    async fn get_command_status(
        &self, 
        command_id: &str, 
        user_id: &str
    ) -> Result<CommandStatusResponse, CommandServiceError> {
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
        self.repository.update_command_execution(execution).await?;
        
        Ok(())
    }
} 