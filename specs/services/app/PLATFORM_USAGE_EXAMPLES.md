# Plugin Sandbox Platform Usage Examples

This document provides concrete usage examples for the plugin sandbox system across different platforms. It demonstrates how to use platform-specific features while maintaining cross-platform compatibility.

## Basic Cross-Platform Usage

These examples work consistently across all supported platforms.

### Creating and Using a Cross-Platform Sandbox

```rust
use crate::plugin::sandbox::CrossPlatformSandbox;
use crate::plugin::resource_monitor::ResourceMonitor;
use crate::plugin::security::{SecurityContext, PermissionLevel};
use std::sync::Arc;
use uuid::Uuid;

async fn sandbox_example() -> Result<()> {
    // Create resource monitor
    let resource_monitor = Arc::new(ResourceMonitor::new());
    
    // Create sandbox with cross-platform support
    let sandbox = CrossPlatformSandbox::new(resource_monitor)?;
    
    // Create a plugin ID
    let plugin_id = Uuid::new_v4();
    
    // Create security context with restricted permissions
    let mut context = SecurityContext::default();
    context.permission_level = PermissionLevel::Restricted;
    context.allowed_capabilities.insert("plugin:execute".to_string());
    context.allowed_paths.push(std::env::current_dir()?);
    
    // Set security context for the plugin
    sandbox.set_security_context(plugin_id, context).await?;
    
    // Create sandbox for the plugin
    sandbox.create_sandbox(plugin_id).await?;
    
    // Check permission for an operation
    sandbox.check_permission(plugin_id, "plugin:execute").await?;
    
    // Track resource usage
    let usage = sandbox.track_resources(plugin_id).await?;
    println!("Memory usage: {} bytes", usage.memory_bytes);
    println!("CPU usage: {}%", usage.cpu_percent);
    
    // Clean up
    sandbox.destroy_sandbox(plugin_id).await?;
    
    Ok(())
}
```

### Using Platform Capabilities

```rust
async fn use_platform_capabilities(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    // Get available capabilities
    let capabilities = sandbox.get_platform_capabilities();
    
    // Print available capabilities
    println!("Available capabilities:");
    for capability in &capabilities {
        println!("  - {}", capability);
    }
    
    // Check for specific capabilities
    if capabilities.contains("memory_limits") {
        println!("Memory limits are supported");
        
        // Apply memory limits
        sandbox.apply_feature(plugin_id, "memory_limits").await?;
    }
    
    // Use graceful degradation for CPU limits
    let cpu_limits_native = sandbox.apply_feature_with_degradation(plugin_id, "cpu_limits").await?;
    if cpu_limits_native {
        println!("CPU limits applied with native implementation");
    } else {
        println!("CPU limits applied with fallback implementation");
    }
    
    Ok(())
}
```

## Windows-Specific Examples

### Using Windows Job Objects

```rust
#[cfg(target_os = "windows")]
async fn windows_specific_example(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    let capabilities = sandbox.get_platform_capabilities();
    
    // Check for Windows-specific capabilities
    if capabilities.contains("windows_job_objects") {
        println!("Using Windows Job Objects for isolation");
        
        // Apply process priority control (Windows-specific)
        if capabilities.contains("process_priority_control") {
            sandbox.apply_feature(plugin_id, "process_priority_control").await?;
        }
        
        // Apply desktop isolation if available
        if capabilities.contains("desktop_isolation") {
            sandbox.apply_feature(plugin_id, "desktop_isolation").await?;
        }
        
        // Apply network isolation if available
        if capabilities.contains("network_isolation") {
            sandbox.apply_feature(plugin_id, "network_isolation").await?;
        }
    }
    
    Ok(())
}
```

### Windows Desktop Isolation Example

```rust
#[cfg(target_os = "windows")]
async fn windows_desktop_isolation(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    let capabilities = sandbox.get_platform_capabilities();
    
    if capabilities.contains("desktop_isolation") {
        // Create a custom security context for UI isolation
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::User;
        context.allowed_capabilities.insert("ui:display".to_string());
        
        // Set the security context
        sandbox.set_security_context(plugin_id, context).await?;
        
        // Apply desktop isolation
        sandbox.apply_feature(plugin_id, "desktop_isolation").await?;
        
        println!("Windows desktop isolation applied");
    } else {
        // Fallback for older Windows versions
        println!("Desktop isolation not available, using basic isolation");
        sandbox.apply_feature(plugin_id, "basic_isolation").await?;
    }
    
    Ok(())
}
```

## Linux-Specific Examples

### Using Linux Seccomp Filtering

```rust
#[cfg(target_os = "linux")]
async fn linux_seccomp_example(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    let capabilities = sandbox.get_platform_capabilities();
    
    if capabilities.contains("seccomp") {
        println!("Using seccomp filtering for syscall restrictions");
        
        // Check for advanced seccomp features
        if capabilities.contains("seccomp_advanced") && capabilities.contains("syscall_arg_filtering") {
            println!("Using advanced seccomp with argument filtering");
            
            // Apply advanced seccomp filtering
            sandbox.apply_feature(plugin_id, "seccomp_advanced").await?;
        } else {
            // Apply basic seccomp filtering
            sandbox.apply_feature(plugin_id, "seccomp").await?;
        }
    } else {
        println!("Seccomp filtering not available, using basic isolation");
        sandbox.apply_feature(plugin_id, "basic_isolation").await?;
    }
    
    Ok(())
}
```

