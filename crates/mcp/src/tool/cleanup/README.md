# MCP Tool Cleanup and Recovery System

This module provides a comprehensive resource management and error recovery system for the MCP Tool Management System. It's designed to handle resource tracking, cleanup procedures, and advanced error recovery strategies during tool lifecycle events.

## Components

### ResourceCleanupHook

A lifecycle hook that tracks and manages resources used by tools:

- **Resource Usage Tracking**: Monitors memory, CPU, file handles, and network connections
- **Resource Limit Enforcement**: Sets and enforces resource limits based on tool security level
- **Automatic Cleanup**: Performs cleanup operations during tool lifecycle events
- **Resource Registration**: Provides APIs for registering and unregistering resources

### RecoveryHook

A lifecycle hook that handles error recovery for tools:

- **Progressive Recovery Strategies**: Implements escalating recovery strategies (Retry → Reset → Restart → Isolate → Unregister)
- **Error History Tracking**: Records error history and recovery attempts
- **Success Rate Monitoring**: Tracks recovery success rates for performance metrics
- **Strategy Selection**: Intelligently selects recovery strategies based on failure patterns

### ResourceUsage and ResourceLimits

Data structures for tracking and limiting tool resource usage:

- **ResourceUsage**: Tracks memory, CPU, file handles, and network connections
- **ResourceLimits**: Defines maximum allowed resources for a tool

## Usage Examples

### Basic Resource Cleanup

```rust
use mcp::tool::{ToolManager, ResourceCleanupHook, CompositeLifecycleHook};

// Create a resource cleanup hook
let cleanup_hook = ResourceCleanupHook::new();

// Create a composite hook including the cleanup hook
let mut composite_hook = CompositeLifecycleHook::new();
composite_hook.add_hook(cleanup_hook);

// Create a tool manager with the hook
let manager = ToolManager::new(composite_hook);

// Resources will be automatically tracked and cleaned up during tool lifecycle events
```

### Setting Custom Resource Limits

```rust
use mcp::tool::{ResourceCleanupHook, ResourceLimits};

// Create a resource cleanup hook
let cleanup_hook = ResourceCleanupHook::new();

// Set custom resource limits for a tool
let limits = ResourceLimits::default()
    .with_max_memory(1024 * 1024 * 50)      // 50 MB
    .with_max_cpu_time(30 * 1000)           // 30 seconds
    .with_max_file_handles(50)
    .with_max_network_connections(10);

// Apply the limits to a tool
cleanup_hook.set_limits("my-tool-id", limits).await;
```

### Combining Cleanup and Recovery

```rust
use mcp::tool::{
    ToolManager, ResourceCleanupHook, RecoveryHook,
    CompositeLifecycleHook
};

// Create hooks
let cleanup_hook = ResourceCleanupHook::new();
let recovery_hook = RecoveryHook::new()
    .with_max_recovery_attempts(5)
    .with_retry_interval(2000);

// Create a composite hook with both
let mut composite_hook = CompositeLifecycleHook::new();
composite_hook.add_hook(cleanup_hook);
composite_hook.add_hook(recovery_hook);

// Create a tool manager with the hooks
let manager = ToolManager::new(composite_hook);

// Now both resource cleanup and error recovery will be performed
// during tool lifecycle events
```

## Testing

The module includes comprehensive tests for:

- Resource tracking and limit enforcement
- File handle and network connection management
- Recovery strategy selection
- Recovery attempt history tracking
- Success rate calculation

## Best Practices

1. **Always use CompositeLifecycleHook** to combine cleanup with other hooks
2. **Set appropriate resource limits** based on tool security level and requirements
3. **Register critical resources** (file handles, network connections) for proper tracking
4. **Configure recovery attempts** based on tool criticality
5. **Monitor success rates** to identify problematic tools 