# Dynamic Plugin Development Guide

## Overview

This guide explains how to create and use dynamically loaded plugins with the Squirrel Plugin System. Dynamic plugins are loaded from shared libraries (.dll on Windows, .so on Linux, .dylib on macOS) and can extend the functionality of the Squirrel application at runtime.

## Table of Contents
1. [Creating a Dynamic Plugin](#creating-a-dynamic-plugin)
2. [Required Entry Points](#required-entry-points)
3. [Plugin Metadata](#plugin-metadata)
4. [Plugin Types](#plugin-types)
5. [Building Shared Libraries](#building-shared-libraries)
6. [Loading Dynamic Plugins](#loading-dynamic-plugins)
7. [Version Compatibility](#version-compatibility)
8. [Resource Management](#resource-management)
9. [State Persistence](#state-persistence)
10. [Best Practices](#best-practices)
11. [Troubleshooting](#troubleshooting)

## Creating a Dynamic Plugin

To create a dynamic plugin, you need to:

1. Implement the `Plugin` trait and any specialized traits (e.g., `CommandsPlugin`, `ToolPlugin`)
2. Export the required entry points
3. Build a shared library

Here's a simple example of a dynamic plugin:

```rust
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;
use squirrel_plugins::interfaces::{Plugin, CommandsPlugin, CommandInfo};
use squirrel_plugins::errors::Result;
use squirrel_plugins::dynamic::{PluginMetadata, PluginDependency};

#[derive(Debug)]
struct MyPlugin {
    id: Uuid,
    name: String,
    version: String,
    description: String,
    // ... other fields
}

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &dyn squirrel_mcp::plugins::interfaces::PluginMetadata {
        // Implementation for converting to MCP PluginMetadata
        unimplemented!()
    }

    async fn initialize(&self) -> Result<()> {
        println!("Initializing plugin: {}", self.name);
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        println!("Starting plugin: {}", self.name);
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        println!("Stopping plugin: {}", self.name);
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        println!("Shutting down plugin: {}", self.name);
        Ok(())
    }
}

// Implement specialized traits (e.g., CommandsPlugin, ToolPlugin)
#[async_trait]
impl CommandsPlugin for MyPlugin {
    fn get_commands(&self) -> Vec<CommandInfo> {
        // Implementation
        vec![]
    }

    async fn execute_command(&self, name: &str, args: Value) -> Result<Value> {
        // Implementation
        Ok(Value::Null)
    }

    fn get_command_help(&self, name: &str) -> Option<CommandHelp> {
        // Implementation
        None
    }

    fn get_command_schema(&self, name: &str) -> Option<Value> {
        // Implementation
        None
    }
}

// Create the plugin instance
fn create_my_plugin() -> Box<dyn Plugin> {
    let plugin = MyPlugin {
        id: Uuid::new_v4(),
        name: "my-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "My dynamic plugin".to_string(),
        // ... initialize other fields
    };
    Box::new(plugin)
}

// Export the required entry points
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    let plugin = create_my_plugin();
    Box::into_raw(plugin)
}

#[no_mangle]
pub extern "C" fn get_plugin_metadata() -> *mut PluginMetadata {
    let metadata = PluginMetadata {
        id: Uuid::new_v4(),
        name: "my-plugin".to_string(),
        version: "1.0.0".to_string(),
        api_version: "1.0.0".to_string(),
        description: "My dynamic plugin".to_string(),
        author: "Your Name".to_string(),
        dependencies: Vec::new(),
    };
    Box::into_raw(Box::new(metadata))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut dyn Plugin) {
    if !plugin.is_null() {
        unsafe {
            let _ = Box::from_raw(plugin);
        }
    }
}
```

## Required Entry Points

Every dynamic plugin must export the following functions:

1. **create_plugin**: Creates and returns an instance of your plugin
   ```rust
   #[no_mangle]
   pub extern "C" fn create_plugin() -> *mut dyn Plugin {
       let plugin = create_my_plugin();
       Box::into_raw(plugin)
   }
   ```

2. **get_plugin_metadata**: Returns metadata about the plugin
   ```rust
   #[no_mangle]
   pub extern "C" fn get_plugin_metadata() -> *mut PluginMetadata {
       let metadata = PluginMetadata {
           id: Uuid::new_v4(),
           name: "my-plugin".to_string(),
           version: "1.0.0".to_string(),
           api_version: "1.0.0".to_string(),
           description: "My dynamic plugin".to_string(),
           author: "Your Name".to_string(),
           dependencies: Vec::new(),
       };
       Box::into_raw(Box::new(metadata))
   }
   ```

3. **destroy_plugin**: Cleans up resources when the plugin is unloaded
   ```rust
   #[no_mangle]
   pub extern "C" fn destroy_plugin(plugin: *mut dyn Plugin) {
       if !plugin.is_null() {
           unsafe {
               let _ = Box::from_raw(plugin);
           }
       }
   }
   ```

## Plugin Metadata

The `PluginMetadata` struct contains information about your plugin:

```rust
pub struct PluginMetadata {
    pub id: Uuid,                    // Unique identifier for the plugin
    pub name: String,                // Plugin name
    pub version: String,             // Plugin version (semver format)
    pub api_version: String,         // API version the plugin is compatible with
    pub description: String,         // Plugin description
    pub author: String,              // Plugin author
    pub dependencies: Vec<PluginDependency>, // Plugin dependencies
}

pub struct PluginDependency {
    pub plugin_id: Uuid,             // Dependency plugin ID
    pub version_requirement: String, // Version requirement (semver format)
}
```

Ensure that:
- The `id` is stable across plugin versions
- The `version` follows semantic versioning (e.g., `1.0.0`)
- The `api_version` matches the plugin system's API version
- All dependencies are properly specified

## Plugin Types

Dynamic plugins can implement various specialized traits:

1. **CommandsPlugin**: For plugins that add commands to the application
   ```rust
   #[async_trait]
   impl CommandsPlugin for MyPlugin {
       fn get_commands(&self) -> Vec<CommandInfo> { /* ... */ }
       async fn execute_command(&self, name: &str, args: Value) -> Result<Value> { /* ... */ }
       fn get_command_help(&self, name: &str) -> Option<CommandHelp> { /* ... */ }
       fn get_command_schema(&self, name: &str) -> Option<Value> { /* ... */ }
   }
   ```

2. **ToolPlugin**: For plugins that add tools to the application
   ```rust
   #[async_trait]
   impl ToolPlugin for MyPlugin {
       fn get_tools(&self) -> Vec<ToolInfo> { /* ... */ }
       async fn execute_tool(&self, name: &str, args: Value) -> Result<Value> { /* ... */ }
       async fn check_tool_availability(&self, name: &str) -> Result<ToolAvailability> { /* ... */ }
       fn get_tool_metadata(&self, name: &str) -> Option<ToolMetadata> { /* ... */ }
   }
   ```

Your plugin can implement multiple traits to provide different functionalities.

## Building Shared Libraries

To build your plugin as a shared library, configure your `Cargo.toml` as follows:

```toml
[package]
name = "my-plugin"
version = "1.0.0"
authors = ["Your Name <your.email@example.com>"]
edition = "2021"

[lib]
name = "my_plugin"
crate-type = ["cdylib"]

[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
squirrel-plugins = { version = "0.1" }
tokio = { version = "1.0", features = ["full"] }
```

The key part is `crate-type = ["cdylib"]`, which tells Cargo to build a dynamic library.

Build your plugin with:

```bash
cargo build --release
```

The shared library will be generated in the `target/release` directory:
- Windows: `my_plugin.dll`
- Linux: `libmy_plugin.so`
- macOS: `libmy_plugin.dylib`

## Loading Dynamic Plugins

To load dynamic plugins:

```rust
use std::path::Path;
use squirrel_plugins::management::PluginRegistry;

async fn load_dynamic_plugin(registry: &PluginRegistry, path: &Path) -> Result<Uuid> {
    // Register the dynamic plugin
    let plugin_id = registry.register_dynamic_plugin(path).await?;
    
    // Initialize and start the plugin
    registry.initialize_plugin(plugin_id).await?;
    registry.start_plugin(plugin_id).await?;
    
    Ok(plugin_id)
}
```

## Version Compatibility

The plugin system includes a `VersionCompatibilityChecker` for verifying plugin compatibility:

```rust
use squirrel_plugins::dynamic::VersionCompatibilityChecker;

async fn check_plugin_compatibility(version: &str, requirement: &str) -> bool {
    let checker = VersionCompatibilityChecker::new("1.0.0")?;
    checker.check_compatibility(version, requirement)?
}
```

Version requirements follow semantic versioning syntax:
- `=1.0.0`: Exactly version 1.0.0
- `>=1.0.0`: Version 1.0.0 or higher
- `^1.0.0`: Compatible with 1.0.0 (same as >=1.0.0 <2.0.0)
- `~1.0.0`: Minor updates allowed (same as >=1.0.0 <1.1.0)

## Resource Management

Dynamic plugins must manage their resources properly:

1. **Memory Management**: Clean up resources in `destroy_plugin`
2. **File Handles**: Close all file handles
3. **Thread Management**: Properly join all threads
4. **Resource Limits**: Respect resource limits

The plugin system monitors resource usage with the `ResourceMonitor`:

```rust
use squirrel_plugins::resource::{ResourceMonitor, ResourceType};

async fn report_resource_usage(monitor: &dyn ResourceMonitor, plugin_id: Uuid) {
    // Report resource allocation
    monitor.report_allocation(plugin_id, ResourceType::Memory, 1024 * 1024).await?;
    
    // Get current resource usage
    let usage = monitor.get_usage(plugin_id).await?;
    println!("Memory usage: {} bytes", usage.memory);
    
    // Check for violations
    let violations = monitor.check_limits(plugin_id).await?;
    for violation in violations {
        println!("Resource violation: {:?}", violation);
    }
    
    // Report resource deallocation
    monitor.report_deallocation(plugin_id, ResourceType::Memory, 1024 * 1024).await?;
}
```

## State Persistence

Plugins can persist state between sessions using the `StateManager`:

```rust
use squirrel_plugins::state::{StateManager, PluginState};
use serde_json::json;

async fn save_plugin_state(state_manager: &dyn StateManager, plugin_id: Uuid) {
    // Create plugin state
    let state = PluginState::new(
        plugin_id,
        "1.0.0",
        json!({
            "settings": {
                "enabled": true,
                "config": {
                    "value": 42
                }
            }
        })
    );
    
    // Save state
    state_manager.save_state(&state).await?;
    
    // Load state later
    if let Some(loaded_state) = state_manager.load_state(plugin_id).await? {
        println!("Loaded state: {:?}", loaded_state);
    }
}
```

## Best Practices

1. **Error Handling**: Properly handle errors in all plugin operations
   ```rust
   async fn plugin_operation() -> Result<()> {
       // Wrap errors with proper context
       let result = some_operation().map_err(|e| {
           PluginError::ExecutionError(format!("Failed to perform operation: {}", e))
       })?;
       
       Ok(())
   }
   ```

2. **Resource Management**: Clean up all resources
   ```rust
   impl Drop for MyPlugin {
       fn drop(&mut self) {
           // Clean up resources
           println!("Cleaning up plugin resources");
       }
   }
   ```

3. **Thread Safety**: Use appropriate synchronization
   ```rust
   use tokio::sync::RwLock;
   
   struct MyPlugin {
       state: RwLock<HashMap<String, String>>,
   }
   
   impl MyPlugin {
       async fn update_state(&self, key: String, value: String) -> Result<()> {
           let mut state = self.state.write().await;
           state.insert(key, value);
           Ok(())
       }
   }
   ```

4. **Dependency Management**: Properly specify and handle dependencies
   ```rust
   PluginMetadata {
       // ...
       dependencies: vec![
           PluginDependency {
               plugin_id: Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap(),
               version_requirement: ">=1.0.0",
           },
       ],
       // ...
   }
   ```

5. **Version Compatibility**: Properly check version compatibility
   ```rust
   let checker = VersionCompatibilityChecker::new(APP_VERSION)?;
   if !checker.check_compatibility(plugin_version, ">=1.0.0")? {
       return Err(PluginError::IncompatibleVersion(
           format!("Plugin version {} is not compatible", plugin_version)
       ));
   }
   ```

## Troubleshooting

### Common Issues

1. **Library Loading Errors**:
   - Ensure the library is in the correct format for the platform (.dll, .so, .dylib)
   - Check for missing dependencies with tools like `ldd` (Linux) or `dependency walker` (Windows)
   - Verify that the library is built for the same architecture (x86, x64)

2. **Symbol Resolution Errors**:
   - Ensure all required entry points are exported with `#[no_mangle]`
   - Check the symbol names with `nm -D` (Linux) or `dumpbin /exports` (Windows)
   - Verify that the plugin implements all required traits

3. **Version Compatibility Issues**:
   - Ensure the plugin's `api_version` is compatible with the application
   - Check dependency version requirements
   - Update the plugin if necessary

4. **Resource Management Issues**:
   - Monitor resource usage with the `ResourceMonitor`
   - Clean up resources properly
   - Respect resource limits

5. **Concurrency Issues**:
   - Use proper synchronization primitives
   - Avoid blocking the event loop
   - Use `tokio::spawn_blocking` for CPU-intensive tasks

### Debugging Techniques

1. **Logging**: Add logging to your plugin
   ```rust
   use tracing::{debug, error, info, warn};
   
   async fn some_operation() -> Result<()> {
       info!("Performing operation");
       debug!("Details: {:?}", some_data);
       
       if let Err(e) = risky_operation() {
           error!("Operation failed: {}", e);
           return Err(PluginError::ExecutionError(e.to_string()));
       }
       
       Ok(())
   }
   ```

2. **Testing**: Write tests for your plugin
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[tokio::test]
       async fn test_plugin_initialization() {
           let plugin = create_my_plugin();
           let result = plugin.initialize().await;
           assert!(result.is_ok());
       }
   }
   ```

3. **Inspecting Libraries**: Use platform-specific tools
   - Linux: `ldd`, `nm`, `objdump`
   - Windows: `dependency walker`, `dumpbin`
   - macOS: `otool`, `nm`

4. **Plugin Validation**: Use the `validate_library` method
   ```rust
   use squirrel_plugins::dynamic::create_library_loader;
   
   async fn validate_plugin(path: &Path) -> Result<PluginMetadata> {
       let loader = create_library_loader();
       loader.validate_library(path).await
   }
   ```

By following this guide, you should be able to create, build, and debug dynamic plugins for the Squirrel Plugin System. 