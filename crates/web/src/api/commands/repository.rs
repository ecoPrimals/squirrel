use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;
use async_trait::async_trait;

use crate::api::commands::models::{
    CommandDefinition, CommandExecution, CommandStatus, CommandSummary
};

#[async_trait]
pub trait CommandRepository: Send + Sync {
    /// Create a new command definition
    async fn create_command_definition(&self, command: CommandDefinition) -> anyhow::Result<()>;
    
    /// Get a command definition by name
    async fn get_command_definition(&self, name: &str) -> anyhow::Result<Option<CommandDefinition>>;
    
    /// List all command definitions
    async fn list_command_definitions(&self) -> anyhow::Result<Vec<CommandDefinition>>;
    
    /// Create a command execution
    async fn create_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()>;
    
    /// Get a command execution by ID
    async fn get_command_execution(&self, id: &str) -> anyhow::Result<Option<CommandExecution>>;
    
    /// Update a command execution
    async fn update_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()>;
    
    /// List command executions for a user
    async fn list_command_executions(&self, user_id: &str, limit: i64, offset: i64) -> anyhow::Result<Vec<CommandSummary>>;
    
    /// Count command executions for a user
    async fn count_command_executions(&self, user_id: &str) -> anyhow::Result<i64>;
}

#[cfg(feature = "db")]
pub struct SqliteCommandRepository {
    pool: SqlitePool,
}

