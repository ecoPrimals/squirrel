# MCP Integration Adapters

This directory contains adapter implementations for integrating various components with the Machine Context Protocol (MCP).

## Available Adapters

- **Core Adapter**: Integrates core system state and commands with MCP (`core_adapter.rs`)

## Using the Adapter Pattern

The adapter pattern is the primary mechanism for integrating external components with MCP. This pattern provides a clean separation of concerns and ensures loose coupling between components.

### Basic Integration Steps

1. **Create an adapter struct**:
   ```rust
   pub struct MyComponentAdapter {
       component: Arc<MyComponent>,
       mcp: Arc<dyn MCPProtocol>,
       // Other dependencies...
   }
   ```

2. **Implement initialization logic**:
   ```rust
   impl MyComponentAdapter {
       pub async fn initialize(&self) -> MCPResult<()> {
           // Register message handlers
           self.mcp.register_handler(
               MessageType::YourMessageType,
               Box::new(self.clone()),
           ).await?;
           
           Ok(())
       }
   }
   ```

3. **Implement the MessageHandler trait**:
   ```rust
   #[async_trait]
   impl MessageHandler for MyComponentAdapter {
       async fn handle_message(&self, message: MCPMessage) -> MCPResult<MCPResponse> {
           // Handle messages from MCP to your component
           // ...
       }
   }
   ```

4. **Implement methods to send messages from your component to MCP**:
   ```rust
   impl MyComponentAdapter {
       pub async fn send_component_event(&self, event: ComponentEvent) -> MCPResult<()> {
           let message = MCPMessage {
               message_type: MessageType::ComponentEvent,
               payload: serde_json::to_value(event)?,
               // ...
           };
           
           self.mcp.send_message(message).await?;
           Ok(())
       }
   }
   ```

### Best Practices

1. **Use dependency injection** - Inject all dependencies through constructors
2. **Handle async operations safely** - Don't hold locks across await points
3. **Implement proper error handling** - Convert between component errors and MCP errors
4. **Add comprehensive logging and metrics** - For observability
5. **Use circuit breakers** - For resilience when communicating with MCP
6. **Write thorough tests** - Both unit tests with mocks and integration tests

## Example Usage

Here's a basic example of using an adapter:

```rust
// Create component and MCP instances
let my_component = Arc::new(MyComponent::new());
let mcp = Arc::new(InMemoryMCPProtocol::new());

// Create the adapter
let adapter = MyComponentAdapter::new(
    my_component.clone(),
    mcp.clone(),
    // Other dependencies...
);

// Initialize the adapter
adapter.initialize().await?;

// Use the adapter to send component events to MCP
adapter.send_component_event(ComponentEvent::Started).await?;

// MCP messages will be handled by the adapter's handle_message method
```

For a complete example, see `examples/core_integration.rs`.

## Adding New Adapters

When adding a new adapter:

1. Create a new file in this directory (e.g., `my_adapter.rs`)
2. Add the module to `mod.rs`
3. Follow the adapter pattern structure shown above
4. Add unit and integration tests
5. Consider adding an example to the `examples` directory

## Further Information

For more detailed guidance, see:
- The MCP Integration Guide in `specs/active/mcp-protocol/MCP_INTEGRATION_GUIDE.md` 