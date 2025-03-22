//! Command Service implementation
//!
//! This module contains the service layer for command execution and management.

use std::sync::Arc;
use async_trait::async_trait;
#[cfg(feature = "db")]
use sqlx::{Executor, Row, SqlitePool};
use uuid::Uuid;
use chrono::Utc;

use crate::api::error::AppError;
use crate::api::commands::{
    CommandDefinition,
    CommandExecution,
    CommandStatus,
};
use crate::mcp::{McpCommandClient, McpError};

/// Command service trait
#[async_trait]
pub trait CommandService: Send + Sync + 'static {
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
}

#[cfg(feature = "db")]
impl DbCommandService {
    /// Create a new DbCommandService
    pub fn new(db: SqlitePool, mcp_client: Arc<dyn McpCommandClient>) -> Self {
        Self { db, mcp_client }
    }
}

#[cfg(feature = "db")]
#[async_trait]
impl CommandService for DbCommandService {
    async fn create_command(
        &self,
        user_id: &str,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, AppError> {
        let command_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Execute command via MCP
        let mcp_command_id = match self.mcp_client.execute_command(command, parameters).await {
            Ok(id) => id,
            Err(err) => return Err(AppError::from(err)),
        };
        
        // Store command in database
        sqlx::query!(
            r#"
            INSERT INTO command_executions (
                id, command_name, user_id, parameters, 
                status, progress, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            command_id,
            command,
            user_id,
            parameters.to_string(),
            CommandStatus::Running.as_str(),
            0.0,
            now,
            now
        )
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to store command execution: {}", e)))?;
        
        Ok(command_id)
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
}

impl MockCommandService {
    /// Create a new MockCommandService
    pub fn new(mcp_client: Arc<dyn McpCommandClient>) -> Self {
        Self { mcp_client }
    }
}

#[async_trait]
impl CommandService for MockCommandService {
    async fn create_command(
        &self,
        _user_id: &str,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, AppError> {
        // Execute command via MCP but don't store in DB
        match self.mcp_client.execute_command(command, parameters).await {
            Ok(id) => Ok(id),
            Err(err) => Err(AppError::from(err)),
        }
    }
    
    async fn get_available_commands(
        &self,
        _user_id: &str,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<CommandDefinition>, u64, u32), AppError> {
        // Get commands from MCP
        let commands = self.mcp_client.list_available_commands().await
            .map_err(AppError::from)?;
        
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
        _user_id: &str,
        command_id: &str,
    ) -> Result<CommandExecution, AppError> {
        // Get status from MCP
        let status = self.mcp_client.get_command_status(command_id).await
            .map_err(AppError::from)?;
        
        // Convert to CommandExecution
        let now = Utc::now();
        let execution = CommandExecution {
            id: status.id,
            command_name: status.command,
            user_id: "mock-user".to_string(),
            parameters: serde_json::Value::Null,
            status: status.status,
            progress: status.progress,
            result: status.result,
            error: status.error,
            started_at: status.started_at.map(|s| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
            completed_at: status.completed_at.map(|s| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
            created_at: now - chrono::Duration::minutes(5),
            updated_at: now,
        };
        
        Ok(execution)
    }
    
    async fn get_command_history(
        &self,
        _user_id: &str,
        page: u32,
        limit: u32,
        _status: Option<CommandStatus>,
        _command: Option<&str>,
    ) -> Result<(Vec<CommandExecution>, u64, u32), AppError> {
        // Create mock history
        let now = Utc::now();
        let executions = vec![
            CommandExecution {
                id: Uuid::new_v4().to_string(),
                command_name: "test-command".to_string(),
                user_id: "mock-user".to_string(),
                parameters: serde_json::json!({"param1": "value1"}),
                status: CommandStatus::Completed,
                progress: 1.0,
                result: Some(serde_json::json!({"result": "success"})),
                error: None,
                started_at: Some(now - chrono::Duration::minutes(10)),
                completed_at: Some(now - chrono::Duration::minutes(5)),
                created_at: now - chrono::Duration::minutes(15),
                updated_at: now - chrono::Duration::minutes(5),
            },
            CommandExecution {
                id: Uuid::new_v4().to_string(),
                command_name: "another-command".to_string(),
                user_id: "mock-user".to_string(),
                parameters: serde_json::json!({"option": "test"}),
                status: CommandStatus::Failed,
                progress: 0.5,
                result: None,
                error: Some("Command failed".to_string()),
                started_at: Some(now - chrono::Duration::minutes(30)),
                completed_at: Some(now - chrono::Duration::minutes(25)),
                created_at: now - chrono::Duration::minutes(35),
                updated_at: now - chrono::Duration::minutes(25),
            },
        ];
        
        let total = executions.len() as u64;
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        
        // Apply pagination
        let start = ((page - 1) * limit) as usize;
        let end = (start + limit as usize).min(executions.len());
        
        let paged_executions = if start < executions.len() {
            executions[start..end].to_vec()
        } else {
            Vec::new()
        };
        
        Ok((paged_executions, total, total_pages))
    }
    
    async fn cancel_command(
        &self,
        _user_id: &str,
        command_id: &str,
    ) -> Result<(), AppError> {
        // Cancel in MCP
        self.mcp_client.cancel_command(command_id).await
            .map_err(AppError::from)
    }
}

// Convert McpError to AppError
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