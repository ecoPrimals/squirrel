use super::*;
use crate::tool::{Tool, Capability};
use std::collections::HashMap;
use tokio::test;

/// Creates a test tool for testing
fn create_test_tool(id: &str, security_level: u8) -> Tool {
    Tool {
        id: id.to_string(),
        name: format!("Test Tool {}", id),
        version: "1.0.0".to_string(),
        description: "Test tool for unit tests".to_string(),
        capabilities: vec![
            Capability {
                name: "test_capability".to_string(),
                description: "Test capability".to_string(),
                parameters: vec![],
                return_type: None,
            }
        ],
        security_level,
    }
}

#[test]
async fn test_resource_tracker_basic() {
    let tracker = ResourceTracker::default();
    let tool_id = "test-tool";
    
    // Initialize tool
    tracker.initialize_tool(tool_id).await.expect("Failed to initialize tool");
    
    // Track resources
    tracker.track_memory_allocation(tool_id, 1024 * 1024).await.expect("Failed to track memory");
    tracker.track_cpu_time(tool_id, 100).await.expect("Failed to track CPU time");
    tracker.track_file_handle(tool_id, 1).await.expect("Failed to track file handle");
    tracker.track_network_connection(tool_id, 1).await.expect("Failed to track network connection");
    
    // Get usage
    let usage = tracker.get_usage(tool_id).await.expect("Failed to get usage");
    
    // Verify usage
    assert_eq!(usage.memory_bytes, 1024 * 1024);
    assert_eq!(usage.cpu_time_ms, 100);
    assert_eq!(usage.file_handles.len(), 1);
    assert_eq!(usage.network_connections.len(), 1);
    
    // Clean up
    tracker.cleanup_tool(tool_id).await.expect("Failed to clean up tool");
}

#[test]
async fn test_enhanced_resource_cleanup_hook() {
    let hook = EnhancedResourceCleanupHook::new();
    let tool = create_test_tool("test-tool", 5); // Medium security level
    
    // Register tool
    hook.on_register(&tool).await.expect("Failed to register tool");
    
    // Get tracker
    let tracker = hook.get_tracker();
    
    // Verify tool was initialized
    let usage = tracker.get_usage(&tool.id).await.expect("Failed to get usage");
    assert_eq!(usage.memory_bytes, 0);
    
    // Track some resources
    tracker.track_memory_allocation(&tool.id, 1024 * 1024).await.expect("Failed to track memory");
    
    // Verify resources were tracked
    let usage = tracker.get_usage(&tool.id).await.expect("Failed to get usage");
    assert_eq!(usage.memory_bytes, 1024 * 1024);
    
    // Deactivate tool
    hook.on_deactivate(&tool.id).await.expect("Failed to deactivate tool");
    
    // Verify resources were released
    let usage = tracker.get_usage(&tool.id).await.expect("Failed to get usage");
    assert_eq!(usage.memory_bytes, 0);
    
    // Unregister tool
    hook.on_unregister(&tool.id).await.expect("Failed to unregister tool");
    
    // Verify tool was cleaned up
    let result = tracker.get_usage(&tool.id).await;
    assert!(result.is_err());
}

#[test]
async fn test_security_based_limits() {
    // Create tools with different security levels
    let low_security_tool = create_test_tool("low-security", 2);
    let medium_security_tool = create_test_tool("medium-security", 5);
    let high_security_tool = create_test_tool("high-security", 8);
    
    let hook = EnhancedResourceCleanupHook::new();
    let tracker = hook.get_tracker();
    
    // Register tools
    hook.on_register(&low_security_tool).await.expect("Failed to register low security tool");
    hook.on_register(&medium_security_tool).await.expect("Failed to register medium security tool");
    hook.on_register(&high_security_tool).await.expect("Failed to register high security tool");
    
    // Get limits
    let low_limits = tracker.get_limits(&low_security_tool.id).await.expect("Failed to get low security limits");
    let medium_limits = tracker.get_limits(&medium_security_tool.id).await.expect("Failed to get medium security limits");
    let high_limits = tracker.get_limits(&high_security_tool.id).await.expect("Failed to get high security limits");
    
    // Verify limits are proportional to security level
    assert!(low_limits.max_memory_bytes < medium_limits.max_memory_bytes);
    assert!(medium_limits.max_memory_bytes < high_limits.max_memory_bytes);
    
    // Verify file handle and network connection limits
    assert!(low_limits.max_file_handles < medium_limits.max_file_handles);
    assert!(medium_limits.max_file_handles < high_limits.max_file_handles);
    
    assert!(low_limits.max_network_connections < medium_limits.max_network_connections);
    assert!(medium_limits.max_network_connections < high_limits.max_network_connections);
}

