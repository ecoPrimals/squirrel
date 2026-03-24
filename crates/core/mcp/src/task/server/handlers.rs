// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC Task Service Handlers
//!
//! Replaces gRPC service handlers with JSON-RPC handler pattern.

use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};

use crate::error::MCPError;
use crate::task::json_rpc_types::{
    AssignTaskRequest, AssignTaskResponse, CancelTaskRequest, CancelTaskResponse,
    CompleteTaskRequest, CompleteTaskResponse, CreateTaskRequest, CreateTaskResponse,
    GetTaskRequest, GetTaskResponse, JsonTask, ListTasksRequest, ListTasksResponse,
    ReportProgressRequest, ReportProgressResponse, UpdateTaskRequest, UpdateTaskResponse,
};
use crate::task::server::service::TaskServiceImpl;
use crate::task::types::{AgentType, Task, TaskPriority};

/// Convert bytes to `HashMap`<String, String> if not empty
fn bytes_to_hashmap(data: &[u8]) -> HashMap<String, String> {
    if data.is_empty() {
        return HashMap::new();
    }

    match serde_json::from_slice::<HashMap<String, serde_json::Value>>(data) {
        Ok(map) => map.into_iter().map(|(k, v)| (k, v.to_string())).collect(),
        Err(e) => {
            warn!("Failed to parse JSON data: {}", e);
            HashMap::new()
        }
    }
}

/// Serialize a value to JSON for JSON-RPC response, mapping errors to MCPError
fn to_json_value<T: serde::Serialize>(value: &T) -> Result<serde_json::Value, MCPError> {
    serde_json::to_value(value).map_err(|e| MCPError::Serialization(e.to_string()))
}

