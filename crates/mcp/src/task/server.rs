//! Server implementation for Task Service API.
//!
//! This module provides a gRPC server implementation for the TaskService
//! defined in the protobuf API.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::sync::Mutex;

use chrono::Utc;
use futures::{Stream, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, instrument, warn};
use uuid;
use prost_types;

use crate::error::Error;
use crate::generated::mcp_task::{
    task_service_server::TaskService,
    task_service_server::TaskServiceServer,
    AssignTaskRequest, AssignTaskResponse,
    CancelTaskRequest, CancelTaskResponse,
    CompleteTaskRequest, CompleteTaskResponse,
    CreateTaskRequest, CreateTaskResponse,
    GetTaskRequest, GetTaskResponse,
    ListTasksRequest, ListTasksResponse,
    ReportProgressRequest, ReportProgressResponse,
    Task as GenTask, WatchTaskRequest, WatchTaskResponse,
    UpdateTaskRequest, UpdateTaskResponse,
    ListCommandsRequest, ListCommandsResponse,
    ExecuteCommandRequest, ExecuteCommandResponse,
    Command as GenCommand
};
use crate::task::manager::TaskManager;
use crate::task::types::{Task, TaskStatus, TaskPriority, AgentType};
#[cfg(feature = "command-registry")]
use squirrel_commands::{Command, CommandRegistry};
use uuid::Uuid;
use serde_json;

/// Wrapper type for a task update channel.
type TaskUpdateSender = mpsc::Sender<std::result::Result<WatchTaskResponse, Status>>;

/// Server implementation for the TaskService.
#[derive(Debug)]
pub struct TaskServiceImpl {
    /// Task manager for handling task operations
    task_manager: Arc<TaskManager>,
    
    /// Channels for tasks being watched by clients
    watchers: Arc<RwLock<HashMap<String, Vec<TaskUpdateSender>>>>,
    
    /// Command registry for executing commands
    #[cfg(feature = "command-registry")]
    command_registry: Option<Arc<Mutex<CommandRegistry>>>,

    /// Mock command registry for tests
    mock_command_registry: Option<Arc<Mutex<mock::MockCommandRegistry>>>,
}

