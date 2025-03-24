# Squirrel CLI Plugin System

## Overview

The Squirrel CLI Plugin System allows extending the functionality of the CLI with custom commands and features. Plugins can be:

1. **Built-in plugins** bundled with the CLI
2. **External plugins** installed by users

## Plugin Architecture

The plugin system follows a modular architecture with these key components:

### Core Components

1. **Plugin Interface** (`Plugin` trait) - Defines the contract for all plugins
2. **Plugin Manager** - Manages plugin lifecycle (loading, enabling, disabling)
3. **Plugin Registry** - Tracks installed and available plugins
4. **Plugin Factory** - Creates plugin instances

### Plugin Lifecycle

Plugins follow a well-defined lifecycle:

1. **Created** - Plugin is instantiated but not initialized
2. **Initialized** - Plugin has set up resources but not started
3. **Started** - Plugin is active and processing
4. **Stopped** - Plugin has stopped processing but resources are still allocated
5. **Cleaned** - Plugin has cleaned up resources
6. **Disposed** - Plugin is ready for removal from memory

## Creating Plugins

### Plugin Trait

All plugins must implement the `Plugin` trait:

```rust
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Get the plugin name
    fn name(&self) -> &str;
    
    /// Get the plugin version
    fn version(&self) -> &str;
    
    /// Get the plugin description
    fn description(&self) -> Option<&str>;
    
    /// Initialize the plugin
    async fn initialize(&self) -> Result<(), PluginError>;
    
    /// Register commands with the command registry
    fn register_commands(&self, registry: &mut CommandRegistry) -> Result<(), PluginError>;
    
    /// Get the commands provided by this plugin
    fn commands(&self) -> Vec<Arc<dyn Command>>;
    
    /// Execute plugin-specific functionality
    async fn execute(&self, args: &[String]) -> Result<String, PluginError>;
    
    /// Clean up plugin resources
    async fn cleanup(&self) -> Result<(), PluginError>;
}
```

### Example Plugin Implementation

Here's a basic plugin implementation:

```rust
pub struct MyPlugin {
    name: String,
    version: String,
    description: String,
    state: PluginState,
    commands: Vec<Arc<dyn Command>>,
}

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> Option<&str> {
        Some(&self.description)
    }
    
    async fn initialize(&self) -> Result<(), PluginError> {
        // Initialization logic
        Ok(())
    }
    
    fn register_commands(&self, registry: &mut CommandRegistry) -> Result<(), PluginError> {
        // Register commands
        for command in &self.commands {
            registry.register(command.name(), command.clone())?;
        }
        Ok(())
    }
    
    fn commands(&self) -> Vec<Arc<dyn Command>> {
        self.commands.clone()
    }
    
    async fn execute(&self, args: &[String]) -> Result<String, PluginError> {
        // Plugin-specific execution logic
        Ok("Plugin executed".to_string())
    }
    
    async fn cleanup(&self) -> Result<(), PluginError> {
        // Cleanup logic
        Ok(())
    }
}
```

### Creating Commands for Plugins

Plugins typically provide custom commands. Each command must implement the `Command` trait:

```rust
pub trait Command: Send + Sync {
    /// Returns the name of the command
    fn name(&self) -> &str;
    
    /// Returns a description of what the command does
    fn description(&self) -> &str;
    
    /// Executes the command with the given arguments
    fn execute(&self, args: &[String]) -> CommandResult<String>;
    
    /// Returns help text for the command
    fn help(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }
    
    /// Returns a parser for the command's arguments
    fn parser(&self) -> clap::Command;
    
    /// Clones the command into a new box
    fn clone_box(&self) -> Box<dyn Command>;
}
```

## Plugin Distribution

### Directory Structure

External plugins should follow this directory structure:

```
my-plugin/
├── plugin.toml        # Plugin metadata
├── lib/               # Plugin libraries
│   └── libplugin.so   # Plugin shared library
└── README.md          # Plugin documentation
```

### Plugin Metadata

The `plugin.toml` file should contain:

```toml
name = "my-plugin"
version = "1.0.0"
description = "My awesome plugin"
author = "Your Name"
homepage = "https://github.com/yourusername/my-plugin"
```

## Plugin Security

- Plugins are sandboxed to prevent system damage
- Resource limits are enforced
- Plugins must request permissions for system access

## Plugin Best Practices

1. **Follow the lifecycle** - Properly implement all lifecycle methods
2. **Clean up resources** - Always properly clean up in the `cleanup` method
3. **Handle errors** - Provide detailed error information
4. **Document your plugin** - Include clear documentation
5. **Validate input** - Always validate user input
6. **Respect resource limits** - Be mindful of memory and CPU usage

## Example Plugin

See `example_plugin.rs` for a complete, working example of a plugin implementation. 