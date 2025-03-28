# Transport Migration Guide

## Overview

As of version 0.3.0, the legacy transport system has been removed from the MCP crate. This guide explains how to migrate your code from the old transport system to the new one.

## Why a New Transport System?

The new transport system offers several advantages:

1. **Thread-Safe API**: All methods operate on `&self` instead of `&mut self`, making it compatible with `Arc` for thread-safe sharing.
2. **Modular Design**: Each transport type (TCP, WebSocket, etc.) is a separate module with its own configuration.
3. **Better Error Handling**: More specific error types for better error handling and recovery.
4. **Improved Performance**: More efficient implementation with less overhead.
5. **Enhanced Security**: Built-in support for encryption and authentication.

## Migration Steps

### 1. Update Imports

```rust
// Old imports
use squirrel_mcp::transport_old::Transport;

// New imports
use squirrel_mcp::transport::Transport; // The trait
use squirrel_mcp::transport::tcp::{TcpTransport, TcpTransportConfig};
use squirrel_mcp::transport::websocket::{WebSocketTransport, WebSocketConfig};
use squirrel_mcp::transport::stdio::{StdioTransport, StdioConfig};
use squirrel_mcp::transport::memory::{MemoryTransport, MemoryChannel};
```

### 2. Creating Transport Instances

#### TCP Transport

```rust
// Old code
let transport = Transport::new_tcp("127.0.0.1:9000");

// New code
let config = TcpTransportConfig::default()
    .with_remote_address("127.0.0.1:9000")
    .with_connection_timeout(5000);
let mut transport = TcpTransport::new(config);

// Connect the transport
transport.connect().await?;
```

#### WebSocket Transport

```rust
// Old code
let transport = Transport::new_websocket("ws://localhost:8080");

// New code
let config = WebSocketConfig::default()
    .with_url("ws://localhost:8080");
let mut transport = WebSocketTransport::new(config);

// Connect the transport
transport.connect().await?;
```

#### Stdio Transport

```rust
// Old code
let transport = Transport::new_stdio();

// New code
let config = StdioConfig::default();
let mut transport = StdioTransport::new(config);

// Connect the transport
transport.connect().await?;
```

#### Memory Transport (for testing)

```rust
// Old code
let (transport1, transport2) = Transport::new_memory_pair();

// New code
let (mut transport1, mut transport2) = MemoryChannel::create_pair();

// Connect the transports
transport1.connect().await?;
transport2.connect().await?;
```

### 3. Thread-Safe Sharing

The new transport system is designed to work well with `Arc` for thread-safe sharing:

```rust
// After connecting
let transport = Arc::new(transport);

// Now you can clone and share between threads
let transport_clone = Arc::clone(&transport);

// Use in another thread
tokio::spawn(async move {
    let message = /* ... */;
    transport_clone.send_message(message).await?;
});
```

### 4. Using with MCP Client

The MCP Client has been updated to work with the new transport system:

```rust
// Create a custom transport
let mut tcp_transport = TcpTransport::new(
    TcpTransportConfig::default()
        .with_remote_address("127.0.0.1:9000")
);

// Connect it
tcp_transport.connect().await?;

// Use it with the client
let client_config = ClientConfig {
    transport: Some(Arc::new(tcp_transport)),
    ..ClientConfig::default()
};

let client = MCPClient::new(client_config);
```

### 5. Common Patterns

#### Sending Messages

```rust
// Old code
transport.send_message(&message).await?;

// New code
transport.send_message(message).await?;
```

#### Receiving Messages

```rust
// Old code
let message = transport.receive_message().await?;

// New code
let message = transport.receive_message().await?;
```

#### Checking Connection Status

```rust
// Old code
if transport.is_connected() {
    // ...
}

// New code
if transport.is_connected().await {
    // ...
}
```

## Transport-Specific Features

Each transport implementation has its own set of features and configuration options. See the documentation for each transport type for details:

- `TcpTransport`: TCP/IP-based transport with support for reconnection, timeouts, and keep-alive.
- `WebSocketTransport`: WebSocket-based transport for browser integration and web applications.
- `StdioTransport`: Standard input/output-based transport for process communication.
- `MemoryTransport`: In-memory transport for testing and internal communication.

## Need Help?

If you encounter issues migrating to the new transport system, please:

1. Check the full documentation for the specific transport you're using
2. Review the examples in the `examples/` directory
3. Open an issue in the GitHub repository if you need additional assistance

## Compatibility Layer

For legacy code, a compatibility layer was available in the previous release, but it has been removed in version 0.3.0. If you need to maintain compatibility with both old and new transport systems during the transition, consider using the 0.2.x version and gradually migrate your code to the new system before upgrading to 0.3.0. 