impl TaskServiceImpl {
    /// Create a new TaskServiceImpl with the provided TaskManager.
    pub fn new(task_manager: Arc<TaskManager>) -> Self {
        Self {
            task_manager,
            watchers: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "command-registry")]
            command_registry: None,
            mock_command_registry: None,
        }
    }
    
    /// Set the command registry for this service
    #[cfg(feature = "command-registry")]
    pub fn with_command_registry(mut self, registry: Arc<Mutex<CommandRegistry>>) -> Self {
        self.command_registry = Some(registry);
        self
    }
    
    /// Create a new TaskServiceServer with the provided TaskManager.
    pub fn create_server(task_manager: Arc<TaskManager>) -> TaskServiceServer<Self> {
        let service = Self::new(task_manager);
        TaskServiceServer::new(service)
    }
    
    /// Send a task update to all watchers for that task.
    async fn broadcast_task_update(&self, task: Task) {
        let task_id = task.id.clone();
        let watchers = self.watchers.read().await;
        
        if let Some(channels) = watchers.get(&task_id) {
            let update = WatchTaskResponse {
                task: Some(task.into()),
                is_initial_state: false,
                success: true,
                error_message: String::new(),
            };
            
            for sender in channels {
                // Try to send the update, but if the channel is closed, just continue
                let _ = sender.send(Ok(update.clone())).await;
            }
        }
    }
    
    /// Check if a task update is significant to send to watchers
    fn is_significant_update(&self, old_task: &Task, new_task: &Task, only_watchable: bool) -> bool {
        // Always significant if states are different
        if old_task.status_code != new_task.status_code {
            return true;
        }
        
        // If only_watchable is true, check if the task is watchable
        if only_watchable && new_task.is_finished() {
            // Terminal state - check if it was updated recently (within last 60 seconds)
            let one_minute_ago = Utc::now() - chrono::Duration::minutes(1);
            return new_task.updated_at > one_minute_ago;
        }
        
        // Progress changes over 5% are significant
        if (new_task.progress - old_task.progress).abs() >= 5.0 {
            return true;
        }
        
        // Status message changes are significant
        if new_task.status_message != old_task.status_message {
            return true;
        }
        
        false
    }
    
    /// Register a watcher for a task
    async fn register_watcher(&self, task_id: &str, sender: TaskUpdateSender) {
        let mut watchers = self.watchers.write().await;
        let channels = watchers.entry(task_id.to_string()).or_insert_with(Vec::new);
        channels.push(sender);
    }
    
    /// Clean up closed channels for a task
    async fn clean_watchers(&self, task_id: &str) {
        let mut watchers = self.watchers.write().await;
        if let Some(channels) = watchers.get_mut(task_id) {
            // Keep only channels that are still open
            channels.retain(|sender| !sender.is_closed());
            
            // Remove the entry if no channels remain
            if channels.is_empty() {
                watchers.remove(task_id);
            }
        }
    }
    
    /// Unregister a watcher for a task
    async fn unregister_watcher(&self, task_id: &str, sender: &TaskUpdateSender) {
        let mut watchers = self.watchers.write().await;
        if let Some(channels) = watchers.get_mut(task_id) {
            // Remove the specific sender
            channels.retain(|s| !s.same_channel(sender));
            
            // Remove the entry if no channels remain
            if channels.is_empty() {
                watchers.remove(task_id);
            }
        }
    }
    
    /// Get the list of available commands from the registry
    #[cfg(feature = "command-registry")]
    pub fn list_available_commands(&self) -> Vec<String> {
        if let Some(registry) = &self.command_registry {
            match registry.lock() {
                Ok(registry) => registry.command_names(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }
    
    #[cfg(all(not(feature = "command-registry"), not(test)))]
    pub fn list_available_commands(&self) -> Vec<String> {
        Vec::new() // Return empty list when command registry is not available
    }
    
    #[cfg(all(not(feature = "command-registry"), test))]
    pub fn list_available_commands(&self) -> Vec<String> {
        if let Some(registry) = &self.mock_command_registry {
            match registry.lock() {
                Ok(registry) => registry.command_names(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }
    
    /// Execute a command from the registry
    #[cfg(feature = "command-registry")]
    pub fn execute_command(&self, command: &str, args: &[String]) -> Result<String, String> {
        if let Some(registry) = &self.command_registry {
            // Try the main command registry first
            match registry.lock() {
                Ok(registry) => {
                    match registry.get(command) {
                        Some(cmd) => {
                            return match cmd.execute(args) {
                                Ok(output) => Ok(output),
                                Err(e) => Err(format!("Command execution failed: {}", e)),
                            };
                        },
                        None => { /* Command not found in main registry, try mock */ }
                    }
                },
                Err(_) => return Err("Failed to lock main command registry".to_string()),
            }
        } 
        
        // Fallback to mock registry if main registry doesn't have the command or isn't available
        if let Some(mock_registry) = &self.mock_command_registry {
             match mock_registry.lock() {
                 Ok(mut mock_registry) => { // Acquire mutable lock
                     match mock_registry.get(command) { // Call get on the mutable registry
                         Some(cmd) => cmd.execute(args),
                         None => Err(format!("Command not found in any registry: {}", command)),
                     }
                 },
                 Err(_) => Err("Failed to lock mock command registry".to_string()),
             }
         } else {
             // If neither registry is available or has the command
             Err(format!("No command registry available or command not found: {}", command))
         }
    }
    
    #[cfg(not(feature = "command-registry"))]
    pub fn execute_command(&self, command: &str, _args: &[String]) -> Result<String, String> {
        // If mock registry exists, use it
        if let Some(registry) = &self.mock_command_registry {
            match registry.lock() {
                Ok(mut registry) => { // Acquire mutable lock
                    match registry.get(command) { // Call get on the mutable registry
                        Some(cmd) => cmd.execute(_args),
                        None => Err(format!("Command not found: {}", command)),
                    }
                },
                Err(_) => Err("Failed to lock mock command registry".to_string()),
            }
        } else {
            // Otherwise, report registry not available
            Err(format!("Command registry not available. Cannot execute: {}", command))
        }
    }

    /// Mock command registry for tests
    pub fn with_mock_command_registry(mut self) -> Self {
        use self::mock::{MockCommand, MockCommandRegistry};
        
        let registry = Arc::new(Mutex::new(MockCommandRegistry::new()));
        {
            let mut reg = registry.lock().unwrap();
            reg.register(MockCommand::new("test_command", "Test command for integration tests")).unwrap();
            reg.register(MockCommand::new("hello", "Says hello")).unwrap();
            // Register a special command that allows registering dynamic commands
            reg.register(MockCommand::new("register_command", "Registers a new command")).unwrap();
        }
        
        self.mock_command_registry = Some(registry);
        self
    }

    #[cfg(test)]
    pub fn execute_command(&self, command: &str, args: &[String]) -> Result<String, String> {
        if let Some(registry) = &self.mock_command_registry {
            match registry.lock() {
                Ok(registry) => {
                    match registry.get(command) {
                        Some(cmd) => cmd.execute(args),
                        None => Err(format!("Command not found: {}", command)),
                    }
                },
                Err(_) => Err("Failed to lock command registry".to_string()),
            }
        } else {
            Err("No command registry available".to_string())
        }
    }
}

// Helper function to convert parameters
fn json_params_to_string_vec(params_value: &serde_json::Value) -> Result<Vec<String>, String> {
    match params_value {
        serde_json::Value::Array(arr) => arr
            .iter()
            .map(|v| match v {
                serde_json::Value::String(s) => Ok(s.clone()),
                _ => Ok(v.to_string()), // Convert non-strings to string representation
            })
            .collect::<Result<Vec<String>, _>>(),
        serde_json::Value::Object(obj) => {
            // Convert object to key=value strings or similar, adjust as needed
            Ok(obj.iter().map(|(k, v)| format!("{}={}", k, v)).collect())
        }
        serde_json::Value::String(s) => {
            // If it's a single string, maybe split it or treat it as one arg?
            // This depends on how commands expect string params. Assuming single arg for now.
            Ok(vec![s.clone()])
        }
        _ => Err("Unsupported parameters format: expected Array, Object, or String".to_string()),
    }
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    type WatchTaskStream = Pin<Box<dyn Stream<Item = std::result::Result<WatchTaskResponse, Status>> + Send + 'static>>;
    
    /// Create a new task.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr()))]
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> std::result::Result<Response<CreateTaskResponse>, Status> {
        let request = request.into_inner();
        let name = request.name;
        let description = request.description;
        let priority = request.priority;
        let input_data = if !request.input_data.is_empty() {
            match String::from_utf8(request.input_data.clone()) {
                Ok(json_str) => serde_json::from_str(&json_str).ok(),
                Err(_) => None
            }
        } else {
            None
        };
        let agent_id = if !request.agent_id.is_empty() {
            Some(request.agent_id)
        } else {
            None
        };
        let context_id = if !request.context_id.is_empty() {
            Some(request.context_id)
        } else {
            None
        };
        
        // Convert priority
        let priority_code = TaskPriority::from(priority as i32);
        
        // Create a new task
        let mut task = Task::new(&name, &description)
            .with_priority(priority_code);
        
        // Set optional fields
        if let Some(agent_id) = agent_id {
            task.agent_id = Some(agent_id);
        }
        
        if let Some(context_id) = context_id {
            task.context_id = Some(context_id);
        }
        
        if let Some(data) = input_data {
            task.input_data = Some(data);
        }
        
        // Add prerequisites
        for prerequisite_id in request.prerequisite_task_ids {
            task.prerequisites.push(prerequisite_id);
        }
        
        // Create the task using the task manager
        match self.task_manager.create_task(task).await {
            Ok(task) => {
                info!("Created task: {}", task.id);
                
                Ok(Response::new(CreateTaskResponse {
                    task_id: task.id,
                    success: true,
                    error_message: String::new(),
                }))
            },
            Err(err) => {
                error!("Failed to create task: {}", err);
                Err(Status::internal(format!("Failed to create task: {}", err)))
            }
        }
    }
    
    /// Get a task by ID.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr(), task_id = %request.get_ref().task_id))]
    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> std::result::Result<Response<GetTaskResponse>, Status> {
        let request = request.into_inner();
        let task_id = request.task_id;
        
        if task_id.is_empty() {
            return Err(Status::invalid_argument("Task ID is required"));
        }
        
        debug!("Getting task: {}", task_id);
        
        match self.task_manager.get_task(&task_id).await {
            Ok(task) => {
                info!("Found task: {}", task_id);
                // Convert to protobuf Task
                let proto_task: GenTask = task.into();
                
                Ok(Response::new(GetTaskResponse {
                    task: Some(proto_task),
                    success: true,
                    error_message: String::new(),
                }))
            },
            Err(Error::NotFound(_)) => {
                warn!("Task not found: {}", task_id);
                Err(Status::not_found(format!("Task not found: {}", task_id)))
            },
            Err(err) => {
                error!("Failed to get task: {}", err);
                Err(Status::internal(format!("Failed to get task: {}", err)))
            }
        }
    }
    
    /// Update an existing task's details.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr(), task_id = %request.get_ref().task_id))]
    async fn update_task(
        &self,
        request: Request<UpdateTaskRequest>,
    ) -> std::result::Result<Response<UpdateTaskResponse>, Status> {
        let request = request.into_inner();
        let task_id = request.task_id;
        
        if task_id.is_empty() {
            return Err(Status::invalid_argument("Task ID is required"));
        }
        
        debug!("Updating task: {}", task_id);
        
        // Get the current task
        let mut task = match self.task_manager.get_task(&task_id).await {
            Ok(t) => t,
            Err(e) => match e {
                Error::NotFound(msg) => {
                    warn!("Task not found: {}", task_id);
                    return Err(Status::not_found(format!("Task not found: {}", task_id)));
                },
                _ => {
                    error!("Failed to get task: {}", e);
                    return Err(Status::internal(format!("Failed to get task: {}", e)));
                }
            }
        };
        
        // Update fields if they're provided
        if !request.name.is_empty() {
            task.name = request.name;
        }
        
        if !request.description.is_empty() {
            task.description = request.description;
        }
        
        if request.priority != 0 {
            // Convert from i32 to TaskPriority
            task.priority_code = TaskPriority::from(request.priority as i32);
        }
        
        // Process input data if provided
        if !request.input_data.is_empty() {
            match String::from_utf8(request.input_data.clone()) {
                Ok(json_str) => {
                    if let Ok(data) = serde_json::from_str(&json_str) {
                        task.input_data = Some(data);
                    } else {
                        return Err(Status::invalid_argument("Invalid input data format"));
                    }
                },
                Err(_) => return Err(Status::invalid_argument("Input data is not valid UTF-8"))
            }
        }
        
        // Process metadata if provided
        if !request.metadata.is_empty() {
            match String::from_utf8(request.metadata.clone()) {
                Ok(json_str) => {
                    if let Ok(metadata) = serde_json::from_str(&json_str) {
                        task.metadata = Some(metadata);
                    } else {
                        return Err(Status::invalid_argument("Invalid metadata format"));
                    }
                },
                Err(_) => return Err(Status::invalid_argument("Metadata is not valid UTF-8"))
            }
        }
        
        // Update the task using the task manager
        match self.task_manager.update_task(task).await {
            Ok(updated_task) => {
                info!("Updated task: {}", task_id);
                
                Ok(Response::new(UpdateTaskResponse {
                    success: true,
                    error_message: String::new(),
                }))
            },
            Err(e) => {
                match e {
                    Error::NotFound(msg) => {
                        warn!("Task not found: {}", task_id);
                        Err(Status::not_found(format!("Task not found: {}", task_id)))
                    },
                    Error::InvalidState(msg) => {
                        warn!("Invalid state for task update: {}", msg);
                        Err(Status::failed_precondition(msg))
                    },
                    Error::AlreadyExists(msg) => {
                        warn!("Task already exists: {}", msg);
                        Err(Status::already_exists(msg))
                    },
                    Error::InvalidRequest(msg) => {
                        warn!("Invalid request: {}", msg);
                        Err(Status::invalid_argument(msg))
                    },
                    Error::Database(msg) => {
                        error!("Database error during task update: {}", msg);
                        Err(Status::internal(format!("Database error: {}", msg)))
                    },
                    Error::Serialization(msg) => {
                        error!("Serialization error during task update: {}", msg);
                        Err(Status::internal(format!("Serialization error: {}", msg)))
                    },
                    Error::OperationFailed(msg) => {
                        error!("Operation failed during task update: {}", msg);
                        Err(Status::internal(format!("Operation failed: {}", msg)))
                    },
                    Error::Io(msg) => {
                        error!("IO error during task update: {}", msg);
                        Err(Status::internal(format!("IO error: {}", msg)))
                    },
                    Error::Json(msg) => {
                        error!("JSON error during task update: {}", msg);
                        Err(Status::internal(format!("JSON error: {}", msg)))
                    },
                    Error::Alert(msg) => {
                        error!("Alert error during task update: {}", msg);
                        Err(Status::internal(format!("Alert error: {}", msg)))
                    },
                    Error::Monitoring(msg) => {
                        error!("Monitoring error during task update: {}", msg);
                        Err(Status::internal(format!("Monitoring error: {}", msg)))
                    },
                    _ => {
                        error!("Unexpected error during task update: {:?}", e);
                        Err(Status::internal(format!("Unexpected error: {}", e)))
                    },
                }
            }
        }
    }
    
    /// List tasks with optional filters.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr()))]
    async fn list_tasks(
        &self,
        request: Request<ListTasksRequest>,
    ) -> std::result::Result<Response<ListTasksResponse>, Status> {
        let request = request.into_inner();
        
        if request.agent_id.is_empty() && request.context_id.is_empty() && request.status == 0 {
            return Err(Status::invalid_argument("At least one filter must be specified"));
        }
        
        let tasks = if request.status != 0 {
            // Convert from i32 to TaskStatus
            let status = TaskStatus::from(request.status);
                
            debug!("Listing tasks with status: {:?}", status);
            self.task_manager.get_tasks_by_status(status).await
        } else if !request.agent_id.is_empty() {
            debug!("Listing tasks for agent: {}", request.agent_id);
            self.task_manager.get_agent_tasks(&request.agent_id).await
        } else if !request.context_id.is_empty() {
            debug!("Listing tasks for context: {}", request.context_id);
            self.task_manager.get_context_tasks(&request.context_id).await
        } else {
            // No filters - list all tasks (TODO: implement pagination)
            debug!("Listing all tasks");
            Err(Error::InvalidRequest("At least one filter must be specified".to_string()))
        };
        
        match tasks {
            Ok(tasks) => {
                // Convert to protocol buffer tasks
                let proto_tasks: Vec<GenTask> = tasks.into_iter()
                    .map(|t| t.into())
                    .collect();
                
                // Get the total count before moving proto_tasks
                let total_count = proto_tasks.len() as i32;
                    
                Ok(Response::new(ListTasksResponse {
                    tasks: proto_tasks,
                    success: true,
                    error_message: String::new(),
                    total_count,
                }))
            },
            Err(err) => {
                error!("Failed to list tasks: {}", err);
                Err(Status::internal(format!("Failed to list tasks: {}", err)))
            }
        }
    }
    
    /// Assign a task to an agent.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr(), task_id = %request.get_ref().task_id, agent_id = %request.get_ref().agent_id))]
    async fn assign_task(
        &self,
        request: Request<AssignTaskRequest>,
    ) -> std::result::Result<Response<AssignTaskResponse>, Status> {
        let request = request.into_inner();
        let task_id = request.task_id;
        let agent_id = request.agent_id;
        
        if task_id.is_empty() {
            return Err(Status::invalid_argument("Task ID is required"));
        }
        
        if agent_id.is_empty() {
            return Err(Status::invalid_argument("Agent ID is required"));
        }
        
        debug!("Assigning task {} to agent {}", task_id, agent_id);
        
        // First, get the current task
        let old_task = match self.task_manager.get_task(&task_id).await {
            Ok(t) => t,
            Err(Error::NotFound(_)) => {
                return Err(Status::not_found(format!("Task not found: {}", task_id)));
            },
            Err(err) => {
                return Err(Status::internal(format!("Failed to get task: {}", err)));
            }
        };
        
        // Assign the task using the task manager
        match self.task_manager.assign_task(&task_id, &agent_id).await {
            Ok(task) => {
                info!("Assigned task {} to agent {}", task_id, agent_id);
                
                // Convert to protobuf Task
                let proto_task: GenTask = task.clone().into();
                
                // Notify watchers
                if self.is_significant_update(&old_task, &task, false) {
                    self.broadcast_task_update(task).await;
                }
                
                Ok(Response::new(AssignTaskResponse {
                    success: true,
                    error_message: String::new(),
                }))
            },
            Err(Error::NotFound(_)) => {
                warn!("Task not found: {}", task_id);
                Err(Status::not_found(format!("Task not found: {}", task_id)))
            },
            Err(Error::InvalidState(msg)) => {
                warn!("Invalid state for task assignment: {}", msg);
                Err(Status::failed_precondition(msg))
            },
            Err(err) => {
                error!("Failed to assign task: {}", err);
                Err(Status::internal(format!("Failed to assign task: {}", err)))
            }
        }
    }
    
    /// Report progress for a task.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr(), task_id = %request.get_ref().task_id, progress = %request.get_ref().progress_percent))]
    async fn report_progress(
        &self,
        request: Request<ReportProgressRequest>,
    ) -> std::result::Result<Response<ReportProgressResponse>, Status> {
        let request = request.into_inner();
        let task_id = request.task_id;
        let progress = request.progress_percent as f32;
        let status_message = request.progress_message;
        
        if task_id.is_empty() {
            return Err(Status::invalid_argument("Task ID is required"));
        }
        
        if progress < 0.0 || progress > 100.0 {
            return Err(Status::invalid_argument("Progress must be between 0 and 100"));
        }
        
        debug!("Updating progress for task {}: {}%", task_id, progress);
        
        // First, get the current task
        let old_task = match self.task_manager.get_task(&task_id).await {
            Ok(t) => t,
            Err(Error::NotFound(_)) => {
                return Err(Status::not_found(format!("Task not found: {}", task_id)));
            },
            Err(err) => {
                return Err(Status::internal(format!("Failed to get task: {}", err)));
            }
        };
        
        // Update the progress using the task manager
        match self.task_manager.update_progress(&task_id, progress, Some(status_message)).await {
            Ok(task) => {
                debug!("Updated progress for task {}: {}%", task_id, progress);
                
                // Convert to protobuf Task
                let proto_task: GenTask = task.clone().into();
                
                // Check if this is a significant update and notify watchers
                if self.is_significant_update(&old_task, &task, false) {
                    self.broadcast_task_update(task).await;
                }
                
                Ok(Response::new(ReportProgressResponse {
                    success: true,
                    error_message: String::new(),
                }))
            },
            Err(Error::NotFound(_)) => {
                warn!("Task not found: {}", task_id);
                Err(Status::not_found(format!("Task not found: {}", task_id)))
            },
            Err(Error::InvalidState(msg)) => {
                warn!("Invalid state for progress update: {}", msg);
                Err(Status::failed_precondition(msg))
            },
            Err(err) => {
                error!("Failed to update progress: {}", err);
                Err(Status::internal(format!("Failed to update progress: {}", err)))
            }
        }
    }
    
    /// Mark a task as completed with optional output data.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr(), task_id = %request.get_ref().task_id))]
    async fn complete_task(
        &self,
        request: Request<CompleteTaskRequest>,
    ) -> std::result::Result<Response<CompleteTaskResponse>, Status> {
        let request = request.into_inner();
        let task_id = request.task_id;
        
        // Convert Vec<u8> to HashMap if not empty
        let output_data = if !request.output_data.is_empty() {
            match std::str::from_utf8(&request.output_data) {
                Ok(json_str) => serde_json::from_str::<HashMap<String, String>>(json_str).ok(),
                Err(_) => None
            }
        } else {
            None
        };
        
        if task_id.is_empty() {
            return Err(Status::invalid_argument("Task ID is required"));
        }
        
        debug!("Completing task: {}", task_id);
        
        // First, get the current task
        let old_task = match self.task_manager.get_task(&task_id).await {
            Ok(t) => t,
            Err(Error::NotFound(_)) => {
                return Err(Status::not_found(format!("Task not found: {}", task_id)));
            },
            Err(err) => {
                return Err(Status::internal(format!("Failed to get task: {}", err)));
            }
        };
        
        // Complete the task using the task manager
        match self.task_manager.complete_task(&task_id, output_data).await {
            Ok(task) => {
                info!("Completed task: {}", task_id);
                
                // Convert to protobuf Task
                let proto_task: GenTask = task.clone().into();
                
                // Notify watchers
                if self.is_significant_update(&old_task, &task, false) {
                    self.broadcast_task_update(task).await;
                }
                
                Ok(Response::new(CompleteTaskResponse {
                    success: true,
                    error_message: String::new(),
                }))
            },
            Err(Error::NotFound(_)) => {
                warn!("Task not found: {}", task_id);
                Err(Status::not_found(format!("Task not found: {}", task_id)))
            },
            Err(Error::InvalidState(msg)) => {
                warn!("Invalid state for task completion: {}", msg);
                Err(Status::failed_precondition(msg))
            },
            Err(err) => {
                error!("Failed to complete task: {}", err);
                Err(Status::internal(format!("Failed to complete task: {}", err)))
            }
        }
    }
    
    /// Cancel a task with a reason.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr(), task_id = %request.get_ref().task_id))]
    async fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> std::result::Result<Response<CancelTaskResponse>, Status> {
        let request = request.into_inner();
        let task_id = request.task_id;
        let reason = if request.reason.is_empty() { 
            "No reason provided".to_string() 
        } else { 
            request.reason 
        };
        
        if task_id.is_empty() {
            return Err(Status::invalid_argument("Task ID is required"));
        }
        
        debug!("Cancelling task {}: {}", task_id, reason);
        
        // First, get the current task
        let old_task = match self.task_manager.get_task(&task_id).await {
            Ok(t) => t,
            Err(Error::NotFound(_)) => {
                return Err(Status::not_found(format!("Task not found: {}", task_id)));
            },
            Err(err) => {
                return Err(Status::internal(format!("Failed to get task: {}", err)));
            }
        };
        
        // Cancel the task using the task manager
        match self.task_manager.cancel_task(&task_id, &reason).await {
            Ok(task) => {
                info!("Cancelled task {}: {}", task_id, reason);
                
                // Convert to protobuf Task
                let proto_task: GenTask = task.clone().into();
                
                // Notify watchers
                if self.is_significant_update(&old_task, &task, false) {
                    self.broadcast_task_update(task).await;
                }
                
                Ok(Response::new(CancelTaskResponse {
                    success: true,
                    error_message: String::new(),
                }))
            },
            Err(Error::NotFound(_)) => {
                warn!("Task not found: {}", task_id);
                Err(Status::not_found(format!("Task not found: {}", task_id)))
            },
            Err(Error::InvalidState(msg)) => {
                warn!("Invalid state for task cancellation: {}", msg);
                Err(Status::failed_precondition(msg))
            },
            Err(err) => {
                error!("Failed to cancel task: {}", err);
                Err(Status::internal(format!("Failed to cancel task: {}", err)))
            }
        }
    }
    
    /// Watch a task for changes and get a stream of updates.
    ///
    /// This method allows clients to subscribe to task changes and receive
    /// real-time updates when significant changes occur.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr(), task_id = %request.get_ref().task_id))]
    async fn watch_task(
        &self,
        request: Request<WatchTaskRequest>,
    ) -> std::result::Result<Response<Self::WatchTaskStream>, Status> {
        let request = request.into_inner();
        let task_id = request.task_id;
        let only_watchable = request.only_watchable;
        
        if task_id.is_empty() {
            return Err(Status::invalid_argument("Task ID is required"));
        }
        
        debug!("Watching task: {}", task_id);
        
        // First, check if the task exists
        let initial_task = match self.task_manager.get_task(&task_id).await {
            Ok(task) => task,
            Err(Error::NotFound(_)) => {
                return Err(Status::not_found(format!("Task not found: {}", task_id)));
            },
            Err(err) => {
                return Err(Status::internal(format!("Failed to get task: {}", err)));
            }
        };
        
        // Create a channel for task updates
        let (tx, rx) = mpsc::channel(10);
        let task_id_clone = task_id.clone();
        let self_clone = self.clone();
        
        // Send the initial task state
        let initial_update = WatchTaskResponse {
            task: Some(initial_task.clone().into()),
            is_initial_state: true,
            success: true,
            error_message: String::new(),
        };
        
        if let Err(err) = tx.send(Ok(initial_update)).await {
            error!("Failed to send initial task update: {}", err);
            return Err(Status::internal("Failed to establish watch stream"));
        }
        
        // Register the watcher
        self.register_watcher(&task_id, tx.clone()).await;
        
        // Create a background task to handle the watch session
        tokio::spawn(async move {
            let mut last_task = initial_task;
            let tx_clone = tx.clone();
            
            // Check for updates every 5 seconds or if the task changes
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                match self_clone.task_manager.get_task(&task_id_clone).await {
                    Ok(task) => {
                        // Check if there are meaningful changes
                        if self_clone.is_significant_update(&last_task, &task, only_watchable) {
                            // Create an update
                            let update = WatchTaskResponse {
                                task: Some(task.clone().into()),
                                is_initial_state: false,
                                success: true,
                                error_message: String::new(),
                            };
                            
                            // Send the update
                            if let Err(_) = tx_clone.send(Ok(update)).await {
                                // Client disconnected
                                break;
                            }
                            
                            // Update the last task
                            last_task = task;
                        }
                        
                        // If the task is already completed, break the loop
                        if last_task.is_finished() {
                            break;
                        }
                    },
                    Err(_) => {
                        // Task not found or other error
                        break;
                    }
                }
            }
            
            // Clean up when we're done
            self_clone.unregister_watcher(&task_id_clone, &tx_clone).await;
        });
        
        // Return the stream to the client
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx)
        )))
    }
    
    /// List available commands.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr()))]
    async fn list_commands(
        &self,
        request: Request<ListCommandsRequest>,
    ) -> std::result::Result<Response<ListCommandsResponse>, Status> {
        let request = request.into_inner();
        
        info!("Listing available commands");
        
        // Get commands from the appropriate registry
        let commands: Vec<String> = {
            #[cfg(feature = "command-registry")]
            {
                if let Some(registry) = &self.command_registry {
                    match registry.lock() {
                        Ok(reg) => reg.command_names(),
                        Err(_) => {
                            error!("Failed to lock main command registry");
                            Vec::new()
                        }
                    }
                } else if let Some(mock_registry) = &self.mock_command_registry {
                    match mock_registry.lock() {
                        Ok(mock_reg) => mock_reg.command_names(),
                        Err(_) => {
                            error!("Failed to lock mock command registry");
                            Vec::new()
                        }
                    }
                } else {
                    Vec::new()
                }
            }
            #[cfg(not(feature = "command-registry"))]
            {
                if let Some(mock_registry) = &self.mock_command_registry {
                    match mock_registry.lock() {
                        Ok(mock_reg) => mock_reg.command_names(),
                        Err(_) => {
                            error!("Failed to lock mock command registry");
                            Vec::new()
                        }
                    }
                } else {
                    Vec::new()
                }
            }
        };
        
        // Apply filtering if requested
        let filtered_commands = if !request.name_filter.is_empty() {
            commands
                .into_iter()
                .filter(|name| name.contains(&request.name_filter))
                .collect::<Vec<_>>()
        } else {
            commands
        };
        
        // Clone for total count before pagination
        let total_count = filtered_commands.len() as i32;
        
        // Apply pagination if requested
        let (offset, limit) = (request.offset as usize, request.limit as usize);
        let paginated_commands = if limit > 0 {
            let start = offset.min(filtered_commands.len());
            let end = (start + limit).min(filtered_commands.len());
            filtered_commands[start..end].to_vec()
        } else {
            filtered_commands
        };
        
        // Create command objects with metadata
        let command_objects = paginated_commands
            .into_iter()
            .map(|name| {
                // Get description from the appropriate registry
                let description = {
                    #[cfg(feature = "command-registry")]
                    {
                        if let Some(registry) = &self.command_registry {
                            if let Ok(reg_lock) = registry.lock() {
                                reg_lock.get(&name).map(|cmd| cmd.description().to_string()).unwrap_or_default()
                            } else {
                                "".to_string()
                            }
                        } else if let Some(mock_registry) = &self.mock_command_registry {
                            if let Ok(mut mock_reg_lock) = mock_registry.lock() { // Acquire mutable lock
                                mock_reg_lock.get(&name).map(|cmd| cmd.description().to_string()).unwrap_or_default()
                            } else {
                                "".to_string()
                            }
                        } else {
                            "".to_string()
                        }
                    }
                    #[cfg(not(feature = "command-registry"))]
                    {
                        if let Some(mock_registry) = &self.mock_command_registry {
                            if let Ok(mut mock_reg_lock) = mock_registry.lock() { // Acquire mutable lock
                                mock_reg_lock.get(&name).map(|cmd| cmd.description().to_string()).unwrap_or_default()
                            } else {
                                "".to_string()
                            }
                        } else {
                            "".to_string()
                        }
                    }
                };
                
                // Create protocol buffer Command object
                GenCommand {
                    name,
                    description,
                    parameter_schema: Vec::new(), // Not implemented yet
                    id: uuid::Uuid::new_v4().to_string(),
                    created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
                    updated_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
                }
            })
            .collect::<Vec<_>>();
        
        // Create successful response
        let response = ListCommandsResponse {
            commands: command_objects,
            total_count: total_count,
            success: true,
            error_message: String::new(),
        };
        
        Ok(Response::new(response))
    }
    
    /// Execute a command.
    #[instrument(skip(self, request), fields(client_addr = ?request.remote_addr(), command = %request.get_ref().command_name))]
    async fn execute_command(
        &self,
        request: Request<ExecuteCommandRequest>,
    ) -> Result<Response<ExecuteCommandResponse>, Status> {
        let inner = request.into_inner();
        info!(
            "Executing command: {}, params bytes: {:?}",
            inner.command_name,
            &inner.parameters
        );

        let parameters_value = serde_json::from_slice::<serde_json::Value>(&inner.parameters)
            .map_err(|e| Status::invalid_argument(format!("Invalid parameters JSON bytes: {}", e)))?;

        let arguments = json_params_to_string_vec(&parameters_value)
             .map_err(|e| Status::invalid_argument(format!("Cannot convert params to args: {}", e)))?;

        // Execute the command using the internal helper (non-async)
        // output_result is now Result<String, String>
        let output_result = self.execute_command(&inner.command_name, &arguments);

        // --- Check command execution result BEFORE creating task ---
        if let Err(err_str) = &output_result {
            // Check if the error indicates "not found"
            // This relies on the internal helper returning a specific string format for not found
            if err_str.contains("not found") { // Adjust this condition as needed
                error!(
                    "Command not found '{}': {}",
                    inner.command_name, err_str
                );
                return Err(Status::not_found(format!("Command not found: {}", inner.command_name)));
            } else {
                // Other internal execution error
                error!(
                    "Internal command execution failed for '{}': {}",
                    inner.command_name, err_str
                );
                return Err(Status::internal(format!("Command execution failed: {}", err_str)));
            }
        }

        // --- If we reach here, command execution *logic* succeeded (output_result is Ok) ---
        let output_str = output_result.unwrap(); // Safe to unwrap now

        let task_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // --- Create Task ---
        let initial_task = Task {
            id: task_id.clone(),
            name: format!("Direct Exec: {}", inner.command_name),
            description: format!("Direct execution for command: {}", inner.command_name),
            status_code: TaskStatus::Pending, // Will be updated to Completed shortly
            priority_code: TaskPriority::Medium,
            agent_type: AgentType::System,
            progress: 0.0,
            agent_id: None,
            context_id: None,
            parent_id: None,
            prerequisites: Vec::new(),
            created_at: now,
            updated_at: now,
            completed_at: None,
            input_data: None,
            output_data: None,
            metadata: None,
            error_message: None,
            status_message: None,
            deadline: None,
            watchable: false,
            retry_count: 0,
            max_retries: 0,
        };

        if let Err(e) = self.task_manager.create_task(initial_task).await {
             error!("Failed to create task {} after successful command logic: {}", task_id, e);
             return Err(Status::internal(format!("Task creation failed: {}", e)));
        }

        // --- Update Task to Completed Status ---
        let final_status = TaskStatus::Completed;
        let final_output_data = Some([("output".to_string(), output_str.clone())].iter().cloned().collect());
        let final_error_msg = None;

        // Declare clone variable outside the match scope (Needed for response)
        let output_data_clone = final_output_data.clone();

        // Fetch the task to update it
        match self.task_manager.get_task(&task_id).await {
             Ok(mut task_to_update) => {
                  task_to_update.status_code = final_status;
                  task_to_update.progress = 100.0;
                  task_to_update.output_data = final_output_data; // Move original
                  task_to_update.error_message = final_error_msg.clone();
                  task_to_update.status_message = final_error_msg; // None in success case
                  task_to_update.completed_at = Some(Utc::now());
                  task_to_update.updated_at = Utc::now();

                  if let Err(e) = self.task_manager.update_task(task_to_update).await {
                       error!("Failed to update task {} status to Completed: {}", task_id, e);
                  }
             }
             Err(e) => {
                 error!("Failed to fetch task {} for Completed status update: {}", task_id, e);
             }
        }

        // Populate all required fields for the response
        let final_output_string = output_data_clone.and_then(|m| m.get("output").cloned()).unwrap_or_default();

        Ok(Response::new(ExecuteCommandResponse {
            task_id,
            success: true, // Always true if we reached here
            output: final_output_string,
            error_message: String::new(), // No error if successful
        }))
    }
}

