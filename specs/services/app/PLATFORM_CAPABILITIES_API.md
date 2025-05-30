# Plugin Sandbox Platform Capabilities API

## Overview

The Platform Capabilities API provides a comprehensive way to detect and query security and sandbox features available on the current platform. This documentation explains how to use the API to determine which features are available, implement graceful degradation for unsupported features, and provide platform-specific optimizations.

## Key Components

### CrossPlatformSandbox

The `CrossPlatformSandbox` is the central component that provides a unified interface to platform-specific implementations. It automatically selects the appropriate implementation based on the current platform and exposes a consistent API for sandbox operations.

```rust
// Create a cross-platform sandbox
let resource_monitor = Arc::new(ResourceMonitor::new());
let sandbox = CrossPlatformSandbox::new(resource_monitor)?;

// Get platform capabilities
let capabilities = sandbox.get_platform_capabilities();

// Check if a specific capability is available
if capabilities.contains("seccomp") {
    // Use seccomp-specific features
}
```

### Platform Capability Detection

The `get_platform_capabilities()` method returns a `HashSet<String>` containing the names of all available capabilities on the current platform. These capabilities are determined at runtime based on:

1. Basic platform identification (Windows, Linux, macOS)
2. Platform-specific feature detection
3. Runtime checks for specific capabilities
4. Resource monitoring capabilities

```rust
// Example capabilities for a Windows system
[
    "basic_isolation",
    "resource_monitoring",
    "path_validation",
    "plugin_lifecycle",
    "windows_job_objects",
    "process_priority_control",
    "memory_limits",
    "cpu_limits",
    "process_limits",
    "integrity_levels",
    "desktop_isolation",
    "network_isolation",
    "app_container",
    "native_sandbox",
    "advanced_resource_monitoring",
    "detailed_resource_metrics",
    "resource_throttling"
]
```

## Available Capabilities

### Common Capabilities (All Platforms)

| Capability | Description |
|------------|-------------|
| `basic_isolation` | Basic plugin isolation features |
| `resource_monitoring` | Resource usage monitoring |
| `path_validation` | Path-based access control |
| `plugin_lifecycle` | Plugin lifecycle management |
| `native_sandbox` | Native sandbox implementation available |
| `advanced_resource_monitoring` | Enhanced resource monitoring features |
| `detailed_resource_metrics` | Detailed process metrics available |
| `resource_throttling` | Resource throttling capabilities |

### Windows-Specific Capabilities

| Capability | Description | Detection Method |
|------------|-------------|------------------|
| `windows_job_objects` | Windows Job Objects for process isolation | Platform detection |
| `process_priority_control` | Process priority manipulation | Platform detection |
| `memory_limits` | Memory usage limitations | Platform detection |
| `cpu_limits` | CPU usage limitations | Platform detection |
| `process_limits` | Process creation limitations | Platform detection |
| `integrity_levels` | Windows integrity levels | Runtime check |
| `desktop_isolation` | Separate desktop for UI isolation | Runtime check |
| `network_isolation` | Network filtering capabilities | Runtime check |
| `app_container` | Windows App Container support | Runtime check |

### Linux-Specific Capabilities

| Capability | Description | Detection Method |
|------------|-------------|------------------|
| `cgroups` | Control groups for resource management | Platform detection |
| `process_limits` | Process creation limitations | Platform detection |
| `memory_limits` | Memory usage limitations | Platform detection |
| `cpu_limits` | CPU usage limitations | Platform detection |
| `io_limits` | Disk I/O limitations | Platform detection |
| `cgroups_v2` | Modern unified cgroups hierarchy | Runtime check |
| `seccomp` | System call filtering | Runtime check |
| `seccomp_advanced` | Advanced system call filtering | Runtime check |
| `syscall_arg_filtering` | Argument-based syscall filtering | Runtime check |
| `namespaces` | Linux namespaces for isolation | Runtime check |
| `namespace_user` | User namespace support | Runtime check |
| `namespace_pid` | PID namespace support | Runtime check |
| `namespace_net` | Network namespace support | Runtime check |
| `namespace_mnt` | Mount namespace support | Runtime check |
| `namespace_ipc` | IPC namespace support | Runtime check |
| `namespace_uts` | UTS namespace support | Runtime check |

### macOS-Specific Capabilities

| Capability | Description | Detection Method |
|------------|-------------|------------------|
| `resource_limits` | Resource usage limitations | Platform detection |
| `memory_limits` | Memory usage limitations | Platform detection |
| `cpu_limits` | CPU usage limitations | Platform detection |
| `app_sandbox` | macOS App Sandbox | Runtime check |
| `system_integrity_protection` | System Integrity Protection (SIP) | Runtime check |
| `transparency_consent_control` | Transparency, Consent, and Control (TCC) | Runtime check |

## Implementing Graceful Degradation

The sandbox system provides built-in support for graceful degradation when features are not available on the current platform. Use the `apply_feature_with_degradation()` method to request a feature with automatic fallback to alternative implementations:

