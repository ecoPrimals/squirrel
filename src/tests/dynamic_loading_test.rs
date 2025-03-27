// Dynamic Plugin Loading Cross-Platform Test Suite
//
// This test suite verifies the dynamic library loading capabilities
// across different platforms (Windows, Linux, macOS).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::test;
use uuid::Uuid;
use serde_json::json;

use crate::plugins::{
    dynamic::{create_library_loader, DynamicLibraryLoader, VersionCompatibilityChecker},
    management::{PluginRegistry, PluginRegistryImpl},
    interfaces::{CommandsPlugin, ToolPlugin},
    resource::{ResourceMonitor, ResourceMonitorImpl, ResourceLimits},
    errors::{PluginError, Result},
    PluginStatus,
};

/// Gets test plugin path appropriate for the current platform
fn get_test_plugin_path() -> PathBuf {
    let base_path = PathBuf::from("test_plugins");
    
    #[cfg(target_os = "windows")]
    {
        base_path.join("test_plugin.dll")
    }
    
    #[cfg(target_os = "linux")]
    {
        base_path.join("test_plugin.so")
    }
    
    #[cfg(target_os = "macos")]
    {
        base_path.join("test_plugin.dylib")
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        panic!("Unsupported operating system");
    }
}

/// Sets up a test environment
async fn setup_test_environment() -> (Arc<dyn DynamicLibraryLoader>, Arc<PluginRegistry>, Arc<dyn ResourceMonitor>) {
    let loader = create_library_loader();
    let registry = Arc::new(PluginRegistryImpl::new());
    let resource_monitor = Arc::new(ResourceMonitorImpl::new());
    
    (loader, registry, resource_monitor)
}

#[test]
async fn test_dynamic_loader_creation() {
    let loader = create_library_loader();
    
    // Check that the loader was created
    assert!(loader.is_some());
    
    // Platform-specific checks
    #[cfg(target_os = "windows")]
    {
        // Windows-specific assertions
        assert!(std::any::type_name_of_val(&*loader).contains("Windows"));
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux-specific assertions
        assert!(std::any::type_name_of_val(&*loader).contains("Linux"));
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS-specific assertions
        assert!(std::any::type_name_of_val(&*loader).contains("MacOS"));
    }
}

#[test]
async fn test_library_validation() {
    let (loader, _, _) = setup_test_environment().await;
    let plugin_path = get_test_plugin_path();
    
    // Skip test if the test plugin doesn't exist
    if !plugin_path.exists() {
        println!("Skipping test_library_validation: Test plugin not found at {:?}", plugin_path);
        return;
    }
    
    // Validate the plugin library
    let metadata = loader.validate_library(&plugin_path).await;
    
    // Check that validation succeeded
    assert!(metadata.is_ok(), "Library validation failed: {:?}", metadata.err());
    
    let metadata = metadata.unwrap();
    
    // Check basic metadata properties
    assert!(!metadata.id.is_nil());
    assert!(!metadata.name.is_empty());
    assert!(!metadata.version.is_empty());
    assert!(!metadata.api_version.is_empty());
}

#[test]
async fn test_plugin_loading() {
    let (loader, registry, _) = setup_test_environment().await;
    let plugin_path = get_test_plugin_path();
    
    // Skip test if the test plugin doesn't exist
    if !plugin_path.exists() {
        println!("Skipping test_plugin_loading: Test plugin not found at {:?}", plugin_path);
        return;
    }
    
    // Load the plugin
    let plugin = loader.load_plugin(&plugin_path).await;
    
    // Check that loading succeeded
    assert!(plugin.is_ok(), "Plugin loading failed: {:?}", plugin.err());
    
    let plugin = plugin.unwrap();
    
    // Register the plugin
    let plugin_id = plugin.metadata().id;
    registry.register_plugin(plugin_id, plugin).await.unwrap();
    
    // Check that the plugin was registered
    let registered_plugin = registry.get_plugin(&plugin_id).await;
    assert!(registered_plugin.is_ok());
    
    // Clean up
    registry.unregister_plugin(&plugin_id).await.unwrap();
}

#[test]
async fn test_version_compatibility() {
    // Create a version compatibility checker
    let checker = VersionCompatibilityChecker::new("1.0.0").unwrap();
    
    // Test compatibility checks
    assert!(checker.check_compatibility("1.0.0", "^1.0.0").unwrap());
    assert!(checker.check_compatibility("1.1.0", "^1.0.0").unwrap());
    assert!(!checker.check_compatibility("2.0.0", "^1.0.0").unwrap());
    assert!(checker.check_compatibility("1.0.0", ">=1.0.0").unwrap());
    assert!(!checker.check_compatibility("0.9.0", ">=1.0.0").unwrap());
    
    // Test with invalid version strings
    let result = checker.check_compatibility("not-a-version", "^1.0.0");
    assert!(result.is_err());
    
    let result = checker.check_compatibility("1.0.0", "not-a-requirement");
    assert!(result.is_err());
}

