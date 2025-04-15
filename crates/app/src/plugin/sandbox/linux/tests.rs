#![cfg(target_family = "unix")]
#![cfg(test)]

//! Test module for LinuxCgroupSandbox

mod tests {
    use crate::plugin::sandbox::linux::LinuxCgroupSandbox;
    use crate::plugin::resource_monitor::ResourceMonitor;
    use crate::plugin::security::{SecurityContext, PermissionLevel};
    use std::env;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use tokio::runtime::Runtime;
    use uuid::Uuid;
    use crate::plugin::sandbox::PluginSandbox;

    // Helper function to check if we can run cgroup tests
    fn can_run_cgroup_tests() -> bool {
        // Check if we are running with sufficient permissions and on Linux
        if !cfg!(target_os = "linux") {
            return false;
        }

        // Check if cgroup v2 is mounted and accessible
        let cgroup_mount = Path::new("/sys/fs/cgroup");
        if !cgroup_mount.exists() {
            return false;
        }

        // Check if it's cgroup v2
        let cgroup_type_path = cgroup_mount.join("cgroup.controllers");
        if !cgroup_type_path.exists() {
            return false;
        }

        // Check if we have permission to create cgroups
        if !cgroup_mount.join("cgroup.procs").exists() {
            return false;
        }

        true
    }

    // Create a test security context with different permission levels
    fn create_test_context(level: PermissionLevel) -> SecurityContext {
        let mut context = SecurityContext::default();
        context.permission_level = level;
        
        match level {
            PermissionLevel::System => {
                context.allowed_capabilities.insert("system:admin".to_string());
                context.allowed_capabilities.insert("file:*".to_string());
                context.allowed_capabilities.insert("network:*".to_string());
            },
            PermissionLevel::User => {
                context.allowed_capabilities.insert("file:read".to_string());
                context.allowed_capabilities.insert("file:write".to_string());
                context.allowed_capabilities.insert("network:connect".to_string());
            },
            PermissionLevel::Restricted => {
                context.allowed_capabilities.insert("file:read".to_string());
                context.allowed_capabilities.insert("plugin:execute".to_string());
            },
        }
        
        // Add allowed paths
        let temp_dir = env::temp_dir();
        context.allowed_paths.push(temp_dir.clone());
        
        // Set resource limits based on permission level
        match level {
            PermissionLevel::System => {
                context.resource_limits.max_memory_bytes = 1024 * 1024 * 1024; // 1GB
                context.resource_limits.max_cpu_percent = 100;
            },
            PermissionLevel::User => {
                context.resource_limits.max_memory_bytes = 512 * 1024 * 1024; // 512MB
                context.resource_limits.max_cpu_percent = 50;
            },
            PermissionLevel::Restricted => {
                context.resource_limits.max_memory_bytes = 256 * 1024 * 1024; // 256MB
                context.resource_limits.max_cpu_percent = 25;
            },
        }
        
        context
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_create_destroy_sandbox() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_create_destroy_sandbox: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Create sandbox
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Check if cgroup path exists
        let cgroup_path = sandbox.get_cgroup_path(&plugin_id);
        assert!(cgroup_path.exists(), "Cgroup path does not exist");
        
        // Check if security context was created
        let context = sandbox.get_security_context(plugin_id).await;
        assert!(context.is_ok(), "Failed to get security context");
        
        // Destroy sandbox
        let result = sandbox.destroy_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to destroy sandbox: {:?}", result);
        
        // Check if cgroup path was removed
        assert!(!cgroup_path.exists(), "Cgroup path still exists after destruction");
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_security_contexts() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_security_contexts: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Set security context
        let context = create_test_context(PermissionLevel::User);
        let result = sandbox.set_security_context(plugin_id, context.clone()).await;
        assert!(result.is_ok(), "Failed to set security context: {:?}", result);
        
        // Get security context
        let retrieved_context = sandbox.get_security_context(plugin_id).await.unwrap();
        assert_eq!(retrieved_context.permission_level, PermissionLevel::User);
        assert!(retrieved_context.allowed_capabilities.contains("file:read"));
        
        // Create sandbox with the context
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Cleanup
        let _ = sandbox.destroy_sandbox(plugin_id).await;
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_resource_limits() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_resource_limits: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Set security context with resource limits
        let mut context = create_test_context(PermissionLevel::User);
        context.resource_limits.max_memory_bytes = 100 * 1024 * 1024; // 100MB
        context.resource_limits.max_cpu_percent = 30;
        
        let result = sandbox.set_security_context(plugin_id, context.clone()).await;
        assert!(result.is_ok(), "Failed to set security context: {:?}", result);
        
        // Create sandbox
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Check cgroup settings
        let cgroup_path = sandbox.get_cgroup_path(&plugin_id);
        
        // Verify memory limit was set properly
        let memory_max = sandbox.read_cgroup_file(&cgroup_path, "memory.max").await;
        assert!(memory_max.is_ok(), "Failed to read memory.max: {:?}", memory_max);
        let memory_max = memory_max.unwrap().trim().parse::<u64>();
        assert!(memory_max.is_ok(), "Failed to parse memory.max: {:?}", memory_max);
        let memory_max = memory_max.unwrap();
        assert_eq!(memory_max, 100 * 1024 * 1024, "Memory limit was not set correctly");
        
        // Verify CPU limit
        let cpu_max = sandbox.read_cgroup_file(&cgroup_path, "cpu.max").await;
        assert!(cpu_max.is_ok(), "Failed to read cpu.max: {:?}", cpu_max);
        
        // Cleanup
        let _ = sandbox.destroy_sandbox(plugin_id).await;
    }

    #[tokio::test]
    #[ignore = "Requires root permissions and cgroup v2"]
    async fn test_path_access() {
        if !can_run_cgroup_tests() {
            println!("Skipping test_path_access: cannot run cgroup tests on this system");
            return;
        }
        
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = LinuxCgroupSandbox::new(resource_monitor).unwrap();
        
        let plugin_id = Uuid::new_v4();
        
        // Set security context with specific path permissions
        let mut context = create_test_context(PermissionLevel::Restricted);
        let temp_file = env::temp_dir().join("test_file.txt");
        let home_dir = PathBuf::from("/home");
        
        context.allowed_paths.push(env::temp_dir());
        
        let result = sandbox.set_security_context(plugin_id, context.clone()).await;
        assert!(result.is_ok(), "Failed to set security context: {:?}", result);
        
        // Create sandbox
        let result = sandbox.create_sandbox(plugin_id).await;
        assert!(result.is_ok(), "Failed to create sandbox: {:?}", result);
        
        // Check access to allowed path
        let result = sandbox.check_path_access(plugin_id, &temp_file, false).await;
        assert!(result.is_ok(), "Access to allowed path denied: {:?}", result);
        
        // Check access to disallowed path
        let result = sandbox.check_path_access(plugin_id, &home_dir, false).await;
        assert!(result.is_err(), "Access to disallowed path granted");
        
        // Check write access (should be denied for Restricted level)
        let result = sandbox.check_path_access(plugin_id, &temp_file, true).await;
        assert!(result.is_err(), "Write access to path granted for Restricted level");
        
        // Cleanup
        let _ = sandbox.destroy_sandbox(plugin_id).await;
    }
} 