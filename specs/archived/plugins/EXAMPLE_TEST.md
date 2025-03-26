# Plugin System Testing Examples

This document provides examples of test implementations for the Squirrel Plugin System based on the testing specification. These examples can be used as templates for developing the test suite.

## 1. Unit Tests for Plugin Registry

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

use squirrel_plugins::plugins::registry::{PluginRegistry, PluginRegistryImpl};
use squirrel_plugins::plugins::dynamic::PluginMetadata;
use squirrel_plugins::plugins::errors::{PluginError, Result};

#[tokio::test]
async fn test_plugin_registry_register_and_lookup() -> Result<()> {
    // Create a new plugin registry
    let registry = PluginRegistryImpl::new();
    
    // Create test plugin metadata
    let plugin_id = uuid::Uuid::new_v4();
    let metadata = PluginMetadata {
        id: plugin_id,
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        api_version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Tester".to_string(),
        dependencies: Vec::new(),
    };
    
    // Register the plugin
    registry.register_plugin(plugin_id, metadata.clone()).await?;
    
    // Look up the plugin
    let retrieved_metadata = registry.get_plugin_metadata(plugin_id).await?;
    
    // Verify that the retrieved metadata matches the original
    assert_eq!(retrieved_metadata.id, metadata.id);
    assert_eq!(retrieved_metadata.name, metadata.name);
    assert_eq!(retrieved_metadata.version, metadata.version);
    assert_eq!(retrieved_metadata.api_version, metadata.api_version);
    assert_eq!(retrieved_metadata.description, metadata.description);
    assert_eq!(retrieved_metadata.author, metadata.author);
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_registry_lookup_nonexistent_plugin() {
    // Create a new plugin registry
    let registry = PluginRegistryImpl::new();
    
    // Generate a random plugin ID
    let plugin_id = uuid::Uuid::new_v4();
    
    // Attempt to look up a plugin that doesn't exist
    let result = registry.get_plugin_metadata(plugin_id).await;
    
    // Verify that the operation returns a NotFoundError
    match result {
        Err(PluginError::NotFoundError(_)) => {
            // This is the expected outcome
            assert!(true);
        }
        _ => {
            // This is unexpected
            assert!(false, "Expected NotFoundError but got {:?}", result);
        }
    }
}

#[tokio::test]
async fn test_plugin_registry_unregister_plugin() -> Result<()> {
    // Create a new plugin registry
    let registry = PluginRegistryImpl::new();
    
    // Create test plugin metadata
    let plugin_id = uuid::Uuid::new_v4();
    let metadata = PluginMetadata {
        id: plugin_id,
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        api_version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Tester".to_string(),
        dependencies: Vec::new(),
    };
    
    // Register the plugin
    registry.register_plugin(plugin_id, metadata).await?;
    
    // Unregister the plugin
    registry.unregister_plugin(plugin_id).await?;
    
    // Attempt to look up the unregistered plugin
    let result = registry.get_plugin_metadata(plugin_id).await;
    
    // Verify that the operation returns a NotFoundError
    match result {
        Err(PluginError::NotFoundError(_)) => {
            // This is the expected outcome
            assert!(true);
        }
        _ => {
            // This is unexpected
            assert!(false, "Expected NotFoundError but got {:?}", result);
        }
    }
    
    Ok(())
}
```

## 2. Integration Tests for Plugin Loading

```rust
use std::path::PathBuf;
use std::sync::Arc;

use squirrel_plugins::plugins::dynamic::{DynamicLoader, DynamicLoaderImpl};
use squirrel_plugins::plugins::registry::{PluginRegistry, PluginRegistryImpl};
use squirrel_plugins::plugins::manager::{PluginManager, PluginManagerImpl};
use squirrel_plugins::plugins::errors::Result;

#[tokio::test]
async fn test_plugin_manager_loads_dynamic_plugin() -> Result<()> {
    // Create dependencies
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    
    // Create plugin manager
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone());
    
    // Path to test plugin (this would be a real .dll/.so file in a real test)
    let plugin_path = PathBuf::from("path/to/test_plugin");
    
    // Load the plugin
    let plugin_id = manager.load_plugin(&plugin_path).await?;
    
    // Verify that the plugin was registered
    let metadata = registry.get_plugin_metadata(plugin_id).await?;
    
    // Verify that the metadata contains expected values
    assert_eq!(metadata.id, plugin_id);
    assert!(!metadata.name.is_empty());
    assert!(!metadata.version.is_empty());
    
    // Unload the plugin
    manager.unload_plugin(plugin_id).await?;
    
    // Verify that the plugin was unregistered
    let result = registry.get_plugin_metadata(plugin_id).await;
    assert!(result.is_err());
    
    Ok(())
}
```

## 3. System Tests for End-to-End Plugin Usage

```rust
use std::path::PathBuf;
use std::sync::Arc;