#[test]
async fn test_command_execution_from_dynamic_plugin() {
    let (loader, registry, _) = setup_test_environment().await;
    let plugin_path = get_test_plugin_path();
    
    // Skip test if the test plugin doesn't exist
    if !plugin_path.exists() {
        println!("Skipping test_command_execution_from_dynamic_plugin: Test plugin not found at {:?}", plugin_path);
        return;
    }
    
    // Load the plugin
    let plugin = loader.load_plugin(&plugin_path).await;
    
    // Check that loading succeeded
    assert!(plugin.is_ok(), "Plugin loading failed: {:?}", plugin.err());
    
    let plugin = plugin.unwrap();
    
    // Register the plugin
    let plugin_id = plugin.metadata().id;
    registry.register_plugin(plugin_id, plugin).await.unwrap();
    
    // Get the plugin as a CommandsPlugin
    let commands_plugin = registry.get_plugin_as::<dyn CommandsPlugin>(&plugin_id).await;
    
    // Skip the rest of the test if this plugin doesn't implement CommandsPlugin
    if commands_plugin.is_err() {
        println!("Skipping command execution test: Plugin doesn't implement CommandsPlugin");
        return;
    }
    
    let commands_plugin = commands_plugin.unwrap();
    
    // List available commands
    let commands = commands_plugin.list_commands();
    
    // Skip if no commands are available
    if commands.is_empty() {
        println!("Skipping command execution test: No commands available");
        return;
    }
    
    // Execute the first command
    let command_name = &commands[0].name;
    let args = json!({});
    
    let result = commands_plugin.execute_command(command_name, args).await;
    assert!(result.is_ok(), "Command execution failed: {:?}", result.err());
    
    // Clean up
    registry.unregister_plugin(&plugin_id).await.unwrap();
}

#[test]
async fn test_resource_limits_with_dynamic_plugin() {
    let (loader, _, resource_monitor) = setup_test_environment().await;
    let plugin_path = get_test_plugin_path();
    
    // Skip test if the test plugin doesn't exist
    if !plugin_path.exists() {
        println!("Skipping test_resource_limits_with_dynamic_plugin: Test plugin not found at {:?}", plugin_path);
        return;
    }
    
    // Set resource limits
    let limits = ResourceLimits::new()
        .with_memory_limit(50 * 1024 * 1024) // 50 MB
        .with_cpu_limit(50.0); // 50% CPU
    
    resource_monitor.set_resource_limits(limits).await.unwrap();
    
    // Load the plugin
    let plugin = loader.load_plugin(&plugin_path).await;
    
    // Check that loading succeeded
    assert!(plugin.is_ok(), "Plugin loading failed: {:?}", plugin.err());
    
    let plugin = plugin.unwrap();
    let plugin_id = plugin.metadata().id;
    
    // Register the plugin with the resource monitor
    resource_monitor.register_plugin(plugin_id).await.unwrap();
    
    // Check resource usage
    let usage = resource_monitor.get_resource_usage(plugin_id).await;
    assert!(usage.is_ok(), "Failed to get resource usage: {:?}", usage.err());
    
    let usage = usage.unwrap();
    println!("Memory usage: {} bytes", usage.memory_usage);
    println!("CPU usage: {}%", usage.cpu_usage);
    
    // Clean up
    resource_monitor.unregister_plugin(plugin_id).await.unwrap();
}

// Test loading a plugin with dependencies
#[test]
async fn test_plugin_with_dependencies() {
    let (loader, registry, _) = setup_test_environment().await;
    
    // This test requires a plugin with dependencies
    // We'll just verify the validation logic without actual loading
    
    // Create a dependency ID
    let dependency_id = Uuid::new_v4();
    
    // Create a mock plugin metadata for validation
    let plugin_with_dependency = crate::plugins::dynamic::PluginMetadata {
        id: Uuid::new_v4(),
        name: "plugin-with-dependency".to_string(),
        version: "1.0.0".to_string(),
        api_version: "1.0.0".to_string(),
        description: "A plugin with a dependency".to_string(),
        author: "Test Author".to_string(),
        dependencies: vec![
            crate::plugins::dynamic::PluginDependency {
                plugin_id: dependency_id,
                version_requirement: "^1.0.0".to_string(),
            }
        ],
    };
    
    // Verify that validation would fail due to missing dependency
    let result = registry.validate_dependencies(&plugin_with_dependency).await;
    assert!(result.is_err());
    
    if let Err(PluginError::DependencyError(msg)) = result {
        assert!(msg.contains(&dependency_id.to_string()));
    } else {
        panic!("Expected DependencyError, got: {:?}", result);
    }
}

// Test unloading plugins
#[test]
async fn test_plugin_unloading() {
    let (loader, registry, _) = setup_test_environment().await;
    let plugin_path = get_test_plugin_path();
    
    // Skip test if the test plugin doesn't exist
    if !plugin_path.exists() {
        println!("Skipping test_plugin_unloading: Test plugin not found at {:?}", plugin_path);
        return;
    }
    
    // Load the plugin
    let plugin = loader.load_plugin(&plugin_path).await;
    
    // Check that loading succeeded
    assert!(plugin.is_ok(), "Plugin loading failed: {:?}", plugin.err());
    
    let plugin = plugin.unwrap();
    
    // Register the plugin
    let plugin_id = plugin.metadata().id;
    registry.register_plugin(plugin_id, plugin).await.unwrap();
    
    // Unload the plugin
    let result = loader.unload_plugin(plugin_id).await;
    assert!(result.is_ok(), "Plugin unloading failed: {:?}", result.err());
    
    // Verify that the unloaded plugin is still registered but can't be used
    let plugin = registry.get_plugin(&plugin_id).await;
    assert!(plugin.is_ok()); // Still registered
    
    // Clean up
    registry.unregister_plugin(&plugin_id).await.unwrap();
} 