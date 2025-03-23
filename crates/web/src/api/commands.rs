//! Command API data models.
//! 
//! This module contains all data models related to the Command API functionality.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct CommandDefinition {
    /// Command ID
    pub id: String,
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// JSON schema for parameters
    pub parameter_schema: serde_json::Value,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Status of a command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandStatus {
    /// Command is queued but not yet running
    Queued,
    /// Command is currently running
    Running,
    /// Command completed successfully
    Completed,
    /// Command failed to execute
    Failed,
    /// Command was cancelled by the user
    Cancelled,
}

impl CommandStatus {
    /// Convert CommandStatus to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::str::FromStr for CommandStatus {
    type Err = ();

    /// Parse a string into a CommandStatus
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Ok(Self::Failed), // Default to failed for unknown status
        }
    }
}

/// Command execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct CommandExecution {
    /// Execution ID
    pub id: String,
    /// Command name
    pub command_name: String,
    /// User ID
    pub user_id: String,
    /// Command parameters
    pub parameters: serde_json::Value,
    /// Execution status
    #[cfg_attr(feature = "db", sqlx(try_from = "String"))]
    pub status: CommandStatus,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Result (if completed)
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<String>,
    /// Start time
    pub started_at: Option<DateTime<Utc>>,
    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommandRequest {
    /// Command name
    pub command: String,
    /// Command parameters
    pub parameters: serde_json::Value,
}

/// Response for a created command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommandResponse {
    /// Command execution ID
    pub id: String,
    /// Status URL
    pub status_url: String,
}

/// Command status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStatusResponse {
    /// Command execution ID
    pub id: String,
    /// Command name
    pub command: String,
    /// Execution status
    pub status: CommandStatus,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Result (if completed)
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<String>,
    /// Start time
    pub started_at: Option<String>,
    /// Completion time
    pub completed_at: Option<String>,
    /// Time since creation
    pub elapsed: String,
}

/// Command list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandListResponse {
    /// Available commands
    pub commands: Vec<CommandDefinition>,
}

/// Command history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryResponse {
    /// Command executions
    pub executions: Vec<CommandStatusResponse>,
}

/// Type of command list update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandListUpdateType {
    /// New command
    Added,
    /// Updated command
    Updated,
    /// Removed command
    Removed,
}

/// Command summary for list updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSummary {
    /// Command ID
    pub id: String,
    /// Command name
    pub name: String,
    /// Command status
    pub status: CommandStatus,
}

/// WebSocket event for command status updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStatusEvent {
    /// Event type
    pub event: String,  // "command-status"
    /// Command execution ID
    pub id: String,
    /// Command name
    pub command: String,
    /// Execution status
    pub status: CommandStatus,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Result (if completed)
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<String>,
    /// Event timestamp (RFC3339)
    pub timestamp: String,
}

/// WebSocket event for command list updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandListUpdateEvent {
    /// Event type
    pub event: String,  // "command-list-update"
    /// Type of update
    pub update_type: CommandListUpdateType,
    /// Command summary
    pub command: CommandSummary,
    /// Event timestamp (RFC3339)
    pub timestamp: String,
} 