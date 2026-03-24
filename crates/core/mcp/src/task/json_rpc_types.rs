// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC types for task service.
//!
//! Replaces protobuf/gRPC types with serde-based equivalents for JSON-RPC transport.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Task representation for JSON-RPC (replaces `GenTask`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTask {
    /// Unique task identifier
    pub id: String,
    /// Human-readable task name
    pub name: String,
    /// Task description
    pub description: String,
    /// Task status code
    pub status: i32,
    /// Task priority
    pub priority: i32,
    /// Type of agent that can execute this task
    pub agent_type: i32,
    /// Progress percentage (0-100)
    pub progress_percent: i32,
    /// ID of the agent assigned to this task
    pub agent_id: String,
    /// ID of the context this task belongs to
    pub context_id: String,
    /// IDs of tasks that must complete before this one
    pub prerequisite_task_ids: Vec<String>,
    /// When the task was created
    pub created_at: Option<DateTime<Utc>>,
    /// When the task was last updated
    pub updated_at: Option<DateTime<Utc>>,
    /// When the task was completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Input data for the task
    pub input_data: Vec<u8>,
    /// Output data from task execution
    pub output_data: Vec<u8>,
    /// Error message if task failed
    pub error_message: String,
    /// Current progress message
    pub progress_message: String,
    /// Additional task metadata
    pub metadata: Vec<u8>,
}

/// Create task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    /// Task name
    pub name: String,
    /// Task description
    pub description: String,
    /// Task priority
    pub priority: i32,
    /// Input data for the task
    pub input_data: Vec<u8>,
    /// Additional metadata
    pub metadata: Vec<u8>,
    /// IDs of prerequisite tasks
    pub prerequisite_task_ids: Vec<String>,
    /// Context ID for the task
    pub context_id: String,
    /// Agent ID to assign
    pub agent_id: String,
    /// Agent type
    pub agent_type: i32,
}

/// Create task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// ID of the created task
    pub task_id: String,
    /// Error message if creation failed
    pub error_message: String,
}

/// Get task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskRequest {
    /// ID of the task to retrieve
    pub task_id: String,
}

/// Get task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// The retrieved task, if found
    pub task: Option<JsonTask>,
    /// Error message if retrieval failed
    pub error_message: String,
}

/// Update task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    /// ID of the task to update
    pub task_id: String,
    /// New task name
    pub name: String,
    /// New task description
    pub description: String,
    /// New task priority
    pub priority: i32,
    /// New input data
    pub input_data: Vec<u8>,
    /// New metadata
    pub metadata: Vec<u8>,
}

/// Update task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if update failed
    pub error_message: String,
}

/// List tasks request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksRequest {
    /// Filter by task status
    pub status: i32,
    /// Filter by agent ID
    pub agent_id: String,
    /// Filter by agent type
    pub agent_type: i32,
    /// Filter by context ID
    pub context_id: String,
    /// Maximum number of tasks to return
    pub limit: i32,
    /// Number of tasks to skip (for pagination)
    pub offset: i32,
}

/// List tasks response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// List of matching tasks
    pub tasks: Vec<JsonTask>,
    /// Total count of matching tasks
    pub total_count: i32,
    /// Error message if listing failed
    pub error_message: String,
}

/// Assign task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignTaskRequest {
    /// ID of the task to assign
    pub task_id: String,
    /// ID of the agent to assign to
    pub agent_id: String,
    /// Type of the agent
    pub agent_type: i32,
}

/// Assign task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignTaskResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if assignment failed
    pub error_message: String,
}

/// Report progress request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportProgressRequest {
    /// ID of the task
    pub task_id: String,
    /// Progress percentage (0-100)
    pub progress_percent: i32,
    /// Human-readable progress message
    pub progress_message: String,
    /// Interim results from partial execution
    pub interim_results: Vec<Vec<u8>>,
}

/// Report progress response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportProgressResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if reporting failed
    pub error_message: String,
}

/// Complete task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTaskRequest {
    /// ID of the task to complete
    pub task_id: String,
    /// Output data from task execution
    pub output_data: Vec<u8>,
    /// Additional metadata
    pub metadata: Vec<u8>,
}

/// Complete task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTaskResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if completion failed
    pub error_message: String,
}

/// Cancel task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskRequest {
    /// ID of the task to cancel
    pub task_id: String,
    /// Reason for cancellation
    pub reason: String,
}

/// Cancel task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if cancellation failed
    pub error_message: String,
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn create_task_request_round_trip(
            name in "[a-zA-Z0-9_-]{1,80}",
            description in "[a-zA-Z0-9 _-]{0,200}",
        ) {
            let req = CreateTaskRequest {
                name: name.clone(),
                description: description.clone(),
                priority: 1,
                input_data: vec![],
                metadata: vec![],
                prerequisite_task_ids: vec![],
                context_id: "ctx".to_string(),
                agent_id: "agent".to_string(),
                agent_type: 0,
            };
            let json = serde_json::to_string(&req).expect("should succeed");
            let deserialized: CreateTaskRequest = serde_json::from_str(&json).expect("should succeed");
            prop_assert_eq!(deserialized.name, name);
            prop_assert_eq!(deserialized.description, description);
        }

        #[test]
        fn json_task_round_trip(
            id in "[a-zA-Z0-9-]{1,80}",
            name in "[a-zA-Z0-9_-]{1,80}",
        ) {
            let task = JsonTask {
                id: id.clone(),
                name: name.clone(),
                description: "desc".to_string(),
                status: 1,
                priority: 1,
                agent_type: 0,
                progress_percent: 0,
                agent_id: String::new(),
                context_id: String::new(),
                prerequisite_task_ids: vec![],
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                completed_at: None,
                input_data: vec![],
                output_data: vec![],
                error_message: String::new(),
                progress_message: String::new(),
                metadata: vec![],
            };
            let json = serde_json::to_string(&task).expect("should succeed");
            let deserialized: JsonTask = serde_json::from_str(&json).expect("should succeed");
            prop_assert_eq!(deserialized.id, id);
            prop_assert_eq!(deserialized.name, name);
        }
    }
}
