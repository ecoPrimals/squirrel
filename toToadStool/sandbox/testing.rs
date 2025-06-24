//! Testing Module for Plugin Sandbox
//!
//! This module contains comprehensive tests for the plugin sandbox functionality,
//! including basic sandbox, cross-platform sandbox, and resource monitoring tests.

#[cfg(test)]
pub mod tests {
    use std::env;
    use std::path::PathBuf;
    use std::sync::Arc;
    use uuid::Uuid;
    use tracing::debug;

    use crate::error::Result;
    use crate::plugin::security::{SecurityContext, PermissionLevel, ResourceLimits};
    use crate::plugin::resource_monitor::{ResourceMonitor, ResourceUsage};
    use super::basic::BasicPluginSandbox;
    use super::cross_platform::CrossPlatformSandbox;
    use super::traits::PluginSandbox;

    /// Helper function to create test security contexts
    fn create_test_context(level: PermissionLevel) -> SecurityContext {
        let mut context = SecurityContext::default();
        context.permission_level = level;
        
        // Add some test capabilities
        context.allowed_capabilities = vec![
            "test:capability".to_string(),
            "file:read".to_string(),
            "network:connect".to_string(),
            "plugin:execute".to_string(),
            "system:resource".to_string(),
        ].into_iter().collect();
        
        // Add test paths
        let temp_dir = env::temp_dir();
        context.allowed_paths = vec![
            temp_dir,
        ];
        
        // Set resource limits
        context.resource_limits = ResourceLimits {
            max_memory_bytes: 512 * 1024 * 1024, // 512 MB
            max_cpu_percent: 50,
            max_disk_mb: 1024,
            max_threads: 4,
        };
        
        context
    }
    
    #[tokio::test]
    async fn test_basic_sandbox() {
        let plugin_id = Uuid::new_v4();
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
        
        // Register the process with the resource monitor
        resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe().unwrap()).await.unwrap();
        
        // Set a security context
        let context = create_test_context(PermissionLevel::User);
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Check capabilities
        sandbox.check_capability(plugin_id, "test:capability").await.unwrap();
        
        // Check paths
        let temp_dir = env::temp_dir();
        sandbox.check_path_access(plugin_id, &temp_dir, true).await.unwrap();
        
        // Destroy sandbox
        sandbox.destroy_sandbox(plugin_id).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_restricted_sandbox() {
        let plugin_id = Uuid::new_v4();
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
        
        // Register the process with the resource monitor
        resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe().unwrap()).await.unwrap();
        
        // Set up the restricted security context
        let mut context = SecurityContext::default();
        let temp_dir = env::temp_dir();
        context.allowed_paths = vec![temp_dir.clone()];
        context.permission_level = PermissionLevel::Restricted;
        
        // Store the permission level before moving the context
        let permission_level = context.permission_level;
        
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Should allow basic operations
        sandbox.check_capability(plugin_id, "file:read").await.unwrap();
        
        // Should deny elevated operations
        let result = sandbox.check_capability(plugin_id, "system:admin").await;
        // This accommodates differences in sandbox implementations
        if permission_level == PermissionLevel::Restricted {
            // For restricted permissions, it should be an error or false
            assert!(result.is_err() || result.unwrap() == false);
        } else {
            // For other levels, the result may vary based on implementation
            let _ = result;
        }
        
        // Clean up
        sandbox.destroy_sandbox(plugin_id).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_path_access() {
        let plugin_id = Uuid::new_v4();
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
        
        // Register the process with the resource monitor
        resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe().unwrap()).await.unwrap();
        
        // Set up test context
        let mut context = SecurityContext::default();
        context.permission_level = PermissionLevel::User;
        
        // Add allowed paths
        let temp_dir = env::temp_dir();
        context.allowed_paths = vec![temp_dir.clone()];
        
        // Set context
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Should allow access to paths in allowed_paths
        let test_path = temp_dir.join("test_file.txt");
        sandbox.check_path_access(plugin_id, &test_path, false).await.unwrap();
        