#[cfg(feature = "db")]
impl SqliteCommandRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "db")]
#[async_trait]
impl CommandRepository for SqliteCommandRepository {
    async fn create_command_definition(&self, command: CommandDefinition) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO commands (id, name, description, schema, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            command.id,
            command.name,
            command.description,
            command.parameter_schema.to_string(),
            command.created_at,
            command.updated_at
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_command_definition(&self, name: &str) -> anyhow::Result<Option<CommandDefinition>> {
        let record = sqlx::query!(
            r#"
            SELECT id, name, description, schema, created_at, updated_at
            FROM commands
            WHERE name = ?
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(record.map(|r| CommandDefinition {
            id: r.id,
            name: r.name,
            description: r.description,
            parameter_schema: serde_json::from_str(&r.schema).unwrap_or(serde_json::Value::Null),
            created_at: r.created_at.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: r.updated_at.parse().unwrap_or_else(|_| Utc::now()),
        }))
    }
    
    async fn list_command_definitions(&self) -> anyhow::Result<Vec<CommandDefinition>> {
        let records = sqlx::query!(
            r#"
            SELECT id, name, description, schema, created_at, updated_at
            FROM commands
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(records.into_iter().map(|r| CommandDefinition {
            id: r.id,
            name: r.name,
            description: r.description,
            parameter_schema: serde_json::from_str(&r.schema).unwrap_or(serde_json::Value::Null),
            created_at: r.created_at.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: r.updated_at.parse().unwrap_or_else(|_| Utc::now()),
        }).collect())
    }
    
    async fn create_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO command_executions (
                id, command_name, user_id, parameters, status, progress,
                result, error, started_at, completed_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            execution.id,
            execution.command_name,
            execution.user_id,
            execution.parameters.to_string(),
            execution.status.to_string(),
            execution.progress,
            execution.result.map(|r| r.to_string()),
            execution.error,
            execution.started_at.map(|t| t.to_rfc3339()),
            execution.completed_at.map(|t| t.to_rfc3339()),
            execution.created_at.to_rfc3339(),
            execution.updated_at.to_rfc3339()
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_command_execution(&self, id: &str) -> anyhow::Result<Option<CommandExecution>> {
        let record = sqlx::query!(
            r#"
            SELECT 
                id, command_name, user_id, parameters, status, progress,
                result, error, started_at, completed_at, created_at, updated_at
            FROM command_executions
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(record.map(|r| CommandExecution {
            id: r.id,
            command_name: r.command_name,
            user_id: r.user_id,
            parameters: serde_json::from_str(&r.parameters).unwrap_or(serde_json::Value::Null),
            status: CommandStatus::from(r.status.as_str()),
            progress: r.progress,
            result: r.result.map(|s| serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)),
            error: r.error,
            started_at: r.started_at.map(|t| t.parse().unwrap_or_else(|_| Utc::now())),
            completed_at: r.completed_at.map(|t| t.parse().unwrap_or_else(|_| Utc::now())),
            created_at: r.created_at.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: r.updated_at.parse().unwrap_or_else(|_| Utc::now()),
        }))
    }
    
    async fn update_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE command_executions
            SET 
                command_name = ?,
                parameters = ?,
                status = ?,
                progress = ?,
                result = ?,
                error = ?,
                started_at = ?,
                completed_at = ?,
                updated_at = ?
            WHERE id = ?
            "#,
            execution.command_name,
            execution.parameters.to_string(),
            execution.status.to_string(),
            execution.progress,
            execution.result.map(|r| r.to_string()),
            execution.error,
            execution.started_at.map(|t| t.to_rfc3339()),
            execution.completed_at.map(|t| t.to_rfc3339()),
            execution.updated_at.to_rfc3339(),
            execution.id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn list_command_executions(&self, user_id: &str, limit: i64, offset: i64) -> anyhow::Result<Vec<CommandSummary>> {
        let records = sqlx::query!(
            r#"
            SELECT 
                id, command_name, status, progress,
                started_at, completed_at, created_at
            FROM command_executions
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(records.into_iter().map(|r| CommandSummary {
            id: r.id,
            command: r.command_name,
            status: CommandStatus::from(r.status.as_str()),
            progress: r.progress,
            created_at: r.created_at.parse().unwrap_or_else(|_| Utc::now()),
            started_at: r.started_at.map(|t| t.parse().unwrap_or_else(|_| Utc::now())),
            completed_at: r.completed_at.map(|t| t.parse().unwrap_or_else(|_| Utc::now())),
        }).collect())
    }
    
    async fn count_command_executions(&self, user_id: &str) -> anyhow::Result<i64> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM command_executions
            WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(count.count)
    }
}

#[cfg(feature = "mock-db")]
pub struct MockCommandRepository {
    command_definitions: Arc<RwLock<HashMap<String, CommandDefinition>>>,
    command_executions: Arc<RwLock<HashMap<String, CommandExecution>>>,
}

#[cfg(feature = "mock-db")]
impl MockCommandRepository {
    pub fn new() -> Self {
        let mut definitions = HashMap::new();
        
        // Add some default commands
        let code_analysis = CommandDefinition {
            id: Uuid::new_v4().to_string(),
            name: "code-analysis".to_string(),
            description: "Analyze code repository for issues and suggestions".to_string(),
            parameter_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "repository_url": {
                        "type": "string",
                        "description": "URL of the repository to analyze"
                    },
                    "depth": {
                        "type": "string",
                        "enum": ["quick", "normal", "deep"],
                        "default": "normal",
                        "description": "Analysis depth"
                    }
                },
                "required": ["repository_url"]
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let code_generation = CommandDefinition {
            id: Uuid::new_v4().to_string(),
            name: "code-generation".to_string(),
            description: "Generate code from a specification".to_string(),
            parameter_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    },
                    "specification": {
                        "type": "string",
                        "description": "Description of the code to generate"
                    },
                    "output_format": {
                        "type": "string",
                        "enum": ["file", "snippet"],
                        "default": "file",
                        "description": "Output format"
                    }
                },
                "required": ["language", "specification"]
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        definitions.insert(code_analysis.name.clone(), code_analysis);
        definitions.insert(code_generation.name.clone(), code_generation);
        
        Self {
            command_definitions: Arc::new(RwLock::new(definitions)),
            command_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[cfg(feature = "mock-db")]
#[async_trait]
impl CommandRepository for MockCommandRepository {
    async fn create_command_definition(&self, command: CommandDefinition) -> anyhow::Result<()> {
        let mut definitions = self.command_definitions.write().unwrap();
        definitions.insert(command.name.clone(), command);
        Ok(())
    }
    
    async fn get_command_definition(&self, name: &str) -> anyhow::Result<Option<CommandDefinition>> {
        let definitions = self.command_definitions.read().unwrap();
        Ok(definitions.get(name).cloned())
    }
    
    async fn list_command_definitions(&self) -> anyhow::Result<Vec<CommandDefinition>> {
        let definitions = self.command_definitions.read().unwrap();
        Ok(definitions.values().cloned().collect())
    }
    
    async fn create_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()> {
        let mut executions = self.command_executions.write().unwrap();
        executions.insert(execution.id.clone(), execution);
        Ok(())
    }
    
    async fn get_command_execution(&self, id: &str) -> anyhow::Result<Option<CommandExecution>> {
        let executions = self.command_executions.read().unwrap();
        Ok(executions.get(id).cloned())
    }
    
    async fn update_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()> {
        let mut executions = self.command_executions.write().unwrap();
        executions.insert(execution.id.clone(), execution);
        Ok(())
    }
    
    async fn list_command_executions(&self, user_id: &str, limit: i64, offset: i64) -> anyhow::Result<Vec<CommandSummary>> {
        let executions = self.command_executions.read().unwrap();
        let mut filtered: Vec<CommandSummary> = executions
            .values()
            .filter(|e| e.user_id == user_id)
            .map(|e| CommandSummary {
                id: e.id.clone(),
                command: e.command_name.clone(),
                status: e.status,
                progress: e.progress,
                created_at: e.created_at,
                started_at: e.started_at,
                completed_at: e.completed_at,
            })
            .collect();
        
        // Sort by created_at (most recent first)
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // Apply pagination
        let offset = offset as usize;
        let limit = limit as usize;
        let filtered = filtered.into_iter().skip(offset).take(limit).collect();
        
        Ok(filtered)
    }
    
    async fn count_command_executions(&self, user_id: &str) -> anyhow::Result<i64> {
        let executions = self.command_executions.read().unwrap();
        let count = executions
            .values()
            .filter(|e| e.user_id == user_id)
            .count();
        
        Ok(count as i64)
    }
}

#[cfg(feature = "mock-db")]
impl Default for MockCommandRepository {
    fn default() -> Self {
        Self::new()
    }
} 