### Using Linux Namespaces

```rust
#[cfg(target_os = "linux")]
async fn linux_namespace_example(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    let capabilities = sandbox.get_platform_capabilities();
    
    if capabilities.contains("namespaces") {
        println!("Using Linux namespaces for isolation");
        
        // Apply specific namespaces if available
        if capabilities.contains("namespace_user") {
            sandbox.apply_feature(plugin_id, "namespace_user").await?;
        }
        
        if capabilities.contains("namespace_pid") {
            sandbox.apply_feature(plugin_id, "namespace_pid").await?;
        }
        
        if capabilities.contains("namespace_net") {
            sandbox.apply_feature(plugin_id, "namespace_net").await?;
        }
        
        if capabilities.contains("namespace_mnt") {
            sandbox.apply_feature(plugin_id, "namespace_mnt").await?;
        }
    } else {
        println!("Namespaces not available, using cgroups for isolation");
        
        if capabilities.contains("cgroups") {
            sandbox.apply_feature(plugin_id, "cgroups").await?;
        } else {
            sandbox.apply_feature(plugin_id, "basic_isolation").await?;
        }
    }
    
    Ok(())
}
```

### Using Linux cgroups v2

```rust
#[cfg(target_os = "linux")]
async fn linux_cgroups_example(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    let capabilities = sandbox.get_platform_capabilities();
    
    // Check for cgroups v2 support
    if capabilities.contains("cgroups_v2") {
        println!("Using cgroups v2 for resource limits");
        
        // Create a security context with strict resource limits
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::Restricted;
        context.resource_limits.max_cpu_percent = 50;
        context.resource_limits.max_memory_bytes = 512 * 1024 * 1024; // 512 MB
        
        // Set the security context
        sandbox.set_security_context(plugin_id, context).await?;
        
        // Apply resource limits
        sandbox.apply_feature(plugin_id, "memory_limits").await?;
        sandbox.apply_feature(plugin_id, "cpu_limits").await?;
        
        if capabilities.contains("io_limits") {
            sandbox.apply_feature(plugin_id, "io_limits").await?;
        }
    } else if capabilities.contains("cgroups") {
        println!("Using legacy cgroups for resource limits");
        sandbox.apply_feature(plugin_id, "cgroups").await?;
    } else {
        println!("Cgroups not available, using basic resource limits");
        sandbox.apply_feature(plugin_id, "basic_isolation").await?;
    }
    
    Ok(())
}
```

## macOS-Specific Examples

### Using macOS App Sandbox

```rust
#[cfg(target_os = "macos")]
async fn macos_sandbox_example(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    let capabilities = sandbox.get_platform_capabilities();
    
    if capabilities.contains("app_sandbox") {
        println!("Using macOS App Sandbox for isolation");
        
        // Create a security context with app sandbox restrictions
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::Restricted;
        context.allowed_capabilities.insert("file:read".to_string());
        context.allowed_capabilities.insert("network:client".to_string());
        
        // Set the security context
        sandbox.set_security_context(plugin_id, context).await?;
        
        // Apply app sandbox
        sandbox.apply_feature(plugin_id, "app_sandbox").await?;
    } else {
        println!("App Sandbox not available, using basic isolation");
        sandbox.apply_feature(plugin_id, "basic_isolation").await?;
    }
    
    Ok(())
}
```

### Using macOS TCC Permissions

```rust
#[cfg(target_os = "macos")]
async fn macos_tcc_example(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    let capabilities = sandbox.get_platform_capabilities();
    
    if capabilities.contains("transparency_consent_control") {
        println!("Using macOS TCC for permission management");
        
        // Create a security context with TCC permissions
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::User;
        
        // Add specific TCC-related capabilities
        context.allowed_capabilities.insert("tcc:camera".to_string());
        context.allowed_capabilities.insert("tcc:microphone".to_string());
        
        // Set the security context
        sandbox.set_security_context(plugin_id, context).await?;
        
        // Apply TCC permissions
        sandbox.apply_feature(plugin_id, "tcc_permissions").await?;
    } else {
        println!("TCC not available, using basic isolation");
        sandbox.apply_feature(plugin_id, "basic_isolation").await?;
    }
    
    Ok(())
}
```

## Cross-Platform Feature Implementation

This example shows how to implement a feature that works across all platforms with appropriate platform-specific optimizations.

