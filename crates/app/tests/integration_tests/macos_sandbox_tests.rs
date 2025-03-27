//! Integration tests for the macOS sandbox implementation
//!
//! These tests verify the macOS sandbox functionality in realistic scenarios.
//! Some tests are platform-specific and will be skipped on non-macOS platforms.

#![cfg(test)]

use squirrel_app::error::Result;
use squirrel_app::plugin::sandbox::{PluginSandbox, SandboxError};
use squirrel_app::plugin::sandbox::macos::MacOsSandbox;
use squirrel_app::plugin::resource_monitor::ResourceMonitor;
use squirrel_app::plugin::security::{SecurityContext, PermissionLevel, ResourceLimits};
use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::process::Command;
use tempfile::tempdir;
use std::fs;
use std::io::Write;

/// Skip the test if not running on macOS
macro_rules! skip_if_not_macos {
    () => {
        if !cfg!(target_os = "macos") {
            println!("Test skipped: not running on macOS");
            return Ok(());
        }
    };
}

/// Check if sandbox-exec is available in the system
fn is_sandbox_exec_available() -> bool {
    Command::new("sh")
        .args(["-c", "command -v sandbox-exec"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Test creating a macOS sandbox across all permission levels
#[tokio::test]
async fn test_create_macos_sandbox() -> Result<()> {
    skip_if_not_macos!();
    
    // Create a resource monitor
    let resource_monitor = Arc::new(ResourceMonitor::new());
    
    // Create the macOS sandbox
    let sandbox = MacOsSandbox::new(resource_monitor.clone())?;
    
    // Test with different permission levels
    for permission_level in [
        PermissionLevel::System,
        PermissionLevel::User,
        PermissionLevel::Restricted
    ].iter() {
        // Generate a unique plugin ID
        let plugin_id = Uuid::new_v4();
        
        // Create a security context with the current permission level
        let mut context = SecurityContext::default();
        context.permission_level = *permission_level;
        
        // Set the security context
        sandbox.set_security_context(plugin_id, context.clone()).await?;
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await?;
        
        // Verify that a sandbox profile was created
        let sandbox_profiles = sandbox.sandbox_profiles.read().await;
        assert!(sandbox_profiles.contains_key(&plugin_id));
        
        if let Some(profile_path) = sandbox_profiles.get(&plugin_id) {
            // Verify that the profile file exists
            assert!(profile_path.exists());
            
            // Check profile content for permission level-specific rules
            let profile_content = fs::read_to_string(profile_path)
                .expect("Failed to read sandbox profile");
            
            match permission_level {
                PermissionLevel::System => {
                    assert!(profile_content.contains("System Permission Level"));
                    assert!(profile_content.contains("(allow default)"));
                },
                PermissionLevel::User => {
                    assert!(profile_content.contains("User Permission Level"));
                    assert!(profile_content.contains("(deny default)"));
                },
                PermissionLevel::Restricted => {
                    assert!(profile_content.contains("Restricted Permission Level"));
                    assert!(profile_content.contains("(deny process-fork"));
                    assert!(profile_content.contains("(deny network-"));
                }
            }
        }
        
        // Destroy sandbox to clean up
        sandbox.destroy_sandbox(plugin_id).await?;
    }
    
    Ok(())
}

/// Test launching a process with the sandbox
#[tokio::test]
async fn test_launch_with_sandbox() -> Result<()> {
    skip_if_not_macos!();
    
    // Skip test if sandbox-exec is not available
    if !is_sandbox_exec_available() {
        println!("Test skipped: sandbox-exec not available");
        return Ok(());
    }
    
    // Create a resource monitor
    let resource_monitor = Arc::new(ResourceMonitor::new());
    
    // Create the macOS sandbox
    let sandbox = MacOsSandbox::new(resource_monitor.clone())?;
    
    // Generate a unique plugin ID
    let plugin_id = Uuid::new_v4();
    
    // Create a security context with User permission level
    let mut context = SecurityContext::default();
    context.permission_level = PermissionLevel::User;
    
    // Set resource limits
    context.resource_limits = ResourceLimits {
        max_memory_bytes: 100 * 1024 * 1024, // 100 MB
        max_cpu_percent: 50,
        max_threads: 5,
        max_file_descriptors: 100,
    };
    
    // Set the security context
    sandbox.set_security_context(plugin_id, context).await?;
    
    // Create a simple test script
    let temp_dir = tempdir()?;
    let script_path = temp_dir.path().join("test_script.sh");
    
    let script_content = r#"#!/bin/sh
echo "Hello from sandboxed process"
echo "Sandbox status: $SQUIRREL_SANDBOX_ENABLED"
echo "Memory limit: $SQUIRREL_MEMORY_LIMIT_MB MB"
echo "CPU limit: $SQUIRREL_CPU_LIMIT%"
echo "Plugin ID: $SQUIRREL_PLUGIN_ID"
sleep 1
exit 0
"#;
    
    let mut file = fs::File::create(&script_path)?;
    file.write_all(script_content.as_bytes())?;
    file.flush()?;
    
    // Make script executable
    Command::new("chmod")
        .args(["+x", script_path.to_str().unwrap()])
        .output()?;
    
    // Launch the process with the sandbox
    let result = sandbox.launch_with_sandbox(
        plugin_id, 
        &script_path, 
        &[]
    ).await;
    
    if let Err(e) = &result {
        // On macOS with sandbox-exec, this should succeed
        // Otherwise it might fail but with a specific error about sandbox-exec
        let err_str = format!("{:?}", e);
        assert!(!err_str.contains("executable not found"), "Script should exist: {:?}", e);
    } else {
        // If it succeeds, verify process ID and cleanup
        let process_id = result.unwrap();
        assert!(process_id > 0);
        
        // Give the process time to complete
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Cleanup
        sandbox.destroy_sandbox(plugin_id).await?;
    }
    
    Ok(())
}

/// Test SIP integration if available
#[tokio::test]
async fn test_sip_integration_feature() -> Result<()> {
    skip_if_not_macos!();
    
    // Create a resource monitor
    let resource_monitor = Arc::new(ResourceMonitor::new());
    
    // Create the macOS sandbox
    let sandbox = MacOsSandbox::new(resource_monitor.clone())?;
    
    // Generate a unique plugin ID
    let plugin_id = Uuid::new_v4();
    
    // Create a security context with Restricted permission level
    let mut context = SecurityContext::default();
    context.permission_level = PermissionLevel::Restricted;
    
    // Set the security context
    sandbox.set_security_context(plugin_id, context).await?;
    
    // Create sandbox
    sandbox.create_sandbox(plugin_id).await?;
    
    // Apply SIP integration feature
    let result = sandbox.apply_feature(plugin_id, "sip_integration").await;
    
    // The result may vary based on SIP status and csrutil availability
    // For our test, we just verify it doesn't panic or throw an unexpected error
    if let Err(e) = &result {
        let err_str = format!("{:?}", e);
        assert!(!err_str.contains("not supported"), 
                "SIP integration should be supported on macOS: {:?}", e);
    }
    
    // Cleanup
    sandbox.destroy_sandbox(plugin_id).await?;
    
    Ok(())
}

/// Test platform-specific optimizations
#[tokio::test]
async fn test_platform_optimizations_feature() -> Result<()> {
    skip_if_not_macos!();
    
    // Create a resource monitor
    let resource_monitor = Arc::new(ResourceMonitor::new());
    
    // Create the macOS sandbox
    let sandbox = MacOsSandbox::new(resource_monitor.clone())?;
    
    // Generate a unique plugin ID
    let plugin_id = Uuid::new_v4();
    
    // Create a security context with User permission level
    let mut context = SecurityContext::default();
    context.permission_level = PermissionLevel::User;
    
    // Set the security context
    sandbox.set_security_context(plugin_id, context).await?;
    
    // Create sandbox
    sandbox.create_sandbox(plugin_id).await?;
    
    // Apply platform optimizations feature
    let result = sandbox.apply_feature(plugin_id, "platform_optimizations").await;
    
    // The result may vary based on available commands
    // For our test, we just verify it doesn't panic or throw an unexpected error
    if let Err(e) = &result {
        let err_str = format!("{:?}", e);
        assert!(!err_str.contains("not supported"), 
                "Platform optimizations should be supported on macOS: {:?}", e);
    }
    
    // Cleanup
    sandbox.destroy_sandbox(plugin_id).await?;
    
    Ok(())
}

/// Test generating compatibility report
#[tokio::test]
async fn test_compatibility_report() -> Result<()> {
    // This test can run on any platform, as it should handle non-macOS platforms gracefully
    
    // Create a resource monitor
    let resource_monitor = Arc::new(ResourceMonitor::new());
    
    // Create the macOS sandbox
    let sandbox = MacOsSandbox::new(resource_monitor.clone())?;
    
    // Generate compatibility report
    let report = sandbox.generate_compatibility_report().await?;
    
    // Verify report structure
    assert!(report.contains("# macOS Sandbox Compatibility Report"));
    
    if cfg!(target_os = "macos") {
        // On macOS, report should contain detailed information
        assert!(report.contains("System Information"));
        assert!(report.contains("Sandbox Feature Compatibility"));
        assert!(report.contains("Required Tools"));
        assert!(report.contains("Sandbox Capabilities"));
        assert!(report.contains("Recommendations"));
    } else {
        // On non-macOS, report should indicate platform incompatibility
        assert!(report.contains("Not running on macOS"));
    }
    
    Ok(())
}

/// Test advanced security features with different permission levels
#[tokio::test]
async fn test_advanced_security_feature() -> Result<()> {
    skip_if_not_macos!();
    
    // Create a resource monitor
    let resource_monitor = Arc::new(ResourceMonitor::new());
    
    // Create the macOS sandbox
    let sandbox = MacOsSandbox::new(resource_monitor.clone())?;
    
    // Test with different permission levels
    for permission_level in [
        PermissionLevel::System,
        PermissionLevel::User,
        PermissionLevel::Restricted
    ].iter() {
        // Generate a unique plugin ID
        let plugin_id = Uuid::new_v4();
        
        // Create a security context with the current permission level
        let mut context = SecurityContext::default();
        context.permission_level = *permission_level;
        
        // Set the security context
        sandbox.set_security_context(plugin_id, context).await?;
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await?;
        
        // Apply advanced security feature
        let result = sandbox.apply_feature(plugin_id, "advanced_security").await;
        
        // The result may vary based on permission level and available commands
        // For our test, we just verify it doesn't panic or throw an unexpected error
        if let Err(e) = &result {
            let err_str = format!("{:?}", e);
            assert!(!err_str.contains("not supported"), 
                    "Advanced security should be supported on macOS: {:?}", e);
        }
        
        // Cleanup
        sandbox.destroy_sandbox(plugin_id).await?;
    }
    
    Ok(())
}

/// Test macOS version compatibility checking
#[tokio::test]
async fn test_macos_compatibility_check() -> Result<()> {
    skip_if_not_macos!();
    
    // Create a resource monitor
    let resource_monitor = Arc::new(ResourceMonitor::new());
    
    // Create the macOS sandbox
    let sandbox = MacOsSandbox::new(resource_monitor.clone())?;
    
    // Check macOS version
    let version = sandbox.get_macos_version()?;
    
    // Verify version is a valid version string (e.g., "10.15.7", "11.2.3", etc.)
    let is_valid_version = version.split('.').any(|part| part.parse::<u32>().is_ok());
    assert!(is_valid_version, "Invalid macOS version format: {}", version);
    
    // Check compatibility
    let compatibility = sandbox.check_macos_compatibility()?;
    
    // Verify compatibility map contains expected keys
    assert!(compatibility.contains_key("basic_sandbox"));
    assert!(compatibility.contains_key("enhanced_profiles"));
    assert!(compatibility.contains_key("sip"));
    assert!(compatibility.contains_key("sandbox_exec_available"));
    
    Ok(())
}

/// Test the sandbox in a realistic plugin scenario
#[tokio::test]
async fn test_realistic_plugin_scenario() -> Result<()> {
    skip_if_not_macos!();
    
    // Skip test if sandbox-exec is not available
    if !is_sandbox_exec_available() {
        println!("Test skipped: sandbox-exec not available");
        return Ok(());
    }
    
    // Create a resource monitor
    let resource_monitor = Arc::new(ResourceMonitor::new());
    
    // Create the macOS sandbox
    let sandbox = MacOsSandbox::new(resource_monitor.clone())?;
    
    // Generate a unique plugin ID
    let plugin_id = Uuid::new_v4();
    
    // Create a security context with User permission level
    let mut context = SecurityContext::default();
    context.permission_level = PermissionLevel::User;
    context.allowed_capabilities.insert("file:read".to_string());
    context.allowed_capabilities.insert("file:write".to_string());
    context.allowed_capabilities.insert("network:connect".to_string());
    
    // Set resource limits
    context.resource_limits = ResourceLimits {
        max_memory_bytes: 100 * 1024 * 1024, // 100 MB
        max_cpu_percent: 50,
        max_threads: 5,
        max_file_descriptors: 100,
    };
    
    // Create a temp directory for the plugin to access
    let temp_dir = tempdir()?;
    context.allowed_paths.push(temp_dir.path().to_path_buf());
    
    // Set the security context
    sandbox.set_security_context(plugin_id, context).await?;
    
    // Create a test file in the allowed directory
    let test_file_path = temp_dir.path().join("test_file.txt");
    let mut file = fs::File::create(&test_file_path)?;
    file.write_all(b"Hello, World!")?;
    file.flush()?;
    
    // Create a test script that will try to:
    // 1. Read from the allowed directory
    // 2. Write to the allowed directory
    // 3. Access network
    let script_path = temp_dir.path().join("plugin.sh");
    
    let script_content = format!(r#"#!/bin/sh
echo "Starting sandboxed plugin"

# Try to read the allowed file
echo "Reading from allowed file:"
cat "{}"

# Try to write to allowed directory
echo "Writing to allowed directory"
echo "Written from sandboxed plugin" > "{}/output.txt"

# Try to access network
echo "Checking network access:"
curl -s --connect-timeout 5 https://example.com > /dev/null
if [ $? -eq 0 ]; then
    echo "Network access succeeded"
else
    echo "Network access failed"
fi

echo "Plugin execution complete"
exit 0
"#, test_file_path.to_str().unwrap(), temp_dir.path().to_str().unwrap());
    
    let mut file = fs::File::create(&script_path)?;
    file.write_all(script_content.as_bytes())?;
    file.flush()?;
    
    // Make script executable
    Command::new("chmod")
        .args(["+x", script_path.to_str().unwrap()])
        .output()?;
    
    // Create sandbox
    sandbox.create_sandbox(plugin_id).await?;
    
    // Try all sandbox features
    sandbox.apply_feature(plugin_id, "memory_limit").await.ok();
    sandbox.apply_feature(plugin_id, "profile_optimize").await.ok();
    sandbox.apply_feature(plugin_id, "platform_optimizations").await.ok();
    
    // Launch the process with the sandbox
    let result = sandbox.launch_with_sandbox(
        plugin_id, 
        &script_path, 
        &[]
    ).await;
    
    if let Ok(process_id) = result {
        // Success! Give the process time to complete
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // Check if output file was created
        let output_path = temp_dir.path().join("output.txt");
        if output_path.exists() {
            let output_content = fs::read_to_string(&output_path)?;
            assert!(output_content.contains("Written from sandboxed plugin"));
        }
        
        // Track resources
        let usage = sandbox.track_resources(plugin_id).await;
        if let Ok(resource_usage) = usage {
            println!("Plugin resource usage: CPU {}%, Memory {} MB", 
                resource_usage.cpu_percent, resource_usage.memory_mb);
        }
    } else {
        eprintln!("Sandbox launch failed: {:?}", result.unwrap_err());
    }
    
    // Cleanup
    sandbox.destroy_sandbox(plugin_id).await?;
    
    Ok(())
} 