# MCP Module Migration Guide

This guide explains how to migrate from the old singleton-based MCP module to the new dependency injection (DI) pattern.

## Overview

The MCP (Message Control Protocol) module has been refactored to remove implicit initialization and global state. The new implementation requires explicit initialization and follows proper dependency injection principles.

## Key Changes

1. Removed "initialize on-demand" fallbacks
2. Added explicit initialization requirements
3. Improved error handling for uninitialized components
4. Standardized factory functions for easier DI usage
5. Implemented proper adapter patterns

## Migration Steps

### Before: Old Approach with Implicit Initialization

```rust
// Old approach that relied on implicit initialization
fn process_message(message: &MCPMessage) -> Result<MCPResponse> {
    // This would implicitly initialize if not already done
    let protocol = MCPProtocolAdapter::new();
    protocol.handle_message(message) // Creates protocol on-demand
}
```

### After: New Approach with Explicit Initialization

#### Approach 1: Manual Initialization

```rust
// Explicitly initialize the adapter
async fn process_message(message: &MCPMessage) -> Result<MCPResponse> {
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize().await?;
    adapter.handle_message(message).await
}
```

#### Approach 2: Using Factory Functions

```rust
// Use the factory function for initialization
async fn process_message(message: &MCPMessage) -> Result<MCPResponse> {
    let adapter = create_initialized_protocol_adapter().await?;
    adapter.handle_message(message).await
}
```

#### Approach 3: With Custom Configuration

```rust
// Initialize with custom configuration
async fn process_message(message: &MCPMessage, config: ProtocolConfig) -> Result<MCPResponse> {
    let adapter = create_protocol_adapter_with_config(config).await?;
    adapter.handle_message(message).await
}
```

#### Approach 4: Dependency Injection

```rust
// Accept the adapter as a parameter (best practice)
async fn process_message(adapter: &MCPProtocolAdapter, message: &MCPMessage) -> Result<MCPResponse> {
    adapter.handle_message(message).await
}

// Then in the calling code:
async fn main() -> Result<()> {
    let adapter = create_initialized_protocol_adapter().await?;
    
    // Pass the adapter to functions that need it
    let message = create_message();
    let response = process_message(&adapter, &message).await?;
    
    // Use the same adapter for multiple operations
    // ...
    
    Ok(())
}
```

## Error Handling

The new pattern includes proper error handling for uninitialized adapters:

```rust
// Creating an adapter without initializing it
let adapter = MCPProtocolAdapter::new();

// This will return an error
match adapter.handle_message(&message).await {
    Ok(response) => println!("Success: {:?}", response),
    Err(e) => {
        // This will print "Protocol not initialized"
        println!("Error: {}", e);
    }
}
```

## Registering Handlers

Handler registration now requires explicit initialization:

```rust
async fn register_handlers(adapter: &MCPProtocolAdapter) -> Result<()> {
    // First, ensure the adapter is initialized
    if !adapter.is_initialized().await {
        return Err(SquirrelError::MCP("Adapter not initialized".to_string()));
    }
    
    // Register handlers
    let command_handler = Box::new(MyCommandHandler::new());
    adapter.register_handler(MessageType::Command, command_handler).await?;
    
    let event_handler = Box::new(MyEventHandler::new());
    adapter.register_handler(MessageType::Event, event_handler).await?;
    
    Ok(())
}
```

## Best Practices

1. **Always initialize explicitly**: Never assume the adapter is already initialized.
2. **Check initialization state**: Use `is_initialized()` before operations if needed.
3. **Use factory functions**: Prefer factory functions for creating and initializing adapters.
4. **Pass adapters as parameters**: Follow dependency injection by passing adapters to functions.
5. **Proper error handling**: Always handle the case where an adapter is not initialized.
6. **Reuse adapters**: Create adapters once and reuse them rather than creating new instances.

## Common Migration Patterns

### Singleton Service to DI Service

Before:
```rust
// Old singleton pattern
static PROTOCOL: OnceCell<MCPProtocolAdapter> = OnceCell::new();

fn get_protocol() -> &'static MCPProtocolAdapter {
    PROTOCOL.get_or_init(|| {
        let adapter = MCPProtocolAdapter::new();
        // Implicit initialization
        adapter
    })
}
```

After:
```rust
// New DI pattern
struct MCPService {
    protocol: Arc<MCPProtocolAdapter>,
}

impl MCPService {
    async fn new() -> Result<Self> {
        let protocol = create_initialized_protocol_adapter().await?;
        Ok(Self { protocol })
    }
    
    async fn handle_message(&self, message: &MCPMessage) -> Result<MCPResponse> {
        self.protocol.handle_message(message).await
    }
}
```

### Global Handler to Local Handler

Before:
```rust
// Global handler registration
fn initialize_handlers() {
    let protocol = get_protocol();
    protocol.register_handler("command", Box::new(CommandHandler));
    protocol.register_handler("event", Box::new(EventHandler));
}
```

After:
```rust
// Local handler registration
async fn initialize_handlers(adapter: &MCPProtocolAdapter) -> Result<()> {
    adapter.register_handler(MessageType::Command, Box::new(CommandHandler)).await?;
    adapter.register_handler(MessageType::Event, Box::new(EventHandler)).await?;
    Ok(())
}
```

## Testing

The new pattern is much easier to test:

```rust
#[tokio::test]
async fn test_message_handling() {
    // Create a test adapter with a mock handler
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize().await.unwrap();
    adapter.register_handler(MessageType::Command, Box::new(MockHandler)).await.unwrap();
    
    // Create a test message
    let message = MCPMessage {
        id: MessageId("test-1".to_string()),
        message_type: MessageType::Command,
        payload: json!({"command": "test"}),
    };
    
    // Test the handler
    let response = adapter.handle_message(&message).await.unwrap();
    assert_eq!(response.status, ResponseStatus::Success);
}
``` 