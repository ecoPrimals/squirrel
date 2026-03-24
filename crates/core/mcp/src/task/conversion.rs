// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Conversion functions between Task and JSON-RPC types.
//!
//! Uses `chrono::DateTime<Utc>` directly for timestamps.

use chrono::Utc;
use std::sync::Arc;

use crate::task::json_rpc_types::JsonTask;
use crate::task::types::{AgentType, Task, TaskPriority, TaskStatus};

/// Convert Task to `JsonTask` for JSON-RPC transport
impl From<Task> for JsonTask {
    fn from(task: Task) -> Self {
        Self {
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

/// Convert `JsonTask` to Task
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

        Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::json_rpc_types::JsonTask;
    use crate::task::types::{AgentType, Task, TaskPriority, TaskStatus};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_task_to_json_task_round_trip() {
        let task = Task {
            id: Arc::from("task-123"),
            name: Arc::from("Test Task"),
            description: "A test".to_string(),
            status_code: TaskStatus::Running,
            priority_code: TaskPriority::High,
            agent_type: AgentType::AI,
            progress: 50.0,
            agent_id: Some("agent-1".to_string()),
            context_id: Some("ctx-1".to_string()),
            parent_id: None,
            prerequisites: vec!["pre-1".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            input_data: Some(HashMap::from([("k".to_string(), "v".to_string())])),
            output_data: None,
            metadata: None,
            error_message: None,
            status_message: Some("in progress".to_string()),
            deadline: None,
            watchable: false,
            retry_count: 0,
            max_retries: 3,
        };
        let json: JsonTask = task.into();
        assert_eq!(json.id, "task-123");
        assert_eq!(json.name, "Test Task");
        assert_eq!(json.status, TaskStatus::Running as i32);
        assert_eq!(json.priority, TaskPriority::High as i32);
        assert_eq!(json.agent_type, AgentType::AI as i32);
        assert_eq!(json.progress_percent, 50);
        assert_eq!(json.agent_id, "agent-1");
        assert_eq!(json.context_id, "ctx-1");
        assert_eq!(json.prerequisite_task_ids, vec!["pre-1"]);
        assert_eq!(json.progress_message, "in progress");

        let back: Task = json.into();
        assert_eq!(back.id.as_ref(), "task-123");
        assert_eq!(back.name.as_ref(), "Test Task");
        assert_eq!(back.status_code, TaskStatus::Running);
        assert_eq!(back.priority_code, TaskPriority::High);
        assert_eq!(back.agent_type, AgentType::AI);
        assert!((back.progress - 50.0_f32).abs() < f32::EPSILON);
        assert_eq!(back.agent_id.as_deref(), Some("agent-1"));
        assert_eq!(back.context_id.as_deref(), Some("ctx-1"));
    }

    #[test]
    fn test_json_task_to_task_empty_optionals() {
        let json = JsonTask {
            id: "id".to_string(),
            name: "name".to_string(),
            description: "desc".to_string(),
            status: 1,
            priority: 1,
            agent_type: 0,
            progress_percent: 0,
            agent_id: String::new(),
            context_id: String::new(),
            prerequisite_task_ids: vec![],
            created_at: None,
            updated_at: None,
            completed_at: None,
            input_data: vec![],
            output_data: vec![],
            error_message: String::new(),
            progress_message: String::new(),
            metadata: vec![],
        };
        let task: Task = json.into();
        assert!(task.agent_id.is_none());
        assert!(task.context_id.is_none());
        assert!(task.input_data.is_none());
        assert!(task.output_data.is_none());
        assert!(task.error_message.is_none());
        assert!(task.status_message.is_none());
    }

    #[test]
    fn test_json_task_to_task_with_json_data() {
        let input_data = serde_json::to_vec(&HashMap::from([("a", "b")])).expect("should succeed");
        let json = JsonTask {
            id: "id".to_string(),
            name: "name".to_string(),
            description: "desc".to_string(),
            status: 2,
            priority: 2,
            agent_type: 2,
            progress_percent: 75,
            agent_id: "agent".to_string(),
            context_id: "ctx".to_string(),
            prerequisite_task_ids: vec![],
            created_at: None,
            updated_at: None,
            completed_at: None,
            input_data,
            output_data: vec![],
            error_message: "err".to_string(),
            progress_message: "progress".to_string(),
            metadata: vec![],
        };
        let task: Task = json.into();
        assert!(task.input_data.is_some());
        assert_eq!(task.error_message.as_deref(), Some("err"));
        assert_eq!(task.status_message.as_deref(), Some("progress"));
    }
}
