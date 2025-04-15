//! Client implementation for task management with the Task Service API.
//!
//! This module provides a client wrapper for the TaskService gRPC service,
//! allowing applications to interact with the task management system.

use std::sync::Arc;
use std::time::Duration;
use anyhow::{Result, anyhow, Context};
use tonic::{Request, Status, transport::Channel};
use futures::StreamExt;
use tokio::sync::Mutex;
use futures::Stream;

use crate::generated::mcp_task::*;
use crate::generated::mcp_task::task_service_client::TaskServiceClient;
use crate::task::Task;

/// Client configuration for connecting to the task service
#[derive(Clone, Debug)]
pub struct TaskClientConfig {
    /// Server address in the format "http://hostname:port"
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

/// Client wrapper for the TaskService gRPC service
/// 
/// This struct provides a high-level API for interacting with the MCP task service.
#[derive(Clone)]
pub struct MCPTaskClient {
    /// Task service client
    client: Arc<Mutex<TaskServiceClient<Channel>>>,
    /// Client configuration
    config: TaskClientConfig,
}

impl MCPTaskClient {
    /// Default task client configuration
    pub fn default_config() -> TaskClientConfig {
        TaskClientConfig {
            server_address: "http://localhost:50051".to_string(),
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
        // Create default channel - will be initialized properly in connect()
        let channel = tonic::transport::Channel::from_shared("http://[::1]:50051")
            .expect("Invalid URI")
            .connect_lazy();
            
        let client = Arc::new(Mutex::new(TaskServiceClient::new(channel)));
        
        MCPTaskClient {
            client,
            config,
        }
    }
    
    /// Connect to the task service and initialize the client
    pub async fn connect(&self) -> Result<()> {
        let config = &self.config;
        
        // Create a new connection
        let connect_timeout = Duration::from_millis(config.connect_timeout_ms);
        let endpoint = tonic::transport::Endpoint::new(config.server_address.clone())?
            .connect_timeout(connect_timeout);
        
        let channel = endpoint.connect().await?;
        let client = TaskServiceClient::new(channel);
        
        // Store the client
        {
            let mut guard = self.client.lock().await;
            *guard = client;
        }
        
        Ok(())
    }

    /// Get the server address from the configuration
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

        // Convert input data to bytes if provided
        if let Some(data) = input_data {
            request.input_data = serde_json::to_vec(&data)
                .context("Failed to serialize input data")?;
        }

        // Convert metadata to bytes if provided
        if let Some(meta) = metadata {
            request.metadata = serde_json::to_vec(&meta)
                .context("Failed to serialize metadata")?;
        }

        let request_clone = request.clone();
        
        let response = self.execute_with_retry(move |mut client| {
            let req = request_clone.clone();
            async move {
                client.create_task(Request::new(req)).await
            }
        }).await?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to create task: {}", inner.error_message));
        }

        Ok(inner.task_id)
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &str) -> Result<Task> {
        let request = GetTaskRequest {
            task_id: task_id.to_string(),
        };
        
        let request_clone = request.clone();
        
        let response = self.execute_with_retry(move |mut client| {
            let req = request_clone.clone();
            async move {
                client.get_task(Request::new(req)).await
            }
        }).await?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to get task: {}", inner.error_message));
        }

