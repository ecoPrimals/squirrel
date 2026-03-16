// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Types for task management.
//!
//! This module contains the core types for task management, including
//! the Task struct and related enums and structs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// Serde helpers for Arc<str> in Task
fn serialize_arc_str<S>(arc_str: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(arc_str)
}

fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}

/// Represents the status of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is waiting to be processed (corresponds to protobuf Pending)
    Waiting = 0,
    /// Task is pending and ready to be assigned (corresponds to protobuf Created)
    Pending = 1,
    /// Task is currently running (corresponds to protobuf Running)
    Running = 2,
    /// Task has been completed successfully (corresponds to protobuf Completed)
    Completed = 3,
    /// Task failed to complete (corresponds to protobuf Failed)
    Failed = 4,
    /// Task was cancelled (corresponds to protobuf Cancelled)
    Cancelled = 5,
    /// Special value used for queries (not in protobuf)
    All = 99,
}

/// Represents the priority of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Unspecified priority
    Unspecified = -1,
    /// Low priority
    Low = 0,
    /// Medium priority
    Medium = 1,
    /// High priority
    High = 2,
    /// Critical priority
    Critical = 3,
    /// Alias for Medium priority
    Normal = 4,
}

/// Represents the type of agent that can handle a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    /// Unspecified agent type
    Unspecified = 0,
    /// Human agent
    Human = 1,
    /// AI agent
    AI = 2,
    /// System agent
    System = 3,
    /// General purpose agent
    General = 4,
    /// Agent specialized for data processing
    DataProcessor = 5,
    /// Agent specialized for file operations
    FileHandler = 6,
    /// Task agent (generic task handler)
    Task = 7,
}

/// Represents a unit of work for an agent.
///
/// A `Task` is the fundamental entity in the task management system.
/// It represents a piece of work that needs to be performed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task — `Arc<str>` for zero-copy (passed around extensively)
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub id: Arc<str>,

    /// Name of the task — `Arc<str>` for zero-copy (task names reused)
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub name: Arc<str>,

    /// Description of the task
    pub description: String,

    /// Current status of the task
    pub status_code: TaskStatus,

    /// Priority of the task
    pub priority_code: TaskPriority,

    /// Type of agent that can handle this task
    pub agent_type: AgentType,

    /// Progress of the task (0.0 to 100.0)
    pub progress: f32,

    /// ID of the agent assigned to this task
    pub agent_id: Option<String>,

    /// ID of the context that this task belongs to
    pub context_id: Option<String>,

    /// ID of the parent task, if this is a subtask
    pub parent_id: Option<String>,

    /// IDs of tasks that must be completed before this one can start
    pub prerequisites: Vec<String>,

    /// When the task was created
    pub created_at: DateTime<Utc>,

    /// When the task was last updated
    pub updated_at: DateTime<Utc>,

    /// When the task was completed
    pub completed_at: Option<DateTime<Utc>>,

    /// Input data for the task
    pub input_data: Option<HashMap<String, String>>,

    /// Output data from the task
    pub output_data: Option<HashMap<String, String>>,

    /// Metadata about the task (additional information)
    pub metadata: Option<HashMap<String, String>>,

    /// Error message if the task failed
    pub error_message: Option<String>,

    /// Status message from the agent
    pub status_message: Option<String>,

    /// When the task must be completed by
    pub deadline: Option<DateTime<Utc>>,

    /// Whether the task should be watchable
    pub watchable: bool,

    /// Number of times this task has been retried
    pub retry_count: i32,

    /// Maximum number of retries allowed
    pub max_retries: i32,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: Arc::from(Uuid::new_v4().to_string()),
            name: Arc::from(""),
            description: String::new(),
            status_code: TaskStatus::Pending,
            priority_code: TaskPriority::Medium,
            agent_type: AgentType::Unspecified,
            progress: 0.0,
            agent_id: None,
            context_id: None,
            parent_id: None,
            prerequisites: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            input_data: None,
            output_data: None,
            metadata: None,
            error_message: None,
            status_message: None,
            deadline: None,
            watchable: false,
            retry_count: 0,
            max_retries: 3,
        }
    }
}

