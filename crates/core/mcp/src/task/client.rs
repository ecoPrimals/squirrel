// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Client implementation for task management with the Task Service API.
//!
//! Uses JSON-RPC over Unix socket.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;

use crate::task::json_rpc_types::*;
use crate::task::types::{AgentType, Task, TaskPriority, TaskStatus};

/// Client configuration for connecting to the task service
#[derive(Clone, Debug)]
pub struct TaskClientConfig {
    /// Unix socket path for the task server (e.g., "/tmp/mcp-task.sock")
    pub server_address: String,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
}

/// Client wrapper for the TaskService JSON-RPC API
#[derive(Clone)]
pub struct MCPTaskClient {
    /// Client configuration
    config: TaskClientConfig,
}

impl MCPTaskClient {
    /// Default task client configuration
    pub fn default_config() -> TaskClientConfig {
        let server_address = std::env::var("TASK_SERVER_SOCKET")
            .or_else(|_| std::env::var("TASK_SERVER_ENDPOINT"))
            .unwrap_or_else(|_| {
                universal_constants::network::get_socket_path("mcp-task")
                    .to_string_lossy()
                    .into_owned()
            });

        TaskClientConfig {
            server_address,
            max_retries: 3,
            connect_timeout_ms: 5000,
            request_timeout_ms: 10000,
            initial_backoff_ms: 100,
            max_backoff_ms: 2000,
        }
    }

    /// Create a new task client with default configuration
    pub fn new() -> Self {
        Self::with_config(Self::default_config())
    }

    /// Create a new task client with the given configuration
    pub fn with_config(config: TaskClientConfig) -> Self {
        MCPTaskClient { config }
    }

    /// Connect to the task service (no-op for Unix socket - connection is per-request)
    pub async fn connect(&self) -> Result<()> {
        Ok(())
    }

    /// Get the server address (socket path) from the configuration
    pub fn server_address(&self) -> String {
        self.config.server_address.clone()
    }

    /// Get the maximum retries from the configuration
    pub fn max_retries(&self) -> u32 {
        self.config.max_retries
    }

    /// Get the connect timeout from the configuration
    pub fn connect_timeout(&self) -> u64 {
        self.config.connect_timeout_ms
    }

    /// Get the request timeout from the configuration
    pub fn request_timeout(&self) -> u64 {
        self.config.request_timeout_ms
    }

    /// Get the initial backoff from the configuration
    pub fn initial_backoff(&self) -> u64 {
        self.config.initial_backoff_ms
    }

    /// Get the maximum backoff from the configuration
    pub fn max_backoff(&self) -> u64 {
        self.config.max_backoff_ms
    }

    /// Execute a JSON-RPC call
    async fn json_rpc_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let socket_path = self.config.server_address.clone();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let request_bytes = serde_json::to_vec(&request).context("Failed to serialize request")?;

        let stream = UnixStream::connect(&socket_path).await.context(format!(
            "Failed to connect to task server at {}",
            socket_path
        ))?;

        let (mut reader, mut writer) = stream.into_split();
        writer.write_all(&request_bytes).await?;
        writer.flush().await?;
        drop(writer);

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        let response: serde_json::Value =
            serde_json::from_slice(&buf).context("Invalid JSON-RPC response")?;

