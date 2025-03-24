# Command Plugin Adapter

This directory contains the implementation of the commands plugin adapter, which integrates the existing command system with the unified plugin architecture. The adapter follows the adapter pattern, allowing commands to be used through the plugin interface while maintaining backward compatibility with the existing API.

## Overview

The command plugin adapter converts the command registry API to the plugin system interface, allowing commands to be:

1. Discovered through the plugin system
2. Executed through the plugin interface
3. Managed through standard plugin lifecycle events
4. Documented through standardized metadata

## Architecture

The implementation uses an adapter pattern:

```
┌────────────────────┐     ┌─────────────────────────┐
│                    │     │                         │
│  CommandRegistry   │◄────┤  CommandsPluginAdapter  │◄─── Plugin API
│                    │     │                         │
└────────────────────┘     └─────────────────────────┘
        │                              │
        │                              │
        ▼                              ▼
┌────────────────────┐     ┌─────────────────────────┐
│                    │     │                         │
│  Command Objects   │     │  Plugin Registry        │
│                    │     │                         │
└────────────────────┘     └─────────────────────────┘
        │
        │
        ▼
   Original API
```

## Key Components

### CommandsPluginAdapter

The core adapter class that implements the `Plugin` and `CommandsPlugin` traits:

- Maintains a reference to the wrapped command registry
- Caches command metadata for performance
- Converts between command registry and plugin interfaces
- Manages plugin lifecycle events

### Command Conversion

Commands are exposed to the plugin system with:
- ID format: `command.<name>` (e.g., `command.help`)
- JSON schema for input/output
- Standard permissions
- Consistent error handling

### Factory Methods

For ease of use, factory methods are provided:
- `create_commands_plugin_adapter()`: Creates a plugin adapter from an existing registry
- `create_command_registry_with_plugin()`: Creates both a registry and adapter in one call

## Usage Example

```rust
use squirrel_commands::factory::create_command_registry_with_plugin;
use squirrel_plugins::commands::CommandsPlugin;
use serde_json::json;

// Create registry and plugin
let (registry, plugin) = create_command_registry_with_plugin()?;

// Use the plugin to get available commands
let commands = plugin.get_available_commands();
println!("Available commands: {}", commands.len());

// Execute a command via the plugin
let result = plugin.execute_command(
    "command.help", 
    json!({ "args": ["help"] })
).await?;

println!("Result: {}", result);

// Continue using the registry directly as needed
let registry_guard = registry.lock().unwrap();
let output = registry_guard.execute("help", &[])?;
println!("Direct output: {}", output);
```

## Current Limitations

1. **No Dynamic Registration**: Changes to the command registry after adapter initialization aren't automatically reflected in the plugin
2. **Limited Schema Information**: Command argument schemas are simplified
3. **No Event System**: Event hooks for command execution via plugins are not implemented yet

## Future Enhancements

1. **Dynamic Command Registration**: Support for commands added/removed after initialization
2. **Enhanced Schema Generation**: Better representation of command parameters
3. **Plugin Events**: Support for command execution events
4. **Integration with Authentication System**: Proper permission handling 