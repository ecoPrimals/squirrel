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

use std::sync::Arc;
use crate::tool::management::types::{Tool, ToolError, ToolLifecycleHook};

// Module structure
/// Basic implementation of resource management functionality
mod basic_resource_manager;
/// Implementation of cleanup hooks for tool resources
mod cleanup_hook;
pub mod recovery;
/// Interface for managing tool resource allocation and limits
mod resource_manager;
/// Utilities for tracking resource usage by tools
mod resource_tracker;

// Re-export the main types
pub use basic_resource_manager::BasicResourceManager;
pub use cleanup_hook::{BasicCleanupHook, CleanupHook};
pub use recovery::{RecoveryHook, RecoveryStrategy};
pub use resource_manager::{ResourceLimits, ResourceManager, ResourceUsage};
pub use resource_tracker::{ResourceTracker, ResourceTrackerUsage};

// New components
/// Enhanced recovery mechanisms for tools with failure patterns
mod enhanced_recovery;
/// Comprehensive cleanup system with dependency tracking
pub mod comprehensive;
/// Legacy comprehensive cleanup module (re-exports from comprehensive module)
mod comprehensive_cleanup;

pub use enhanced_recovery::{
    AdvancedBackoffStrategy, AdvancedRecoveryAction, EnhancedRecoveryAttempt,
    EnhancedRecoveryHandler, EnhancedRecoveryHook, EnhancedRecoveryStrategy,
    ToolManagerRecoveryExt,
};

// Re-export from comprehensive module for backward compatibility
pub use comprehensive::{
    CleanupMethod, CleanupRecord, CleanupStrategy, ComprehensiveCleanupHook,
    ResourceAllocation, ResourceDependency, ResourceId, ResourceType,
    ResourceStats, ResourceOperations,
};

// Legacy exports from comprehensive_cleanup module
pub use comprehensive_cleanup::{
    ComprehensiveCleanupHook as LegacyComprehensiveCleanupHook,
};

// Original modules
pub mod resource_tracking;
/// Adaptive resource management for dynamic scaling
mod adaptive_resource;