        let task: Task = inner.task.ok_or_else(|| anyhow!("No task in response"))?.into();
        Ok(task)
    }

    /// Update a task
    pub async fn update_task(&self, task: &Task) -> Result<()> {
        let proto_task: crate::generated::mcp_task::Task = task.clone().into();
        
        let request = UpdateTaskRequest {
            task_id: proto_task.id.clone(),
            name: proto_task.name.clone(),
            description: proto_task.description.clone(),
            priority: proto_task.priority,
            input_data: proto_task.input_data.clone(),
            metadata: proto_task.metadata.clone(),
        };
        
        let request_clone = request.clone();
        
        let response = self.execute_with_retry(move |mut client| {
            let req = request_clone.clone();
            async move {
                client.update_task(Request::new(req)).await
            }
        }).await?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to update task: {}", inner.error_message));
        }

        Ok(())
    }

    /// List tasks with optional filtering
    pub async fn list_tasks(
        &self,
        status: Option<TaskStatus>,
        priority: Option<TaskPriority>,
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
            limit: limit.unwrap_or(100) as i32,
            offset: offset.unwrap_or(0) as i32,
        };
        
        let request_clone = request.clone();
        
        let response = self.execute_with_retry(move |mut client| {
            let req = request_clone.clone();
            async move {
                client.list_tasks(Request::new(req)).await
            }
        }).await?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to list tasks: {}", inner.error_message));
        }

        let tasks = inner.tasks
            .into_iter()
            .map(|t| t.into())
            .collect();

        Ok(tasks)
    }

    /// Assign a task to an agent
    pub async fn assign_task(&self, task_id: &str, agent_id: &str, agent_type: AgentType) -> Result<()> {
        let request = AssignTaskRequest {
            task_id: task_id.to_string(),
            agent_id: agent_id.to_string(),
            agent_type: agent_type as i32,
        };
        
        let request_clone = request.clone();
        
        let response = self.execute_with_retry(move |mut client| {
            let req = request_clone.clone();
            async move {
                client.assign_task(Request::new(req)).await
            }
        }).await?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to assign task: {}", inner.error_message));
        }

        Ok(())
    }

    /// Report progress for a task
    pub async fn report_progress(
        &self, 
        task_id: &str, 
        progress_percent: i32, 
        message: Option<&str>
    ) -> Result<()> {
        let request = ReportProgressRequest {
            task_id: task_id.to_string(),
            progress_percent,
            progress_message: message.unwrap_or("").to_string(),
            interim_results: Vec::new(),
        };
        
        let request_clone = request.clone();
        
        let response = self.execute_with_retry(move |mut client| {
            let req = request_clone.clone();
            async move {
                client.report_progress(Request::new(req)).await
            }
        }).await?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to report progress: {}", inner.error_message));
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

        // Convert output data to bytes if provided
        if let Some(data) = output_data {
            request.output_data = serde_json::to_vec(&data)
                .context("Failed to serialize output data")?;
        }

        // Convert metadata to bytes if provided
        if let Some(meta) = metadata {
            request.metadata = serde_json::to_vec(&meta)
                .context("Failed to serialize metadata")?;
        }
        
        let request_clone = request.clone();
        
        let response = self.execute_with_retry(move |mut client| {
            let req = request_clone.clone();
            async move {
                client.complete_task(Request::new(req)).await
            }
        }).await?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to complete task: {}", inner.error_message));
        }

        Ok(())
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str, reason: &str) -> Result<()> {
        let request = CancelTaskRequest {
            task_id: task_id.to_string(),
            reason: reason.to_string(),
        };
        
        let request_clone = request.clone();
        
        let response = self.execute_with_retry(move |mut client| {
            let req = request_clone.clone();
            async move {
                client.cancel_task(Request::new(req)).await
            }
        }).await?;

        let inner = response.into_inner();
        if !inner.success {
            return Err(anyhow!("Failed to cancel task: {}", inner.error_message));
        }

        Ok(())
    }

    /// Watch a task for changes.
    /// 
    /// This method returns a streaming response that will emit task updates as they happen.
    /// The stream remains open until explicitly closed or the server terminates it.
    /// 
    /// # Arguments
    /// 
    /// * `task_id` - The unique identifier of the task to watch
    /// * `include_initial_state` - Whether to include the initial task state in the response
    /// * `timeout_seconds` - Optional timeout in seconds (0 = no timeout)
    /// * `watchable` - Whether to only include watchable tasks in the response
    /// * `filter_updates` - Whether to filter duplicate updates (only emit when task state changes)
    /// 
    /// # Returns
    /// 
    /// A `Result` containing a stream of task updates or an error
    pub async fn watch_task(
        &self, 
        task_id: &str, 
        include_initial_state: bool, 
        timeout_seconds: i32,
        watchable: bool,
        filter_updates: bool
    ) -> Result<impl Stream<Item = Result<Task, Status>>> {
        // Connect to the task service - handle conversion from anyhow::Error to tonic::Status
        let mut client = {
            // Ensure we're connected
            if let Err(e) = self.connect().await {
                return Err(anyhow!("Failed to connect to task service: {}", e));
            }
            
            let guard = self.client.lock().await;
            guard.clone()
        };
        
        let request = tonic::Request::new(crate::generated::mcp_task::WatchTaskRequest {
            task_id: task_id.to_string(),
            include_initial_state,
            timeout_seconds,
            only_watchable: watchable,
            filter_updates,
        });
        
        let response = match client.watch_task(request).await {
            Ok(response) => response,
            Err(status) => {
                return Err(anyhow!("Failed to watch task: {}", status));
            }
        };
        
        let stream = response.into_inner();
        
        // Transform the stream to convert protobuf tasks to our Task type
        Ok(stream.map(|result| {
            match result {
                Ok(proto_response) => {
                    if !proto_response.success {
                        Err(Status::internal(proto_response.error_message))
                    } else if let Some(proto_task) = proto_response.task {
                        Ok(Task::from(proto_task))
                    } else {
                        Err(Status::internal("No task in watch response"))
                    }
                },
                Err(status) => Err(status),
            }
        }))
    }

    /// Execute a client operation with retry logic
    async fn execute_with_retry<F, Fut, R>(&self, operation: F) -> Result<R>
    where
        F: for<'a> Fn(TaskServiceClient<Channel>) -> Fut,
        Fut: std::future::Future<Output = std::result::Result<R, Status>>,
    {
        let mut backoff = self.config.initial_backoff_ms;
        let mut attempts = 0;

        loop {
            attempts += 1;
            let client = {
                let guard = self.client.lock().await;
                guard.clone()
            };

            match operation(client).await {
                Ok(response) => return Ok(response),
                Err(status) => {
                    if attempts >= self.config.max_retries || !is_retryable_error(&status) {
                        return Err(anyhow!("Task service error: {}", status));
                    }

                    // Exponential backoff
                    tokio::time::sleep(Duration::from_millis(backoff)).await;
                    backoff = std::cmp::min(backoff * 2, self.config.max_backoff_ms);
                }
            }
        }
    }
}

/// Helper function to determine if an error is retryable
fn is_retryable_error(status: &Status) -> bool {
    match status.code() {
        tonic::Code::Unavailable | 
        tonic::Code::DeadlineExceeded |
        tonic::Code::ResourceExhausted |
        tonic::Code::Internal => true,
        _ => false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_task() {
        // This is just a stub for now
        // Real tests will need a mock server or integration tests
    }
} 