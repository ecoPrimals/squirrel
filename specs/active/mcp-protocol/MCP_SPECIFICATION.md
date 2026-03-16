---
version: 1.0.0
date: 2024-08-17
status: active
author: ecoPrimals Contributors
---

# Machine Context Protocol (MCP) Specification

## Overview

The Machine Context Protocol (MCP) provides a standardized communication layer for exchanging messages between AI models, tools, and their runtime environments. This specification defines the architecture, message format, and interfaces of the MCP implementation.

## Core Components

### 1. Architecture

The MCP system is structured in layers with clear separation of concerns:

```
┌───────────────────────────────────────────────────────────┐
│                                                           │
│  ┌─────────────┐   ┌──────────────┐   ┌─────────────────┐ │
│  │ Application │───│ MCPProtocol  │───│ MessageHandler  │ │
│  └─────────────┘   └──────────────┘   └─────────────────┘ │
│                           │                               │
│                           │                               │
│  ┌─────────────────┐     ▼     ┌───────────────────────┐ │
│  │   Serializer    │◄──────────┤     MessageRouter     │ │
│  └─────────────────┘           └───────────────────────┘ │
│                                         │                 │
│                                         │                 │
│  ┌─────────────────┐     ▼     ┌───────────────────────┐ │
│  │ Security Layer  │◄──────────┤    Transport Layer    │ │
│  └─────────────────┘           └───────────────────────┘ │
│                                         │                 │
│                                         │                 │
│           ┌───────────┬─────────────┬───▼──────┐         │
│           │           │             │          │         │
│  ┌────────▼───┐ ┌─────▼─────┐ ┌─────▼─────┐ ┌─▼────────┐ │
│  │    TCP     │ │ WebSocket │ │   stdio   │ │  Custom  │ │
│  └────────────┘ └───────────┘ └───────────┘ └──────────┘ │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

Each layer is responsible for a specific aspect of the protocol:

- **Application Layer**: Interfaces directly with the client code
- **Protocol Layer**: Manages message transmission and handling
- **Message Router**: Directs messages to appropriate handlers
- **Transport Layer**: Handles communication mechanism details
- **Security Layer**: Provides authentication, authorization, and encryption

### 2. Message Format

All MCP messages follow a standardized format:

```rust
pub struct MCPMessage {
    pub id: MessageId,                                // Unique message identifier
    pub type_: MessageType,                           // Type of message
    pub payload: serde_json::Value,                   // Message content
    pub metadata: Option<serde_json::Value>,          // Optional additional information
    pub security: SecurityMetadata,                   // Security-related metadata
    pub timestamp: chrono::DateTime<chrono::Utc>,     // Message creation time
    pub version: ProtocolVersion,                     // Protocol version
    pub trace_id: Option<String>,                     // Optional tracing identifier
}
```

### 3. Message Types

The protocol supports several types of messages:

- **Command**: Request for an action to be performed
- **Response**: Results of a command execution
- **Event**: One-way notification without expected response
- **Error**: Indication of a problem
- **Heartbeat**: Connection health check
- **Sync**: State synchronization message

### 4. Transport Options

MCP supports multiple transport mechanisms:

- **TCP**: Stream-based networking using TcpStream
- **WebSocket**: Web-compatible bidirectional communication
- **stdio**: Standard input/output for process communication
- **Custom**: Extensible for additional transport types

## Key Interfaces

### 1. Transport Interface

All transport implementations must implement the Transport trait:

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message over the transport
    async fn send_message(&self, message: MCPMessage) -> Result<(), TransportError>;

    /// Receive a message from the transport
    async fn receive_message(&self) -> Result<MCPMessage, TransportError>;

    /// Connect to the transport target
    async fn connect(&self) -> Result<(), TransportError>;

    /// Disconnect from the transport target
    async fn disconnect(&self) -> Result<(), TransportError>;

    /// Check if the transport is connected
    async fn is_connected(&self) -> bool;

    /// Get transport metadata
    fn get_metadata(&self) -> TransportMetadata;
}
```

### 2. Protocol Interface

The MCPProtocol trait defines the main protocol operations:

```rust
#[async_trait]
pub trait MCPProtocol: Send + Sync {
    /// Send a message through the protocol and get a response
    async fn send_message(&self, message: MCPMessage) -> Result<MCPResponse, MCPError>;

    /// Register a message handler for specific message types
    async fn register_handler(
        &self,
        handler: Box<dyn MessageHandler>,
    ) -> Result<(), MCPError>;

    /// Subscribe to a specific message type
    async fn subscribe(
        &self,
        message_type: MessageType,
        callback: Box<dyn Fn(MCPMessage) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    ) -> Result<SubscriptionId, MCPError>;

    /// Unsubscribe from a specific subscription
    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), MCPError>;

    /// Connect the protocol
    async fn connect(&self) -> Result<(), MCPError>;

    /// Disconnect the protocol
    async fn disconnect(&self) -> Result<(), MCPError>;

    /// Check if the protocol is connected
    async fn is_connected(&self) -> bool;
}
```

### 3. Message Handler Interface

Custom message handlers implement the MessageHandler trait:

```rust
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle a received message
    async fn handle_message(&self, message: MCPMessage) -> Result<Option<MCPMessage>, MCPError>;

    /// Report which message types this handler supports
    fn supported_message_types(&self) -> Vec<MessageType>;
}
```

