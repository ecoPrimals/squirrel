//! Command Service implementation
//!
//! This module contains the service layer for command execution and management.

use chrono::Utc;
use std::convert::TryInto;
use std::sync::Arc;
use async_trait::async_trait;
#[cfg(feature = "db")]
use sqlx::{Executor, Row, SqlitePool, sqlite::SqliteRow};
use tracing::{error, info};

use crate::api::error::AppError;
use crate::api::commands::{
    CommandDefinition,
    CommandExecution,
    CommandStatus,
    CommandSummary,
};
use crate::mcp::{McpCommandClient, McpClient, McpError, ConnectionStatus};
// Import the CommandService trait from the new API for proper access to methods
use crate::api::commands::service::CommandService as NewCommandService;
use crate::api::commands::repository::CommandRepository;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Command service trait
#[async_trait]
pub trait LocalCommandService: Send + Sync + 'static {
    /// Create and execute a new command
    async fn create_command(
        &self,
        user_id: &str,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, AppError>;
    
    /// Get available commands
    async fn get_available_commands(
        &self,
        user_id: &str,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<CommandDefinition>, u64, u32), AppError>;
    
    /// Get command status
    async fn get_command_status(
        &self,
        user_id: &str,
        command_id: &str,
    ) -> Result<CommandExecution, AppError>;
    
    /// Get command history
    async fn get_command_history(
        &self,
        user_id: &str,
        page: u32,
        limit: u32,
        status: Option<CommandStatus>,
        command: Option<&str>,
    ) -> Result<(Vec<CommandExecution>, u64, u32), AppError>;
    
    /// Cancel command execution
    async fn cancel_command(
        &self,
        user_id: &str,
        command_id: &str,
    ) -> Result<(), AppError>;
}

/// Database implementation of the command service
#[cfg(feature = "db")]
pub struct DbCommandService {
    db: SqlitePool,
    mcp_client: Arc<dyn McpCommandClient>,
    ws_manager: Arc<crate::websocket::ConnectionManager>,
}

#[cfg(feature = "db")]
impl DbCommandService {
    /// Create a new DbCommandService
    pub fn new(db: SqlitePool, mcp_client: Arc<dyn McpCommandClient>, ws_manager: Arc<crate::websocket::ConnectionManager>) -> Self {
        Self { db, mcp_client, ws_manager }
    }
}

