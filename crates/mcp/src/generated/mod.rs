// Re-export all generated types from mcp.sync
pub mod mcp_sync;
// Re-export all generated types from mcp.task
pub mod mcp_task;

// Re-export sync service client
pub use mcp_sync::sync_service_client::SyncServiceClient;
// Re-export task service client
pub use mcp_task::task_service_client::TaskServiceClient;

// Re-export message types
pub use mcp_sync::{ContextChange, SyncRequest, SyncResponse};
pub use mcp_task::{Task, CreateTaskRequest, CreateTaskResponse, GetTaskRequest, 
                  GetTaskResponse, UpdateTaskRequest, UpdateTaskResponse,
                  ListTasksRequest, ListTasksResponse, AssignTaskRequest, 
                  AssignTaskResponse, ReportProgressRequest, ReportProgressResponse,
                  CompleteTaskRequest, CompleteTaskResponse, CancelTaskRequest,
                  CancelTaskResponse, WatchTaskRequest, WatchTaskResponse};

// Re-export enum types
pub use mcp_sync::context_change;
pub use mcp_task::{TaskStatus, TaskPriority, AgentType}; 