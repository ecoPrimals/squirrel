// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! JSON-RPC types for task service.
//!
//! Replaces protobuf/gRPC types with serde-based equivalents for JSON-RPC transport.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Task representation for JSON-RPC (replaces GenTask)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: i32,
    pub priority: i32,
    pub agent_type: i32,
    pub progress_percent: i32,
    pub agent_id: String,
    pub context_id: String,
    pub prerequisite_task_ids: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub input_data: Vec<u8>,
    pub output_data: Vec<u8>,
    pub error_message: String,
    pub progress_message: String,
    pub metadata: Vec<u8>,
}

/// Create task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: String,
    pub priority: i32,
    pub input_data: Vec<u8>,
    pub metadata: Vec<u8>,
    pub prerequisite_task_ids: Vec<String>,
    pub context_id: String,
    pub agent_id: String,
    pub agent_type: i32,
}

/// Create task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskResponse {
    pub success: bool,
    pub task_id: String,
    pub error_message: String,
}

/// Get task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskRequest {
    pub task_id: String,
}

/// Get task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskResponse {
    pub success: bool,
    pub task: Option<JsonTask>,
    pub error_message: String,
}

/// Update task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub task_id: String,
    pub name: String,
    pub description: String,
    pub priority: i32,
    pub input_data: Vec<u8>,
    pub metadata: Vec<u8>,
}

/// Update task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskResponse {
    pub success: bool,
    pub error_message: String,
}

/// List tasks request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksRequest {
    pub status: i32,
    pub agent_id: String,
    pub agent_type: i32,
    pub context_id: String,
    pub limit: i32,
    pub offset: i32,
}

/// List tasks response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksResponse {
    pub success: bool,
    pub tasks: Vec<JsonTask>,
    pub total_count: i32,
    pub error_message: String,
}

/// Assign task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignTaskRequest {
    pub task_id: String,
    pub agent_id: String,
    pub agent_type: i32,
}

/// Assign task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignTaskResponse {
    pub success: bool,
    pub error_message: String,
}

/// Report progress request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportProgressRequest {
    pub task_id: String,
    pub progress_percent: i32,
    pub progress_message: String,
    pub interim_results: Vec<Vec<u8>>,
}

/// Report progress response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportProgressResponse {
    pub success: bool,
    pub error_message: String,
}

/// Complete task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTaskRequest {
    pub task_id: String,
    pub output_data: Vec<u8>,
    pub metadata: Vec<u8>,
}

/// Complete task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTaskResponse {
    pub success: bool,
    pub error_message: String,
}

/// Cancel task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskRequest {
    pub task_id: String,
    pub reason: String,
}

/// Cancel task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskResponse {
    pub success: bool,
    pub error_message: String,
}