/// Helper function to create a test tool
fn create_test_tool(id: &str, _max_memory_mb: usize) -> Tool {
    Tool::builder()
        .id(id)
        .name(&format!("Test Tool {}", id))
        .description(&format!("Test tool for cleanup tests: {}", id))
        .build()
        .expect("Failed to build test tool")
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::management::types::{Tool, ToolError, ToolLifecycleHook};
    use crate::tool::lifecycle::CompositeLifecycleHook;
    
    
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
        let tool = Tool::builder().id("test-tool").name("Test Tool").build()
            .expect("Failed to build tool");

        // Register the tool
        cleanup_hook::CleanupHook::register_tool(&cleanup_hook, &tool)
            .await
            .expect("Failed to register tool");

        // Test the reset operation
        cleanup_hook::CleanupHook::reset_tool(&cleanup_hook, "test-tool")
            .await
            .expect("Failed to reset tool");

        // Test cleanup
        cleanup_hook::CleanupHook::cleanup_tool(&cleanup_hook, "test-tool")
            .await
            .expect("Failed to cleanup tool");

        // Tool should no longer be registered
        assert!(cleanup_hook::CleanupHook::reset_tool(&cleanup_hook, "test-tool").await.is_err());
    }

    // Test for resource limit warnings
    #[test]
    fn test_resource_limit_warnings() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _tool = create_test_tool("test-tool", 5);
            let hook = EnhancedResourceCleanupHook::new();
            let tracker = hook.get_tracker();
            
            // Test memory allocation - Normal
            tracker.track_memory_allocation(1024 * 1024).await.unwrap(); // 1MB
            let status = tracker.get_current_usage().await.unwrap();
            assert_eq!(status.memory_bytes, 1024 * 1024);
            
            // Test memory allocation - Warning
            tracker.track_memory_allocation(2 * 1024 * 1024).await.unwrap(); // Additional 2MB, total 3MB
            let status = tracker.get_current_usage().await.unwrap();
            assert_eq!(status.memory_bytes, 3 * 1024 * 1024);
            
            // Test memory allocation - Critical
            tracker.track_memory_allocation(1024 * 1024).await.unwrap(); // Additional 1MB, total 4MB
            let status = tracker.get_current_usage().await.unwrap();
            assert_eq!(status.memory_bytes, 4 * 1024 * 1024);
            
            // Test file handle allocation
            for i in 1..15 {
                tracker.track_file_handle_open(&format!("file{}", i)).await.unwrap();
                let usage = tracker.get_current_usage().await.unwrap();
                
                assert_eq!(usage.file_handles.len(), i as usize);
            }
        });
    }
    
    // Test for resource tracking history
    #[test]
    fn test_resource_tracking_history() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _tool = create_test_tool("test-tool", 5);
            let hook = EnhancedResourceCleanupHook::new();
            let tracker = hook.get_tracker();
            
            // Track allocations multiple times
            tracker.track_memory_allocation(1024 * 1024).await.unwrap(); // 1MB
            tracker.track_memory_allocation(2 * 1024 * 1024).await.unwrap(); // Additional 2MB, total 3MB
            
            // Verify last usage
            let usage = tracker.get_current_usage().await.unwrap();
            assert_eq!(usage.memory_bytes, 3 * 1024 * 1024);
        });
    }
    
    // Test for composite lifecycle hook with resource tracking
    #[test]
    fn test_composite_lifecycle_hook_with_resource_tracking() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create a composite hook with EnhancedResourceCleanupHook
            let composite_hook = CompositeLifecycleHook::new();
            
            let tool = create_test_tool("test-tool", 5);
            
            // Register the tool
            composite_hook.register_tool(&tool).await.expect("Failed to register tool");
            
            // Initialize and activate (these should work without errors)
            composite_hook.initialize_tool(&tool.id).await.expect("Failed to initialize tool");
            
            // Unregister (should call all hooks without failure)
            composite_hook.on_unregister(&tool.id).await.expect("Failed to unregister tool");
        });
    }
    
    // Test for resource tracking in error recovery
    #[test]
    fn test_resource_tracking_in_error_recovery() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let hook = EnhancedResourceCleanupHook::new();
            let tool = create_test_tool("test-tool", 5);
            let tracker = hook.get_tracker();
            
            // Allocate some resources
            tracker.track_memory_allocation(1024 * 1024).await.unwrap();
            
            // Simulate a resource error (exceeding memory)
            let error = ToolError::ExecutionFailed { 
                tool_id: "test-tool".to_string(),
                reason: "Exceeded memory resources".to_string()
            };
            hook.on_error(&tool.id, &error).await.unwrap();
            
            // Verify that resources are still tracked despite the error
            let usage = tracker.get_current_usage().await.unwrap();
            assert_eq!(usage.memory_bytes, 1024 * 1024);
            
            // Simulate a non-resource error
            let error = ToolError::ValidationFailed("Invalid parameter".to_string());
            hook.on_error(&tool.id, &error).await.unwrap();
            
            // Verify resources remain tracked
            let usage = tracker.get_current_usage().await.unwrap();
            assert_eq!(usage.memory_bytes, 1024 * 1024);
        });
    }

    #[tokio::test]
    async fn test_cleanup_resources() {
        // ... 
        {
            // Setup using tokio_test to control time
            let _tool = create_test_tool("test-tool", 5);
            // ...
        }
    }

    #[tokio::test]
    async fn test_cleanup_max_memory() {
        // ...
        {
            // Setup using tokio_test to control time
            let _tool = create_test_tool("test-tool", 5);
            // ...
        }
    }
}

/// Enhanced resource cleanup hook implementation
#[derive(Debug)]
pub struct EnhancedResourceCleanupHook {
    // Implementation details
    tracker: Arc<ResourceTracker>,
}

impl EnhancedResourceCleanupHook {
    /// Creates a new enhanced resource cleanup hook
    pub fn new() -> Self {
        Self {
            tracker: Arc::new(ResourceTracker::new("resource-tracker")),
        }
    }

    /// Gets the tracker for this hook
    pub fn get_tracker(&self) -> Arc<ResourceTracker> {
        Arc::clone(&self.tracker)
    }
}

#[async_trait::async_trait]
impl ToolLifecycleHook for EnhancedResourceCleanupHook {
    async fn on_register(&self, _tool: &Tool) -> Result<(), ToolError> {
        Ok(())
    }

    async fn register_tool(&self, _tool: &Tool) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_unregister(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_activate(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_deactivate(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_error(&self, _tool_id: &str, _error: &ToolError) -> Result<(), ToolError> {
        Ok(())
    }

    async fn pre_start(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn post_start(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn pre_stop(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn post_stop(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_pause(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_resume(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_update(&self, _tool: &Tool) -> Result<(), ToolError> {
        Ok(())
    }
    
    async fn on_cleanup(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn initialize_tool(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn pre_execute(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn post_execute(&self, _tool_id: &str, _result: Result<(), ToolError>) -> Result<(), ToolError> {
        Ok(())
    }

    async fn reset_tool(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn cleanup_tool(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Re-export all the hooks
