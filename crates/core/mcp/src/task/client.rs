// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Client implementation for task management with the Task Service API.
//!
//! Uses JSON-RPC over Unix socket.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use anyhow::{Context, Result, anyhow};
use chrono::Utc;

use crate::task::json_rpc_types::{
    AssignTaskRequest, AssignTaskResponse, CancelTaskRequest, CancelTaskResponse,
    CompleteTaskRequest, CompleteTaskResponse, CreateTaskRequest, CreateTaskResponse,
    GetTaskRequest, GetTaskResponse, JsonTask, ListTasksRequest, ListTasksResponse,
    ReportProgressRequest, ReportProgressResponse, UpdateTaskRequest, UpdateTaskResponse,
};
use crate::task::types::{AgentType, Task, TaskPriority, TaskStatus};

/// Parameters for creating a new task
#[derive(Debug)]
pub struct CreateTaskParams {
    /// Task name
    pub name: String,
    /// Task description
    pub description: String,
    /// Task priority
    pub priority: TaskPriority,
    /// Optional input data
    pub input_data: Option<serde_json::Value>,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
    /// Optional context ID
    pub context_id: Option<String>,
    /// Prerequisite task IDs
    pub prerequisites: Vec<String>,
}

/// Parameters for listing tasks with optional filters
#[derive(Debug, Default)]
pub struct ListTasksParams {
    /// Filter by status
    pub status: Option<TaskStatus>,
    /// Filter by agent ID
    pub agent_id: Option<String>,
    /// Filter by agent type
    pub agent_type: Option<AgentType>,
    /// Filter by context ID
    pub context_id: Option<String>,
    /// Maximum number of tasks to return
    pub limit: Option<u32>,
    /// Number of tasks to skip
    pub offset: Option<u32>,
}

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

/// Client wrapper for the `TaskService` JSON-RPC API
#[derive(Clone)]
pub struct MCPTaskClient {
    /// Client configuration
    config: TaskClientConfig,
}

impl MCPTaskClient {
    /// Default task client configuration
    #[must_use]
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
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(Self::default_config())
    }

    /// Create a new task client with the given configuration
    #[must_use]
    pub const fn with_config(config: TaskClientConfig) -> Self {
        Self { config }
    }
}

impl Default for MCPTaskClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MCPTaskClient {
    /// Connect to the task service (no-op for Unix socket - connection is per-request)
    pub const fn connect(&self) -> Result<()> {
        Ok(())
    }

    /// Get the server address (socket path) from the configuration
    #[must_use]
    pub fn server_address(&self) -> String {
        self.config.server_address.clone()
    }

    /// Get the maximum retries from the configuration
    #[must_use]
    pub const fn max_retries(&self) -> u32 {
        self.config.max_retries
    }

    /// Get the connect timeout from the configuration
    #[must_use]
    pub const fn connect_timeout(&self) -> u64 {
        self.config.connect_timeout_ms
    }

    /// Get the request timeout from the configuration
    #[must_use]
    pub const fn request_timeout(&self) -> u64 {
        self.config.request_timeout_ms
    }

    /// Get the initial backoff from the configuration
    #[must_use]
    pub const fn initial_backoff(&self) -> u64 {
        self.config.initial_backoff_ms
    }