### 4. Security Interface

Security functionality is defined through the SecurityManager trait:

```rust
#[async_trait]
pub trait SecurityManager: Send + Sync {
    /// Authenticate a user with credentials
    async fn authenticate(&self, credentials: &Credentials) -> Result<Session, SecurityError>;
    
    /// Verify authorization for an action
    async fn authorize(&self, session: &Session, resource: &str, action: &str) -> Result<bool, SecurityError>;
    
    /// Encrypt data
    async fn encrypt(&self, session: &Session, data: &[u8]) -> Result<Vec<u8>, SecurityError>;
    
    /// Decrypt data
    async fn decrypt(&self, session: &Session, data: &[u8]) -> Result<Vec<u8>, SecurityError>;
    
    /// Validate message security
    async fn validate_message(&self, message: &MCPMessage) -> Result<(), SecurityError>;
    
    /// Sign a message
    async fn sign_message(&self, message: &mut MCPMessage) -> Result<(), SecurityError>;
}
```

## Client Usage

Applications interact with MCP primarily through the Client interface:

```rust
// Creating a client with TCP transport
let transport = TcpTransport::new(TcpTransportConfig {
    remote_address: "127.0.0.1:8080".to_string(),
    // other config...
});
let client = Client::new(transport);

// Connect to the remote endpoint
client.connect().await?;

// Send a command
let response = client.send_command(
    "execute_tool", 
    serde_json::json!({
        "tool_name": "file_reader",
        "parameters": { "path": "/path/to/file" }
    })
).await?;

// Register a handler for event messages
client.register_handler(Box::new(MyEventHandler::new())).await?;

// Subscribe to a specific message type
let subscription_id = client.subscribe(
    MessageType::Event,
    Box::new(|message| Box::pin(async move {
        println!("Received event: {:?}", message);
    }))
).await?;

// Disconnect when done
client.disconnect().await?;
```

## Server Implementation

Servers use the Server interface to handle incoming connections:

```rust
// Creating a server with TCP transport
let transport = TcpTransport::new(TcpTransportConfig {
    bind_address: "0.0.0.0".to_string(),
    port: 8080,
    // other config...
});
let server = Server::new(transport);

// Register command handlers
server.register_handler(Box::new(ToolExecutionHandler::new())).await?;

// Start the server (blocking operation)
server.start().await?;
```

## Error Handling

MCP implements comprehensive error types:

```rust
pub enum MCPError {
    Transport(TransportError),     // Transport-related errors
    Protocol(String),              // Protocol-related errors  
    Security(SecurityError),       // Security-related errors
    Serialization(SerdeError),     // Serialization errors
    Authentication(String),        // Authentication errors
    Authorization(String),         // Authorization errors
}
```

Error handling should always provide context:

```rust
// Good error handling example
match client.send_command("execute_tool", params).await {
    Ok(response) => {
        process_response(response);
    },
    Err(MCPError::Transport(e)) => {
        log::error!("Transport error during tool execution: {}", e);
        notify_user("Connection to the server failed");
    },
    Err(MCPError::Authentication(e)) => {
        log::warn!("Authentication failed: {}", e);
        prompt_for_credentials();
    },
    Err(e) => {
        log::error!("Unexpected error: {}", e);
        report_error("An unexpected error occurred");
    }
}
```

## Security Considerations

The MCP implementation addresses several security concerns:

1. **Authentication**: Validates identity through credentials
2. **Authorization**: Controls access to resources and actions
3. **Encryption**: Protects message content during transmission
4. **Message Integrity**: Verifies messages haven't been tampered with
5. **Session Management**: Tracks authenticated sessions

Security levels can be configured based on requirements:

```rust
pub enum SecurityLevel {
    None,        // No security (development only)
    Basic,       // Basic authentication
    Standard,    // Standard encryption and authentication
    High,        // Strong encryption, authentication, and integrity checks
}
```

## Performance Characteristics

The MCP implementation is designed to meet these performance targets:

1. **Throughput**: At least 1000 messages per second
2. **Latency**: Message processing in under 30ms
3. **Resource Usage**: Moderate memory footprint with efficient connection pooling
4. **Scalability**: Supports many concurrent connections

## Extension Points

The system can be extended in several ways:

1. **Custom Transports**: Implement the Transport trait
2. **Custom Handlers**: Implement the MessageHandler trait
3. **Security Providers**: Implement the SecurityManager trait
4. **Protocol Extensions**: Enhanced message formats or types

## Development Guidelines

When working with MCP, follow these practices:

1. **Always use async/await pattern** for I/O operations
2. **Implement proper error handling** with contextual information
3. **Use Arc and proper synchronization** for shared state
4. **Write comprehensive tests** for all components
5. **Document security considerations** for each component
6. **Benchmark performance-critical sections** during development

## Next Steps for Teams

Teams working with the MCP implementation should:

1. **Review the transport options** and select the appropriate one
2. **Implement appropriate message handlers** for their use cases
3. **Configure security levels** based on their requirements
4. **Write integration tests** to verify their components
5. **Document their specific protocol extensions** if any

---

*Specification produced by ecoPrimals Contributors.* 