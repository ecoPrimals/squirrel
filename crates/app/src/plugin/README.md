# Squirrel Plugin System

This module provides a comprehensive plugin system for extending the Squirrel AI Coding Assistant with custom functionality.

## Overview

The plugin system allows for extending various parts of the application:

- **Command Plugins**: Add new commands to the application
- **Tool Plugins**: Implement custom tools for specific operations
- **MCP Plugins**: Extend the Machine Context Protocol with new message types
- **UI Plugins**: (Sunsetted in MVP)

## Key Components

### Core Components

- `Plugin`: The base trait all plugins must implement
- `PluginManager`: Manages the lifecycle of plugins
- `PluginMetadata`: Describes a plugin including its capabilities and dependencies
- `PluginState`: Manages persistent state for plugins

### Plugin Types

- `CommandPlugin`: Interface for command-based plugins
- `ToolPlugin`: Interface for tool-based plugins
- `McpPlugin`: Interface for MCP protocol extensions

### Plugin Discovery

- `EnhancedPluginDiscovery`: Advanced plugin discovery system with caching and monitoring
- `EnhancedPluginLoader`: Automatically creates appropriate plugin types based on metadata

### Builder Patterns

- `CommandPluginBuilder`: Simplifies creation of command plugins
- `ToolPluginBuilder`: Simplifies creation of tool plugins
- `McpPluginBuilder`: Simplifies creation of MCP plugins

## Creating a Plugin

### 1. Create Plugin Metadata

Every plugin must have metadata to describe its capabilities and dependencies:

```rust
use squirrel_app::PluginMetadata;
use uuid::Uuid;

let metadata = PluginMetadata {
    id: Uuid::new_v4(),
    name: "my-plugin".to_string(),
    version: "0.1.0".to_string(),
    description: "My awesome plugin".to_string(),
    author: "Plugin Author".to_string(),
    dependencies: vec![],
    capabilities: vec!["command".to_string()],
};
```

### 2. Choose Plugin Type and Implement

#### Command Plugin Example

```rust
use squirrel_app::{CommandPluginBuilder, PluginMetadata};
use uuid::Uuid;
use serde_json::json;

// Create a command plugin
let plugin = CommandPluginBuilder::new(PluginMetadata {
    id: Uuid::new_v4(),
    name: "my-command-plugin".to_string(),
    version: "0.1.0".to_string(),
    description: "Custom commands".to_string(),
    author: "Plugin Author".to_string(),
    dependencies: vec![],
    capabilities: vec!["command".to_string()],
})
.with_command("hello", "Say hello to someone")
.with_command("goodbye", "Say goodbye to someone")
.build();
```

#### Tool Plugin Example

```rust
use squirrel_app::{ToolPluginBuilder, PluginMetadata};
use uuid::Uuid;
use serde_json::json;

// Create a tool plugin
let plugin = ToolPluginBuilder::new(PluginMetadata {
    id: Uuid::new_v4(),
    name: "my-tool-plugin".to_string(),
    version: "0.1.0".to_string(),
    description: "Custom tools".to_string(),
    author: "Plugin Author".to_string(),
    dependencies: vec![],
    capabilities: vec!["tool".to_string()],
})
.with_tool("analyze", json!({
    "language": "rust",
    "options": {
        "detailed": true
    }
}))
.with_tool("format", json!({
    "style": "default"
}))
.build();
```

#### MCP Plugin Example

```rust
use squirrel_app::{McpPluginBuilder, PluginMetadata};
use uuid::Uuid;

// Create an MCP plugin
let plugin = McpPluginBuilder::new(PluginMetadata {
    id: Uuid::new_v4(),
    name: "my-mcp-plugin".to_string(),
    version: "0.1.0".to_string(),
    description: "MCP extensions".to_string(),
    author: "Plugin Author".to_string(),
    dependencies: vec![],
    capabilities: vec!["mcp".to_string()],
})
.with_extension("custom-protocol")
.with_extension("advanced-context")
.build();

// Register message handlers
plugin.register_handler("custom-message", |msg| {
    Box::pin(async move {
        // Process the message
        Ok(json!({
            "status": "success",
            "message": "Custom message processed"
        }))
    })
}).await;
```

### 3. Advanced Plugin Implementation

For more complex plugins, implement the traits directly:

```rust
use squirrel_app::{Plugin, CommandPlugin, PluginMetadata, PluginState};
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug)]
struct MyPlugin {
    metadata: PluginMetadata,
    state: RwLock<Option<PluginState>>,
    // Custom fields
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Initialization logic
            Ok(())
        })
    }
    
    fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Cleanup logic
            Ok(())
        })
    }
    
    fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>> {
        Box::pin(async move {
            let guard = self.state.read().await;
            Ok(guard.clone())
        })
    }
    
    fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let mut guard = self.state.write().await;
            *guard = Some(state);
            Ok(())
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(Self {
            metadata: self.metadata.clone(),
            state: RwLock::new(None),
            // Clone other fields
        })
    }
}

#[async_trait]
impl CommandPlugin for MyPlugin {
    async fn execute_command(&self, command: &str, args: Value) -> Result<Value> {
        match command {
            "hello" => {
                let name = args.get("name").and_then(Value::as_str).unwrap_or("world");
                Ok(json!({ "message": format!("Hello, {}!", name) }))
            }
            "goodbye" => {
                let name = args.get("name").and_then(Value::as_str).unwrap_or("world");
                Ok(json!({ "message": format!("Goodbye, {}!", name) }))
            }
            _ => {
                Ok(json!({ "error": "Command not found" }))
            }
        }
    }
    
    async fn get_commands(&self) -> Result<Vec<String>> {
        Ok(vec!["hello".to_string(), "goodbye".to_string()])
    }
    
    fn get_command_help(&self, command: &str) -> Option<String> {
        match command {
            "hello" => Some("Say hello to someone. Args: name".to_string()),
            "goodbye" => Some("Say goodbye to someone. Args: name".to_string()),
            _ => None,
        }
    }
    
    fn list_commands(&self) -> Vec<String> {
        vec!["hello".to_string(), "goodbye".to_string()]
    }
    
    fn registry(&self) -> Arc<CommandRegistry> {
        // Return command registry
        Arc::new(CommandRegistry::new())
    }
}
```

