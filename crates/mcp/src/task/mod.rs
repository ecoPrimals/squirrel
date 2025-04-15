//! Task management system for the MCP.
//!
//! This module provides a comprehensive task management system that can be used
//! to create, track, and manage units of work that need to be performed by agents.

// Main task module - import and re-export from submodules

// Import types from the generated protobuf code

// Define the submodules
pub mod types;
pub mod client;
pub mod server;
pub mod manager;
pub mod conversion;

// Re-export the types
pub use self::types::Task;
pub use self::client::{MCPTaskClient, TaskClientConfig};
pub use self::manager::TaskManager;

#[cfg(test)]
pub mod tests;

// Re-export generated types from the protobuf
pub use crate::generated::mcp_task::task_service_client::TaskServiceClient; 