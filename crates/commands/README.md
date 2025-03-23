# Squirrel Commands

This crate provides functionality for command registration, validation, and execution within the Squirrel system.

## Commands API

The core API revolves around the `Command` trait and `CommandRegistry`:

```rust
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: &[String]) -> CommandResult<String>;
    fn help(&self) -> String;
    fn parser(&self) -> clap::Command;
    fn clone_box(&self) -> Box<dyn Command>;
}

pub struct CommandRegistry {
    // Internal implementation
}

impl CommandRegistry {
    pub fn new() -> Self;
    pub fn register(&self, name: &str, command: Arc<dyn Command>) -> CommandResult<()>;
    pub fn execute(&self, name: &str, args: &Vec<String>) -> CommandResult<String>;
    pub fn list_commands(&self) -> CommandResult<Vec<String>>;
    pub fn get_help(&self, name: &str) -> CommandResult<String>;
    // More methods...
}
```

## Plugin System Integration

This crate now integrates with the unified plugin system via the adapter pattern. The core command system functionality is unchanged, but adapters provide compatibility with the new plugin architecture.

### Using Commands as Plugins

```rust
use squirrel_commands::register_plugin;
use squirrel_plugins::registry::PluginRegistry;
use squirrel_plugins::commands::CommandsPlugin;

// Create a plugin registry
let mut registry = PluginRegistry::new();

// Register commands as a plugin
let plugin_id = register_plugin(&mut registry)?;

// Get the commands plugin
let commands_plugin = registry.get_plugin_by_capability::<dyn CommandsPlugin>("command_execution")?;

// Execute commands through the plugin interface
let result = commands_plugin.execute_command("command.help", serde_json::json!({
    "args": []
})).await?;
```

### Using Both APIs Together

You can use both the original API and the plugin API together:

```rust
use squirrel_commands::factory::create_command_registry_with_plugin;
use squirrel_commands::CommandResult;

// Create both registry and plugin in one call
let (registry, plugin) = create_command_registry_with_plugin()?;

// Use the original API
let registry_guard = registry.lock()?;
let result1: CommandResult<String> = registry_guard.execute("help", &vec![]);

// Use the plugin API
let result2 = plugin.execute_command("command.help", serde_json::json!({
    "args": []
})).await?;
```

## Additional Documentation

- See `adapter/plugins/README.md` for detailed documentation on the plugin adapter
- See factory.rs for ways to create command registries and plugins
- See registry.rs for command registry documentation 