#[test]
async fn test_resource_limit_warnings() {
    let tool = create_test_tool("test-tool", 5);
    let hook = EnhancedResourceCleanupHook::new();
    let tracker = hook.get_tracker();
    
    // Register tool
    hook.on_register(&tool).await.expect("Failed to register tool");
    
    // Set custom limits for testing
    let custom_limits = ResourceLimits {
        max_memory_bytes: 10 * 1024 * 1024, // 10MB
        max_cpu_time_ms: 1000,              // 1 second
        max_file_handles: 5,
        max_network_connections: 2,
    };
    
    tracker.set_limits(&tool.id, custom_limits).await.expect("Failed to set limits");
    
    // Test memory allocation - normal
    let status = tracker.track_memory_allocation(&tool.id, 2 * 1024 * 1024).await
        .expect("Failed to track memory");
    assert_eq!(status, ResourceStatus::Normal);
    
    // Test memory allocation - warning
    let status = tracker.track_memory_allocation(&tool.id, 6 * 1024 * 1024).await
        .expect("Failed to track memory");
    assert_eq!(status, ResourceStatus::Warning);
    
    // Test memory allocation - critical
    let status = tracker.track_memory_allocation(&tool.id, 3 * 1024 * 1024).await
        .expect("Failed to track memory");
    assert_eq!(status, ResourceStatus::Critical);
    
    // Test file handle allocation
    for i in 0..5 {
        let status = tracker.track_file_handle(&tool.id, i).await
            .expect("Failed to track file handle");
        
        if i < 3 {
            assert_eq!(status, ResourceStatus::Normal);
        } else if i < 4 {
            assert_eq!(status, ResourceStatus::Warning);
        } else {
            assert_eq!(status, ResourceStatus::Critical);
        }
    }
}

#[test]
async fn test_resource_tracking_history() {
    let tool = create_test_tool("test-tool", 5);
    let hook = EnhancedResourceCleanupHook::new();
    let tracker = hook.get_tracker();
    
    // Register tool
    hook.on_register(&tool).await.expect("Failed to register tool");
    
    // Track resources multiple times
    for i in 0..5 {
        tracker.track_memory_allocation(&tool.id, 1024 * 1024).await
            .expect("Failed to track memory");
        
        tracker.track_cpu_time(&tool.id, 100).await
            .expect("Failed to track CPU time");
    }
    
    // Get history
    let history = tracker.get_history(&tool.id).await.expect("Failed to get history");
    
    // Verify history
    assert!(history.len() >= 5);
    
    // Verify history records
    for record in history {
        assert_eq!(record.tool_id, tool.id);
        assert!(record.usage.memory_bytes > 0);
        assert!(record.usage.cpu_time_ms > 0);
    }
}

#[test]
async fn test_composite_lifecycle_hook_with_resource_tracking() {
    // Create a composite hook with EnhancedResourceCleanupHook
    let composite_hook = CompositeLifecycleHook::new(vec![
        Arc::new(BasicLifecycleHook::new()),
        Arc::new(EnhancedResourceCleanupHook::new()),
    ]);
    
    let tool = create_test_tool("test-tool", 5);
    
    // Register tool
    composite_hook.on_register(&tool).await.expect("Failed to register tool");
    
    // Verify all hooks were called
    // (This is hard to test directly, but we can ensure it doesn't fail)
    
    // Unregister tool
    composite_hook.on_unregister(&tool.id).await.expect("Failed to unregister tool");
}

#[test]
async fn test_resource_tracking_in_error_recovery() {
    let hook = EnhancedResourceCleanupHook::new();
    let tool = create_test_tool("test-tool", 5);
    let tracker = hook.get_tracker();
    
    // Register tool
    hook.on_register(&tool).await.expect("Failed to register tool");
    
    // Track some resources
    tracker.track_memory_allocation(&tool.id, 5 * 1024 * 1024).await.expect("Failed to track memory");
    tracker.track_file_handle(&tool.id, 1).await.expect("Failed to track file handle");
    
    // Simulate a resource-related error
    let error = ToolError::ExecutionFailed("Exceeded memory resources".to_string());
    hook.on_error(&tool.id, &error).await.expect("Failed to handle error");
    
    // Verify resources were released
    let usage = tracker.get_usage(&tool.id).await.expect("Failed to get usage");
    assert_eq!(usage.memory_bytes, 0);
    assert_eq!(usage.file_handles.len(), 0);
    
    // Simulate a non-resource-related error
    let error = ToolError::ValidationFailed("Invalid parameter".to_string());
    
    // Track resources again
    tracker.track_memory_allocation(&tool.id, 1024 * 1024).await.expect("Failed to track memory");
    
    // Handle error
    hook.on_error(&tool.id, &error).await.expect("Failed to handle error");
    
    // Verify resources were not released (since it's not a resource-related error)
    let usage = tracker.get_usage(&tool.id).await.expect("Failed to get usage");
    assert_eq!(usage.memory_bytes, 1024 * 1024);
} 