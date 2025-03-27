# Squirrel Plugin System Implementation

This document provides an overview of the implementation of the Squirrel Plugin System based on our implementation plan.

## Implementation Overview

The plugin system has been implemented with the following key components:

1. **Plugin Registry**: A central registry for managing and tracking plugins
2. **Plugin Loader**: Functionality for loading plugins from various sources
3. **Plugin Manager**: High-level API for plugin system management
4. **Example Plugins**: Example implementations of plugins to demonstrate usage

## Key Files

- `src/plugins/mod.rs`: Main module definitions and plugin manager
- `src/plugins/registry.rs`: Implementation of the plugin registry
- `src/plugins/loader.rs`: Implementation of plugin loading functionality
- `crates/example-plugins`: Example plugin implementations
- `crates/interfaces/src/plugins.rs`: Core interface definitions for plugins

## Using the Plugin System

### Initialization

To initialize the plugin system in your application, use the following code:

```rust
use crate::plugins::create_plugin_manager;
use std::path::PathBuf;

// Create the plugin manager
let plugin_manager = create_plugin_manager();

// Define plugin directories to search for dynamic plugins
let plugin_dirs = vec![PathBuf::from("./plugins")];

// Initialize the plugin system
let plugin_ids = plugin_manager.initialize(&plugin_dirs).await?;
```

### Accessing Plugins

Plugins can be accessed by ID or capability:

```rust
// Get the plugin registry
let registry = plugin_manager.registry();

// Get a plugin by ID
if let Some(plugin) = registry.get_plugin("plugin-id").await {
    // Use the plugin
}

// Get a plugin by capability
if let Some(cmd_plugin) = registry.get_plugin_by_capability("command_execution").await {
    // Use the command execution plugin
}

// Get a typed plugin by capability
if let Some(cmd_plugin) = registry.get_plugin_by_type_and_capability::<dyn CommandsPlugin>("command_execution").await {
    // Execute a command
    let result = cmd_plugin.execute_command("command.echo", serde_json::json!({
        "args": ["Hello", "World"]
    })).await?;
}
```

### Shutting Down

Make sure to shut down the plugin system when your application exits:

```rust
// Shut down the plugin system
plugin_manager.shutdown().await?;
```

## Creating Plugins

### Implementing a Plugin

To create a plugin, implement the `Plugin` trait and any specialized traits:

```rust
use async_trait::async_trait;
use anyhow::Result;
use squirrel_interfaces::plugins::{Plugin, PluginMetadata};

#[derive(Debug)]
struct MyPlugin {
    metadata: PluginMetadata,
}

impl MyPlugin {
    fn new() -> Self {
        let metadata = PluginMetadata::new(
            "my-plugin",
            env!("CARGO_PKG_VERSION"),
            "My plugin description",
            "Author Name",
        )
        .with_capability("my_capability");

        Self { metadata }
    }
}

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> Result<()> {
        // Initialization code here
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        // Cleanup code here
        Ok(())
    }
}
```

### Creating a Commands Plugin

To create a plugin that provides commands, implement the `CommandsPlugin` trait:

```rust
use async_trait::async_trait;
use serde_json::Value;
use squirrel_interfaces::plugins::{CommandsPlugin, CommandMetadata};

#[async_trait]
impl CommandsPlugin for MyPlugin {
    fn get_available_commands(&self) -> Vec<CommandMetadata> {
        // Return available commands
        vec![
            CommandMetadata {
                id: "my.command".to_string(),
                name: "my-command".to_string(),
                description: "My command description".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "param": {
                            "type": "string"
                        }
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {
                            "type": "string"
                        }
                    }
                }),
                permissions: vec!["my.permission".to_string()],
            }
        ]
    }

    fn get_command_metadata(&self, command_id: &str) -> Option<CommandMetadata> {
        // Return metadata for a specific command
        self.get_available_commands()
            .into_iter()
            .find(|cmd| cmd.id == command_id)
    }

    async fn execute_command(&self, command_id: &str, input: Value) -> Result<Value> {
        // Execute the command
        match command_id {
            "my.command" => {
                let param = input.get("param")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                
                Ok(serde_json::json!({
                    "result": format!("Command executed with param: {}", param)
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown command: {}", command_id)),
        }
    }

    fn get_command_help(&self, command_id: &str) -> Option<String> {
        // Return help text for a command
        match command_id {
            "my.command" => Some("Help text for my command".to_string()),
            _ => None,
        }
    }
}
```

### Creating a Dynamic Plugin

To create a plugin that can be loaded dynamically, export the required entry points:

```rust
use squirrel_interfaces::plugins::{Plugin, PluginMetadata};

#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    // Create the plugin
    let plugin = MyPlugin::new();
    Box::into_raw(Box::new(plugin))
}

#[no_mangle]
pub extern "C" fn get_plugin_metadata() -> *mut PluginMetadata {
    // Create the metadata
    let metadata = PluginMetadata::new(
        "my-plugin",
        "1.0.0",
        "My plugin description",
        "Author Name",
    );
    Box::into_raw(Box::new(metadata))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut dyn Plugin) {
    // Clean up resources
    if !plugin.is_null() {
        unsafe {
            let _ = Box::from_raw(plugin);
        }
    }
}
```

## Building and Loading Plugins

### Building a Dynamic Plugin

Configure your `Cargo.toml` to build a dynamic library:

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
anyhow = "1.0.75"
async-trait = "0.1.74"
serde = { version = "1.0.188", features = ["derive"] }
serde_json = "1.0.107"
tracing = "0.1.37"
squirrel-interfaces = { path = "../interfaces" }
```

### Loading Dynamic Plugins

Place your compiled plugin (`.dll`, `.so`, or `.dylib`) in a directory that the plugin loader will search:

```
squirrel/
├── plugins/
│   ├── my-plugin.dll
│   └── another-plugin.dll
```

Then configure the plugin manager to search this directory:

```rust
let plugin_dirs = vec![PathBuf::from("./plugins")];
plugin_manager.initialize(&plugin_dirs).await?;
```

## Security Considerations

1. **Plugin Isolation**: Plugins run in the same process as the main application, but can have limited access to resources.
2. **Capability-Based Security**: Plugins declare capabilities that can be checked before performing operations.
3. **Resource Limits**: Resource usage by plugins can be monitored and limited.
4. **Validation**: Plugin metadata and code can be validated before loading.

## Conclusion

The Squirrel Plugin System provides a robust and flexible way to extend the application's functionality. By following this implementation guide, you can create, load, and manage plugins for various purposes.

For more details, refer to the implementation code and the `specs/plugins/PLUGIN_IMPLEMENTATION_PLAN.md` document.

DataScienceBioLab; 