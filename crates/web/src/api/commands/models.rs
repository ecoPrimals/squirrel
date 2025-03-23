use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Status of a command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for CommandStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandStatus::Queued => write!(f, "queued"),
            CommandStatus::Running => write!(f, "running"),
            CommandStatus::Completed => write!(f, "completed"),
            CommandStatus::Failed => write!(f, "failed"),
            CommandStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl From<&str> for CommandStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "queued" => CommandStatus::Queued,
            "running" => CommandStatus::Running,
            "completed" => CommandStatus::Completed,
            "failed" => CommandStatus::Failed,
            "cancelled" | "canceled" => CommandStatus::Cancelled,
            _ => CommandStatus::Queued,
        }
    }
}

/// Command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Command execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Create command request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommandRequest {
    /// Command name
    pub command: String,
    /// Command parameters
    pub parameters: serde_json::Value,
}

/// Create command response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommandResponse {
    /// Command execution ID
    pub id: String,
    /// Command name
    pub command: String,
    /// Status
    pub status: CommandStatus,
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
    pub started_at: Option<DateTime<Utc>>,
    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

/// Command list filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandListFilter {
    /// Filter by status
    pub status: Option<CommandStatus>,
    /// Filter by command name
    pub command: Option<String>,
    /// Filter by from date
    pub from_date: Option<DateTime<Utc>>,
    /// Filter by to date
    pub to_date: Option<DateTime<Utc>>,
}

/// Command summary for list view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSummary {
    /// Command execution ID
    pub id: String,
    /// Command name
    pub command: String,
    /// Execution status
    pub status: CommandStatus,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Start time (if started)
    pub started_at: Option<DateTime<Utc>>,
    /// Completion time (if completed)
    pub completed_at: Option<DateTime<Utc>>,
}

/// Available command info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableCommand {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Parameter schema
    pub parameter_schema: serde_json::Value,
} 