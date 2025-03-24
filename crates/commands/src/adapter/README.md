# Command Adapters

This directory contains adapter implementations for integrating the command system with external systems. The adapters follow the adapter pattern to provide a bridge between the command system's API and various external interfaces.

## Available Adapters

### MCP Adapter (`mcp.rs`)

The MCP (Machine Context Protocol) adapter enables command execution through the MCP protocol:

- Converts MCP requests to command execution
- Handles serialization/deserialization of command data
- Manages execution context

### Plugin Adapter (`plugins.rs`)

The Plugin adapter integrates the command system with the unified plugin architecture:

- Implements the `Plugin` and `CommandsPlugin` traits
- Enables command discovery through the plugin registry
- Converts between command arguments and JSON formats
- Provides command metadata for external systems
- Manages plugin lifecycle events

### Helper Adapter (`helper.rs`)

The Helper adapter provides utility functions and simplified interfaces for common command operations:

- Streamlined command registration
- Common command patterns
- Simplified error handling

## Usage

Each adapter follows a similar pattern:

1. Create/obtain a command registry
2. Create the appropriate adapter
3. Use the adapter to interact with the system from the external interface

### Example: Plugin Adapter

```rust
use squirrel_commands::factory::create_command_registry;
use squirrel_commands::adapter::plugins::create_commands_plugin_adapter;
use squirrel_plugins::registry::PluginRegistry;

// Create a command registry
let registry = create_command_registry()?;

// Create the plugin adapter
let plugin = create_commands_plugin_adapter(registry);

// Register with the plugin registry
let mut plugin_registry = PluginRegistry::new();
plugin_registry.register_plugin(plugin)?;

// Now commands can be accessed through the plugin registry
```

### Example: MCP Adapter

```rust
use squirrel_commands::factory::create_command_registry;
use squirrel_commands::adapter::mcp::{McpCommandAdapter, McpCommandRequest};

// Create a command registry
let registry = create_command_registry()?;

// Create the MCP adapter
let adapter = McpCommandAdapter::new(registry);

// Execute command via MCP
let request = McpCommandRequest {
    command: "help".to_string(),
    args: vec![], 
    execution_id: "exec-1".to_string(),
};

let response = adapter.execute(request).await?;
println!("MCP response: {}", response.output);
```

## Extending Adapters

When adding new adapter implementations:

1. Create a new module in the adapter directory
2. Implement the adapter following the existing patterns
3. Add the module to the `mod.rs` file
4. Update this README with documentation for the new adapter

## Implementation Notes

- All adapters should maintain thread safety
- Proper error handling and propagation is essential
- Adapters should maintain backward compatibility
- Performance considerations are important for adapters

## Modules

- `helper.rs`: Provides the `CommandRegistryAdapter` for interacting with the command registry
- `mcp.rs`: Implements the MCP adapter for command execution via the Machine Context Protocol
- `tests.rs`: Contains tests for the adapter functionality

## Documentation

For more detailed information about the adapter pattern implementation, see:
- [Command Adapter Pattern](../../../../specs/patterns/command-adapter-pattern.md)
- [Adapter Implementation Guide](../../../../specs/patterns/adapter-implementation-guide.md)

## Testing

To run tests for this module:

```
cargo test adapter::tests
cargo test adapter::mcp::tests
cargo test tests::mcp_integration_test
``` 