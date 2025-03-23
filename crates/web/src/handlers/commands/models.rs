use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Command definition for the legacy API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDefinition>,
}

/// Parameter definition for command parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub parameter_type: String,
    pub default_value: Option<serde_json::Value>,
}

/// Command execution status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Command execution details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    pub id: String,
    pub command_name: String,
    pub parameters: serde_json::Value,
    pub status: CommandStatus,
    pub progress: f32,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CommandExecution> for crate::api::commands::models::CommandStatusResponse {
    fn from(execution: CommandExecution) -> Self {
        Self {
            id: execution.id,
            command: execution.command_name,
            status: match execution.status {
                CommandStatus::Queued => crate::api::commands::CommandStatus::Queued,
                CommandStatus::Running => crate::api::commands::CommandStatus::Running,
                CommandStatus::Completed => crate::api::commands::CommandStatus::Completed,
                CommandStatus::Failed => crate::api::commands::CommandStatus::Failed,
                CommandStatus::Cancelled => crate::api::commands::CommandStatus::Cancelled,
            },
            progress: execution.progress,
            result: execution.result,
            error: execution.error,
            started_at: execution.started_at,
            completed_at: execution.completed_at,
            created_at: execution.created_at,
            updated_at: execution.updated_at,
        }
    }
} 