use squirrel_plugins::plugins::dynamic::{DynamicLoader, DynamicLoaderImpl};
use squirrel_plugins::plugins::registry::{PluginRegistry, PluginRegistryImpl};
use squirrel_plugins::plugins::manager::{PluginManager, PluginManagerImpl};
use squirrel_plugins::plugins::commands::{CommandsPlugin, CommandResult};
use squirrel_plugins::plugins::context::PluginContext;
use squirrel_plugins::plugins::errors::Result;

// This test simulates a full workflow of loading a plugin and executing commands
#[tokio::test]
async fn test_end_to_end_command_plugin_execution() -> Result<()> {
    // Create dependencies
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    
    // Create plugin manager
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone());
    
    // Load the test command plugin
    let plugin_path = PathBuf::from("path/to/command_plugin");
    let plugin_id = manager.load_plugin(&plugin_path).await?;
    
    // Create a plugin context
    let context = PluginContext::new();
    
    // Get the CommandsPlugin interface
    let commands_plugin = manager.get_plugin_as::<dyn CommandsPlugin>(plugin_id).await?;
    
    // List available commands
    let commands = commands_plugin.list_commands().await?;
    assert!(!commands.is_empty());
    
    // Execute a test command
    let command_name = commands[0].name.clone();
    let args = serde_json::json!({
        "parameter1": "value1",
        "parameter2": 42
    });
    
    let result = commands_plugin.execute_command(&command_name, &args, &context).await?;
    
    // Verify the command result
    match result {
        CommandResult::Success(data) => {
            // Verify that the returned data is as expected
            assert!(data.is_object() || data.is_array() || data.is_string());
        }
        CommandResult::Error(message) => {
            panic!("Command execution failed: {}", message);
        }
    }
    
    // Unload the plugin
    manager.unload_plugin(plugin_id).await?;
    
    Ok(())
}
```

## 4. Performance Tests

```rust
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use criterion::{criterion_group, criterion_main, Criterion};
use squirrel_plugins::plugins::dynamic::{DynamicLoader, DynamicLoaderImpl};
use squirrel_plugins::plugins::registry::{PluginRegistry, PluginRegistryImpl};
use squirrel_plugins::plugins::manager::{PluginManager, PluginManagerImpl};

fn bench_plugin_load_time(c: &mut Criterion) {
    // Set up the test environment
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone());
    let plugin_path = PathBuf::from("path/to/test_plugin");
    
    // Benchmark plugin loading
    c.bench_function("plugin_load", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let start = Instant::now();
                let plugin_id = manager.load_plugin(&plugin_path).await.unwrap();
                let load_time = start.elapsed();
                
                // Clean up
                manager.unload_plugin(plugin_id).await.unwrap();
                
                load_time
            })
        });
    });
}