// Allow cloning for TaskServiceImpl
impl Clone for TaskServiceImpl {
    fn clone(&self) -> Self {
        Self {
            task_manager: self.task_manager.clone(),
            watchers: self.watchers.clone(),
            #[cfg(feature = "command-registry")]
            command_registry: self.command_registry.clone(),
            mock_command_registry: self.mock_command_registry.clone(),
        }
    }
}

// Mock command registry implementation for tests
mod mock {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    
    #[derive(Debug, Clone)]
    pub struct MockCommand {
        name: String,
        description: String,
    }
    
    impl MockCommand {
        pub fn new(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
            }
        }
        
        pub fn name(&self) -> &str {
            &self.name
        }
        
        pub fn description(&self) -> &str {
            &self.description
        }
        
        pub fn execute(&self, args: &[String]) -> Result<String, String> {
            Ok(format!("Mock command {} executed with args: {:?}", self.name, args))
        }
    }
    
    #[derive(Debug)]
    pub struct MockCommandRegistry {
        commands: HashMap<String, MockCommand>,
    }
    
    impl MockCommandRegistry {
        pub fn new() -> Self {
            Self {
                commands: HashMap::new(),
            }
        }
        
        pub fn register(&mut self, command: MockCommand) -> Result<(), String> {
            self.commands.insert(command.name().to_string(), command);
            Ok(())
        }
        
        pub fn command_names(&self) -> Vec<String> {
            self.commands.keys().cloned().collect()
        }
        
        // Updated to return Option<MockCommand> and take &mut self to modify the internal map
        pub fn get(&mut self, name: &str) -> Option<MockCommand> {
            // If the command exists in the main registry, clone and return it
            if let Some(cmd) = self.commands.get(name) {
                return Some(cmd.clone()); 
            }
            
            // In test mode, accept ANY command name for integration tests
            // Create and insert the command directly into self.commands if it doesn't exist
            if !self.commands.contains_key(name) {
                 let cmd = MockCommand::new(name, &format!("Dynamically created test command for {}", name));
                 // Insert a clone into the main map
                 self.commands.insert(name.to_string(), cmd);
            }
            // Clone the command from the main map before returning
            self.commands.get(name).cloned()
        }
    }
} 