        if let Some(err) = response.get("error") {
            let msg = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(anyhow!("JSON-RPC error: {}", msg));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow!("Missing result in JSON-RPC response"))
    }

    /// Create a new task
    pub async fn create_task(
        &self,
        name: &str,
        description: &str,
        priority: TaskPriority,
        input_data: Option<serde_json::Value>,
        metadata: Option<serde_json::Value>,
        context_id: Option<&str>,
        prerequisites: Vec<String>,
    ) -> Result<String> {
        let mut request = CreateTaskRequest {
            name: name.to_string(),
            description: description.to_string(),
            priority: priority as i32,
            input_data: Vec::new(),
            metadata: Vec::new(),
            prerequisite_task_ids: prerequisites,
            context_id: context_id.unwrap_or("").to_string(),
            agent_id: "".to_string(),
            agent_type: 0,
        };

        if let Some(data) = input_data {
            request.input_data =
                serde_json::to_vec(&data).context("Failed to serialize input data")?;
        }
        if let Some(meta) = metadata {
            request.metadata = serde_json::to_vec(&meta).context("Failed to serialize metadata")?;
        }

        let params = serde_json::to_value(&request).context("Failed to serialize request")?;
        let result = self.json_rpc_call("create_task", params).await?;
        let response: CreateTaskResponse = serde_json::from_value(result)?;

        if !response.success {
            return Err(anyhow!("Failed to create task: {}", response.error_message));
        }

        Ok(response.task_id)
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &str) -> Result<Task> {
        let request = GetTaskRequest {
            task_id: task_id.to_string(),
        };
        let params = serde_json::to_value(&request)?;
        let result = self.json_rpc_call("get_task", params).await?;
        let response: GetTaskResponse = serde_json::from_value(result)?;

        if !response.success {
            return Err(anyhow!("Failed to get task: {}", response.error_message));
        }

        let json_task = response
            .task
            .ok_or_else(|| anyhow!("No task in response"))?;

        Ok(json_task_to_task(json_task))
    }

    /// Update a task
    pub async fn update_task(&self, task: &Task) -> Result<()> {
        let request = UpdateTaskRequest {
            task_id: task.id.as_ref().to_string(),
            name: task.name.as_ref().to_string(),
            description: task.description.clone(),
            priority: task.priority_code as i32,
            input_data: task
                .input_data
                .as_ref()
                .map(|m| serde_json::to_vec(m).unwrap_or_default())
                .unwrap_or_default(),
            metadata: task
                .metadata
                .as_ref()
                .map(|m| serde_json::to_vec(m).unwrap_or_default())
                .unwrap_or_default(),
        };

        let params = serde_json::to_value(&request)?;
        let result = self.json_rpc_call("update_task", params).await?;
        let response: UpdateTaskResponse = serde_json::from_value(result)?;

        if !response.success {
            return Err(anyhow!("Failed to update task: {}", response.error_message));
        }

        Ok(())
    }

    /// List tasks with optional filtering
    pub async fn list_tasks(
        &self,
        status: Option<TaskStatus>,
        _priority: Option<TaskPriority>,
        agent_id: Option<&str>,
        agent_type: Option<AgentType>,
        context_id: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Task>> {
        let request = ListTasksRequest {
            status: status.map(|s| s as i32).unwrap_or(-1),
            agent_id: agent_id.unwrap_or("").to_string(),
            agent_type: agent_type.map(|a| a as i32).unwrap_or(-1),
            context_id: context_id.unwrap_or("").to_string(),
            limit: limit.unwrap_or(100).min(i32::MAX as u32) as i32,
            offset: offset.unwrap_or(0).min(i32::MAX as u32) as i32,
        };

        let params = serde_json::to_value(&request)?;
        let result = self.json_rpc_call("list_tasks", params).await?;
        let response: ListTasksResponse = serde_json::from_value(result)?;

        if !response.success {
            return Err(anyhow!("Failed to list tasks: {}", response.error_message));
        }

        Ok(response.tasks.into_iter().map(json_task_to_task).collect())
    }

    /// Assign a task to an agent
    pub async fn assign_task(
        &self,
        task_id: &str,
        agent_id: &str,
        agent_type: AgentType,
    ) -> Result<()> {
        let request = AssignTaskRequest {
            task_id: task_id.to_string(),
            agent_id: agent_id.to_string(),
            agent_type: agent_type as i32,
        };

        let params = serde_json::to_value(&request)?;
        let result = self.json_rpc_call("assign_task", params).await?;
        let response: AssignTaskResponse = serde_json::from_value(result)?;

        if !response.success {
            return Err(anyhow!("Failed to assign task: {}", response.error_message));
        }

        Ok(())
    }

    /// Report progress for a task
    pub async fn report_progress(
        &self,
        task_id: &str,
        progress_percent: i32,
        message: Option<&str>,
    ) -> Result<()> {
        let request = ReportProgressRequest {
            task_id: task_id.to_string(),
            progress_percent,
            progress_message: message.unwrap_or("").to_string(),
            interim_results: Vec::new(),
        };

        let params = serde_json::to_value(&request)?;
        let result = self.json_rpc_call("report_progress", params).await?;
        let response: ReportProgressResponse = serde_json::from_value(result)?;

        if !response.success {
            return Err(anyhow!(
                "Failed to report progress: {}",
                response.error_message
            ));
        }

        Ok(())
    }

    /// Complete a task
    pub async fn complete_task(
        &self,
        task_id: &str,
        output_data: Option<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut request = CompleteTaskRequest {
            task_id: task_id.to_string(),
            output_data: Vec::new(),
            metadata: Vec::new(),
        };

        if let Some(data) = output_data {
            request.output_data =
                serde_json::to_vec(&data).context("Failed to serialize output data")?;
        }
        if let Some(meta) = metadata {
            request.metadata = serde_json::to_vec(&meta).context("Failed to serialize metadata")?;
        }

        let params = serde_json::to_value(&request)?;
        let result = self.json_rpc_call("complete_task", params).await?;
        let response: CompleteTaskResponse = serde_json::from_value(result)?;

        if !response.success {
            return Err(anyhow!(
                "Failed to complete task: {}",
                response.error_message
            ));
        }

        Ok(())
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str, reason: &str) -> Result<()> {
        let request = CancelTaskRequest {
            task_id: task_id.to_string(),
            reason: reason.to_string(),
        };

        let params = serde_json::to_value(&request)?;
        let result = self.json_rpc_call("cancel_task", params).await?;
        let response: CancelTaskResponse = serde_json::from_value(result)?;

        if !response.success {
            return Err(anyhow!("Failed to cancel task: {}", response.error_message));
        }

        Ok(())
    }
}

/// Convert JsonTask to Task
fn json_task_to_task(json: JsonTask) -> Task {
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
        id: std::sync::Arc::from(json.id),
        name: std::sync::Arc::from(json.name),
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

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_create_task() {
        // Stub for testing - requires running task server
    }
}