fn bench_plugin_api_call_latency(c: &mut Criterion) {
    // Set up the test environment
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone());
    let plugin_path = PathBuf::from("path/to/test_plugin");
    
    // Load the plugin
    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin_id = rt.block_on(async {
        manager.load_plugin(&plugin_path).await.unwrap()
    });
    
    // Benchmark API call latency
    c.bench_function("plugin_api_call", |b| {
        b.iter(|| {
            rt.block_on(async {
                let commands_plugin = manager.get_plugin_as::<dyn CommandsPlugin>(plugin_id).await.unwrap();
                let context = PluginContext::new();
                
                let start = Instant::now();
                let _ = commands_plugin.execute_command("test_command", &serde_json::json!({}), &context).await.unwrap();
                start.elapsed()
            })
        });
    });
    
    // Clean up
    rt.block_on(async {
        manager.unload_plugin(plugin_id).await.unwrap();
    });
}

criterion_group!(benches, bench_plugin_load_time, bench_plugin_api_call_latency);
criterion_main!(benches);
```

## 5. Security Tests

```rust
use std::path::PathBuf;
use std::sync::Arc;

use squirrel_plugins::plugins::dynamic::{DynamicLoader, DynamicLoaderImpl};
use squirrel_plugins::plugins::registry::{PluginRegistry, PluginRegistryImpl};
use squirrel_plugins::plugins::manager::{PluginManager, PluginManagerImpl};
use squirrel_plugins::plugins::security::{PluginSecurityManager, SecurityPolicy};
use squirrel_plugins::plugins::errors::Result;

#[tokio::test]
async fn test_plugin_security_unauthorized_access_blocked() -> Result<()> {
    // Create dependencies
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    let security_manager = Arc::new(PluginSecurityManager::new());
    
    // Create a restrictive security policy
    let policy = SecurityPolicy::new()
        .with_filesystem_access(false)
        .with_network_access(false)
        .with_process_creation(false);
    
    // Apply the policy to the security manager
    security_manager.set_default_policy(policy).await?;
    
    // Create plugin manager with security
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone())
        .with_security_manager(security_manager.clone());
    
    // Load a plugin that might try to access restricted resources
    let plugin_path = PathBuf::from("path/to/potentially_unsafe_plugin");
    let plugin_id = manager.load_plugin(&plugin_path).await?;
    
    // Create a plugin context
    let context = PluginContext::new();
    
    // Get the CommandsPlugin interface
    let commands_plugin = manager.get_plugin_as::<dyn CommandsPlugin>(plugin_id).await?;
    
    // Execute a command that attempts to access the filesystem
    let args = serde_json::json!({
        "file_path": "/etc/passwd"
    });
    
    let result = commands_plugin.execute_command("read_file", &args, &context).await;
    
    // The command should fail due to security policy
    assert!(result.is_err());
    
    // Verify that the error is a security error
    match result {
        Err(PluginError::SecurityError(_)) => {
            // This is the expected outcome
            assert!(true);
        }
        _ => {
            // This is unexpected
            assert!(false, "Expected SecurityError but got {:?}", result);
        }
    }
    
    // Unload the plugin
    manager.unload_plugin(plugin_id).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_security_invalid_signature_rejected() -> Result<()> {
    // Create dependencies
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    let security_manager = Arc::new(PluginSecurityManager::new());
    
    // Enable signature verification
    security_manager.enable_signature_verification(true).await?;
    
    // Add a trusted key
    security_manager.add_trusted_key("trusted_key").await?;
    
    // Create plugin manager with security
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone())
        .with_security_manager(security_manager.clone());
    
    // Attempt to load a plugin with an invalid signature
    let plugin_path = PathBuf::from("path/to/plugin_with_invalid_signature");
    let result = manager.load_plugin(&plugin_path).await;
    
    // The plugin loading should fail due to invalid signature
    assert!(result.is_err());
    
    // Verify that the error is a signature verification error
    match result {
        Err(PluginError::SecurityError(_)) => {
            // This is the expected outcome
            assert!(true);
        }
        _ => {
            // This is unexpected
            assert!(false, "Expected SecurityError but got {:?}", result);
        }
    }
    
    Ok(())
}
```

## 6. Cross-Platform Tests

```rust
use std::path::PathBuf;
use std::sync::Arc;

