// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Conversion functions between Task and JSON-RPC types.
//!
//! Uses `chrono::DateTime<Utc>` directly for timestamps.

use chrono::Utc;
use std::sync::Arc;

use crate::task::json_rpc_types::JsonTask;
use crate::task::types::{AgentType, Task, TaskPriority, TaskStatus};

/// Convert Task to JsonTask for JSON-RPC transport
impl From<Task> for JsonTask {
    fn from(task: Task) -> Self {
        JsonTask {
            id: task.id.as_ref().to_string(),
            name: task.name.as_ref().to_string(),
            description: task.description,
            status: task.status_code as i32,
            priority: task.priority_code as i32,
            agent_type: task.agent_type as i32,
            progress_percent: task.progress as i32,
            agent_id: task.agent_id.unwrap_or_default(),
            context_id: task.context_id.unwrap_or_default(),
            prerequisite_task_ids: task.prerequisites,
            created_at: Some(task.created_at),
            updated_at: Some(task.updated_at),
            completed_at: task.completed_at,
            input_data: task
                .input_data
                .as_ref()
                .map(|m| serde_json::to_vec(m).unwrap_or_default())
                .unwrap_or_default(),
            output_data: task
                .output_data
                .as_ref()
                .map(|m| serde_json::to_vec(m).unwrap_or_default())
                .unwrap_or_default(),
            error_message: task.error_message.unwrap_or_default(),
            progress_message: task.status_message.unwrap_or_default(),
            metadata: task
                .metadata
                .as_ref()
                .map(|m| serde_json::to_vec(m).unwrap_or_default())
                .unwrap_or_default(),
        }
    }
}

/// Convert JsonTask to Task
impl From<JsonTask> for Task {
    fn from(json: JsonTask) -> Self {
        let input_data = if json.input_data.is_empty() {
            None
        } else {
            serde_json::from_slice(&json.input_data).ok()
        };
        let output_data = if json.output_data.is_empty() {
            None
        } else {
            serde_json::from_slice(&json.output_data).ok()
        };
        let metadata = if json.metadata.is_empty() {
            None
        } else {
            serde_json::from_slice(&json.metadata).ok()
        };

        Task {
            id: Arc::from(json.id),
            name: Arc::from(json.name),
            description: json.description,
            status_code: TaskStatus::from(json.status),
            priority_code: TaskPriority::from(json.priority),
            agent_type: AgentType::from(json.agent_type),
            progress: json.progress_percent as f32,
            agent_id: if json.agent_id.is_empty() {
                None
            } else {
                Some(json.agent_id)
            },
            context_id: if json.context_id.is_empty() {
                None
            } else {
                Some(json.context_id)
            },
            parent_id: None,
            prerequisites: json.prerequisite_task_ids,
            created_at: json.created_at.unwrap_or_else(Utc::now),
            updated_at: json.updated_at.unwrap_or_else(Utc::now),
            completed_at: json.completed_at,
            input_data,
            output_data,
            metadata,
            error_message: if json.error_message.is_empty() {
                None
            } else {
                Some(json.error_message)
            },
            status_message: if json.progress_message.is_empty() {
                None
            } else {
                Some(json.progress_message)
            },
            deadline: None,
            watchable: false,
            retry_count: 0,
            max_retries: 3,
        }
    }
}
