# Adapter Pattern Implementation

This crate provides a standalone implementation of the adapter pattern in Rust, with a focus on testing and demonstration purposes. It's designed to be completely independent of the main codebase, allowing it to be used for learning and testing the adapter pattern without any dependencies on potentially broken code.

## Overview

The adapter pattern is a structural design pattern that allows objects with incompatible interfaces to collaborate. This crate demonstrates several adaptations:

1. **Command Registry Adapter** - Adapts a command registry for asynchronous operations
2. **MCP Command Adapter** - Adapts commands for Machine Context Protocol (MCP) with authentication
3. **Plugin Adapter** - Adapts commands for a plugin system

## Features

- Thread-safe command registry with Arc/Mutex
- Asynchronous execution of commands
- Authentication and authorization for MCP commands
- Proper error handling with specific error types
- Comprehensive test suite
- Well-documented code with examples

## Architecture

The crate is organized into several modules:

- `command`: Defines the `MockCommand` trait and implementations
- `registry`: Provides the `MockCommandRegistry` for registering and executing commands
- `error`: Contains error types and result aliases
- `adapter`: Implements the adapter types:
  - `CommandRegistryAdapter`: Basic adapter for command registry operations
  - `McpCommandAdapter`: Adapter for MCP operations with authentication
  - `CommandsPluginAdapter`: Adapter for plugin system integration

## Usage

### Basic Usage Example

```rust
use adapter_tests::{TestCommand, CommandRegistryAdapter, Auth, McpCommandAdapter};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a registry adapter
    let adapter = CommandRegistryAdapter::new();
    
    // Create and register a test command
    let cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
    adapter.register_command(Arc::new(cmd))?;
    
    // Execute the command
    let result = adapter.execute("hello", vec![]).await?;
    println!("Result: {}", result);  // Output: "Hello, world!"
    
    // Using the MCP adapter with authentication
    let mcp_adapter = McpCommandAdapter::new();
    let cmd = TestCommand::new("secure", "Secure command", "Secret data");
    mcp_adapter.register_command(Arc::new(cmd))?;
    
    // Execute with authentication
    let result = mcp_adapter.execute_with_auth(
        "secure",
        vec![],
        Auth::User("admin".to_string(), "password".to_string())
    ).await?;
    println!("Authenticated result: {}", result);
    
    Ok(())
}
```

### Advanced Example

For a more comprehensive example demonstrating all three adapters, see the included example:

```bash
cargo run --example adapter_showcase
```

## Key Design Principles

This implementation follows these key design principles:

1. **Separation of Concerns**: Core command functionality is separated from protocol-specific details
2. **Interface Isolation**: Clear interfaces between components allow independent evolution
3. **Testability**: Components can be tested in isolation without dependencies on external systems
4. **Flexibility**: Adapters can be swapped or modified without changing core command logic
5. **Async Safety**: Async operations are handled safely to prevent deadlocks and ensure performance

## Testing

Run the tests with:

```bash
cargo test -p adapter-tests
```

The tests demonstrate the adapter pattern in action, verifying the functionality of:
- Command registration
- Command execution
- Authentication and authorization
- Error handling
- Thread safety

## Async Safety Considerations

When implementing adapters that involve asynchronous operations, special care must be taken to ensure proper handling of locks and resources across await points. Our implementation follows these key principles:

1. **Scoped Lock Usage**: Locks are acquired and released within explicit scopes to control their lifetime
2. **Lock Duration Minimization**: Locks are held for the minimum time necessary to extract required data
3. **No Locks Across Await**: We ensure that no locks are held when calling `.await` on futures

These practices help prevent deadlocks, improve concurrency, and ensure efficient resource utilization in async contexts.

## Error Handling

This implementation uses a dedicated error type `AdapterError` for all adapter operations, with specific variants for different error conditions. The error type implements `std::error::Error` and provides detailed error messages.

## License

This code is licensed under either the MIT license or the Apache License 2.0, at your option. 