//! Tool management module for MCP
//!
//! This module provides the core tool management functionality for MCP.

// Core tool modules
pub mod cleanup;
pub mod executor;
pub mod lifecycle;
pub mod management;

// Re-export core types and traits from management
pub use management::types::{
    Tool, ToolState, ToolError, ToolInfo, ToolExecutor, ToolContext, 
    ToolExecutionResult, ExecutionStatus, ToolLifecycleHook
};
pub use management::{ToolManager, CoreToolManager};

// Re-export from cleanup for compatibility
pub use cleanup::RecoveryHook; 