## Using the Plugin Manager

The `PluginManager` provides comprehensive management of plugins:

```rust
use squirrel_app::{PluginManager, EnhancedPluginDiscovery, Plugin};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create plugin discovery
    let discovery = EnhancedPluginDiscovery::new(Path::new("./plugins"))?;
    
    // Create plugin manager
    let manager = PluginManager::new();
    
    // Discover and register plugins
    let plugins = discovery.scan().await?;
    
    for metadata in plugins {
        match discovery.get_plugin_path(metadata.id).await {
            Some(path) => {
                // Load and register the plugin
                let loader = EnhancedPluginLoader::new(discovery.clone());
                let plugin = loader.load_plugin(metadata.id).await?;
                manager.register_plugin(plugin).await?;
            }
            None => continue,
        }
    }
    
    // Resolve dependencies and load plugins
    manager.resolve_dependencies().await?;
    manager.load_all_plugins().await?;
    
    // Get plugins by capability
    let command_plugins = manager.get_all_command_plugins().await;
    
    // Work with plugins
    for (id, plugin) in command_plugins {
        let commands = plugin.list_commands();
        println!("Plugin {} provides commands: {:?}", id, commands);
        
        // Execute a command
        if commands.contains(&"hello".to_string()) {
            let result = plugin.execute_command("hello", json!({ "name": "User" })).await?;
            println!("Command result: {}", result);
        }
    }
    
    // Shutdown when done
    manager.unload_all_plugins().await?;
    manager.shutdown().await?;
    
    Ok(())
}
```

## Plugin Discovery

The enhanced plugin discovery system scans directories for plugin metadata files:

```rust
use squirrel_app::EnhancedPluginDiscovery;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create plugin discovery
    let discovery = EnhancedPluginDiscovery::new(Path::new("./plugins"))?
        .with_scan_interval(30); // Scan every 30 seconds
    
    // Scan for plugins
    let plugins = discovery.scan().await?;
    
    // Find plugins by capability
    let command_plugins = discovery.find_plugins_by_capability("command").await;
    
    // Find a specific plugin
    let my_plugin = discovery.find_plugin_by_name("my-plugin").await;
    
    Ok(())
}
```

## Plugin State Persistence

Plugins can persist state between sessions:

```rust
use squirrel_app::{PluginState, PluginManager};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = PluginManager::new();
    
    // Register and load plugins...
    
    // Get a plugin
    let plugin_id = Uuid::parse_str("...")?;
    
    // Save plugin state
    let state = PluginState {
        plugin_id,
        data: json!({
            "count": 42,
            "settings": {
                "enabled": true
            }
        }),
        last_modified: Utc::now(),
    };
    
    manager.set_plugin_state(state).await?;
    
    // Later, load the state
    if let Some(state) = manager.get_plugin_state(plugin_id).await {
        let count = state.data.get("count").and_then(|v| v.as_i64()).unwrap_or(0);
        println!("Saved count: {}", count);
    }
    
    Ok(())
}
```

## Security Features

The plugin system includes security features:

```rust
use squirrel_app::{PluginManager, SecurityValidator, ResourceLimits};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create plugin manager with security enabled
    let mut manager = PluginManager::new();
    manager.with_security();
    
    // Register and load plugins...
    
    // Validate operations
    let plugin_id = Uuid::parse_str("...")?;
    manager.validate_operation(plugin_id, "file_access").await?;
    
    // Track resource usage
    if let Some(usage) = manager.track_resources(plugin_id).await? {
        println!("Memory: {} bytes, CPU: {}%", usage.memory_bytes, usage.cpu_percent);
    }
    
    Ok(())
}
```

## Best Practices

1. **Dependency Management**: Properly specify dependencies in plugin metadata
2. **Error Handling**: Use proper error propagation and recovery
3. **Resource Management**: Be mindful of resource usage in plugins
4. **Security**: Don't request more permissions than needed
5. **State Management**: Use plugin state for persistent data
6. **Documentation**: Document your plugin capabilities and commands
7. **Testing**: Write comprehensive tests for your plugins

## Tutorials

See the `examples/` directory for complete plugin examples:

- `examples/command_plugin/`: Creating a command plugin
- `examples/tool_plugin/`: Creating a tool plugin
- `examples/mcp_plugin/`: Creating an MCP plugin
- `examples/advanced_plugin/`: Advanced plugin implementation

## API Reference

For detailed API documentation, see the rustdoc comments in the source code. 