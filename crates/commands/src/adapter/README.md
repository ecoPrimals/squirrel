# Command System Adapters

This module provides adapter implementations for integrating the command system with external protocols and services.

## Overview

The adapter pattern is used to create a bridge between our command system and external interfaces such as the Machine Context Protocol (MCP). This allows the core command functionality to remain independent of specific integration details.

## Modules

- `helper.rs`: Provides the `CommandRegistryAdapter` for interacting with the command registry
- `mcp.rs`: Implements the MCP adapter for command execution via the Machine Context Protocol
- `tests.rs`: Contains tests for the adapter functionality

## Usage

### Basic Usage with MCP

```rust
// Create auth manager
let auth_manager = AuthManager::with_provider(Box::new(BasicAuthProvider::new()));

// Create command registry adapter
let registry_adapter = create_initialized_registry_adapter().unwrap();

// Create MCP adapter
let mcp_adapter = Arc::new(McpCommandAdapter::new(
    Arc::new(auth_manager),
    registry_adapter.clone()
));

// Execute command via MCP
let request = McpCommandRequest {
    command: "my_command".to_string(),
    arguments: vec!["arg1".to_string(), "arg2".to_string()],
    credentials: Some(AuthCredentials::Basic {
        username: "user".to_string(),
        password: "password".to_string(),
    }),
    context: McpExecutionContext {
        working_directory: None,
        environment: None,
        session_id: None,
        timestamp: None,
    },
};

let response = mcp_adapter.handle_command(&request).await;
```

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