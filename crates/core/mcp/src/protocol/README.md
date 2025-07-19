# MCP Protocol Module

## Overview

The Machine Context Protocol (MCP) module provides a standardized way for components to communicate within the Squirrel system. This module implements dependency injection (DI) patterns to ensure testability, maintainability, and proper initialization control.

## Core Components

- `MCPProtocolAdapter`: Main adapter for the protocol system
- `CommandHandler`: Trait for handling command messages
- `MCPMessage`: Standard message format for communication
- `ProtocolConfig`: Configuration for the protocol

## Dependency Injection Patterns

The MCP protocol module implements proper DI patterns through adapters and factories:

### Using Adapters

Adapters provide a clean way to interact with the protocol without relying on global state. Each adapter must be explicitly initialized before use.

```rust
// Create an adapter
let adapter = MCPProtocolAdapter::new();

// Initialize the adapter
adapter.initialize().await?;

// Use the adapter
let message = MCPMessage::default();
let response = adapter.handle_message(&message).await?;
```

### Using Factory Functions

Factory functions simplify adapter creation and initialization:

```rust
// Create an uninitialized adapter
let adapter = create_protocol_adapter();

// Create and initialize an adapter with custom config
let config = ProtocolConfig {
    version: "2.0".to_string(),
    max_message_size: 4096,
    timeout_ms: 5000,
};
let adapter = create_protocol_adapter_with_config(config).await?;

// Create and initialize an adapter with default config
let adapter = create_initialized_protocol_adapter().await?;
```

## Working with Message Handlers

Message handlers process specific message types. They must be registered with the adapter:

```rust
#[async_trait::async_trait]
impl CommandHandler for MyHandler {
    async fn handle_command(&self, message: &MCPMessage) -> Result<MCPResponse> {
        // Process message
        Ok(MCPResponse { /* ... */ })
    }
}

// Register the handler
let handler = MyHandler::new();
adapter.register_handler(MessageType::Command, Box::new(handler)).await?;
```

## Error Handling

The adapter properly handles initialization errors:

```rust
// Check if adapter is initialized
if !adapter.is_initialized().await {
    // Initialize it
    match adapter.initialize().await {
        Ok(_) => println!("Adapter initialized"),
        Err(e) => eprintln!("Failed to initialize: {}", e),
    }
}

// Handle messages with proper error handling
match adapter.handle_message(&message).await {
    Ok(response) => println!("Received response: {:?}", response),
    Err(e) => eprintln!("Error handling message: {}", e),
}
```

## Protocol State Management

The adapter maintains protocol state:

```rust
// Get current state
let state = adapter.get_state().await;

// Set new state
adapter.set_state(serde_json::json!({
    "state": "Initialized",
    "details": { "connected_clients": 5 }
})).await;
```

## Migration from Global State

### Before (using global state or implicit initialization)

```rust
// Old approach with implicit initialization
let adapter = MCPProtocolAdapter::new();
// This would create the protocol on-demand (internally) if not initialized
let response = adapter.handle_message(&message).await?;
```

### After (using explicit DI)

```rust
// Approach 1: Explicit initialization
let adapter = MCPProtocolAdapter::new();
adapter.initialize().await?;
let response = adapter.handle_message(&message).await?;

// Approach 2: Using factory function
let adapter = create_initialized_protocol_adapter().await?;
let response = adapter.handle_message(&message).await?;

// Approach 3: With custom configuration
let config = ProtocolConfig::default();
let adapter = create_protocol_adapter_with_config(config).await?;
let response = adapter.handle_message(&message).await?;
```

## Testing

The module is designed to be easily testable:

```rust
#[tokio::test]
async fn test_protocol_adapter() {
    // Create adapter with mock components for testing
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize().await.unwrap();
    
    // Register mock handler
    adapter.register_handler(MessageType::Command, Box::new(MockHandler::new())).await.unwrap();
    
    // Test message handling
    let message = MCPMessage::default();
    let response = adapter.handle_message(&message).await.unwrap();
    
    assert_eq!(response.status, ResponseStatus::Success);
}
``` 