# Commands Plugin Adapter

This module provides adapters to integrate the existing commands crate with the unified plugin system architecture.

## Overview

The Commands Plugin Adapter allows the existing command system to be used with the new plugin architecture without modification to existing commands. It functions as a bridge between the two systems.

## Key Features

- Exposes existing commands as `CommandsPlugin` implementations
- Converts between command arguments and JSON input/output formats
- Preserves all existing command functionality
- Adds plugin lifecycle management (initialization, shutdown)
- Supports command help and metadata

## Usage

### Registering Commands as Plugins

```rust
use squirrel_commands::register_plugin;
use squirrel_plugins::registry::PluginRegistry;

// Create a plugin registry
let mut registry = PluginRegistry::new();

// Register commands as a plugin
let plugin_id = register_plugin(&mut registry)?;

// Now you can use the plugin registry to execute commands
let command_plugin = registry.get_plugin_by_capability::<dyn CommandsPlugin>("command_execution")?;
```

### Creating a Custom Plugin Adapter

```rust
use squirrel_commands::adapter::plugins::create_commands_plugin_adapter;
use squirrel_commands::factory::create_command_registry;
use squirrel_plugins::commands::CommandsPlugin;
use std::sync::Arc;

// Create a command registry with built-in commands
let registry = create_command_registry()?;

// Create the plugin adapter
let plugin = create_commands_plugin_adapter(registry);

// Initialize the plugin
plugin.initialize().await?;

// Use the plugin to execute commands
let commands = plugin.get_available_commands();
let input = serde_json::json!({ "args": ["arg1", "arg2"] });
let result = plugin.execute_command("command.test", input).await?;
```

### Using Factory Methods

```rust
use squirrel_commands::factory::create_command_registry_with_plugin;

// Create both registry and plugin in one call
let (registry, plugin) = create_command_registry_with_plugin()?;

// Now you can use both independently
```

## Implementation Details

The adapter implements the following interfaces:

1. `Plugin` - Core plugin interface with lifecycle methods
2. `CommandsPlugin` - Command-specific plugin interface

The adapter maintains a cache of command metadata for efficient access and converts between the command registry's string-based interface and the plugin system's JSON-based interface.

## Error Handling

Errors from the command registry are properly propagated through the plugin interface, with appropriate conversions to maintain context and traceability.

## Testing

The adapter includes comprehensive tests to ensure proper integration between the command system and plugin architecture. 