# Adapter Pattern Examples

This crate demonstrates the Adapter Pattern in Rust with a command-based architecture.

## Overview

The adapter pattern is a structural design pattern that allows objects with incompatible interfaces to collaborate. This implementation focuses on adapting command execution interfaces for different contexts:

1. **Registry Adapter** - Basic adapter for command registry operations
2. **MCP Adapter** - Adapter with authentication and authorization
3. **Plugin Adapter** - Adapter for plugin system integration

## Features

- Command registration and execution
- Authentication and authorization
- Command logging and audit
- Plugin integration

## Examples

This project includes several example applications that demonstrate different aspects of the adapter pattern:

### Demo Example (`cargo run --bin demo`)

Shows the basic usage of all three adapter types:
- Registry adapter for basic command operations
- MCP adapter with authentication and authorization
- Plugin adapter for extensibility

### Custom Command Example (`cargo run --bin custom_command`)

Demonstrates how to create custom commands:
- Calculator command with multiple operations
- Weather forecast command

### Authentication Example (`cargo run --bin auth_example`)

Interactive example focusing on authentication and authorization:
- Login/logout functionality
- Role-based permissions
- Command access control

### CLI Application Example (`cargo run --bin cli_app`)

A more complete command-line application using the adapter pattern:
- Command registration and discovery
- Help system
- Authentication for secure commands
- Multiple adapter composition

Example usage:
- `cargo run --bin cli_app help` - Display help information
- `cargo run --bin cli_app echo hello world` - Echo the arguments
- `cargo run --bin cli_app greet formal John` - Greet a user formally
- `cargo run --bin cli_app login user password secure` - Access a secure command with authentication

## Usage

```rust
use adapter_pattern_examples::{
    Command, CommandAdapter, TestCommand, 
    RegistryAdapter, McpAdapter, PluginAdapter, Auth
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create an adapter
    let adapter = RegistryAdapter::new();
    
    // Register commands
    let hello_cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
    adapter.register_command(Arc::new(hello_cmd)).unwrap();
    
    // Execute commands
    let result = adapter.execute_command("hello", vec![]).await.unwrap();
    println!("Result: {}", result);
}
```

## Running the Tests

```bash
cargo test
```

## License

AGPL-3.0-only — part of the ecoPrimals [scyBorg](../../LICENSE) triple-copyleft framework.