        // Should allow write access to paths in write_allowed_paths
        sandbox.check_path_access(plugin_id, &test_path, true).await.unwrap();
        
        // Should deny access to paths not in allowed_paths
        let root_path = PathBuf::from("/some_restricted_path");
        let result = sandbox.check_path_access(plugin_id, &root_path, false).await;
        assert!(result.is_err());
        
        // Clean up
        sandbox.destroy_sandbox(plugin_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_resource_monitoring() {
        let plugin_id = Uuid::new_v4();
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
        
        // Register the process with the resource monitor
        resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe().unwrap()).await.unwrap();
        
        // Set resource limits
        let mut context = SecurityContext::default();
        context.resource_limits = ResourceLimits {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_percent: 10,
            max_disk_mb: 10,
            max_threads: 2,
        };
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Mock resource usage for testing
        let usage = ResourceUsage {
            cpu_percent: 10.0,
            memory_bytes: 100 * 1024 * 1024, // 100 MB
            disk_mb: 50.0,
            network_mb: 5.0,
            timestamp: chrono::Utc::now(),
        };
        resource_monitor.set_resource_usage_for_testing(plugin_id, usage.clone()).await.unwrap();
        
        // Track resources
        let tracked_usage = sandbox.track_resources(plugin_id).await.unwrap();
        
        // Verify tracking results
        assert_eq!(tracked_usage.cpu_percent, usage.cpu_percent);
        assert_eq!(tracked_usage.memory_bytes, usage.memory_bytes);
        
        // Clean up
        sandbox.destroy_sandbox(plugin_id).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_sandbox_capabilities() {
        let plugin_id = Uuid::new_v4();
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
        
        // Register the process with the resource monitor
        resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe().unwrap()).await.unwrap();
        
        // Set security context with specific capabilities
        let mut context = SecurityContext::default();
        context.allowed_capabilities = vec![
            "test:capability".to_string(),
            "namespace:*".to_string(), // Wildcard capability
        ].into_iter().collect();
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Should allow exact capability
        sandbox.check_capability(plugin_id, "test:capability").await.unwrap();
        
        // Should allow capability matching wildcard
        sandbox.check_capability(plugin_id, "namespace:specific").await.unwrap();
        
        // Should deny capability not in list
        let result = sandbox.check_capability(plugin_id, "other:capability").await;
        // This accommodates differences in sandbox implementations
        match result {
            Ok(has_capability) => assert!(!has_capability, "Capability should be denied"),
            Err(_) => {} // Error is also acceptable
        }
        
        // Clean up
        sandbox.destroy_sandbox(plugin_id).await.unwrap();
    }
    
    #[tokio::test]
    #[ignore] // This test requires platform detection so we'll ignore it in CI
    async fn test_cross_platform_sandbox() {
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = CrossPlatformSandbox::new(resource_monitor).unwrap();
        
        // Get platform capabilities
        let capabilities = sandbox.get_platform_capabilities();
        debug!("Platform capabilities: {:?}", capabilities);
        
        // Should return valid platform info
        let info = sandbox.get_platform_info();
        assert!(info.contains_key("platform"));
        assert!(info.contains_key("has_native_sandbox"));
    }
    
    #[tokio::test]
    async fn test_resource_monitor_integration() {
        // Create sandbox with resource monitor
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = BasicPluginSandbox::new(resource_monitor.clone());
        let plugin_id = Uuid::new_v4();
        
        // Register the process with the resource monitor
        resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe().unwrap()).await.unwrap();
        
        // Create a resource-limited context
        let mut context = SecurityContext::default();
        context.allowed_capabilities = vec![
            "system:resource".to_string()
        ].into_iter().collect();
        context.permission_level = PermissionLevel::Restricted;
        context.resource_limits = ResourceLimits {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_percent: 10,
            max_disk_mb: 10,
            max_threads: 2,
        };
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await.unwrap();
        
        // Set strict resource limits
        let mut context = SecurityContext::default();
        context.resource_limits = ResourceLimits {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_percent: 10,
            max_disk_mb: 10,
            max_threads: 2,
        };
        sandbox.set_security_context(plugin_id, context).await.unwrap();
        