```rust
async fn implement_cross_platform_feature(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    let capabilities = sandbox.get_platform_capabilities();
    let platform = sandbox.get_platform_name();
    
    println!("Implementing network isolation on platform: {}", platform);
    
    // Platform-specific implementations with fallbacks
    match platform.as_str() {
        "windows" => {
            if capabilities.contains("network_isolation") {
                // Windows Firewall integration
                sandbox.apply_feature(plugin_id, "network_isolation").await?;
            } else {
                // Fallback for Windows
                println!("Using port restriction fallback for Windows");
                apply_port_restrictions(sandbox, plugin_id).await?;
            }
        },
        "linux" => {
            if capabilities.contains("namespace_net") {
                // Linux network namespace
                sandbox.apply_feature(plugin_id, "namespace_net").await?;
            } else if capabilities.contains("seccomp") {
                // Seccomp socket filtering
                sandbox.apply_feature(plugin_id, "seccomp_network").await?;
            } else {
                // Fallback for Linux
                println!("Using port restriction fallback for Linux");
                apply_port_restrictions(sandbox, plugin_id).await?;
            }
        },
        "macos" => {
            if capabilities.contains("app_sandbox") {
                // macOS App Sandbox network restrictions
                sandbox.apply_feature(plugin_id, "app_sandbox_network").await?;
            } else {
                // Fallback for macOS
                println!("Using port restriction fallback for macOS");
                apply_port_restrictions(sandbox, plugin_id).await?;
            }
        },
        _ => {
            // Generic fallback for unknown platforms
            println!("Using port restriction fallback for unknown platform");
            apply_port_restrictions(sandbox, plugin_id).await?;
        }
    }
    
    println!("Network isolation implemented successfully");
    Ok(())
}

// Cross-platform fallback implementation
async fn apply_port_restrictions(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    // Implement basic port restrictions that work on all platforms
    println!("Applying basic port restrictions");
    
    // Restrict allowed capabilities
    let mut context = sandbox.get_security_context(plugin_id).await?;
    context.allowed_capabilities.remove("network:bind");
    context.allowed_capabilities.remove("network:listen");
    sandbox.set_security_context(plugin_id, context).await?;
    
    // Apply basic isolation
    sandbox.apply_feature(plugin_id, "basic_isolation").await?;
    
    Ok(())
}
```

## Error Handling Examples

### Standardized Error Handling

```rust
async fn error_handling_example(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    // Try to access a path
    let path = std::path::Path::new("/restricted/path");
    let result = sandbox.check_path_access(plugin_id, path, true).await;
    
    match result {
        Ok(_) => {
            println!("Path access granted");
        },
        Err(e) => {
            // Handle specific error types
            if let Some(error_string) = e.to_string().to_lowercase() {
                if error_string.contains("path access denied") {
                    println!("Path access denied: {}", path.display());
                    
                    // Try a different path
                    let allowed_path = std::env::current_dir()?.join("allowed_file.txt");
                    sandbox.check_path_access(plugin_id, &allowed_path, true).await?;
                    println!("Access granted to alternative path: {}", allowed_path.display());
                } else if error_string.contains("plugin not found") {
                    println!("Plugin not registered: {}", plugin_id);
                    
                    // Create the plugin sandbox first
                    sandbox.create_sandbox(plugin_id).await?;
                    println!("Plugin sandbox created, retrying operation");
                    
                    // Retry the operation
                    sandbox.check_path_access(plugin_id, path, true).await?;
                } else {
                    // Other errors
                    println!("Error: {}", e);
                }
            } else {
                // Unknown error
                println!("Unknown error: {}", e);
            }
        }
    }
    
    Ok(())
}
```

### Graceful Degradation with Error Recovery

```rust
async fn degradation_with_recovery(sandbox: &CrossPlatformSandbox, plugin_id: Uuid) -> Result<()> {
    println!("Attempting to apply advanced features with degradation");
    
    // Try to apply process isolation with graceful degradation
    match sandbox.apply_feature_with_degradation(plugin_id, "process_isolation").await {
        Ok(true) => {
            println!("Process isolation applied with native implementation");
        },
        Ok(false) => {
            println!("Process isolation applied with fallback implementation");
            
            // Apply additional security measures to compensate
            println!("Applying additional security measures");
            sandbox.apply_feature(plugin_id, "path_validation").await?;
            
            // Restrict capabilities further
            let mut context = sandbox.get_security_context(plugin_id).await?;
            context.allowed_capabilities.clear();
            context.allowed_capabilities.insert("file:read".to_string());
            sandbox.set_security_context(plugin_id, context).await?;
        },
        Err(e) => {
            println!("Failed to apply process isolation: {}", e);
            
            // Apply strong fallback security
            println!("Applying strong fallback security");
            
            // Set highly restricted context
            let mut context = SecurityContext::default();
            context.permission_level = PermissionLevel::Restricted;
            context.allowed_paths = vec![std::env::current_dir()?];
            context.allowed_capabilities = HashSet::new();
            sandbox.set_security_context(plugin_id, context).await?;
            
            // Apply basic isolation
            sandbox.apply_feature(plugin_id, "basic_isolation").await?;
        }
    }
    
    println!("Security measures applied with appropriate degradation");
    Ok(())
}
```

## References

- [Platform Capabilities API](PLATFORM_CAPABILITIES_API.md)
- [Implementation Progress & Sandbox Summary](IMPLEMENTATION_PROGRESS.md)
- [Task Tracking](TASK_TRACKING.md) 