```rust
// Try to apply a feature with graceful degradation
match sandbox.apply_feature_with_degradation(plugin_id, "process_isolation").await {
    Ok(true) => {
        // Feature was applied with native implementation
        println!("Process isolation enabled with native support");
    },
    Ok(false) => {
        // Feature was applied with fallback implementation
        println!("Process isolation enabled with fallback support");
    },
    Err(e) => {
        // Feature could not be applied
        println!("Failed to enable process isolation: {}", e);
    }
}
```

### Fallback Behavior

When a feature is not available or fails to apply, the system will:

1. Attempt to use a platform-specific fallback if available
2. Fall back to a cross-platform alternative implementation
3. Use a basic path-based restriction as a last resort
4. Provide clear error messages explaining why the feature could not be applied

### Security Considerations

- Security-related errors will not trigger fallbacks to prevent security bypasses
- All fallback implementations maintain the security guarantees of the original feature
- Path-based restrictions are always enforced regardless of other features
- All fallback actions are logged for audit purposes

## Resource Monitoring Capabilities

The resource monitoring system provides platform-specific implementations with capabilities that can be queried:

```rust
// Check if advanced metrics are available
if ResourceMonitor::has_advanced_metrics() {
    // Use advanced metrics
}

// Check if resource throttling is supported
if ResourceMonitor::supports_resource_throttling() {
    // Use resource throttling
}
```

### Advanced Metrics

Platform-specific advanced metrics include:

- **Windows**: Performance Data Helper (PDH) metrics
- **Linux**: Detailed memory mapping via `/proc/[pid]/smaps` and `perf` integration
- **macOS**: DTrace-based process monitoring

### Resource Throttling

Platform-specific resource throttling includes:

- **Windows**: Job Objects with CPU rate control
- **Linux**: cgroups CPU and memory controllers
- **macOS**: Resource limits via `setrlimit()`

## Error Handling

The system provides standardized error handling across all platforms through the `standardize_error()` method. This ensures consistent error messages and proper categorization regardless of the underlying platform implementation.

```rust
// Example of standardized error handling
match sandbox.check_permission(plugin_id, "filesystem:write").await {
    Ok(_) => {
        // Permission granted
    },
    Err(e) => {
        // Error is standardized with proper context
        println!("Permission error: {}", e);
    }
}
```

### Error Categories

Errors are categorized into specific types:

- `PluginNotFound`: Plugin not registered with the sandbox
- `Permission`: Permission denied for an operation
- `ResourceLimit`: Resource limit exceeded
- `PathAccess`: Path access denied
- `Capability`: Capability not allowed
- `Platform`: Platform-specific error
- `Unsupported`: Feature not supported on the platform
- `Internal`: Internal error in the sandbox implementation

## Best Practices

1. **Always check capabilities before using platform-specific features**:
   ```rust
   if capabilities.contains("seccomp_advanced") {
       // Use advanced seccomp features
   } else if capabilities.contains("seccomp") {
       // Use basic seccomp features
   } else {
       // Fall back to alternative
   }
   ```

2. **Prefer apply_feature_with_degradation for automatic fallbacks**:
   ```rust
   sandbox.apply_feature_with_degradation(plugin_id, "process_isolation").await?;
   ```

3. **Handle security errors specially**:
   ```rust
   match result {
       Err(SquirrelError::Security(msg)) => {
           // Security error, do not bypass
           return Err(SquirrelError::Security(msg));
       },
       Err(e) => {
           // Try fallback for other errors
       },
       Ok(_) => { /* Success */ }
   }
   ```

4. **Log degradation events**:
   ```rust
   if let Ok(false) = result {
       warn!("Using fallback implementation for {}", feature);
   }
   ```

5. **Check platform name for platform-specific optimizations**:
   ```rust
   match sandbox.get_platform_name().as_str() {
       "windows" => { /* Windows-specific code */ },
       "linux" => { /* Linux-specific code */ },
       "macos" => { /* macOS-specific code */ },
       _ => { /* Fallback code */ }
   }
   ```

## Example: Implementing a New Feature With Cross-Platform Support

```rust
async fn apply_network_isolation(&self, plugin_id: Uuid) -> Result<()> {
    let capabilities = self.get_platform_capabilities();
    
    // Check for platform-specific capabilities
    if capabilities.contains("network_isolation") {
        // Use native network isolation
        self.apply_feature(plugin_id, "network_isolation").await
    } else if capabilities.contains("namespace_net") {
        // Use Linux network namespace as alternative
        self.apply_feature(plugin_id, "namespace_net").await
    } else {
        // Fall back to basic port restrictions
        warn!("Native network isolation not available, using port restrictions");
        self.apply_basic_network_restrictions(plugin_id).await
    }
}
```

## References

- [Sandbox Implementation Summary](SANDBOX_IMPLEMENTATION_SUMMARY.md)
- [Implementation Progress](IMPLEMENTATION_PROGRESS.md)
- [Cross-Platform Integration Update](IMPLEMENTATION_UPDATE_2024_08_10.md)
- [Task Tracking](TASK_TRACKING.md) 