        // Register test process
        let process_id = std::process::id();
        let executable_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("unknown"));
        resource_monitor.register_process(plugin_id, process_id, &executable_path).await.unwrap();
        
        // Test within limits
        let within_limits = ResourceUsage {
            cpu_percent: 5.0,
            memory_bytes: 50 * 1024 * 1024, // 50 MB
            disk_mb: 50.0,
            network_mb: 5.0,
            timestamp: chrono::Utc::now(),
        };
        resource_monitor.set_resource_usage_for_testing(plugin_id, within_limits).await.unwrap();
        
        // Track resources - should succeed
        let usage = sandbox.track_resources(plugin_id).await.unwrap();
        assert_eq!(usage.cpu_percent, 5.0);
        
        // Test beyond limits (for testing only - would trigger alerts in production)
        let beyond_limits = ResourceUsage {
            cpu_percent: 50.0, // Exceeds max_cpu_percent
            memory_bytes: 200 * 1024 * 1024, // 200 MB
            disk_mb: 50.0,
            network_mb: 5.0,
            timestamp: chrono::Utc::now(),
        };
        resource_monitor.set_resource_usage_for_testing(plugin_id, beyond_limits).await.unwrap();
        
        // Track resources - should still work in test mode
        let usage = sandbox.track_resources(plugin_id).await.unwrap();
        assert_eq!(usage.cpu_percent, 50.0);
        
        // Clean up
        sandbox.destroy_sandbox(plugin_id).await.unwrap();
    }
    
    /// Comprehensive test of all sandbox functionality
    #[tokio::test]
    pub async fn test_sandbox_functionality() -> Result<()> {
        // Create a sandbox
        let resource_monitor = Arc::new(ResourceMonitor::new());
        let sandbox = Arc::new(BasicPluginSandbox::new(resource_monitor.clone()));
        let plugin_id = Uuid::new_v4();
        
        // Register the process with the resource monitor
        resource_monitor.register_process(plugin_id, std::process::id(), &std::env::current_exe().unwrap()).await?;
        
        // Create security context
        let mut context = SecurityContext::default();
        context.resource_limits = ResourceLimits {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_percent: 10,
            max_disk_mb: 10,
            max_threads: 2,
        };
        
        // Set security context
        sandbox.set_security_context(plugin_id, context).await?;
        
        // Create sandbox
        sandbox.create_sandbox(plugin_id).await?;
        
        // Step 1: Create sandbox with system permission level
        sandbox.create_sandbox(plugin_id).await?;
        
        // Step 2: Set system permission context for full capability testing
        let system_context = create_test_context(PermissionLevel::System);
        sandbox.set_security_context(plugin_id, system_context).await?;
        
        // Step 3: Check higher-level capabilities (should pass with system level)
        sandbox.check_capability(plugin_id, "system:admin").await?;
        
        // Step 4: Check operation permission (uses capability mapping)
        sandbox.check_permission(plugin_id, "system:admin").await?;
        
        // Step 5: Downgrade to user level and verify restrictions
        let user_context = create_test_context(PermissionLevel::User);
        sandbox.set_security_context(plugin_id, user_context).await?;
        
        // Should allow user operations
        sandbox.check_permission(plugin_id, "filesystem:read").await?;
        
        // Should deny system operations
        let result = sandbox.check_permission(plugin_id, "system:admin").await;
        assert!(result.is_err());
        
        // Step 6: Try restricted level
        let restricted_context = create_test_context(PermissionLevel::Restricted);
        sandbox.set_security_context(plugin_id, restricted_context).await?;
        
        // Should still allow basic operations
        sandbox.check_permission(plugin_id, "filesystem:read").await?;
        
        // Should deny even user level operations
        let result = sandbox.check_permission(plugin_id, "filesystem:delete").await;
        assert!(result.is_err());
        
        // Step 7: Clean up
        sandbox.destroy_sandbox(plugin_id).await?;
        
        Ok(())
    }
} 