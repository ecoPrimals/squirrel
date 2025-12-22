//! Tool lifecycle hooks
//!
//! This module defines the ToolLifecycleHook trait for tool lifecycle management.

use std::any::Any;
use std::fmt;

use super::error::ToolError;
use super::tool::Tool;

/// Trait for tool lifecycle hooks
pub trait ToolLifecycleHook: fmt::Debug + Send + Sync {
    /// Converts to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Called when a tool is registered
    fn on_register(&self, tool: &Tool) -> impl std::future::Future<Output = Result<(), ToolError>> + Send;

    /// Called when a tool is unregistered
    fn on_unregister(&self, tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send;

    /// Called when a tool is activated
    fn on_activate(&self, tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send;

    /// Called when a tool is deactivated
    fn on_deactivate(&self, tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send;

    /// Called when a tool encounters an error
    fn on_error(&self, tool_id: &str, error: &ToolError) -> impl std::future::Future<Output = Result<(), ToolError>> + Send;

    /// Called before a tool is started
    fn pre_start(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called after a tool is started
    fn post_start(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called before a tool is stopped
    fn pre_stop(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called after a tool is stopped
    fn post_stop(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called when a tool is paused
    fn on_pause(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called when a tool is resumed
    fn on_resume(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called when a tool is updated
    fn on_update(&self, _tool: &Tool) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called when a tool is being cleaned up
    fn on_cleanup(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called when registering a tool (new name for on_register)
    fn register_tool(&self, tool: &Tool) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        self.on_register(tool)
    }

    /// Called when initializing a tool after registration
    fn initialize_tool(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called before executing a tool
    fn pre_execute(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called after executing a tool
    fn post_execute(&self, _tool_id: &str, result: Result<(), ToolError>) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async move { result }
    }

    /// Called when resetting a tool
    fn reset_tool(&self, _tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async { Ok(()) }
    }

    /// Called when cleaning up a tool (new name for on_cleanup)
    fn cleanup_tool(&self, tool_id: &str) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        self.on_cleanup(tool_id)
    }
}

