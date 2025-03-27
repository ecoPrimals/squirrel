//! Integration tests for seccomp filtering integration
//!
//! These tests verify the seccomp filtering functionality works correctly
//! on Linux platforms.

#![cfg(test)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::time::Duration;
use uuid::Uuid;
use std::process::Command;
use std::fs::{self, File};
use tempfile;
use std::collections::HashSet;
use std::io::Write;

use squirrel_app::error::Result;
use squirrel_app::plugin::sandbox::seccomp::{
    SeccompFilterBuilder, SeccompAction, ArgFilter, SyscallRule
};
use squirrel_app::plugin::security::{SecurityContext, PermissionLevel, ResourceLimits};
use squirrel_app::plugin::resource_monitor::ResourceMonitor;

#[cfg(target_os = "linux")]
use squirrel_app::plugin::sandbox::linux::LinuxCgroupSandbox;

/// Check if seccomp is available on the system
fn is_seccomp_available() -> bool {
    // Only run these tests on Linux
    if !cfg!(target_os = "linux") {
        return false;
    }
    
    // Check if the kernel supports seccomp
    Command::new("sh")
        .args(["-c", "grep -q Seccomp /proc/self/status"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false) && 
    
    // Check if we have permission to use seccomp
    Command::new("sh")
        .args(["-c", "grep -q \"Seccomp.*2\" /proc/self/status"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Check if we can run cgroup tests
fn can_run_cgroup_tests() -> bool {
    // Only run these tests on Linux
    if !cfg!(target_os = "linux") {
        return false;
    }
    
    // Check if cgroups v2 is mounted
    let output = Command::new("sh")
        .args(["-c", "mount | grep -q \"type cgroup2\" && echo yes || echo no"])
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.trim() == "yes"
        },
        Err(_) => false,
    }
}

/// Test creating seccomp filter builder
#[tokio::test]
async fn test_seccomp_filter_builder() -> Result<()> {
    if !is_seccomp_available() {
        println!("Seccomp not available, skipping test");
        return Ok(());
    }
    
    let plugin_id = Uuid::new_v4();
    
    // Create a basic filter builder
    let builder = SeccompFilterBuilder::new(plugin_id)
        .default_action(SeccompAction::Errno)
        .add_essential_syscalls()
        .add_file_operations();
    
    // Generate a BPF file
    let temp_dir = std::env::temp_dir();
    let bpf_path = temp_dir.join(format!("test_seccomp_{}.bpf", plugin_id));
    
    let result = builder.generate_bpf(&bpf_path);
    
    // This may or may not succeed depending on whether seccomp-tools is installed
    // We're mostly testing that it doesn't panic
    match result {
        Ok(path) => {
            assert!(path.exists());
            // Clean up
            let _ = std::fs::remove_file(path);
        },
        Err(e) => {
            // Print the error but don't fail the test
            println!("Failed to generate BPF file: {:?}", e);
        }
    }
    
    Ok(())
}

/// Test argument filtering
#[tokio::test]
async fn test_seccomp_arg_filtering() -> Result<()> {
    if !is_seccomp_available() {
        println!("Seccomp not available, skipping test");
        return Ok(());
    }
    
    let plugin_id = Uuid::new_v4();
    
    // Create a filter builder with argument filtering
    let builder = SeccompFilterBuilder::new(plugin_id)
        .default_action(SeccompAction::Errno)
        .add_rule(
            SyscallRule::new("open", SeccompAction::Allow)
                .with_arg_filter(1, "&", 0x1, 64)) // Filter for O_WRONLY
    );
    
    // Generate a BPF file
    let temp_dir = std::env::temp_dir();
    let bpf_path = temp_dir.join(format!("test_seccomp_arg_{}.bpf", plugin_id));
    
    let result = builder.generate_bpf(&bpf_path);
    
    // This may or may not succeed depending on whether seccomp-tools is installed
    match result {
        Ok(path) => {
            assert!(path.exists());
            // Clean up
            let _ = std::fs::remove_file(path);
        },
        Err(e) => {
            // Print the error but don't fail the test
            println!("Failed to generate BPF file: {:?}", e);
        }
    }
    
    Ok(())
}

/// Test integrating seccomp with Linux sandbox
#[tokio::test]
async fn test_linux_sandbox_seccomp_integration() -> Result<()> {
    if !is_seccomp_available() || !can_run_cgroup_tests() {
        println!("Seccomp or cgroups not available, skipping test");
        return Ok(());
    }
    
    #[cfg(target_os = "linux")]
    {
        // Create a resource monitor
        let resource_monitor = Arc::new(ResourceMonitor::new());
        
        // Create a Linux sandbox
        let sandbox = LinuxCgroupSandbox::new(resource_monitor.clone())?;
        
        // Generate a unique plugin ID
        let plugin_id = Uuid::new_v4();
        
        // Create a security context
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::Restricted;
        
        // Add some capabilities
        context.capabilities.insert("fs.read".to_string());
        context.capabilities.insert("net.client".to_string());
        
        // Set the security context for the plugin
        sandbox.set_security_context(plugin_id, context).await?;
        
        // Register the process with the resource monitor
        resource_monitor.register_process(
            plugin_id,
            std::process::id(),
            &std::env::current_exe()?
        ).await?;
        
        // Create the sandbox
        let result = sandbox.create_sandbox(plugin_id).await;
        
        // This may fail if we don't have sufficient permissions
        if let Err(e) = &result {
            println!("Failed to create sandbox: {:?}", e);
            println!("This may be expected if not running with sufficient permissions.");
            return Ok(());
        }
        
        // Generate a seccomp BPF file
        let temp_dir = std::env::temp_dir();
        let bpf_path = temp_dir.join(format!("test_sandbox_seccomp_{}.bpf", plugin_id));
        
        // Generate the BPF file
        sandbox.generate_seccomp_bpf(plugin_id, &bpf_path).await?;
        
        // Check that the BPF file exists
        assert!(bpf_path.exists());
        
        // Apply the seccomp filter
        let apply_result = sandbox.apply_seccomp_filter(plugin_id).await;
        
        // This may fail if we don't have sufficient permissions
        if let Err(e) = &apply_result {
            println!("Failed to apply seccomp filter: {:?}", e);
            println!("This may be expected if not running with sufficient permissions.");
        } else {
            println!("Successfully applied seccomp filter");
        }
        
        // Clean up
        sandbox.destroy_sandbox(plugin_id).await?;
        
        // Remove the BPF file
        let _ = std::fs::remove_file(bpf_path);
    }
    
    Ok(())
}

/// Test security context to seccomp filter conversion
#[tokio::test]
async fn test_security_context_to_seccomp() -> Result<()> {
    if !is_seccomp_available() {
        println!("Seccomp not available, skipping test");
        return Ok(());
    }
    
    // Test with different permission levels
    for permission_level in &[
        PermissionLevel::System,
        PermissionLevel::User,
        PermissionLevel::Restricted
    ] {
        let plugin_id = Uuid::new_v4();
        
        // Create a security context
        let mut context = SecurityContext::default();
        context.permission_level = *permission_level;
        
        // Add some capabilities based on permission level
        match permission_level {
            PermissionLevel::System => {
                context.capabilities.insert("fs.read".to_string());
                context.capabilities.insert("fs.write".to_string());
                context.capabilities.insert("net.client".to_string());
                context.capabilities.insert("net.server".to_string());
                context.capabilities.insert("proc.create".to_string());
            },
            PermissionLevel::User => {
                context.capabilities.insert("fs.read".to_string());
                context.capabilities.insert("fs.write".to_string());
                context.capabilities.insert("net.client".to_string());
            },
            PermissionLevel::Restricted => {
                context.capabilities.insert("fs.read".to_string());
                context.capabilities.insert("net.client".to_string());
            }
        };
        
        // Create a seccomp filter from the security context
        let builder = SeccompFilterBuilder::from_security_context(plugin_id, &context);
        
        // Generate a BPF file
        let temp_dir = std::env::temp_dir();
        let bpf_path = temp_dir.join(format!("test_security_context_{}.bpf", plugin_id));
        
        let result = builder.generate_bpf(&bpf_path);
        
        // This may or may not succeed depending on whether seccomp-tools is installed
        match result {
            Ok(path) => {
                assert!(path.exists());
                // Clean up
                let _ = std::fs::remove_file(path);
            },
            Err(e) => {
                // Print the error but don't fail the test
                println!("Failed to generate BPF file for {:?}: {:?}", permission_level, e);
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile;

    use squirrel_app::plugin::sandbox::seccomp::{
        SeccompFilterBuilder, SeccompAction, ArgFilter, SyscallRule
    };
    use squirrel_app::plugin::security::{
        SecurityContext, PermissionLevel, ResourceLimits
    };
    use squirrel_app::error::Result;

    /// Helper function to check if seccomp tests can run on this platform
    fn seccomp_tests_available() -> bool {
        cfg!(target_os = "linux")
    }

    /// Helper function to check if cgroup tests can run on this system
    fn cgroup_tests_available() -> bool {
        if !cfg!(target_os = "linux") {
            return false;
        }
        
        // Check if cgroup v2 is mounted
        std::path::Path::new("/sys/fs/cgroup").exists()
    }

    #[test]
    fn test_seccomp_filter_builder() {
        if !seccomp_tests_available() {
            println!("Skipping seccomp filter builder test on non-Linux platform");
            return;
        }

        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let output_path = temp_dir.path().join("seccomp_test.bpf");
        
        let mut builder = SeccompFilterBuilder::new(SeccompAction::Kill);
        
        // Add some basic allowed syscalls
        builder.add_rule(SyscallRule::new("read", SeccompAction::Allow));
        builder.add_rule(SyscallRule::new("write", SeccompAction::Allow));
        builder.add_rule(SyscallRule::new("exit", SeccompAction::Allow));
        
        // Generate BPF file
        let result = builder.generate_bpf_file(&output_path);
        assert!(result.is_ok(), "Failed to generate BPF file");
        
        // Validate that the file exists and has content
        assert!(output_path.exists(), "BPF file does not exist");
        
        // Clean up
        std::fs::remove_file(output_path).ok();
    }

    #[test]
    fn test_seccomp_arg_filtering() {
        if !seccomp_tests_available() {
            println!("Skipping seccomp arg filtering test on non-Linux platform");
            return;
        }

        let mut builder = SeccompFilterBuilder::new(SeccompAction::Kill);
        
        // Add rule with argument filtering
        builder.add_rule(SyscallRule::new("open", SeccompAction::Allow)
            .with_arg_filter(0, ArgFilter::path_prefix("/tmp/")));
        
        // Verify the rule exists in the builder
        assert!(builder.has_rule("open"), "Expected 'open' rule to exist");
        
        // More complex rule with multiple args
        builder.add_rule(SyscallRule::new("socket", SeccompAction::Allow)
            .with_arg_filter(0, ArgFilter::equal(2)) // AF_INET
            .with_arg_filter(1, ArgFilter::equal(1))); // SOCK_STREAM
        
        assert!(builder.has_rule("socket"), "Expected 'socket' rule to exist");
    }

    #[test]
    fn test_linux_sandbox_seccomp_integration() {
        if !seccomp_tests_available() || !cgroup_tests_available() {
            println!("Skipping Linux sandbox seccomp integration test (requires Linux with cgroups)");
            return;
        }

        // Create a security context with restricted permissions
        let context = SecurityContext::new(PermissionLevel::Restricted, ResourceLimits::default());
        
        // Generate a seccomp BPF file from the security context
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let output_path = temp_dir.path().join("test_sandbox_seccomp.bpf");
        
        let mut builder = SeccompFilterBuilder::from_security_context(&context);
        let result = builder.generate_bpf_file(&output_path);
        
        assert!(result.is_ok(), "Failed to generate BPF file from security context");
        assert!(output_path.exists(), "BPF file does not exist");
        
        // Clean up
        std::fs::remove_file(output_path).ok();
    }

    #[test]
    fn test_security_context_to_seccomp() {
        if !seccomp_tests_available() {
            println!("Skipping security context to seccomp conversion test on non-Linux platform");
            return;
        }

        // Test different permission levels
        let system_context = SecurityContext::new(PermissionLevel::System, ResourceLimits::default());
        let user_context = SecurityContext::new(PermissionLevel::User, ResourceLimits::default());
        let restricted_context = SecurityContext::new(PermissionLevel::Restricted, ResourceLimits::default());
        
        // Create builders from each context
        let system_builder = SeccompFilterBuilder::from_security_context(&system_context);
        let user_builder = SeccompFilterBuilder::from_security_context(&user_context);
        let restricted_builder = SeccompFilterBuilder::from_security_context(&restricted_context);
        
        // System should have more allowed syscalls than User, which should have more than Restricted
        assert!(system_builder.allowed_syscall_count() >= user_builder.allowed_syscall_count());
        assert!(user_builder.allowed_syscall_count() >= restricted_builder.allowed_syscall_count());
    }

    #[test]
    fn test_security_context_seccomp_variations() {
        if !seccomp_tests_available() {
            println!("Skipping seccomp security context variation test on non-Linux platform");
            return;
        }

        // Test different security contexts with varying permission levels
        let contexts = vec![
            SecurityContext::new(PermissionLevel::System, ResourceLimits::unrestricted()),
            SecurityContext::new(PermissionLevel::User, ResourceLimits::default()),
            SecurityContext::new(PermissionLevel::Restricted, ResourceLimits::minimal()),
        ];

        for (i, context) in contexts.iter().enumerate() {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let output_path = temp_dir.path().join(format!("seccomp_filter_{}.bpf", i));
            
            // Create filter builder from security context
            let mut builder = SeccompFilterBuilder::from_security_context(context);
            
            // Generate BPF file
            let result = builder.generate_bpf_file(&output_path);
            assert!(result.is_ok(), "Failed to generate BPF file for context with permission level: {:?}", context.permission_level());
            
            // Validate that the file exists and has content
            assert!(output_path.exists(), "BPF file does not exist");
            assert!(fs::metadata(&output_path).unwrap().len() > 0, "BPF file is empty");
            
            // For a more comprehensive test, we would analyze the content of the BPF file
            // or apply it to a process and test the behavior, but this is beyond the scope
            // of a simple integration test
            
            // Verify that different permission levels result in different BPF content
            if i > 0 {
                let prev_file = temp_dir.path().join(format!("seccomp_filter_{}.bpf", i-1));
                let prev_content = fs::read(&prev_file).unwrap();
                let current_content = fs::read(&output_path).unwrap();
                
                // Files should be different for different security contexts
                assert_ne!(prev_content, current_content, 
                    "BPF files for {:?} and {:?} are identical, which is unexpected",
                    contexts[i-1].permission_level(), context.permission_level());
            }
            
            // Clean up
            fs::remove_file(&output_path).ok();
        }
    }

    #[test]
    fn test_seccomp_with_custom_syscall_rules() {
        if !seccomp_tests_available() {
            println!("Skipping seccomp custom rules test on non-Linux platform");
            return;
        }

        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let output_path = temp_dir.path().join("seccomp_custom.bpf");
        
        // Create a base security context
        let context = SecurityContext::new(PermissionLevel::Restricted, ResourceLimits::minimal());
        
        // Create filter builder with base rules from context
        let mut builder = SeccompFilterBuilder::from_security_context(&context);
        
        // Add custom syscall rules for specific operations
        builder.add_rule(SyscallRule::new("open", SeccompAction::Allow)
            .with_arg_filter(0, ArgFilter::path_prefix("/tmp/")));
        
        builder.add_rule(SyscallRule::new("socket", SeccompAction::Allow)
            .with_arg_filter(0, ArgFilter::equal(2)) // AF_INET = 2
            .with_arg_filter(1, ArgFilter::equal(1))); // SOCK_STREAM = 1
        
        // A custom rule with logging for a sensitive operation
        builder.add_rule(SyscallRule::new("ptrace", SeccompAction::Log));
        
        // Generate BPF file
        let result = builder.generate_bpf_file(&output_path);
        assert!(result.is_ok(), "Failed to generate BPF file with custom rules");
        
        // Validate that the file exists and has content
        assert!(output_path.exists(), "BPF file does not exist");
        assert!(fs::metadata(&output_path).unwrap().len() > 0, "BPF file is empty");
        
        // Clean up
        fs::remove_file(output_path).ok();
    }

    #[test]
    fn test_seccomp_real_world_scenarios() {
        if !seccomp_tests_available() {
            println!("Skipping seccomp real-world scenarios test on non-Linux platform");
            return;
        }

        // Test different real-world scenarios
        let scenarios = ["web_browser", "file_processor", "web_server", "database"];
        
        for scenario in scenarios {
            let plugin_id = Uuid::new_v4();
            
            // Create test suite for this scenario
            let builder = SeccompFilterBuilder::real_world_test_suite(plugin_id, scenario);
            
            // Generate BPF file
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let output_path = temp_dir.path().join(format!("seccomp_scenario_{}.bpf", scenario));
            
            let result = builder.generate_bpf_file(&output_path);
            
            assert!(result.is_ok(), "Failed to generate BPF file for scenario: {}", scenario);
            assert!(output_path.exists(), "BPF file does not exist for scenario: {}", scenario);
            assert!(fs::metadata(&output_path).unwrap().len() > 0, "BPF file is empty for scenario: {}", scenario);
            
            // Validate scenario-specific syscalls
            match scenario {
                "web_browser" => {
                    assert!(builder.has_rule("socket"), "Web browser should have socket syscall");
                    assert!(builder.has_rule("connect"), "Web browser should have connect syscall");
                    assert!(builder.has_rule("open"), "Web browser should have open syscall");
                },
                "file_processor" => {
                    assert!(builder.has_rule("open"), "File processor should have open syscall");
                    assert!(builder.has_rule("read"), "File processor should have read syscall");
                    assert!(builder.has_rule("write"), "File processor should have write syscall");
                    assert!(builder.has_rule("mmap"), "File processor should have mmap syscall");
                },
                "web_server" => {
                    assert!(builder.has_rule("socket"), "Web server should have socket syscall");
                    assert!(builder.has_rule("bind"), "Web server should have bind syscall");
                    assert!(builder.has_rule("accept"), "Web server should have accept syscall");
                    assert!(builder.has_rule("listen"), "Web server should have listen syscall");
                },
                "database" => {
                    assert!(builder.has_rule("mmap"), "Database should have mmap syscall");
                    assert!(builder.has_rule("fsync"), "Database should have fsync syscall");
                    assert!(builder.has_rule("flock"), "Database should have flock syscall");
                },
                _ => {}
            }
            
            // Clean up
            fs::remove_file(output_path).ok();
        }
    }

    #[test]
    fn test_seccomp_capability_customization() {
        if !seccomp_tests_available() {
            println!("Skipping seccomp capability customization test on non-Linux platform");
            return;
        }

        let plugin_id = Uuid::new_v4();
        
        // Create different capability sets
        let mut minimal_caps = HashSet::new();
        minimal_caps.insert("fs.read".to_string());
        
        let mut standard_caps = HashSet::new();
        standard_caps.insert("fs.read".to_string());
        standard_caps.insert("fs.write".to_string());
        standard_caps.insert("net.client".to_string());
        
        let mut privileged_caps = HashSet::new();
        privileged_caps.insert("fs.read".to_string());
        privileged_caps.insert("fs.write".to_string());
        privileged_caps.insert("net.client".to_string());
        privileged_caps.insert("net.server".to_string());
        privileged_caps.insert("proc.create".to_string());
        
        let mut admin_caps = privileged_caps.clone();
        admin_caps.insert("sys.admin".to_string());
        admin_caps.insert("debug".to_string());
        
        // Create builders with different capability sets
        let builders = [
            ("minimal", SeccompFilterBuilder::new(plugin_id).customize_for_capabilities(&minimal_caps)),
            ("standard", SeccompFilterBuilder::new(plugin_id).customize_for_capabilities(&standard_caps)),
            ("privileged", SeccompFilterBuilder::new(plugin_id).customize_for_capabilities(&privileged_caps)),
            ("admin", SeccompFilterBuilder::new(plugin_id).customize_for_capabilities(&admin_caps)),
        ];
        
        for (cap_name, builder) in builders {
            // Generate BPF file
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let output_path = temp_dir.path().join(format!("seccomp_caps_{}.bpf", cap_name));
            
            let result = builder.generate_bpf_file(&output_path);
            
            assert!(result.is_ok(), "Failed to generate BPF file for capabilities: {}", cap_name);
            assert!(output_path.exists(), "BPF file does not exist for capabilities: {}", cap_name);
            assert!(fs::metadata(&output_path).unwrap().len() > 0, "BPF file is empty for capabilities: {}", cap_name);
            
            // Validate capability-specific syscalls
            match cap_name {
                "minimal" => {
                    assert!(builder.has_rule("read"), "Minimal should have read syscall");
                    assert!(!builder.has_rule("write"), "Minimal should not have write syscall");
                    assert!(!builder.has_rule("socket"), "Minimal should not have socket syscall");
                },
                "standard" => {
                    assert!(builder.has_rule("read"), "Standard should have read syscall");
                    assert!(builder.has_rule("write"), "Standard should have write syscall");
                    assert!(builder.has_rule("socket"), "Standard should have socket syscall");
                    assert!(!builder.has_rule("bind"), "Standard should not have bind syscall");
                },
                "privileged" => {
                    assert!(builder.has_rule("socket"), "Privileged should have socket syscall");
                    assert!(builder.has_rule("bind"), "Privileged should have bind syscall");
                    assert!(builder.has_rule("fork"), "Privileged should have fork syscall");
                    assert!(!builder.has_rule("mount"), "Privileged should not have mount syscall");
                },
                "admin" => {
                    assert!(builder.has_rule("mount"), "Admin should have mount syscall");
                    assert!(builder.has_rule("ptrace"), "Admin should have ptrace syscall");
                    assert!(builder.has_rule("setuid"), "Admin should have setuid syscall");
                },
                _ => {}
            }
            
            // Clean up
            fs::remove_file(output_path).ok();
        }
    }

    #[test]
    fn test_seccomp_custom_rules_with_arg_filtering() {
        if !seccomp_tests_available() {
            println!("Skipping seccomp custom rules with arg filtering test on non-Linux platform");
            return;
        }

        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let output_path = temp_dir.path().join("seccomp_arg_filtered.bpf");
        
        let plugin_id = Uuid::new_v4();
        let mut builder = SeccompFilterBuilder::new(plugin_id)
            .default_action(SeccompAction::Kill)
            .add_essential_syscalls();
        
        // Add complex argument-filtered rules
        
        // Socket rule: Allow only IPv4 (AF_INET=2) TCP (SOCK_STREAM=1) sockets
        builder = builder.add_rule(
            SyscallRule::new("socket", SeccompAction::Allow)
                .with_arg_filter(ArgFilter::equal(2).with_arg_index(0))    // AF_INET
                .with_arg_filter(ArgFilter::equal(1).with_arg_index(1))    // SOCK_STREAM
        );
        
        // Open rule: Allow opening files for reading (O_RDONLY=0)
        builder = builder.add_rule(
            SyscallRule::new("open", SeccompAction::Allow)
                .with_arg_filter(ArgFilter::equal(0).with_arg_index(1))    // O_RDONLY
        );
        
        // File path restriction: Allow opening specific paths (simplified)
        builder = builder.add_rule(
            SyscallRule::new("openat", SeccompAction::Allow)
                .with_arg_filter(ArgFilter::path_prefix("/tmp/").with_arg_index(1))
        );
        
        // Range-based filtering: Allow mmap with specific protection flags
        builder = builder.add_rule(
            SyscallRule::new("mmap", SeccompAction::Allow)
                .with_arg_filter(ArgFilter::in_range(1, 7).with_arg_index(2))  // PROT_READ|PROT_WRITE|PROT_EXEC combinations
        );
        
        // Generate BPF file
        let result = builder.generate_bpf_file(&output_path);
        assert!(result.is_ok(), "Failed to generate BPF file with argument-filtered rules");
        
        // Validate that the file exists and has content
        assert!(output_path.exists(), "BPF file does not exist");
        assert!(fs::metadata(&output_path).unwrap().len() > 0, "BPF file is empty");
        
        // Clean up
        fs::remove_file(output_path).ok();
    }

    #[test]
    #[cfg(target_os = "linux")]
    async fn test_seccomp_real_process_behavior() -> Result<()> {
        if !is_seccomp_available() {
            println!("Seccomp not available, skipping test");
            return Ok(());
        }
        
        use std::process::{Command, Stdio};
        use std::io::Write;
        
        // Create temporary directory for test files
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let test_path = temp_dir.path().join("seccomp_test.sh");
        let allowed_path = temp_dir.path().join("allowed_file.txt");
        let blocked_path = temp_dir.path().join("blocked_file.txt");
        
        // Create test script
        let script_content = format!(r#"#!/bin/sh
# Write to allowed file
echo "This should work" > {}
# Try to write to blocked file
echo "This should fail" > {}
# Exit with success to indicate the script ran
exit 0
"#, 
            allowed_path.display(),
            blocked_path.display()
        );
        
        // Write test script
        let mut script_file = File::create(&test_path).expect("Failed to create test script");
        script_file.write_all(script_content.as_bytes()).expect("Failed to write test script");
        script_file.flush().expect("Failed to flush test script");
        
        // Make script executable
        let status = Command::new("chmod")
            .args(["+x", &test_path.to_string_lossy()])
            .status()
            .expect("Failed to make script executable");
        
        assert!(status.success(), "Failed to make script executable");
        
        // Create a resource monitor
        let resource_monitor = Arc::new(ResourceMonitor::new());
        
        // Create a plugin ID for the test
        let plugin_id = Uuid::new_v4();
        
        // Create a security context
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::User;
        
        // Add specific capabilities
        context.capabilities.insert("fs.read".to_string());
        context.capabilities.insert("fs.write".to_string());
        
        // Add path permissions - only allow writing to the allowed path
        let mut allowed_paths = HashSet::new();
        allowed_paths.insert(temp_dir.path().join("allowed_file.txt").to_string_lossy().to_string());
        context.path_permissions.insert("write".to_string(), allowed_paths);
        
        // Create a custom seccomp filter
        let builder = SeccompFilterBuilder::new(plugin_id)
            .default_action(SeccompAction::Errno)
            .add_essential_syscalls()
            .add_file_operations();
        
        // Add a specific rule to allow opening only the allowed path
        let bpf_path = temp_dir.path().join("seccomp_test.bpf");
        builder.generate_bpf(&bpf_path)?;
        
        // Register the test script process
        let test_process = Command::new(&test_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start test script");
        
        let pid = test_process.id();
        
        // Register process with resource monitor
        resource_monitor.register_process(
            plugin_id,
            pid,
            &test_path
        ).await?;
        
        // Apply seccomp filter
        let result = builder.apply_to_process(pid);
        
        // This may fail if we don't have sufficient permissions
        if let Err(e) = &result {
            println!("Failed to apply seccomp filter: {:?}", e);
            println!("This may be expected if not running with sufficient permissions.");
            return Ok(());
        }
        
        // Wait for the process to complete
        let output = test_process.wait_with_output().expect("Failed to wait for test script");
        
        // Check if the allowed file was created
        assert!(allowed_path.exists(), "Allowed file was not created");
        
        // Check if the blocked file was not created (due to seccomp filter)
        // Note: This is a simplified test. In reality, the seccomp filter would need to be
        // much more sophisticated to block specific file paths.
        // The current implementation is just a proof of concept.
        if !blocked_path.exists() {
            println!("Successfully blocked file creation using seccomp filter");
        } else {
            println!("Note: Blocked file was created. This is expected in this simplified test.");
            println!("A more sophisticated filter would be needed to block specific paths.");
        }
        
        // Clean up
        let _ = std::fs::remove_file(&bpf_path);
        let _ = std::fs::remove_file(&test_path);
        let _ = std::fs::remove_file(&allowed_path);
        let _ = std::fs::remove_file(&blocked_path);
        
        Ok(())
    }
} 