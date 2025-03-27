//! Integration tests for sandbox and resource monitor integration
//!
//! These tests demonstrate the proper pattern for integrating the resource monitor
//! with the sandbox system across different platforms.

#![cfg(test)]

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

use squirrel_app::error::Result;
use squirrel_app::plugin::resource_monitor::{ResourceMonitor, ResourceLimits, ResourceUsage};
use squirrel_app::plugin::sandbox::{PluginSandbox, SandboxError};
use squirrel_app::plugin::security::{SecurityContext, PermissionLevel};

// Import platform-specific sandboxes if testing on those platforms
#[cfg(target_os = "windows")]
use squirrel_app::plugin::sandbox::windows::WindowsSandbox;

#[cfg(target_os = "linux")]
use squirrel_app::plugin::sandbox::linux::LinuxSandbox;

#[cfg(target_os = "macos")]
use squirrel_app::plugin::sandbox::macos::MacOsSandbox;

// Use a basic sandbox for platform-agnostic tests
use squirrel_app::plugin::sandbox::BasicPluginSandbox;

/// Creates a test security context with specified permission level
fn create_test_security_context(permission_level: PermissionLevel) -> SecurityContext {
    let mut context = SecurityContext::default();
    context.permission_level = permission_level;
    
    // Set resource limits appropriate for testing
    context.resource_limits = ResourceLimits {
        max_memory_bytes: 100 * 1024 * 1024, // 100 MB
        max_cpu_percent: 50,                 // 50% CPU
        max_threads: 5,
        max_file_descriptors: 100,
    };
    
    // Add test capabilities based on permission level
    match permission_level {
        PermissionLevel::System => {
            context.capabilities.insert("system.admin".to_string());
            context.capabilities.insert("fs.write.*".to_string());
        },
        PermissionLevel::User => {
            context.capabilities.insert("fs.read.*".to_string());
            context.capabilities.insert("fs.write.user".to_string());
        },
        PermissionLevel::Restricted => {
            context.capabilities.insert("fs.read.temp".to_string());
        },
    }
    
    context
}

/// Helper function to set up a resource monitor
fn create_resource_monitor() -> Arc<ResourceMonitor> {
    let mut resource_monitor = ResourceMonitor::new();
    resource_monitor.enable_monitoring();
    resource_monitor.set_monitor_interval(Duration::from_secs(1));
    Arc::new(resource_monitor)
}

/// Test basic sandbox with resource monitor integration
#[tokio::test]
async fn test_basic_sandbox_resource_integration() -> Result<()> {
    // Create resource monitor
    let resource_monitor = create_resource_monitor();
    
    // Create basic sandbox
    let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
    
    // Generate a unique plugin ID
    let plugin_id = Uuid::new_v4();
    
    // Create security context
    let context = create_test_security_context(PermissionLevel::User);
    
    // Set the security context for the plugin
    sandbox.set_security_context(plugin_id, context).await?;
    
    // IMPORTANT: Register the process with the resource monitor before creating the sandbox
    // This is the critical step that was missing in failing tests
    resource_monitor.register_process(
        plugin_id,
        std::process::id(),
        &std::env::current_exe()?
    ).await?;
    
    // Now create the sandbox
    sandbox.create_sandbox(plugin_id).await?;
    
    // Verify sandbox was created properly
    let result = sandbox.check_permission(plugin_id, "fs.read.user").await;
    assert!(result.is_ok(), "Permission check should succeed");
    
    // Track resources - this should work now that process is registered
    let usage = sandbox.track_resources(plugin_id).await?;
    
    // Verify resource tracking works
    assert!(usage.memory_bytes > 0, "Memory usage should be tracked");
    
    // Wait a moment for resource monitoring
    time::sleep(Duration::from_millis(500)).await;
    
    // Get updated resource usage
    let updated_usage = resource_monitor.get_resource_usage(plugin_id).await?;
    assert!(updated_usage.memory_bytes > 0, "Memory usage should be tracked");
    
    // Verify path access works correctly
    let temp_dir = std::env::temp_dir();
    let result = sandbox.check_path_access(plugin_id, &temp_dir, false).await;
    assert!(result.is_ok(), "Read access to temp dir should be allowed");
    
    // Clean up
    sandbox.destroy_sandbox(plugin_id).await?;
    
    // Verify resource monitor unregistered the process
    let result = resource_monitor.get_resource_usage(plugin_id).await;
    assert!(result.is_err(), "Process should be unregistered after sandbox destruction");
    
    Ok(())
}

