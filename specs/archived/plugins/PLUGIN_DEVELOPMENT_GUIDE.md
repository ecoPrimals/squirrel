# Squirrel CLI Plugin Development Guide

## Overview

This guide provides detailed instructions for developing plugins for the Squirrel CLI. Plugins extend the CLI's functionality by adding new commands, features, and integrations.

## Plugin Architecture

The Squirrel CLI plugin system follows a modular architecture with these key components:

1. **Plugin Interface** - Core interface that all plugins must implement
2. **Plugin Lifecycle** - State transitions that plugins go through
3. **Command Integration** - Integration with the command system
4. **Security Model** - Constraints to ensure plugin security

## Creating a Plugin

### Step 1: Set Up Your Development Environment

First, set up your development environment:

```bash
# Create a new Rust library crate
cargo new --lib my-squirrel-plugin

# Add required dependencies to Cargo.toml
[dependencies]
squirrel-cli = "0.1.0"
async-trait = "0.1.0"
log = "0.4.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
clap = "3.0"
```

### Step 2: Implement the Plugin Trait

Create a struct that implements the `Plugin` trait:

```rust
use async_trait::async_trait;
use squirrel_cli::plugins::{Plugin, PluginError};
use squirrel_cli::commands::{Command, CommandRegistry};
use std::sync::Arc;

pub struct MyPlugin {
    name: String,
    version: String,
    description: String,
    commands: Vec<Arc<dyn Command>>,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            name: "my-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "My awesome plugin".to_string(),
            commands: vec![
                Arc::new(MyCommand::new()),
            ],
        }
    }
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
    
    async fn start(&self) -> Result<(), PluginError> {
        // Start any background tasks
        Ok(())
    }
    
    fn register_commands(&self, registry: &mut CommandRegistry) -> Result<(), PluginError> {
        for command in &self.commands {
            match registry.register(command.name(), command.clone()) {
                Ok(_) => log::info!("Registered command: {}", command.name()),
                Err(e) => {
                    return Err(PluginError::RegisterError(
                        format!("Failed to register command {}: {}", command.name(), e)
                    ));
                }
            }
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
    
    async fn stop(&self) -> Result<(), PluginError> {
        // Stop any background tasks
        Ok(())
    }
    
    async fn cleanup(&self) -> Result<(), PluginError> {
        // Cleanup resources
        Ok(())
    }
}
```

### Step 3: Implement Plugin Commands

Create commands that your plugin will provide:

```rust
use squirrel_cli::Command;
use clap::{App, Arg, ArgMatches};

pub struct MyCommand;

impl MyCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for MyCommand {
    fn name(&self) -> &str {
        "my-command"
    }
    
    fn description(&self) -> &str {
        "A command provided by my plugin"
    }
    
    fn execute(&self, args: &[String]) -> Result<String, CommandError> {
        Ok("My command executed successfully!".to_string())
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("my-command")
            .about("A command provided by my plugin")
            .arg(
                Arg::new("option")
                    .long("option")
                    .help("An option for my command")
                    .takes_value(true)
            )
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self::new())
    }
}
```

### Step 4: Create a Plugin Factory

Create a factory for your plugin:

```rust
use squirrel_cli::plugins::{PluginFactory, Plugin, PluginError};
use std::sync::Arc;

pub struct MyPluginFactory;

impl PluginFactory for MyPluginFactory {
    fn create(&self) -> Result<Arc<dyn Plugin>, PluginError> {
        Ok(Arc::new(MyPlugin::new()))
    }
}

// Create a C-compatible export function
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn PluginFactory {
    Box::into_raw(Box::new(MyPluginFactory))
}
```

### Step 5: Create the Plugin Metadata

Create a `plugin.toml` file:

```toml
name = "my-plugin"
version = "1.0.0"
description = "My awesome plugin"
author = "Your Name"
homepage = "https://github.com/yourusername/my-plugin"
```

## Plugin Lifecycle

Understand the plugin lifecycle to properly implement your plugin:

1. **Created** - Plugin is instantiated
2. **Initialized** - `initialize()` called, resources set up
3. **Started** - `start()` called, background tasks started
4. **Stopped** - `stop()` called, tasks stopped but resources still allocated
5. **Cleaned** - `cleanup()` called, resources released
6. **Disposed** - Plugin removed from memory

## Security Best Practices

1. **Validate all input** - Never trust user input without validation
2. **Handle resources carefully** - Always clean up resources
3. **Limit permissions** - Request only permissions you need
4. **Secure storage** - Use secure methods for storing sensitive data
5. **Error handling** - Provide detailed error information without leaking sensitive data

## Testing Your Plugin

### Unit Testing

Create unit tests for your plugin:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let plugin = MyPlugin::new();
        
        // Test initialization
        assert!(plugin.initialize().await.is_ok());
        
        // Test starting
        assert!(plugin.start().await.is_ok());
        
        // Test stopping
        assert!(plugin.stop().await.is_ok());
        
        // Test cleanup
        assert!(plugin.cleanup().await.is_ok());
    }
    
    #[test]
    fn test_commands() {
        let plugin = MyPlugin::new();
        let commands = plugin.commands();
        
        assert!(!commands.is_empty());
        assert_eq!(commands[0].name(), "my-command");
    }
}
```

### Integration Testing

Test your plugin with the Squirrel CLI:

```rust
#[cfg(test)]
mod integration_tests {
    use squirrel_cli::plugins::{PluginManager, PluginStatus};
    use squirrel_cli::commands::CommandRegistry;
    
    #[tokio::test]
    async fn test_plugin_registration() {
        let mut manager = PluginManager::new();
        let factory = super::MyPluginFactory;
        
        // Register factory
        assert!(manager.register_plugin_factory("my-plugin", Arc::new(factory)).is_ok());
        
        // Load plugin
        assert!(manager.load_plugin("my-plugin").is_ok());
        
        // Check plugin status
        let plugin = manager.get_plugin("my-plugin").unwrap();
        assert_eq!(plugin.status(), &PluginStatus::Enabled);
    }
}
```

## Distributing Your Plugin

### Plugin Package Structure

Your plugin package should have this structure:

```
my-plugin/
├── plugin.toml        # Plugin metadata
├── lib/               # Compiled libraries
│   └── libmyplugin.so # Linux
│   └── myplugin.dll   # Windows
│   └── libmyplugin.dylib # macOS
└── README.md          # Documentation
```

### Cross-Platform Considerations

- Compile for multiple platforms if possible
- Document platform-specific requirements
- Handle platform-specific paths and resources correctly

### Versioning

- Use semantic versioning (MAJOR.MINOR.PATCH)
- Document breaking changes clearly
- Provide upgrade/migration guides

## Advanced Topics

### Resource Management

- Track and limit resource usage
- Use appropriate data structures for performance
- Clean up resources in the correct order

### Dependency Management

- Declare plugin dependencies in plugin.toml
- Handle missing dependencies gracefully
- Validate dependency versions

### Persistence

- Use config API for persistent storage
- Save and restore plugin state
- Handle data migration between versions

## Troubleshooting

### Common Issues

1. **Plugin not loading**
   - Check file permissions
   - Verify plugin.toml format
   - Check for missing dependencies

2. **Commands not registering**
   - Ensure command names are unique
   - Check for registration errors

3. **Resource leaks**
   - Ensure cleanup is called
   - Use RAII pattern where possible

### Debugging

- Enable debug logging
- Use proper error handling
- Add telemetry to your plugin

## Examples

For complete examples, see the codebase:

- `crates/cli/src/plugins/example_plugin.rs` - Example plugin implementation
- `crates/cli/src/plugins/README.md` - Plugin system documentation

## Resources

- [Rust Plugin Development Documentation](https://doc.rust-lang.org/book/)
- [Squirrel CLI GitHub Repository](https://github.com/squirrel/squirrel-cli)
- [Rust Async Programming Guide](https://rust-lang.github.io/async-book/)
- [Command Pattern in Rust](https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html) 