impl Task {
    /// Create a new task with the given name and description.
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: Arc::from(name),
            description: description.to_string(),
            ..Self::default()
        }
    }

    /// Set the priority of the task.
    pub const fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority_code = priority;
        self
    }

    /// Set the agent type that can handle this task.
    pub const fn with_agent_type(mut self, agent_type: AgentType) -> Self {
        self.agent_type = agent_type;
        self
    }

    /// Set the task's context.
    pub fn with_context(mut self, context_id: &str) -> Self {
        self.context_id = Some(context_id.to_string());
        self
    }

    /// Add input data to the task.
    pub fn with_input_data(mut self, input_data: HashMap<String, String>) -> Self {
        self.input_data = Some(input_data);
        self
    }

    /// Add metadata to the task.
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add a prerequisite task that must be completed before this one can start.
    pub fn with_prerequisite(mut self, prerequisite_id: &str) -> Self {
        self.prerequisites.push(prerequisite_id.to_string());
        self
    }

    /// Set the deadline for the task.
    pub const fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Make the task watchable for live updates.
    pub const fn watchable(mut self) -> Self {
        self.watchable = true;
        self
    }

    /// Check if the task is pending.
    pub fn is_pending(&self) -> bool {
        self.status_code == TaskStatus::Pending
    }

    /// Check if the task is running.
    pub fn is_running(&self) -> bool {
        self.status_code == TaskStatus::Running
    }

    /// Check if the task is completed.
    pub fn is_completed(&self) -> bool {
        self.status_code == TaskStatus::Completed
    }

    /// Check if the task has failed.
    pub fn is_failed(&self) -> bool {
        self.status_code == TaskStatus::Failed
    }

    /// Check if the task has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.status_code == TaskStatus::Cancelled
    }

    /// Check if the task is finished (completed, failed, or cancelled).
    pub fn is_finished(&self) -> bool {
        self.is_completed() || self.is_failed() || self.is_cancelled()
    }

    /// Mark the task as running and assign it to an agent.
    pub fn mark_running(&mut self, agent_id: &str) {
        self.status_code = TaskStatus::Running;
        self.agent_id = Some(agent_id.to_string());
        self.updated_at = Utc::now();
    }

    /// Mark the task as completed with optional output data.
    pub fn mark_completed(&mut self, output_data: Option<HashMap<String, String>>) {
        self.status_code = TaskStatus::Completed;
        self.progress = 100.0;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        if let Some(data) = output_data {
            self.output_data = Some(data);
        }
    }

    /// Mark the task as failed with an error message.
    pub fn mark_failed(&mut self, error_message: &str) {
        self.status_code = TaskStatus::Failed;
        self.error_message = Some(error_message.to_string());
        self.updated_at = Utc::now();
    }

    /// Mark the task as cancelled with a reason.
    pub fn mark_cancelled(&mut self, reason: &str) {
        self.status_code = TaskStatus::Cancelled;
        self.status_message = Some(reason.to_string());
        self.updated_at = Utc::now();
    }

    /// Update the progress of the task.
    pub fn update_progress(&mut self, progress: f32, message: Option<String>) {
        self.progress = progress.clamp(0.0, 100.0);
        if let Some(msg) = message {
            self.status_message = Some(msg);
        }
        self.updated_at = Utc::now();
    }

    /// Check if the task is overdue.
    pub fn is_overdue(&self) -> bool {
        if let Some(deadline) = self.deadline {
            Utc::now() > deadline && !self.is_finished()
        } else {
            false
        }
    }

    /// Check if the task can be retried.
    pub fn can_retry(&self) -> bool {
        self.is_failed() && self.retry_count < self.max_retries
    }

    /// Retry the task.
    pub fn retry(&mut self) -> bool {
        if !self.can_retry() {
            return false;
        }

        self.status_code = TaskStatus::Pending;
        self.retry_count += 1;
        self.progress = 0.0;
        self.error_message = None;
        self.agent_id = None;
        self.updated_at = Utc::now();

        true
    }

    /// Set maximum retries for the task.
    pub const fn set_max_retries(&mut self, max_retries: i32) {
        self.max_retries = max_retries;
    }
}

/*
 * Module containing task types and enums
 *
 * This module defines the types and enums used for task management,
 * including status codes, priority levels, and agent types.
 */

impl From<i32> for TaskStatus {
    fn from(code: i32) -> Self {
        match code {
            0 => Self::Waiting,
            1 => Self::Pending,
            2 => Self::Running,
            3 => Self::Completed,
            4 => Self::Failed,
            5 => Self::Cancelled,
            99 => Self::All,
            _ => Self::Waiting,
        }
    }
}

impl From<TaskStatus> for i32 {
    fn from(status: TaskStatus) -> Self {
        match status {
            TaskStatus::Waiting => 0,
            TaskStatus::Pending => 1,
            TaskStatus::Running => 2,
            TaskStatus::Completed => 3,
            TaskStatus::Failed => 4,
            TaskStatus::Cancelled => 5,
            TaskStatus::All => 99,
        }
    }
}

/// Task priority enum that maps to the protobuf `PriorityCode`
impl From<i32> for TaskPriority {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Low,
            1 => Self::Medium,
            2 => Self::High,
            3 => Self::Critical,
            4 => Self::Normal,
            _ => Self::Unspecified,
        }
    }
}

impl From<TaskPriority> for i32 {
    fn from(priority: TaskPriority) -> Self {
        match priority {
            TaskPriority::Low => 0,
            TaskPriority::Medium => 1,
            TaskPriority::Normal => 1,
            TaskPriority::High => 2,
            TaskPriority::Critical => 3,
            TaskPriority::Unspecified => -1,
        }
    }
}

/// Agent type enum that maps to the protobuf `AgentTypeCode`
impl From<i32> for AgentType {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Unspecified,
            1 => Self::Human,
            2 => Self::AI,
            3 => Self::System,
            4 => Self::General,
            5 => Self::DataProcessor,
            6 => Self::FileHandler,
            7 => Self::Task,
            _ => Self::Unspecified,
        }
    }
}

impl From<AgentType> for i32 {
    fn from(agent_type: AgentType) -> Self {
        match agent_type {
            AgentType::Unspecified => 0,
            AgentType::Human => 1,
            AgentType::AI => 2,
            AgentType::System => 3,
            AgentType::General => 4,
            AgentType::DataProcessor => 5,
            AgentType::FileHandler => 6,
            AgentType::Task => 7,
        }
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn task_round_trip_serde(
            name in "[a-zA-Z0-9_-]{1,100}",
            desc in ".*",
        ) {
            let task = Task::new(&name, &desc);
            let json = serde_json::to_string(&task).unwrap();
            let deserialized: Task = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(deserialized.name.as_ref(), name);
            prop_assert_eq!(deserialized.description, desc);
        }

        #[test]
        fn task_status_round_trip(status in 0i32..6) {
            let task_status: TaskStatus = status.into();
            let json = serde_json::to_string(&task_status).unwrap();
            let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(task_status, deserialized);
        }

        #[test]
        fn task_priority_round_trip(priority in 0i32..5) {
            let p = TaskPriority::from(priority);
            let json = serde_json::to_string(&p).unwrap();
            let deserialized: TaskPriority = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(p, deserialized);
        }
    }
}