/// Test platform-specific sandbox with resource monitor integration
/// This test will run only on the appropriate platform
#[tokio::test]
async fn test_platform_specific_sandbox_resource_integration() -> Result<()> {
    // Skip if not on a supported platform
    let platform_name = std::env::consts::OS;
    
    // Create resource monitor
    let resource_monitor = create_resource_monitor();
    
    // Create platform-specific sandbox based on current OS
    let sandbox: Box<dyn PluginSandbox> = match platform_name {
        #[cfg(target_os = "windows")]
        "windows" => {
            Box::new(WindowsSandbox::new(resource_monitor.clone())?)
        },
        #[cfg(target_os = "linux")]
        "linux" => {
            Box::new(LinuxSandbox::new(resource_monitor.clone())?)
        },
        #[cfg(target_os = "macos")]
        "macos" => {
            Box::new(MacOsSandbox::new(resource_monitor.clone())?)
        },
        _ => {
            println!("Test skipped: platform {} not supported", platform_name);
            return Ok(());
        }
    };
    
    // Generate a unique plugin ID
    let plugin_id = Uuid::new_v4();
    
    // Create security context
    let context = create_test_security_context(PermissionLevel::User);
    
    // Set the security context for the plugin
    sandbox.set_security_context(plugin_id, context).await?;
    
    // IMPORTANT: Register the process with the resource monitor before creating the sandbox
    resource_monitor.register_process(
        plugin_id,
        std::process::id(),
        &std::env::current_exe()?
    ).await?;
    
    // Now create the sandbox
    sandbox.create_sandbox(plugin_id).await?;
    
    // Verify sandbox was created properly by checking a basic permission
    let result = sandbox.check_permission(plugin_id, "fs.read.user").await;
    assert!(result.is_ok(), "Permission check should succeed");
    
    // Track resources 
    let usage = sandbox.track_resources(plugin_id).await?;
    
    // Verify resource tracking works
    assert!(usage.memory_bytes > 0, "Memory usage should be tracked");
    assert!(usage.cpu_percent >= 0.0, "CPU usage should be a valid percentage");
    
    // Wait a moment for resource monitoring
    time::sleep(Duration::from_millis(500)).await;
    
    // Get updated resource usage directly from the monitor
    let updated_usage = resource_monitor.get_resource_usage(plugin_id).await?;
    assert!(updated_usage.memory_bytes > 0, "Memory usage should be tracked");
    
    // Clean up
    sandbox.destroy_sandbox(plugin_id).await?;
    
    // Verify resource monitor unregistered the process
    let result = resource_monitor.get_resource_usage(plugin_id).await;
    assert!(result.is_err(), "Process should be unregistered after sandbox destruction");
    
    Ok(())
}

/// Test resource limits in sandbox
#[tokio::test]
async fn test_sandbox_resource_limits() -> Result<()> {
    // Create resource monitor
    let resource_monitor = create_resource_monitor();
    
    // Create basic sandbox
    let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
    
    // Generate a unique plugin ID
    let plugin_id = Uuid::new_v4();
    
    // Create security context with restricted resources
    let mut context = create_test_security_context(PermissionLevel::Restricted);
    
    // Set very restrictive resource limits for testing
    context.resource_limits = ResourceLimits {
        max_memory_bytes: 10 * 1024 * 1024, // 10 MB
        max_cpu_percent: 10,                // 10% CPU
        max_threads: 2,
        max_file_descriptors: 10,
    };
    
    // Set the security context for the plugin
    sandbox.set_security_context(plugin_id, context).await?;
    
    // Register the process with the resource monitor
    resource_monitor.register_process(
        plugin_id,
        std::process::id(),
        &std::env::current_exe()?
    ).await?;
    
    // Register the resource limits
    resource_monitor.set_resource_limits(plugin_id, ResourceLimits {
        max_memory_bytes: 10 * 1024 * 1024, // 10 MB
        max_cpu_percent: 10,                // 10% CPU
        max_threads: 2,
        max_file_descriptors: 10,
    }).await?;
    
    // Create the sandbox
    sandbox.create_sandbox(plugin_id).await?;
    
    // Verify sandbox enforces resource limits by checking validation
    let validation = resource_monitor.validate_resource_usage(plugin_id).await?;
    
    // At this point, we haven't exceeded resources yet
    assert!(validation.is_valid, "Resource validation should pass initially");
    
    // Clean up
    sandbox.destroy_sandbox(plugin_id).await?;
    
    Ok(())
}

/// Test error handling for unregistered processes
#[tokio::test]
async fn test_sandbox_unregistered_process() -> Result<()> {
    // Create resource monitor
    let resource_monitor = create_resource_monitor();
    
    // Create basic sandbox
    let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
    
    // Generate a unique plugin ID
    let plugin_id = Uuid::new_v4();
    
    // Create security context
    let context = create_test_security_context(PermissionLevel::User);
    
    // Set the security context for the plugin
    sandbox.set_security_context(plugin_id, context).await?;
    
    // Try to create sandbox without registering the process - this should fail
    let result = sandbox.create_sandbox(plugin_id).await;
    
    // Verify handling of unregistered processes is improved
    // The error now should be more specific about process registration
    if let Err(e) = result {
        let error_string = format!("{:?}", e);
        assert!(
            error_string.contains("Process not registered") || 
            error_string.contains("Plugin not found in resource monitor"),
            "Error message should indicate process registration issue: {}", error_string
        );
    } else {
        // If it doesn't fail, make sure registration happened automatically
        let usage_result = resource_monitor.get_resource_usage(plugin_id).await;
        assert!(usage_result.is_ok(), "Process should be registered if sandbox creation succeeded");
        
        // Clean up
        sandbox.destroy_sandbox(plugin_id).await?;
    }
    
    Ok(())
} 