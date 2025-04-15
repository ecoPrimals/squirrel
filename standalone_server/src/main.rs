use anyhow::{Result, anyhow};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, broadcast};
use tokio::signal;
use tokio::time::{Duration, timeout};
use tonic::{transport::Server, Status, Request, Response};
use tracing::{info, error, debug, Level};
use tracing_subscriber::FmtSubscriber;
use std::collections::HashMap;
use uuid::Uuid;
use tonic::codegen::tokio_stream::{StreamExt, wrappers::ReceiverStream};

// Import from the crate level modules
use taskserver_standalone::mcp_task::{
    task_service_server::{TaskService, TaskServiceServer},
    AgentType, CompleteTaskRequest, CompleteTaskResponse, CreateTaskRequest, CreateTaskResponse,
    GetTaskRequest, GetTaskResponse, ListTasksRequest, ListTasksResponse, ReportProgressRequest,
    ReportProgressResponse, Task, TaskPriority, TaskStatus, UpdateTaskRequest, UpdateTaskResponse,
    AssignTaskRequest, AssignTaskResponse, CancelTaskRequest, CancelTaskResponse,
    WatchTaskRequest, WatchTaskResponse,
};

// Define command-line options
#[derive(Parser, Debug)]
#[clap(name = "task_server", about = "Standalone Task Management Server")]
struct Opt {
    /// Socket address to listen on
    #[clap(short, long, default_value = "[::1]:50052")]
    address: String,

    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,

    /// Request timeout in seconds
    #[clap(long, default_value = "30")]
    request_timeout: u64,
}

// Simple in-memory task store
#[derive(Debug, Clone)]
struct TaskStore {
    tasks: HashMap<String, Task>,
}

impl TaskStore {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    fn create_task(&mut self, mut task: Task) -> String {
        if task.id.is_empty() {
            task.id = Uuid::new_v4().to_string();
        }
        
        // Set timestamps if not already set
        if task.created_at.is_none() {
            task.created_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        }
        
        if task.updated_at.is_none() {
            task.updated_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        }
        
        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task);
        task_id
    }

    fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.get(task_id).cloned()
    }

    fn update_task(&mut self, task_id: &str, updates: Task) -> Result<(), Status> {
        let task = self.tasks.get_mut(task_id).ok_or_else(|| {
            Status::not_found(format!("Task not found: {}", task_id))
        })?;
        
        // Update fields
        if !updates.name.is_empty() {
            task.name = updates.name;
        }
        
        if !updates.description.is_empty() {
            task.description = updates.description;
        }
        
        if updates.priority != 0 {
            task.priority = updates.priority;
        }
        
        if !updates.input_data.is_empty() {
            task.input_data = updates.input_data;
        }
        
        if !updates.metadata.is_empty() {
            task.metadata = updates.metadata;
        }
        
        // Update timestamp
        task.updated_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        
        Ok(())
    }

    fn list_tasks(&self, status: i32, agent_id: &str, context_id: &str) -> Vec<Task> {
        self.tasks
            .values()
            .filter(|task| {
                (status <= 0 || task.status == status) &&
                (agent_id.is_empty() || task.agent_id == agent_id) &&
                (context_id.is_empty() || task.context_id == context_id)
            })
            .cloned()
            .collect()
    }

    fn assign_task(&mut self, task_id: &str, agent_id: &str, agent_type: i32) -> Result<(), Status> {
        let task = self.tasks.get_mut(task_id).ok_or_else(|| {
            Status::not_found(format!("Task not found: {}", task_id))
        })?;
        
        task.agent_id = agent_id.to_string();
        task.agent_type = agent_type;
        task.status = 2; // ASSIGNED
        task.updated_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        
        Ok(())
    }

    fn update_progress(&mut self, task_id: &str, percent: i32, message: &str) -> Result<(), Status> {
        let task = self.tasks.get_mut(task_id).ok_or_else(|| {
            Status::not_found(format!("Task not found: {}", task_id))
        })?;
        
        // Only update tasks that are in ASSIGNED or RUNNING state
        if task.status != 2 && task.status != 3 {
            return Err(Status::failed_precondition(
                format!("Cannot update progress for task in state: {}", task.status)
            ));
        }
        
        // If first progress update, set to RUNNING
        if task.status == 2 {
            task.status = 3; // RUNNING
            task.started_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        }
        
        task.progress_percent = percent;
        task.progress_message = message.to_string();
        task.updated_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        
        Ok(())
    }

    fn complete_task(&mut self, task_id: &str, output_data: Vec<u8>, metadata: Vec<u8>) -> Result<(), Status> {
        let task = self.tasks.get_mut(task_id).ok_or_else(|| {
            Status::not_found(format!("Task not found: {}", task_id))
        })?;
        
        // Only complete tasks that are in ASSIGNED or RUNNING state
        if task.status != 2 && task.status != 3 {
            return Err(Status::failed_precondition(
                format!("Cannot complete task in state: {}", task.status)
            ));
        }
        
        task.status = 4; // COMPLETED
        task.output_data = output_data;
        task.progress_percent = 100;
        
        if !metadata.is_empty() {
            task.metadata = metadata;
        }
        
        task.completed_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        task.updated_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        
        Ok(())
    }

    fn cancel_task(&mut self, task_id: &str, reason: &str) -> Result<(), Status> {
        let task = self.tasks.get_mut(task_id).ok_or_else(|| {
            Status::not_found(format!("Task not found: {}", task_id))
        })?;
        
        // Only cancel tasks that are not already in terminal state
        if task.status == 4 || task.status == 5 || task.status == 6 {
            return Err(Status::failed_precondition(
                format!("Cannot cancel task in state: {}", task.status)
            ));
        }
        
        task.status = 6; // CANCELLED
        task.error_message = reason.to_string();
        task.completed_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        task.updated_at = Some(prost_types::Timestamp::from(std::time::SystemTime::now()));
        
        Ok(())
    }
}