#[cfg(feature = "db")]
#[async_trait]
impl LocalCommandService for DbCommandService {
    async fn create_command(
        &self,
        user_id: &str,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, AppError> {
        // Execute command via MCP - this now returns the task_id
        let task_id = self.mcp_client.execute_command(command, parameters).await
            .map_err(AppError::from)?;

        // Here, instead of creating a new CommandExecution locally,
        // we would ideally rely on the task system. For now, we return the task_id.
        // The caller (HTTP handler) will use this ID to construct the status URL.
        // In a real implementation, we might query the task manager via MCP 
        // immediately after creation to get the initial state if needed, 
        // or simply return the task_id for polling.
        
        Ok(task_id) // Return the task_id received from the gRPC call
    }
    
    async fn get_available_commands(
        &self,
        _user_id: &str,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<CommandDefinition>, u64, u32), AppError> {
        // Get commands from MCP
        let commands = self.mcp_client.list_available_commands().await
            .map_err(|e| AppError::from(e))?;
        
        let total = commands.len() as u64;
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        
        // Apply pagination
        let start = ((page - 1) * limit) as usize;
        let end = (start + limit as usize).min(commands.len());
        
        let paged_commands = if start < commands.len() {
            commands[start..end].to_vec()
        } else {
            Vec::new()
        };
        
        Ok((paged_commands, total, total_pages))
    }
    
    async fn get_command_status(
        &self,
        user_id: &str,
        command_id: &str,
    ) -> Result<CommandExecution, AppError> {
        // Get the command from the database
        let db_command = sqlx::query_as!(
            CommandExecution,
            r#"
            SELECT 
                id,
                command_name,
                user_id,
                parameters as "parameters: serde_json::Value",
                status as "status: CommandStatus",
                progress,
                result as "result: serde_json::Value",
                error,
                started_at,
                completed_at,
                created_at,
                updated_at
            FROM command_executions
            WHERE id = ? AND user_id = ?
            "#,
            command_id,
            user_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound(format!("Command execution {} not found", command_id)))?;
        
        // For ongoing commands, check status from MCP
        if db_command.status == CommandStatus::Queued || db_command.status == CommandStatus::Running {
            match self.mcp_client.get_command_status(command_id).await {
                Ok(status) => {
                    // Update status in database if changed
                    if status.status != db_command.status || (status.progress - db_command.progress).abs() > f32::EPSILON {
                        sqlx::query!(
                            r#"
                            UPDATE command_executions
                            SET status = ?, progress = ?, updated_at = ?
                            WHERE id = ?
                            "#,
                            status.status.as_str(),
                            status.progress,
                            Utc::now(),
                            command_id
                        )
                        .execute(&self.db)
                        .await
                        .map_err(|e| AppError::Database(e))?;
                        
                        // If completed or failed, update result/error and completion time
                        if status.status == CommandStatus::Completed || status.status == CommandStatus::Failed {
                            let result_json = status.result.map(|r| r.to_string());
                            
                            sqlx::query!(
                                r#"
                                UPDATE command_executions
                                SET result = ?, error = ?, completed_at = ?
                                WHERE id = ?
                                "#,
                                result_json,
                                status.error,
                                Utc::now(),
                                command_id
                            )
                            .execute(&self.db)
                            .await
                            .map_err(|e| AppError::Database(e))?;
                        }
                        
                        // Fetch updated command
                        return self.get_command_status(user_id, command_id).await;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get command status from MCP: {:?}", e);
                    // Continue with database status if MCP fails
                }
            }
        }
        
        Ok(db_command)
    }
    
    async fn get_command_history(
        &self,
        user_id: &str,
        page: u32,
        limit: u32,
        status: Option<CommandStatus>,
        command: Option<&str>,
    ) -> Result<(Vec<CommandExecution>, u64, u32), AppError> {
        // Build the query based on filters
        let mut base_query = String::from("
            SELECT COUNT(*) as count
            FROM command_executions
            WHERE user_id = ?
        ");
        
        let mut params = vec![user_id.to_string()];
        
        if let Some(status_filter) = status {
            base_query.push_str(" AND status = ?");
            params.push(status_filter.as_str().to_string());
        }
        
        if let Some(command_filter) = command {
            base_query.push_str(" AND command_name = ?");
            params.push(command_filter.to_string());
        }
        
        // Get total count for pagination
        let mut query = sqlx::query(&base_query);
        
        // Bind parameters
        for param in &params {
            query = query.bind(param);
        }
        
        let count_row = query
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Database(e))?;
        
        let total_count: i64 = count_row.get(0);
        let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;
        
        // Get the command history with pagination
        let offset = (page - 1) * limit;
        
        let mut data_query = String::from("
            SELECT 
                id,
                command_name,
                user_id,
                parameters,
                status,
                progress,
                result,
                error,
                started_at,
                completed_at,
                created_at,
                updated_at
            FROM command_executions
            WHERE user_id = ?
        ");
        
        if let Some(status_filter) = status {
            data_query.push_str(" AND status = ?");
        }
        
        if let Some(command_filter) = command {
            data_query.push_str(" AND command_name = ?");
        }
        
        data_query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        
        let mut all_params = params.clone();
        all_params.push(limit.to_string());
        all_params.push(offset.to_string());
        
        // Execute the query
        let mut query = sqlx::query(&data_query);
        
        // Bind parameters
        for param in &all_params {
            query = query.bind(param);
        }
        
        let rows = query
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Database(e))?;
        
        // Convert rows to CommandExecution objects
        let mut executions = Vec::new();
        for row in rows {
            let parameters_str: String = row.get("parameters");
            let result_str: Option<String> = row.get("result");
            
            let execution = CommandExecution {
                id: row.get("id"),
                command_name: row.get("command_name"),
                user_id: row.get("user_id"),
                parameters: serde_json::from_str(&parameters_str)
                    .unwrap_or(serde_json::Value::Null),
                status: CommandStatus::from_str(row.get("status")),
                progress: row.get("progress"),
                result: result_str
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
                error: row.get("error"),
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            
            executions.push(execution);
        }
        
        Ok((executions, total_count as u64, total_pages))
    }
    
    async fn cancel_command(
        &self,
        user_id: &str,
        command_id: &str,
    ) -> Result<(), AppError> {
        // Check if the command exists and belongs to the user
        let exists = sqlx::query!(
            "SELECT id FROM command_executions WHERE id = ? AND user_id = ?",
            command_id,
            user_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::Database(e))?
        .is_some();
        
        if !exists {
            return Err(AppError::NotFound(format!("Command execution {} not found", command_id)));
        }
        
        // Cancel the command in MCP
        match self.mcp_client.cancel_command(command_id).await {
            Ok(_) => {
                // Update the command status in the database
                sqlx::query!(
                    "UPDATE command_executions SET status = ?, updated_at = ? WHERE id = ?",
                    CommandStatus::Cancelled.as_str(),
                    Utc::now(),
                    command_id
                )
                .execute(&self.db)
                .await
                .map_err(|e| AppError::Database(e))?;
                
                Ok(())
            }
            Err(e) => Err(AppError::from(e)),
        }
    }
}

/// Mock implementation of the command service for testing
pub struct MockCommandService {
    mcp_client: Arc<dyn McpCommandClient>,
    ws_manager: Arc<crate::websocket::ConnectionManager>,
}

impl MockCommandService {
    /// Create a new MockCommandService
    pub fn new(mcp_client: Arc<dyn McpCommandClient>, ws_manager: Arc<crate::websocket::ConnectionManager>) -> Self {
        Self { mcp_client, ws_manager }
    }
    
    // Helper method to convert McpCommandClient to McpClient for compatibility
    fn get_mcp_client(&self) -> Arc<dyn McpClient> {
        // Use our adapter to convert McpCommandClient to McpClient
        Arc::new(McpClientAdapter::new(self.mcp_client.clone()))
    }
    
    // Helper method to get the McpCommandClient
    fn get_mcp_command_client(&self) -> Arc<dyn McpCommandClient> {
        self.mcp_client.clone()
    }
}

#[async_trait]
impl LocalCommandService for MockCommandService {
    async fn create_command(
        &self,
        user_id: &str,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, AppError> {
        // Execute command via MCP - This now returns a Task ID
        let task_id = self.mcp_client.execute_command(command, parameters).await
            .map_err(|e| {
                error!("Failed to execute command via MCP: {}", e);
                AppError::from(e)
            })?;

        info!(task_id, command, user_id, "Command execution initiated via MCP");

        // Return the task_id received from the gRPC call
        Ok(task_id)
    }
    
    async fn get_available_commands(
        &self,
        _user_id: &str,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<CommandDefinition>, u64, u32), AppError> {
        let real_service = crate::api::commands::CommandServiceImpl::new(
            #[cfg(feature = "mock-db")]
            Arc::new(crate::api::commands::repository::MockCommandRepository::new()),
            #[cfg(not(feature = "mock-db"))]
            Arc::new(DefaultCommandRepository::new()),
            self.get_mcp_command_client(),
            self.ws_manager.clone()
        );
        
        // Use the trait to access the method
        let command_service: &dyn NewCommandService = &real_service;
        match command_service.get_available_commands().await {
            Ok(commands) => {
                let total = commands.len() as u64;
                let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
                
                // Apply pagination
                let start = ((page - 1) * limit) as usize;
                let end = (start + limit as usize).min(commands.len());
                
                // Convert AvailableCommand to CommandDefinition
                let mut definitions = Vec::new();
                for cmd in &commands[start..end] {
                    definitions.push(CommandDefinition {
                        id: format!("mock-id-{}", cmd.name),
                        name: cmd.name.clone(),
                        description: cmd.description.clone(),
                        parameter_schema: cmd.parameter_schema.clone(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    });
                }
                
                Ok((definitions, total, total_pages))
            },
            Err(e) => Err(AppError::Internal(format!("Failed to get available commands: {}", e))),
        }
    }
    
    async fn get_command_status(
        &self,
        user_id: &str,
        command_id: &str,
    ) -> Result<CommandExecution, AppError> {
        let real_service = crate::api::commands::CommandServiceImpl::new(
            #[cfg(feature = "mock-db")]
            Arc::new(crate::api::commands::repository::MockCommandRepository::new()),
            #[cfg(not(feature = "mock-db"))]
            Arc::new(DefaultCommandRepository::new()),
            self.get_mcp_command_client(),
            self.ws_manager.clone()
        );
        
        // Use the trait to access the method
        let command_service: &dyn NewCommandService = &real_service;
        match command_service.get_command_status(command_id, user_id).await {
            Ok(status) => {
                // Convert CommandStatusResponse to CommandExecution
                Ok(CommandExecution {
                    id: status.id,
                    command_name: status.command,
                    user_id: user_id.to_string(),
                    parameters: serde_json::json!({}),
                    status: status.status,
                    progress: status.progress,
                    result: status.result,
                    error: status.error,
                    started_at: status.started_at,
                    completed_at: status.completed_at,
                    created_at: status.created_at,
                    updated_at: status.updated_at,
                })
            },
            Err(e) => Err(AppError::Internal(format!("Failed to get command status: {}", e))),
        }
    }
    
    async fn get_command_history(
        &self,
        user_id: &str,
        page: u32,
        limit: u32,
        status: Option<CommandStatus>,
        command: Option<&str>,
    ) -> Result<(Vec<CommandExecution>, u64, u32), AppError> {
        let real_service = crate::api::commands::CommandServiceImpl::new(
            #[cfg(feature = "mock-db")]
            Arc::new(crate::api::commands::repository::MockCommandRepository::new()),
            #[cfg(not(feature = "mock-db"))]
            Arc::new(DefaultCommandRepository::new()),
            self.get_mcp_command_client(),
            self.ws_manager.clone()
        );
        
        // Use the trait to access the method
        let command_service: &dyn NewCommandService = &real_service;
        match command_service.list_user_commands(user_id, page as i64, limit as i64).await {
            Ok((summaries, total)) => {
                let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
                
                // Filter by status and command if needed
                let filtered: Vec<_> = summaries
                    .into_iter()
                    .filter(|s| {
                        let status_match = match status.as_ref() {
                            None => true,
                            Some(st) => s.status == *st,
                        };
                        let command_match = match command.as_ref() {
                            None => true,
                            Some(cmd) => s.command == *cmd,
                        };
                        status_match && command_match
                    })
                    .collect();
                
                // Convert CommandSummary to CommandExecution
                let executions = filtered
                    .into_iter()
                    .map(|s| {
                        CommandExecution {
                            id: s.id,
                            command_name: s.command.clone(),
                            user_id: user_id.to_string(),
                            parameters: serde_json::Value::Null,
                            status: s.status,
                            progress: s.progress,
                            result: None,
                            error: None,
                            started_at: None,
                            completed_at: None,
                            created_at: s.created_at,
                            updated_at: s.created_at,
                        }
                    })
                    .collect();
                
                Ok((executions, total.try_into().unwrap_or(0), total_pages))
            },
            Err(e) => Err(AppError::Internal(format!("Failed to get command history: {}", e))),
        }
    }
    
    async fn cancel_command(
        &self,
        _user_id: &str,
        _command_id: &str,
    ) -> Result<(), AppError> {
        // No direct equivalent in new API yet, just return success
        Ok(())
    }
}

/// Adapter to convert McpCommandClient into McpClient
pub struct McpClientAdapter {
    inner: Arc<dyn McpCommandClient>
}

impl McpClientAdapter {
    pub fn new(client: Arc<dyn McpCommandClient>) -> Self {
        Self { inner: client }
    }
}

#[async_trait]
impl McpClient for McpClientAdapter {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        self.inner.send_message(message).await
    }
    
    async fn get_status(&self) -> Result<ConnectionStatus, McpError> {
        // Since we don't have direct access to status from command client, return Connected
        Ok(ConnectionStatus::Connected)
    }
}

// If there's a From<McpError> for AppError implementation at the bottom of the file,
// comment it out or remove it entirely
/*
impl From<McpError> for AppError {
    fn from(error: McpError) -> Self {
        match error {
            McpError::ConnectionError(msg) => AppError::Internal(format!("MCP connection error: {}", msg)),
            McpError::CommandError(msg) => AppError::Internal(format!("MCP command error: {}", msg)),
            McpError::InvalidResponse(msg) => AppError::Internal(format!("MCP invalid response: {}", msg)),
            McpError::CommandNotFound(msg) => AppError::NotFound(format!("Command not found: {}", msg)),
            McpError::Timeout(msg) => AppError::Custom(
                axum::http::StatusCode::REQUEST_TIMEOUT,
                format!("MCP timeout: {}", msg)
            ),
            McpError::Internal(msg) => AppError::Internal(format!("MCP internal error: {}", msg)),
        }
    }
}
*/

// Define a default command repository implementation for use when no feature is enabled
#[cfg(not(feature = "mock-db"))]
struct DefaultCommandRepository {
    command_definitions: RwLock<HashMap<String, CommandDefinition>>,
    command_executions: RwLock<HashMap<String, CommandExecution>>,
}

#[cfg(not(feature = "mock-db"))]
impl DefaultCommandRepository {
    fn new() -> Self {
        Self {
            command_definitions: RwLock::new(HashMap::new()),
            command_executions: RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(not(feature = "mock-db"))]
#[async_trait::async_trait]
impl CommandRepository for DefaultCommandRepository {
    async fn create_command_definition(&self, command: CommandDefinition) -> anyhow::Result<()> {
        let mut definitions = self.command_definitions.write().await;
        definitions.insert(command.name.clone(), command);
        Ok(())
    }
    
    async fn get_command_definition(&self, name: &str) -> anyhow::Result<Option<CommandDefinition>> {
        let definitions = self.command_definitions.read().await;
        Ok(definitions.get(name).cloned())
    }
    
    async fn list_command_definitions(&self) -> anyhow::Result<Vec<CommandDefinition>> {
        let definitions = self.command_definitions.read().await;
        Ok(definitions.values().cloned().collect())
    }
    
    async fn upsert_command_definition(&self, command: CommandDefinition) -> anyhow::Result<()> {
        let mut definitions = self.command_definitions.write().await;
        definitions.insert(command.name.clone(), command);
        Ok(())
    }
    
    async fn create_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()> {
        let mut executions = self.command_executions.write().await;
        executions.insert(execution.id.clone(), execution);
        Ok(())
    }
    
    async fn get_command_execution(&self, id: &str) -> anyhow::Result<Option<CommandExecution>> {
        let executions = self.command_executions.read().await;
        Ok(executions.get(id).cloned())
    }
    
    async fn update_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()> {
        let mut executions = self.command_executions.write().await;
        executions.insert(execution.id.clone(), execution);
        Ok(())
    }
    
    async fn list_command_executions(&self, user_id: &str, limit: i64, offset: i64) -> anyhow::Result<Vec<CommandSummary>> {
        let executions = self.command_executions.read().await;
        
        let mut filtered: Vec<_> = executions.values()
            .filter(|e| e.user_id == user_id)
            .cloned()
            .collect();
        
        // Sort by created_at descending
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // Apply offset and limit
        let start = offset as usize;
        let end = (offset + limit) as usize;
        
        let paginated = filtered.into_iter()
            .skip(start)
            .take(end - start)
            .map(|e| CommandSummary {
                id: e.id,
                command: e.command_name,
                status: e.status,
                progress: e.progress,
                created_at: e.created_at,
                started_at: None,
                completed_at: None,
            })
            .collect();
        
        Ok(paginated)
    }
    
    async fn count_command_executions(&self, user_id: &str) -> anyhow::Result<i64> {
        let executions = self.command_executions.read().await;
        let count = executions.values()
            .filter(|e| e.user_id == user_id)
            .count();
        
        Ok(count as i64)
    }
} 