    /// Get the maximum backoff from the configuration
    #[must_use]
    pub const fn max_backoff(&self) -> u64 {
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

        let stream = UnixStream::connect(&socket_path)
            .await
            .context(format!("Failed to connect to task server at {socket_path}"))?;

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
            return Err(anyhow!("JSON-RPC error: {msg}"));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow!("Missing result in JSON-RPC response"))
    }

    /// Create a new task
    pub async fn create_task(&self, params: CreateTaskParams) -> Result<String> {
        let mut request = CreateTaskRequest {
            name: params.name,
            description: params.description,
            priority: params.priority as i32,
            input_data: Vec::new(),
            metadata: Vec::new(),
            prerequisite_task_ids: params.prerequisites,
            context_id: params.context_id.unwrap_or_default(),
            agent_id: String::new(),
            agent_type: 0,
        };

        if let Some(data) = params.input_data {
            request.input_data =
                serde_json::to_vec(&data).context("Failed to serialize input data")?;
        }
        if let Some(meta) = params.metadata {
            request.metadata = serde_json::to_vec(&meta).context("Failed to serialize metadata")?;
        }

        let request_value =
            serde_json::to_value(&request).context("Failed to serialize request")?;
        let result = self.json_rpc_call("create_task", request_value).await?;
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
    pub async fn list_tasks(&self, params: ListTasksParams) -> Result<Vec<Task>> {
        let request = ListTasksRequest {
            status: params.status.map_or(-1, |s| s as i32),
            agent_id: params.agent_id.unwrap_or_default(),
            agent_type: params.agent_type.map_or(-1, |a| a as i32),
            context_id: params.context_id.unwrap_or_default(),
            limit: params.limit.unwrap_or(100).min(i32::MAX as u32) as i32,
            offset: params.offset.unwrap_or(0).min(i32::MAX as u32) as i32,
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

/// Convert `JsonTask` to Task
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
    use super::*;
    use crate::task::json_rpc_types::{
        AssignTaskRequest, CancelTaskRequest, CompleteTaskRequest, CreateTaskRequest,
        GetTaskRequest, JsonTask, ListTasksRequest, ReportProgressRequest, UpdateTaskRequest,
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    fn test_client(socket_path: std::path::PathBuf) -> MCPTaskClient {
        MCPTaskClient::with_config(TaskClientConfig {
            server_address: socket_path.to_string_lossy().into_owned(),
            max_retries: 1,
            connect_timeout_ms: 5000,
            request_timeout_ms: 10000,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        })
    }

    /// Single-shot JSON-RPC server: reads one request, writes one response, then closes.
    async fn run_mock_rpc_server(
        socket_path: std::path::PathBuf,
        response: serde_json::Value,
    ) -> tokio::task::JoinHandle<()> {
        let _ = std::fs::remove_file(&socket_path);
        let listener = UnixListener::bind(&socket_path).expect("bind unix socket");
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await.expect("read request");
            assert!(!buf.is_empty(), "expected JSON-RPC request bytes");
            let body = serde_json::to_vec(&response).expect("serialize response");
            stream.write_all(&body).await.expect("write response");
        })
    }

    #[test]
    fn default_config_and_with_config_accessors() {
        let cfg = TaskClientConfig {
            server_address: "/tmp/unit-test.sock".to_string(),
            max_retries: 7,
            connect_timeout_ms: 100,
            request_timeout_ms: 200,
            initial_backoff_ms: 10,
            max_backoff_ms: 50,
        };
        let client = MCPTaskClient::with_config(cfg.clone());
        assert!(client.connect().is_ok());
        assert_eq!(client.server_address(), cfg.server_address);
        assert_eq!(client.max_retries(), 7);
        assert_eq!(client.connect_timeout(), 100);
        assert_eq!(client.request_timeout(), 200);
        assert_eq!(client.initial_backoff(), 10);
        assert_eq!(client.max_backoff(), 50);

        let def = MCPTaskClient::default();
        assert!(!def.server_address().is_empty());
    }

    #[test]
    fn create_task_params_and_list_tasks_params_default() {
        let list = ListTasksParams::default();
        assert!(list.status.is_none());
        assert!(list.limit.is_none());

        let params = CreateTaskParams {
            name: "n".to_string(),
            description: "d".to_string(),
            priority: TaskPriority::High,
            input_data: Some(serde_json::json!({"k": 1})),
            metadata: Some(serde_json::json!({"m": true})),
            context_id: Some("ctx".to_string()),
            prerequisites: vec!["p1".to_string()],
        };
        let mut req = CreateTaskRequest {
            name: params.name.clone(),
            description: params.description.clone(),
            priority: params.priority as i32,
            input_data: Vec::new(),
            metadata: Vec::new(),
            prerequisite_task_ids: params.prerequisites.clone(),
            context_id: params.context_id.clone().unwrap_or_default(),
            agent_id: String::new(),
            agent_type: 0,
        };
        if let Some(data) = params.input_data.clone() {
            req.input_data = serde_json::to_vec(&data).unwrap();
        }
        if let Some(meta) = params.metadata.clone() {
            req.metadata = serde_json::to_vec(&meta).unwrap();
        }
        let v = serde_json::to_value(&req).unwrap();
        let back: CreateTaskRequest = serde_json::from_value(v).unwrap();
        assert_eq!(back.name, "n");
        assert!(!back.input_data.is_empty());
    }

    #[test]
    fn json_rpc_request_types_roundtrip() {
        let ct = CreateTaskRequest {
            name: "t".to_string(),
            description: "".to_string(),
            priority: 1,
            input_data: vec![1, 2],
            metadata: vec![],
            prerequisite_task_ids: vec![],
            context_id: "c".to_string(),
            agent_id: "a".to_string(),
            agent_type: 2,
        };
        let v = serde_json::to_value(&ct).unwrap();
        let ct2: CreateTaskRequest = serde_json::from_value(v).unwrap();
        assert_eq!(ct2.agent_type, 2);

        let gt = GetTaskRequest {
            task_id: "id-1".to_string(),
        };
        let gt_rt: GetTaskRequest =
            serde_json::from_value(serde_json::to_value(&gt).unwrap()).unwrap();
        assert_eq!(gt_rt.task_id, gt.task_id);

        let lt = ListTasksRequest {
            status: -1,
            agent_id: "".to_string(),
            agent_type: -1,
            context_id: "".to_string(),
            limit: 50,
            offset: 0,
        };
        let lt2: ListTasksRequest =
            serde_json::from_value(serde_json::to_value(&lt).unwrap()).unwrap();
        assert_eq!(lt2.limit, 50);

        let assign = AssignTaskRequest {
            task_id: "t".to_string(),
            agent_id: "ag".to_string(),
            agent_type: AgentType::AI as i32,
        };
        let _: AssignTaskRequest =
            serde_json::from_value(serde_json::to_value(&assign).unwrap()).unwrap();

        let rp = ReportProgressRequest {
            task_id: "t".to_string(),
            progress_percent: 50,
            progress_message: "m".to_string(),
            interim_results: vec![],
        };
        let _: ReportProgressRequest =
            serde_json::from_value(serde_json::to_value(&rp).unwrap()).unwrap();

        let comp = CompleteTaskRequest {
            task_id: "t".to_string(),
            output_data: vec![0],
            metadata: vec![],
        };
        let _: CompleteTaskRequest =
            serde_json::from_value(serde_json::to_value(&comp).unwrap()).unwrap();

        let can = CancelTaskRequest {
            task_id: "t".to_string(),
            reason: "r".to_string(),
        };
        let _: CancelTaskRequest =
            serde_json::from_value(serde_json::to_value(&can).unwrap()).unwrap();
    }

    #[test]
    fn json_task_to_task_maps_fields() {
        let jt = JsonTask {
            id: "tid".to_string(),
            name: "nm".to_string(),
            description: "desc".to_string(),
            status: TaskStatus::Running as i32,
            priority: TaskPriority::Medium as i32,
            agent_type: AgentType::AI as i32,
            progress_percent: 42,
            agent_id: "agent-1".to_string(),
            context_id: "ctx-1".to_string(),
            prerequisite_task_ids: vec!["pre".to_string()],
            created_at: None,
            updated_at: None,
            completed_at: None,
            input_data: serde_json::to_vec(&serde_json::json!({"x": "1"})).unwrap(),
            output_data: vec![],
            error_message: "".to_string(),
            progress_message: "working".to_string(),
            metadata: serde_json::to_vec(&serde_json::json!({"k": "v"})).unwrap(),
        };
        let task = json_task_to_task(jt);
        assert_eq!(task.id.as_ref(), "tid");
        assert_eq!(task.status_code, TaskStatus::Running);
        assert_eq!(task.priority_code, TaskPriority::Medium);
        assert_eq!(task.agent_type, AgentType::AI);
        assert_eq!(task.progress, 42.0);
        assert_eq!(task.agent_id.as_deref(), Some("agent-1"));
        assert_eq!(task.context_id.as_deref(), Some("ctx-1"));
        assert!(task.input_data.is_some());
        assert!(task.metadata.is_some());
        assert_eq!(task.status_message.as_deref(), Some("working"));
    }

    #[test]
    fn json_task_empty_optional_bytes() {
        let jt = JsonTask {
            id: "1".to_string(),
            name: "n".to_string(),
            description: "".to_string(),
            status: 0,
            priority: 0,
            agent_type: 0,
            progress_percent: 0,
            agent_id: "".to_string(),
            context_id: "".to_string(),
            prerequisite_task_ids: vec![],
            created_at: None,
            updated_at: None,
            completed_at: None,
            input_data: vec![],
            output_data: vec![],
            error_message: "".to_string(),
            progress_message: "".to_string(),
            metadata: vec![],
        };
        let task = json_task_to_task(jt);
        assert!(task.input_data.is_none());
        assert!(task.output_data.is_none());
        assert!(task.metadata.is_none());
        assert!(task.agent_id.is_none());
        assert!(task.status_message.is_none());
    }

    #[test]
    fn update_task_request_serializes_from_task() {
        let mut input = std::collections::HashMap::new();
        input.insert("k".to_string(), "v".to_string());
        let task = Task {
            id: std::sync::Arc::from("u1"),
            name: std::sync::Arc::from("name"),
            description: "d".to_string(),
            status_code: TaskStatus::Pending,
            priority_code: TaskPriority::Low,
            agent_type: AgentType::Unspecified,
            progress: 0.0,
            agent_id: None,
            context_id: None,
            parent_id: None,
            prerequisites: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            completed_at: None,
            input_data: Some(input),
            output_data: None,
            metadata: None,
            error_message: None,
            status_message: None,
            deadline: None,
            watchable: false,
            retry_count: 0,
            max_retries: 1,
        };
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
        let _: UpdateTaskRequest =
            serde_json::from_value(serde_json::to_value(&request).unwrap()).unwrap();
    }

    #[test]
    fn json_task_to_task_invalid_json_bytes_yield_none() {
        let jt = JsonTask {
            id: "1".to_string(),
            name: "n".to_string(),
            description: "".to_string(),
            status: 0,
            priority: 0,
            agent_type: 0,
            progress_percent: 0,
            agent_id: "".to_string(),
            context_id: "".to_string(),
            prerequisite_task_ids: vec![],
            created_at: None,
            updated_at: None,
            completed_at: None,
            input_data: vec![0xff, 0xfe],
            output_data: vec![0x01],
            error_message: "err".to_string(),
            progress_message: "".to_string(),
            metadata: vec![],
        };
        let task = json_task_to_task(jt);
        assert!(task.input_data.is_none());
        assert!(task.output_data.is_none());
        assert_eq!(task.error_message.as_deref(), Some("err"));
    }

    #[tokio::test]
    async fn json_rpc_create_task_success_and_rpc_error() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "mcp_task_client_test_{}.sock",
            uuid::Uuid::new_v4()
        ));
        let ok = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "success": true,
                "task_id": "new-task-id",
                "error_message": ""
            }
        });
        let h = run_mock_rpc_server(path.clone(), ok).await;
        let client = test_client(path.clone());
        let id = client
            .create_task(CreateTaskParams {
                name: "a".into(),
                description: "b".into(),
                priority: TaskPriority::Low,
                input_data: None,
                metadata: None,
                context_id: None,
                prerequisites: vec![],
            })
            .await
            .expect("create_task");
        assert_eq!(id, "new-task-id");
        h.await.unwrap();

        let path2 = dir.join(format!(
            "mcp_task_client_test_{}.sock",
            uuid::Uuid::new_v4()
        ));
        let err_resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "message": "server boom" }
        });
        let h2 = run_mock_rpc_server(path2.clone(), err_resp).await;
        let client2 = test_client(path2.clone());
        let e = client2
            .create_task(CreateTaskParams {
                name: "a".into(),
                description: "b".into(),
                priority: TaskPriority::Low,
                input_data: None,
                metadata: None,
                context_id: None,
                prerequisites: vec![],
            })
            .await
            .expect_err("expected rpc error");
        assert!(e.to_string().contains("server boom"));
        h2.await.unwrap();
    }

    #[tokio::test]
    async fn json_rpc_get_task_missing_task_and_list_success_false() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "mcp_task_client_test_{}.sock",
            uuid::Uuid::new_v4()
        ));
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "success": true,
                "task": null,
                "error_message": ""
            }
        });
        let h = run_mock_rpc_server(path.clone(), resp).await;
        let client = test_client(path.clone());
        let err = client.get_task("x").await.expect_err("no task in body");
        assert!(err.to_string().contains("No task"));
        h.await.unwrap();

        let path2 = dir.join(format!(
            "mcp_task_client_test_{}.sock",
            uuid::Uuid::new_v4()
        ));
        let resp2 = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "success": false,
                "tasks": [],
                "total_count": 0,
                "error_message": "list failed"
            }
        });
        let h2 = run_mock_rpc_server(path2.clone(), resp2).await;
        let client2 = test_client(path2.clone());
        let e = client2
            .list_tasks(ListTasksParams::default())
            .await
            .expect_err("list failed");
        assert!(e.to_string().contains("list failed"));
        h2.await.unwrap();
    }

    #[tokio::test]
    async fn json_rpc_assign_report_complete_cancel_branches() {
        let dir = std::env::temp_dir();
        for (path, method_result, call) in [
            (
                dir.join(format!("as_{}.sock", uuid::Uuid::new_v4())),
                serde_json::json!({
                    "success": false,
                    "error_message": "assign bad"
                }),
                "assign",
            ),
            (
                dir.join(format!("rp_{}.sock", uuid::Uuid::new_v4())),
                serde_json::json!({
                    "success": false,
                    "error_message": "progress bad"
                }),
                "progress",
            ),
            (
                dir.join(format!("cp_{}.sock", uuid::Uuid::new_v4())),
                serde_json::json!({
                    "success": false,
                    "error_message": "complete bad"
                }),
                "complete",
            ),
            (
                dir.join(format!("cn_{}.sock", uuid::Uuid::new_v4())),
                serde_json::json!({
                    "success": false,
                    "error_message": "cancel bad"
                }),
                "cancel",
            ),
        ] {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": method_result
            });
            let _ = std::fs::remove_file(&path);
            let h = run_mock_rpc_server(path.clone(), resp).await;
            let client = test_client(path.clone());
            let r = match call {
                "assign" => client
                    .assign_task("t1", "a1", AgentType::AI)
                    .await
                    .map(|_| ()),
                "progress" => client
                    .report_progress("t1", 50, Some("hi"))
                    .await
                    .map(|_| ()),
                "complete" => client.complete_task("t1", None, None).await.map(|_| ()),
                "cancel" => client.cancel_task("t1", "why").await.map(|_| ()),
                _ => unreachable!(),
            };
            assert!(r.is_err());
            h.await.unwrap();
        }
    }

    #[tokio::test]
    async fn json_rpc_update_task_failure_response() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("upd_{}.sock", uuid::Uuid::new_v4()));
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "success": false,
                "error_message": "cannot update"
            }
        });
        let h = run_mock_rpc_server(path.clone(), resp).await;
        let client = test_client(path.clone());
        let task = Task {
            id: std::sync::Arc::from("tid"),
            name: std::sync::Arc::from("n"),
            description: "d".into(),
            status_code: TaskStatus::Pending,
            priority_code: TaskPriority::Low,
            agent_type: AgentType::Unspecified,
            progress: 0.0,
            agent_id: None,
            context_id: None,
            parent_id: None,
            prerequisites: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            completed_at: None,
            input_data: None,
            output_data: None,
            metadata: None,
            error_message: None,
            status_message: None,
            deadline: None,
            watchable: false,
            retry_count: 0,
            max_retries: 1,
        };
        let err = client
            .update_task(&task)
            .await
            .expect_err("update should fail");
        assert!(err.to_string().contains("cannot update"));
        h.await.unwrap();
    }

    #[tokio::test]
    async fn list_tasks_request_builds_filters_and_limits() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("lt_{}.sock", uuid::Uuid::new_v4()));
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "success": true,
                "tasks": [],
                "total_count": 0,
                "error_message": ""
            }
        });
        let h = run_mock_rpc_server(path.clone(), resp).await;
        let client = test_client(path.clone());
        client
            .list_tasks(ListTasksParams {
                status: Some(TaskStatus::Running),
                agent_id: Some("ag".into()),
                agent_type: Some(AgentType::Human),
                context_id: Some("ctx".into()),
                limit: Some(2000),
                offset: Some(5),
            })
            .await
            .expect("list");
        h.await.unwrap();
    }
}
