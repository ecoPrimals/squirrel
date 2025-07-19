//! gRPC Service Handler Implementations
//!
//! This module contains all the gRPC service method implementations for the TaskService.

use chrono::Utc;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, instrument, warn};
use std::collections::HashMap;
use serde_json;

use crate::generated::mcp_task::{
    task_service_server::TaskService,
    AssignTaskRequest, AssignTaskResponse,
    CancelTaskRequest, CancelTaskResponse,
    CompleteTaskRequest, CompleteTaskResponse,
    CreateTaskRequest, CreateTaskResponse,
    GetTaskRequest, GetTaskResponse,
    ListTasksRequest, ListTasksResponse,
    ReportProgressRequest, ReportProgressResponse,
    Task as GenTask,
    UpdateTaskRequest, UpdateTaskResponse,
};
use crate::task::types::{Task, TaskPriority, AgentType};
use crate::task::server::service::TaskServiceImpl;

/// Convert bytes to HashMap<String, String> if not empty
fn bytes_to_hashmap(data: &[u8]) -> HashMap<String, String> {
    if data.is_empty() {
        return HashMap::new();
    }
    
    match serde_json::from_slice::<HashMap<String, serde_json::Value>>(data) {
        Ok(map) => {
            map.into_iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect()
        }
        Err(e) => {
            warn!("Failed to parse JSON data: {}", e);
            HashMap::new()
        }
    }
}

