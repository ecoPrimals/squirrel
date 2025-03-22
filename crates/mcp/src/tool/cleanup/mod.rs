//! Tool cleanup and resource management.
//!
//! This module provides utilities for managing tool resources and ensuring
//! proper cleanup after tool execution. It includes mechanisms for tracking
//! resource usage, enforcing resource limits, and cleaning up resources.
//!
//! Key components:
//! - `ResourceManager`: Interface for managing tool resources
//! - `BasicResourceManager`: Standard implementation of resource management
//! - `ResourceTracker`: Tracks resource usage for individual tools
//! - `CleanupHook`: Trait for hooks that perform cleanup actions
//! - `RecoveryHook`: Hook for handling tool errors and recovery

// Module structure
mod basic_resource_manager;
mod cleanup_hook;
pub mod recovery;
mod resource_manager;
mod resource_tracker;

// Re-export the main types
pub use basic_resource_manager::BasicResourceManager;
pub use cleanup_hook::{BasicCleanupHook, CleanupHook};
pub use recovery::{RecoveryHook, RecoveryStrategy};
pub use resource_manager::{ResourceLimits, ResourceManager, ResourceUsage};
pub use resource_tracker::{ResourceTracker, ResourceTrackerUsage};

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::Tool;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_basic_resource_manager() {
        let rm = BasicResourceManager::new();

        // Initialize a tool
        rm.initialize_tool(
            "test-tool",
            ResourceLimits::default(),
            ResourceLimits::default(),
        )
        .await
        .expect("Failed to initialize tool");

        // Check initial usage
        let usage = rm
            .get_usage("test-tool")
            .await
            .expect("Failed to get usage");
        assert_eq!(usage.memory_bytes, 0);
        assert_eq!(usage.cpu_time_ms, 0);
        assert_eq!(usage.file_handles, 0);
        assert_eq!(usage.network_connections, 0);

        // Test limits checking
        let within_limits = rm
            .check_limits("test-tool")
            .await
            .expect("Failed to check limits");
        assert!(within_limits);

        // Test cleanup
        rm.cleanup_tool("test-tool")
            .await
            .expect("Failed to cleanup tool");

        // Test non-existent tool
        assert!(rm.get_usage("non-existent").await.is_err());
    }

    #[tokio::test]
    async fn test_basic_cleanup_hook() {
        let rm = Arc::new(BasicResourceManager::new());
        let cleanup_hook = BasicCleanupHook::new(rm);

        // Create a mock tool
        let tool = Tool::builder().id("test-tool").name("Test Tool").build();

        // Register the tool
        cleanup_hook
            .register_tool(&tool)
            .await
            .expect("Failed to register tool");

        // Test the reset operation
        cleanup_hook
            .reset_tool("test-tool")
            .await
            .expect("Failed to reset tool");

        // Test cleanup
        cleanup_hook
            .cleanup_tool("test-tool")
            .await
            .expect("Failed to cleanup tool");

        // Tool should no longer be registered
        assert!(cleanup_hook.reset_tool("test-tool").await.is_err());
    }
}