/// Convert Task to `JsonTask` for JSON-RPC response
fn task_to_json_task(task: Task) -> JsonTask {
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

impl TaskServiceImpl {
    /// Handle JSON-RPC request - dispatches to appropriate method based on "method" field
    #[instrument(skip(self, request))]
    pub async fn handle_json_rpc_request(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .ok_or_else(|| MCPError::InvalidArgument("Missing method".to_string()))?;

        let params = request
            .get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let id = request
            .get("id")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let result = match method {
            "create_task" => self.handle_create_task(params).await,
            "get_task" => self.handle_get_task(params).await,
            "update_task" => self.handle_update_task(params).await,
            "list_tasks" => self.handle_list_tasks(params).await,
            "assign_task" => self.handle_assign_task(params).await,
            "report_progress" => self.handle_report_progress(params).await,
            "complete_task" => self.handle_complete_task(params).await,
            "cancel_task" => self.handle_cancel_task(params).await,
            _ => Err(MCPError::InvalidArgument(format!(
                "Unknown method: {method}"
            ))),
        }?;

        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }))
    }

    #[instrument(skip(self, params))]
    async fn handle_create_task(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let req: CreateTaskRequest = serde_json::from_value(params)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid create_task params: {e}")))?;

        debug!("Creating task with name: {}", req.name);

        let priority = TaskPriority::from(req.priority);
        let agent_type = AgentType::from(req.agent_type);

        let mut task = Task::new(&req.name, &req.description);
        task.priority_code = priority;
        task.agent_type = agent_type;

        if !req.input_data.is_empty() {
            task.input_data = Some(bytes_to_hashmap(&req.input_data));
        }
        if !req.metadata.is_empty() {
            task.metadata = Some(bytes_to_hashmap(&req.metadata));
        }

        let task_manager = self.task_manager.lock().await;
        match task_manager.create_task(task).await {
            Ok(created_task) => {
                info!("Task created successfully with ID: {}", created_task.id);
                to_json_value(&CreateTaskResponse {
                    success: true,
                    task_id: created_task.id.as_ref().to_string(),
                    error_message: String::new(),
                })
            }
            Err(e) => {
                error!("Failed to create task: {}", e);
                to_json_value(&CreateTaskResponse {
                    success: false,
                    task_id: String::new(),
                    error_message: e.to_string(),
                })
            }
        }
    }

    #[instrument(skip(self, params))]
    async fn handle_get_task(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let req: GetTaskRequest = serde_json::from_value(params)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid get_task params: {e}")))?;

        debug!("Getting task with ID: {}", req.task_id);

        let task_manager = self.task_manager.lock().await;
        match task_manager.get_task(&req.task_id).await {
            Ok(task) => {
                debug!("Task found: {}", task.id);
                to_json_value(&GetTaskResponse {
                    task: Some(task_to_json_task(task)),
                    success: true,
                    error_message: String::new(),
                })
            }
            Err(e) => {
                error!("Failed to get task {}: {}", req.task_id, e);
                to_json_value(&GetTaskResponse {
                    task: None,
                    success: false,
                    error_message: e.to_string(),
                })
            }
        }
    }

    #[instrument(skip(self, params))]
    async fn handle_update_task(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let req: UpdateTaskRequest = serde_json::from_value(params)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid update_task params: {e}")))?;

        debug!("Updating task with ID: {}", req.task_id);

        let task_manager = self.task_manager.lock().await;
        let current_task = match task_manager.get_task(&req.task_id).await {
            Ok(task) => task,
            Err(e) => {
                error!("Failed to get task for update {}: {}", req.task_id, e);
                return to_json_value(&UpdateTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                });
            }
        };

        let mut updated_task = current_task.clone();
        if !req.name.is_empty() {
            updated_task.name = std::sync::Arc::from(req.name);
        }
        if !req.description.is_empty() {
            updated_task.description = req.description;
        }
        if !req.input_data.is_empty() {
            updated_task.input_data = Some(bytes_to_hashmap(&req.input_data));
        }
        if !req.metadata.is_empty() {
            updated_task.metadata = Some(bytes_to_hashmap(&req.metadata));
        }
        if req.priority != 0 {
            updated_task.priority_code = TaskPriority::from(req.priority);
        }
        updated_task.updated_at = Utc::now();

        match task_manager.update_task(updated_task.clone()).await {
            Ok(_) => {
                info!("Task updated successfully: {}", req.task_id);
                to_json_value(&UpdateTaskResponse {
                    success: true,
                    error_message: String::new(),
                })
            }
            Err(e) => {
                error!("Failed to update task {}: {}", req.task_id, e);
                to_json_value(&UpdateTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                })
            }
        }
    }

    #[instrument(skip(self, params))]
    async fn handle_list_tasks(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let req: ListTasksRequest = serde_json::from_value(params)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid list_tasks params: {e}")))?;

        debug!("Listing tasks for agent: {}", req.agent_id);

        let agent_id = if req.agent_id.is_empty() {
            None
        } else {
            Some(req.agent_id.as_str())
        };

        let task_manager = self.task_manager.lock().await;
        match task_manager.list_tasks(agent_id).await {
            Ok(tasks) => {
                debug!("Found {} tasks", tasks.len());
                let total_count = tasks.len().min(i32::MAX as usize) as i32;
                let json_tasks: Vec<JsonTask> = tasks.into_iter().map(task_to_json_task).collect();
                to_json_value(&ListTasksResponse {
                    tasks: json_tasks,
                    total_count,
                    success: true,
                    error_message: String::new(),
                })
            }
            Err(e) => {
                error!("Failed to list tasks: {}", e);
                to_json_value(&ListTasksResponse {
                    tasks: Vec::new(),
                    total_count: 0,
                    success: false,
                    error_message: e.to_string(),
                })
            }
        }
    }

    #[instrument(skip(self, params))]
    async fn handle_assign_task(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let req: AssignTaskRequest = serde_json::from_value(params)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid assign_task params: {e}")))?;

        debug!("Assigning task {} to agent {}", req.task_id, req.agent_id);

        let task_manager = self.task_manager.lock().await;
        match task_manager.assign_task(&req.task_id, &req.agent_id).await {
            Ok(_) => {
                info!(
                    "Task assigned successfully: {} to {}",
                    req.task_id, req.agent_id
                );
                to_json_value(&AssignTaskResponse {
                    success: true,
                    error_message: String::new(),
                })
            }
            Err(e) => {
                error!("Failed to assign task {}: {}", req.task_id, e);
                to_json_value(&AssignTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                })
            }
        }
    }

    #[instrument(skip(self, params))]
    async fn handle_report_progress(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let req: ReportProgressRequest = serde_json::from_value(params).map_err(|e| {
            MCPError::InvalidArgument(format!("Invalid report_progress params: {e}"))
        })?;

        info!("Reporting progress for task: {}", req.task_id);

        let task_manager = self.task_manager.lock().await;
        match task_manager
            .update_task_progress(
                &req.task_id,
                req.progress_percent as f32,
                &req.progress_message,
            )
            .await
        {
            Ok(_) => {
                info!("Progress updated for task: {}", req.task_id);
                to_json_value(&ReportProgressResponse {
                    success: true,
                    error_message: String::new(),
                })
            }
            Err(e) => {
                error!("Failed to update progress for task {}: {}", req.task_id, e);
                to_json_value(&ReportProgressResponse {
                    success: false,
                    error_message: format!("Failed to update progress: {e}"),
                })
            }
        }
    }

    #[instrument(skip(self, params))]
    async fn handle_complete_task(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let req: CompleteTaskRequest = serde_json::from_value(params)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid complete_task params: {e}")))?;

        debug!("Completing task: {}", req.task_id);

        let output_data = if req.output_data.is_empty() {
            None
        } else {
            Some(bytes_to_hashmap(&req.output_data))
        };

        let task_manager = self.task_manager.lock().await;
        match task_manager.complete_task(&req.task_id, output_data).await {
            Ok(_) => {
                info!("Task completed successfully: {}", req.task_id);
                to_json_value(&CompleteTaskResponse {
                    success: true,
                    error_message: String::new(),
                })
            }
            Err(e) => {
                error!("Failed to complete task {}: {}", req.task_id, e);
                to_json_value(&CompleteTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                })
            }
        }
    }

    #[instrument(skip(self, params))]
    async fn handle_cancel_task(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let req: CancelTaskRequest = serde_json::from_value(params)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid cancel_task params: {e}")))?;

        debug!("Cancelling task: {}", req.task_id);

        let task_manager = self.task_manager.lock().await;
        match task_manager.cancel_task(&req.task_id, &req.reason).await {
            Ok(_) => {
                info!("Task cancelled successfully: {}", req.task_id);
                to_json_value(&CancelTaskResponse {
                    success: true,
                    error_message: String::new(),
                })
            }
            Err(e) => {
                error!("Failed to cancel task {}: {}", req.task_id, e);
                to_json_value(&CancelTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::task::manager::TaskManager;
    use crate::task::server::service::TaskServiceImpl;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn create_task_params(name: &str) -> serde_json::Value {
        json!({
            "name": name,
            "description": "desc",
            "priority": 1,
            "input_data": serde_json::to_vec(&json!({"k": "v"})).expect("should succeed"),
            "metadata": [],
            "prerequisite_task_ids": [],
            "context_id": "",
            "agent_id": "",
            "agent_type": 0
        })
    }

    fn svc() -> TaskServiceImpl {
        TaskServiceImpl::create_server(Arc::new(Mutex::new(TaskManager::new())))
    }

    #[tokio::test]
    async fn handle_json_rpc_missing_method_returns_invalid_argument() {
        let svc = svc();
        let err = svc
            .handle_json_rpc_request(json!({"jsonrpc": "2.0", "id": 1}))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("method") || err.to_string().contains("Missing"));
    }

    #[tokio::test]
    async fn handle_json_rpc_unknown_method_returns_invalid_argument() {
        let svc = svc();
        let err = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "not_a_real_method",
                "params": null
            }))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Unknown"));
    }

    #[tokio::test]
    async fn handle_create_task_invalid_params_returns_mcp_error() {
        let svc = svc();
        let err = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "create_task",
                "params": "not-an-object"
            }))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Invalid create_task params"));
    }

    #[tokio::test]
    #[expect(
        clippy::too_many_lines,
        reason = "Exhaustive table or handler; splitting hurts clarity"
    )]
    async fn task_json_rpc_create_get_list_update_assign_progress_complete_and_cancel() {
        let svc = svc();
        let created = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "create_task",
                "params": create_task_params("lifecycle")
            }))
            .await
            .expect("should succeed");
        let r = created["result"].clone();
        assert_eq!(r["success"], true);
        let task_id = r["task_id"].as_str().expect("should succeed").to_string();

        let got = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "get_task",
                "params": { "task_id": task_id }
            }))
            .await
            .expect("should succeed");
        assert_eq!(got["result"]["success"], true);
        assert!(got["result"]["task"].is_object());

        let listed = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "list_tasks",
                "params": {
                    "status": 0,
                    "agent_id": "",
                    "agent_type": 0,
                    "context_id": "",
                    "limit": 100,
                    "offset": 0
                }
            }))
            .await
            .expect("should succeed");
        assert_eq!(listed["result"]["success"], true);
        assert!(
            listed["result"]["total_count"]
                .as_i64()
                .expect("should succeed")
                >= 1
        );

        let updated = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "update_task",
                "params": {
                    "task_id": task_id,
                    "name": "lifecycle2",
                    "description": "d2",
                    "priority": 2,
                    "input_data": [],
                    "metadata": []
                }
            }))
            .await
            .expect("should succeed");
        assert_eq!(updated["result"]["success"], true);

        let assigned = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "assign_task",
                "params": { "task_id": task_id, "agent_id": "agent-1", "agent_type": 0 }
            }))
            .await
            .expect("should succeed");
        assert_eq!(assigned["result"]["success"], true);

        let prog = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 6,
                "method": "report_progress",
                "params": {
                    "task_id": task_id,
                    "progress_percent": 50,
                    "progress_message": "half",
                    "interim_results": []
                }
            }))
            .await
            .expect("should succeed");
        assert_eq!(prog["result"]["success"], true);

        let done = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "complete_task",
                "params": { "task_id": task_id, "output_data": [], "metadata": [] }
            }))
            .await
            .expect("should succeed");
        assert_eq!(done["result"]["success"], true);

        let t2 = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 8,
                "method": "create_task",
                "params": create_task_params("to-cancel")
            }))
            .await
            .expect("should succeed");
        let id2 = t2["result"]["task_id"].as_str().expect("should succeed");
        let cancelled = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 9,
                "method": "cancel_task",
                "params": { "task_id": id2, "reason": "test" }
            }))
            .await
            .expect("should succeed");
        assert_eq!(cancelled["result"]["success"], true);

        let missing = svc
            .handle_json_rpc_request(json!({
                "jsonrpc": "2.0",
                "id": 10,
                "method": "get_task",
                "params": { "task_id": "00000000-0000-0000-0000-000000000000" }
            }))
            .await
            .expect("should succeed");
        assert_eq!(missing["result"]["success"], false);
    }
}