// Task Service Implementation
struct TaskServiceImpl {
    store: Arc<RwLock<TaskStore>>,
    task_updates: Arc<broadcast::Sender<String>>,
}

impl TaskServiceImpl {
    fn new(store: Arc<RwLock<TaskStore>>) -> Self {
        let (task_updates, _) = broadcast::channel(100);
        Self {
            store,
            task_updates: Arc::new(task_updates),
        }
    }
    
    // Helper method to notify about task updates
    async fn notify_task_update(&self, task_id: &str) {
        let _ = self.task_updates.send(task_id.to_string());
    }
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Create task request received: {}", req.name);
        
        // Create a new task
        let task = Task {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            description: req.description,
            status: 1, // CREATED
            priority: req.priority,
            created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            updated_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            started_at: None,
            completed_at: None,
            agent_id: req.agent_id,
            agent_type: req.agent_type,
            input_data: req.input_data,
            output_data: Vec::new(),
            metadata: req.metadata,
            error_message: String::new(),
            prerequisite_task_ids: req.prerequisite_task_ids,
            dependent_task_ids: Vec::new(),
            progress_percent: 0,
            progress_message: String::new(),
            context_id: req.context_id,
        };
        
        // Store the task
        let task_id = {
            let mut store = self.store.write().await;
            store.create_task(task)
        };
        
        // Notify about the update
        self.notify_task_update(&task_id).await;
        
        // Return response
        let response = CreateTaskResponse {
            task_id,
            success: true,
            error_message: String::new(),
        };
        