/// Convert HashMap<String, String> to bytes
fn hashmap_to_bytes(map: &Option<HashMap<String, String>>) -> Vec<u8> {
    match map {
        Some(m) => serde_json::to_vec(m).unwrap_or_default(),
        None => Vec::new(),
    }
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    #[instrument(skip(self, request))]
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> std::result::Result<Response<CreateTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Creating task with name: {}", req.name);

        let priority = TaskPriority::from(req.priority);
        let agent_type = AgentType::from(req.agent_type);

        let mut task = Task::new(&req.name, &req.description);
        task.priority_code = priority;
        task.agent_type = agent_type;
        
        // Set input data if provided
        if !req.input_data.is_empty() {
            task.input_data = Some(bytes_to_hashmap(&req.input_data));
        }
        
        // Set metadata if provided
        if !req.metadata.is_empty() {
            task.metadata = Some(bytes_to_hashmap(&req.metadata));
        }

        let task_id = task.id.clone();

        let task_manager = self.task_manager.lock().await;
        match task_manager.create_task(task).await {
            Ok(created_task) => {
                info!("Task created successfully with ID: {}", task_id);
                let response = CreateTaskResponse {
                    success: true,
                    task_id: created_task.id,
                    error_message: String::new(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Failed to create task: {}", e);
                let response = CreateTaskResponse {
                    success: false,
                    task_id: String::new(),
                    error_message: e.to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> std::result::Result<Response<GetTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Getting task with ID: {}", req.task_id);

        let task_manager = self.task_manager.lock().await;
        match task_manager.get_task(&req.task_id).await {
            Ok(task) => {
                debug!("Task found: {}", task.id);
                let response = GetTaskResponse {
                    task: Some(task.into()),
                    success: true,
                    error_message: String::new(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Failed to get task {}: {}", req.task_id, e);
                let response = GetTaskResponse {
                    task: None,
                    success: false,
                    error_message: e.to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn update_task(
        &self,
        request: Request<UpdateTaskRequest>,
    ) -> std::result::Result<Response<UpdateTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Updating task with ID: {}", req.task_id);

        let task_manager = self.task_manager.lock().await;
        
        // Get the current task first
        let current_task = match task_manager.get_task(&req.task_id).await {
            Ok(task) => task,
            Err(e) => {
                error!("Failed to get task for update {}: {}", req.task_id, e);
                let response = UpdateTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                };
                return Ok(Response::new(response));
            }
        };

        // Create updated task with new values
        let mut updated_task = current_task.clone();
        
        if !req.name.is_empty() {
            updated_task.name = req.name;
        }
        if !req.description.is_empty() {
            updated_task.description = req.description;
        }
        
        // Set input data if provided
        if !req.input_data.is_empty() {
            updated_task.input_data = Some(bytes_to_hashmap(&req.input_data));
        }
        
        // Set metadata if provided
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
                
                let response = UpdateTaskResponse {
                    success: true,
                    error_message: String::new(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Failed to update task {}: {}", req.task_id, e);
                let response = UpdateTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn list_tasks(
        &self,
        request: Request<ListTasksRequest>,
    ) -> std::result::Result<Response<ListTasksResponse>, Status> {
        let req = request.into_inner();
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
                let total_count = tasks.len() as i32;
                let gen_tasks: Vec<GenTask> = tasks.into_iter().map(|t| t.into()).collect();
                let response = ListTasksResponse {
                    tasks: gen_tasks,
                    total_count,
                    success: true,
                    error_message: String::new(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Failed to list tasks: {}", e);
                let response = ListTasksResponse {
                    tasks: Vec::new(),
                    total_count: 0,
                    success: false,
                    error_message: e.to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn assign_task(
        &self,
        request: Request<AssignTaskRequest>,
    ) -> std::result::Result<Response<AssignTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Assigning task {} to agent {}", req.task_id, req.agent_id);

        let task_manager = self.task_manager.lock().await;
        match task_manager.assign_task(&req.task_id, &req.agent_id).await {
            Ok(_) => {
                info!("Task assigned successfully: {} to {}", req.task_id, req.agent_id);
                
                let response = AssignTaskResponse {
                    success: true,
                    error_message: String::new(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Failed to assign task {}: {}", req.task_id, e);
                let response = AssignTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn report_progress(
        &self,
        request: Request<ReportProgressRequest>,
    ) -> Result<Response<ReportProgressResponse>, Status> {
        let req = request.into_inner();
        
        info!("Reporting progress for task: {}", req.task_id);
        
        let task_manager = self.task_manager.lock().await;
        match task_manager.update_task_progress(&req.task_id, req.progress_percent as f32, &req.progress_message).await {
            Ok(_) => {
                info!("Progress updated for task: {}", req.task_id);
                Ok(Response::new(ReportProgressResponse {
                    success: true,
                    error_message: String::new(),
                }))
            }
            Err(e) => {
                error!("Failed to update progress for task {}: {}", req.task_id, e);
                Ok(Response::new(ReportProgressResponse {
                    success: false,
                    error_message: format!("Failed to update progress: {}", e),
                }))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn complete_task(
        &self,
        request: Request<CompleteTaskRequest>,
    ) -> std::result::Result<Response<CompleteTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Completing task: {}", req.task_id);

        // Convert bytes to HashMap if not empty
        let output_data = if req.output_data.is_empty() {
            None
        } else {
            Some(bytes_to_hashmap(&req.output_data))
        };

        let task_manager = self.task_manager.lock().await;
        match task_manager.complete_task(&req.task_id, output_data).await {
            Ok(_) => {
                info!("Task completed successfully: {}", req.task_id);
                
                let response = CompleteTaskResponse {
                    success: true,
                    error_message: String::new(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Failed to complete task {}: {}", req.task_id, e);
                let response = CompleteTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> std::result::Result<Response<CancelTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Cancelling task: {}", req.task_id);

        let task_manager = self.task_manager.lock().await;
        match task_manager.cancel_task(&req.task_id, &req.reason).await {
            Ok(_) => {
                info!("Task cancelled successfully: {}", req.task_id);
                
                let response = CancelTaskResponse {
                    success: true,
                    error_message: String::new(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Failed to cancel task {}: {}", req.task_id, e);
                let response = CancelTaskResponse {
                    success: false,
                    error_message: e.to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }
} 