use squirrel_plugins::plugins::dynamic::{DynamicLoader, DynamicLoaderImpl};
use squirrel_plugins::plugins::registry::{PluginRegistry, PluginRegistryImpl};
use squirrel_plugins::plugins::manager::{PluginManager, PluginManagerImpl};
use squirrel_plugins::plugins::errors::Result;

#[tokio::test]
#[cfg(target_os = "windows")]
async fn test_plugin_loading_windows() -> Result<()> {
    // Windows-specific test for loading a .dll plugin
    
    // Create dependencies
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    
    // Create plugin manager
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone());
    
    // Load a Windows-specific .dll plugin
    let plugin_path = PathBuf::from("path/to/windows_plugin.dll");
    let plugin_id = manager.load_plugin(&plugin_path).await?;
    
    // Verify that the plugin was loaded
    let metadata = registry.get_plugin_metadata(plugin_id).await?;
    assert_eq!(metadata.id, plugin_id);
    
    // Unload the plugin
    manager.unload_plugin(plugin_id).await?;
    
    Ok(())
}

#[tokio::test]
#[cfg(target_os = "linux")]
async fn test_plugin_loading_linux() -> Result<()> {
    // Linux-specific test for loading a .so plugin
    
    // Create dependencies
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    
    // Create plugin manager
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone());
    
    // Load a Linux-specific .so plugin
    let plugin_path = PathBuf::from("path/to/linux_plugin.so");
    let plugin_id = manager.load_plugin(&plugin_path).await?;
    
    // Verify that the plugin was loaded
    let metadata = registry.get_plugin_metadata(plugin_id).await?;
    assert_eq!(metadata.id, plugin_id);
    
    // Unload the plugin
    manager.unload_plugin(plugin_id).await?;
    
    Ok(())
}

#[tokio::test]
#[cfg(target_os = "macos")]
async fn test_plugin_loading_macos() -> Result<()> {
    // macOS-specific test for loading a .dylib plugin
    
    // Create dependencies
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    
    // Create plugin manager
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone());
    
    // Load a macOS-specific .dylib plugin
    let plugin_path = PathBuf::from("path/to/macos_plugin.dylib");
    let plugin_id = manager.load_plugin(&plugin_path).await?;
    
    // Verify that the plugin was loaded
    let metadata = registry.get_plugin_metadata(plugin_id).await?;
    assert_eq!(metadata.id, plugin_id);
    
    // Unload the plugin
    manager.unload_plugin(plugin_id).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_path_handling_cross_platform() -> Result<()> {
    // Test that path handling works correctly on all platforms
    
    // Create dependencies
    let registry = Arc::new(PluginRegistryImpl::new());
    let loader = Arc::new(DynamicLoaderImpl::new());
    
    // Create plugin manager
    let manager = PluginManagerImpl::new(registry.clone(), loader.clone());
    
    // Test path with platform-specific separators
    let mut plugin_path = PathBuf::new();
    
    #[cfg(target_os = "windows")]
    {
        plugin_path.push("path\\to\\plugin.dll");
    }
    
    #[cfg(target_os = "linux")]
    {
        plugin_path.push("path/to/plugin.so");
    }
    
    #[cfg(target_os = "macos")]
    {
        plugin_path.push("path/to/plugin.dylib");
    }
    
    // This should normalize the path correctly for the current platform
    let normalized_path = manager.normalize_plugin_path(&plugin_path)?;
    
    // Verify that the path is normalized correctly
    assert!(normalized_path.is_absolute() || normalized_path.starts_with("./"));
    
    Ok(())
}
```

## Conclusion

The examples provided in this document illustrate how to implement tests for different aspects of the Squirrel Plugin System. These examples can be expanded and adapted to provide comprehensive test coverage for the plugin system components.

When implementing these tests, it's important to:

1. Create realistic test plugins for integration and system tests
2. Use mocks when testing in isolation
3. Consider platform-specific behaviors
4. Test both normal and error cases
5. Verify proper resource cleanup

By following these guidelines and expanding on the examples, a robust test suite can be created to ensure the plugin system works correctly in all scenarios. 