        info!("Task created: {}", response.task_id);
        Ok(Response::new(response))
    }
    
    async fn update_task(
        &self,
        request: Request<UpdateTaskRequest>,
    ) -> Result<Response<UpdateTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Update task request received: {}", req.task_id);
        
        // Create updates
        let updates = Task {
            id: req.task_id.clone(),
            name: req.name,
            description: req.description,
            status: 0, // Don't update status
            priority: req.priority,
            created_at: None,
            updated_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            started_at: None,
            completed_at: None,
            agent_id: String::new(),
            agent_type: 0,
            input_data: req.input_data,
            output_data: Vec::new(),
            metadata: req.metadata,
            error_message: String::new(),
            prerequisite_task_ids: Vec::new(),
            dependent_task_ids: Vec::new(),
            progress_percent: 0,
            progress_message: String::new(),
            context_id: String::new(),
        };
        
        // Update the task
        {
            let mut store = self.store.write().await;
            store.update_task(&req.task_id, updates)?;
        }
        
        // Notify about the update
        self.notify_task_update(&req.task_id).await;
        
        // Return response
        let response = UpdateTaskResponse {
            success: true,
            error_message: String::new(),
        };
        
        info!("Task updated: {}", req.task_id);
        Ok(Response::new(response))
    }
    
    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<GetTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Get task request received: {}", req.task_id);
        
        // Get the task
        let task = {
            let store = self.store.read().await;
            store.get_task(&req.task_id).ok_or_else(|| {
                Status::not_found(format!("Task not found: {}", req.task_id))
            })?
        };
        
        // Return response
        let response = GetTaskResponse {
            task: Some(task),
            success: true,
            error_message: String::new(),
        };
        
        Ok(Response::new(response))
    }
    
    async fn list_tasks(
        &self,
        request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        let req = request.into_inner();
        debug!("List tasks request received");
        
        // Get tasks matching criteria
        let tasks = {
            let store = self.store.read().await;
            store.list_tasks(req.status, &req.agent_id, &req.context_id)
        };
        
        // Apply pagination
        let total_count = tasks.len() as i32;
        let offset = req.offset as usize;
        let limit = if req.limit > 0 { req.limit as usize } else { 100 };
        
        let tasks = tasks
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        
        // Return response
        let response = ListTasksResponse {
            tasks,
            total_count,
            success: true,
            error_message: String::new(),
        };
        
        info!("Listed {} tasks", response.tasks.len());
        Ok(Response::new(response))
    }
    
    async fn assign_task(
        &self,
        request: Request<AssignTaskRequest>,
    ) -> Result<Response<AssignTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Assign task request received: {}", req.task_id);
        
        // Assign the task
        {
            let mut store = self.store.write().await;
            store.assign_task(&req.task_id, &req.agent_id, req.agent_type)?;
        }
        
        // Notify about the update
        self.notify_task_update(&req.task_id).await;
        
        // Return response
        let response = AssignTaskResponse {
            success: true,
            error_message: String::new(),
        };
        
        info!("Task assigned: {} to agent {}", req.task_id, req.agent_id);
        Ok(Response::new(response))
    }
    
    async fn report_progress(
        &self,
        request: Request<ReportProgressRequest>,
    ) -> Result<Response<ReportProgressResponse>, Status> {
        let req = request.into_inner();
        debug!("Report progress request received: {}", req.task_id);
        
        // Update progress
        {
            let mut store = self.store.write().await;
            store.update_progress(&req.task_id, req.progress_percent, &req.progress_message)?;
        }
        
        // Notify about the update
        self.notify_task_update(&req.task_id).await;
        
        // Return response
        let response = ReportProgressResponse {
            success: true,
            error_message: String::new(),
        };
        
        debug!("Progress updated for task {}: {}%", req.task_id, req.progress_percent);
        Ok(Response::new(response))
    }
    
    async fn complete_task(
        &self,
        request: Request<CompleteTaskRequest>,
    ) -> Result<Response<CompleteTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Complete task request received: {}", req.task_id);
        
        // Complete the task
        {
            let mut store = self.store.write().await;
            store.complete_task(&req.task_id, req.output_data, req.metadata)?;
        }
        
        // Notify about the update
        self.notify_task_update(&req.task_id).await;
        
        // Return response
        let response = CompleteTaskResponse {
            success: true,
            error_message: String::new(),
        };
        
        info!("Task completed: {}", req.task_id);
        Ok(Response::new(response))
    }
    
    async fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> Result<Response<CancelTaskResponse>, Status> {
        let req = request.into_inner();
        debug!("Cancel task request received: {}", req.task_id);
        
        // Cancel the task
        {
            let mut store = self.store.write().await;
            store.cancel_task(&req.task_id, &req.reason)?;
        }
        
        // Notify about the update
        self.notify_task_update(&req.task_id).await;
        
        // Return response
        let response = CancelTaskResponse {
            success: true,
            error_message: String::new(),
        };
        
        info!("Task cancelled: {} - {}", req.task_id, req.reason);
        Ok(Response::new(response))
    }

    // Implementation of the watch_task method
    type WatchTaskStream = ReceiverStream<Result<WatchTaskResponse, Status>>;
    
    async fn watch_task(
        &self,
        request: Request<WatchTaskRequest>,
    ) -> Result<Response<Self::WatchTaskStream>, Status> {
        let req = request.into_inner();
        let task_id = req.task_id;
        let include_initial_state = req.include_initial_state;
        let timeout_secs = req.timeout_seconds;
        let only_watchable = req.only_watchable;
        let filter_updates = req.filter_updates;
        
        debug!("Watch task request received: {}", task_id);
        
        // Verify task exists
        let initial_task = {
            let store = self.store.read().await;
            store.get_task(&task_id).ok_or_else(|| {
                Status::not_found(format!("Task not found: {}", task_id))
            })?
        };
        
        // Check if task is watchable if that filter is enabled
        if only_watchable {
            // Define which statuses are watchable (usually tasks in progress)
            let watchable_statuses = vec![
                TaskStatus::Running as i32, 
                TaskStatus::Assigned as i32,
                TaskStatus::Created as i32,
            ];
            
            if !watchable_statuses.contains(&initial_task.status) {
                return Err(Status::failed_precondition(
                    format!("Task {} is not in a watchable state", task_id)
                ));
            }
        }
        
        // Create channel for streaming responses
        let (tx, rx) = mpsc::channel(10);
        
        // Subscribe to task updates
        let mut task_updates = self.task_updates.subscribe();
        
        // Clone values for async move
        let store_clone = self.store.clone();
        let task_id_clone = task_id.clone();
        
        // Keep track of last task state for filtering
        let mut last_task_status = if filter_updates {
            Some(initial_task.status)
        } else {
            None
        };
        
        // Spawn task to handle the streaming
        tokio::spawn(async move {
            // Send initial state if requested
            if include_initial_state {
                let response = WatchTaskResponse {
                    task: Some(initial_task),
                    is_initial_state: true,
                    success: true,
                    error_message: String::new(),
                };
                
                if tx.send(Ok(response)).await.is_err() {
                    // Client disconnected
                    return;
                }
            }
            
            let watch_duration = if timeout_secs > 0 {
                Some(Duration::from_secs(timeout_secs as u64))
            } else {
                None
            };
            
            // Watch for updates
            loop {
                let update_future = task_updates.recv();
                
                let update_result = if let Some(duration) = watch_duration {
                    match timeout(duration, update_future).await {
                        Ok(result) => result,
                        Err(_) => {
                            // Timeout occurred
                            let _ = tx.send(Ok(WatchTaskResponse {
                                task: None,
                                is_initial_state: false,
                                success: true,
                                error_message: "Timeout".to_string(),
                            })).await;
                            break;
                        }
                    }
                } else {
                    update_future.await
                };
                
                // Process the update
                match update_result {
                    Ok(updated_task_id) => {
                        if updated_task_id == task_id_clone {
                            // Task was updated, send the new state
                            let updated_task = {
                                let store = store_clone.read().await;
                                match store.get_task(&task_id_clone) {
                                    Some(task) => task,
                                    None => {
                                        // Task was deleted
                                        let _ = tx.send(Ok(WatchTaskResponse {
                                            task: None,
                                            is_initial_state: false,
                                            success: false,
                                            error_message: "Task was deleted".to_string(),
                                        })).await;
                                        break;
                                    }
                                }
                            };
                            
                            // Check if we should filter this update
                            if filter_updates {
                                if let Some(last_status) = last_task_status {
                                    if last_status == updated_task.status {
                                        // Status hasn't changed, skip this update
                                        continue;
                                    }
                                }
                                // Update the last status
                                last_task_status = Some(updated_task.status);
                            }
                            
                            let response = WatchTaskResponse {
                                task: Some(updated_task),
                                is_initial_state: false,
                                success: true,
                                error_message: String::new(),
                            };
                            
                            if tx.send(Ok(response)).await.is_err() {
                                // Client disconnected
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        // Error in the broadcast channel
                        let _ = tx.send(Ok(WatchTaskResponse {
                            task: None,
                            is_initial_state: false,
                            success: false,
                            error_message: format!("Error watching task: {}", e),
                        })).await;
                        break;
                    }
                }
            }
        });
        
        // Return the receiver as a stream
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let opt = Opt::parse();

    // Initialize tracing
    let level = if opt.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    // Parse socket address
    let addr: SocketAddr = match opt.address.parse() {
        Ok(addr) => addr,
        Err(e) => {
            error!("Failed to parse address '{}': {}", opt.address, e);
            return Err(anyhow!("Invalid address: {}", e));
        }
    };

    info!("Starting Standalone Task Server on {}", addr);

    // Create task store
    let store = Arc::new(RwLock::new(TaskStore::new()));
    
    // Create task service
    let task_service = TaskServiceImpl::new(store);
    
    // Build and serve the service with graceful shutdown
    let service = TaskServiceServer::new(task_service);
    
    info!("Task Server is ready to accept connections");
    
    Server::builder()
        .timeout(Duration::from_secs(opt.request_timeout))
        .add_service(service)
        .serve_with_shutdown(addr, async {
            match signal::ctrl_c().await {
                Ok(()) => {
                    info!("Received shutdown signal, stopping server gracefully...");
                },
                Err(err) => {
                    error!("Failed to listen for shutdown signal: {}", err);
                }
            }
        })
        .await?;
    
    info!("Task Server shut down successfully");
    Ok(())
} 