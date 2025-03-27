# Adapter Test Module

This crate demonstrates the adapter pattern in Rust with a focus on testing. It provides a standalone implementation of the adapter pattern that can be used for testing purposes.

## Overview

The adapter pattern is a structural design pattern that allows objects with incompatible interfaces to collaborate. This crate provides a simplified implementation of the adapter pattern with a focus on command execution and authentication.

## Features

- `MockCommandRegistry`: A simple registry for commands
- `CommandRegistryAdapter`: Adapts the command registry for async execution
- `McpCommandAdapter`: Adapts commands for Machine Context Protocol (MCP) with authentication
- `CommandsPluginAdapter`: Adapts commands for plugin systems

## Usage

Here's a simple example of how to use the adapter pattern:

```rust
use adapter_test::{TestCommand, CommandRegistryAdapter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create registry adapter
    let adapter = CommandRegistryAdapter::new();
    
    // Create test command
    let test_cmd = TestCommand::new(
        "test", 
        "A test command", 
        "Test command result"
    );
    
    // Register command
    adapter.register_command(Box::new(test_cmd))?;
    
    // Execute command
    let result = adapter.execute("test", &[]).await?;
    println!("Result: {}", result);
    
    Ok(())
}
```

## Testing

The crate includes comprehensive tests for all adapter implementations:

- Basic command registry functionality
- MCP adapter with authentication
- Plugin adapter for command execution
- Authentication validation

Run the tests with:

```bash
cargo test -p adapter-test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 