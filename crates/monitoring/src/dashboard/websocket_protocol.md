# WebSocket Protocol Documentation

## Overview

This document describes the WebSocket communication protocol used by the monitoring dashboard system. The WebSocket server provides real-time updates for dashboard components, health status, metrics, and alerts. This protocol enables bidirectional communication between clients and the server, allowing for efficient real-time data exchange.

## Connection Establishment

### WebSocket Endpoint

- **URL Path**: `/ws`
- **Protocol**: `ws://` or `wss://` (for secure connections)

### Authentication

Authentication is performed via a Bearer token in the Authorization header:

```
Authorization: Bearer <token>
```

If authentication is enabled in the configuration, the server will validate the token before establishing a WebSocket connection. If validation fails, the server will respond with an HTTP 401 Unauthorized status.

### Connection Lifecycle

1. Client initiates a WebSocket connection to the server
2. Server authenticates the request (if configured)
3. Server validates origin (if configured)
4. Server accepts the connection and assigns a unique client ID
5. Client sends subscription requests for components of interest
6. Server sends real-time updates for subscribed components
7. Either party can close the connection at any time

## Message Format

All messages between client and server are formatted as JSON objects with a `type` field indicating the message type. Messages may be compressed when they exceed the configured size threshold.

### Binary Message Format (Compressed)

When compression is enabled, larger messages are sent in binary format using Gzip compression. To detect compressed messages:

- Text messages are always JSON
- Binary messages are always compressed JSON

To decompress a binary message:
1. Apply Gzip decompression to the binary data
2. Parse the resulting text as JSON

## Client to Server Messages

### Subscribe to Component

Subscribe to receive updates for a specific component:

```json
{
  "type": "subscribe",
  "componentId": "system_cpu"
}
```

### Unsubscribe from Component

Unsubscribe from a component to stop receiving updates:

```json
{
  "type": "unsubscribe",
  "componentId": "system_cpu"
}
```

### Ping

Send a ping to check connection health:

```json
{
  "type": "ping"
}
```

## Server to Client Messages

### Component Update

A single component update:

```json
{
  "type": "update",
  "componentId": "system_cpu",
  "payload": {
    "usage": 45.2,
    "cores": [32.1, 56.7, 48.3, 43.9]
  },
  "timestamp": 1653472392000,
  "compressed": false
}
```

### Batch Update

Multiple component updates batched into a single message:

```json
{
  "type": "batch",
  "batch_id": "550e8400-e29b-41d4-a716-446655440000",
  "messages": [
    {
      "type": "update",
      "componentId": "system_cpu",
      "payload": {
        "usage": 45.2,
        "cores": [32.1, 56.7, 48.3, 43.9]
      },
      "timestamp": 1653472392000,
      "compressed": false
    },
    {
      "type": "update",
      "componentId": "system_memory",
      "payload": {
        "used": 8590000000,
        "total": 16000000000
      },
      "timestamp": 1653472392000,
      "compressed": false
    }
  ],
  "timestamp": 1653472392000,
  "compressed": false
}
```

### Subscription Confirmation

Confirms successful subscription to a component:

```json
{
  "type": "subscription_confirmed",
  "componentId": "system_cpu",
  "timestamp": 1653472392000
}
```

### Unsubscription Confirmation

Confirms successful unsubscription from a component:

```json
{
  "type": "unsubscription_confirmed",
  "componentId": "system_cpu",
  "timestamp": 1653472392000
}
```

### Pong

Response to a ping message:

```json
{
  "type": "pong",
  "timestamp": 1653472392000
}
```

### Error Message

Sent when an error occurs:

```json
{
  "type": "error",
  "code": "invalid_request",
  "message": "Invalid request format",
  "timestamp": 1653472392000
}
```

Common error codes:
- `invalid_request`: The request format is invalid
- `not_found`: The requested component was not found
- `rate_limit_exceeded`: Client has exceeded rate limits
- `unauthorized`: Client is not authorized to access the requested component
- `internal_error`: An internal server error occurred

## Compression and Batching

### Message Compression

For efficient communication, the server compresses large messages before sending. The compression is applied based on configuration settings:

- `compression_enabled`: Whether compression is enabled
- `min_compression_size`: Minimum message size (in bytes) for compression to be applied
- `compression_level`: Compression level (0-9, where 9 is maximum compression)

When receiving a binary message, clients should decompress it using Gzip decompression.

### Message Batching

To reduce the number of messages for high-frequency updates, the server batches multiple component updates into a single message:

- `batching_enabled`: Whether batching is enabled
- `max_messages`: Maximum number of messages per batch
- `max_interval_ms`: Maximum batch interval in milliseconds

## Error Handling

### Connection Errors

If the WebSocket connection is lost, clients should implement a reconnection strategy with exponential backoff:

1. Start with a short delay (e.g., 100ms)
2. Double the delay for each failed reconnection attempt
3. Cap the maximum delay (e.g., 30 seconds)
4. Reset the delay after a successful connection

### Protocol Errors

When receiving an error message, clients should handle it based on the error code:

- For transient errors (like rate limiting), retry with backoff
- For permanent errors (like unauthorized), fix the issue before retrying
- For internal errors, log the error and retry with backoff

## Security Considerations

### Authentication and Authorization

- Use HTTPS and WSS for secure connections
- Keep authentication tokens secure and use appropriate expiration
- Validate all input from clients to prevent injection attacks

### Rate Limiting

The server implements rate limiting to prevent abuse:

- Connection rate limiting: Limits the number of connection attempts per IP
- Message rate limiting: Limits the number of messages per client
- Subscription rate limiting: Limits the number of subscriptions per client

### Data Privacy

- Sensitive data should be masked or encrypted
- Use the data masking configuration to automatically mask sensitive fields

## Performance Considerations

### Client Implementations

For optimal performance, clients should:

1. Only subscribe to components they need
2. Implement efficient processing of batched updates
3. Handle compressed messages correctly
4. Use a dedicated thread or event loop for WebSocket processing
5. Implement proper reconnection handling

### Minimizing Network Traffic

To minimize network traffic:

1. Subscribe only to essential components
2. Process batch updates efficiently
3. Close connections when not in use
4. Implement proper error handling to avoid connection cycling

## Example Implementation

### JavaScript Client Example

```javascript
const connectToDashboard = (url, token, onUpdate) => {
  const socket = new WebSocket(url);
  
  // Handle connection open
  socket.onopen = () => {
    console.log('Connected to dashboard');
    
    // Subscribe to components
    socket.send(JSON.stringify({
      type: 'subscribe',
      componentId: 'system_cpu'
    }));
  };
  
  // Handle incoming messages
  socket.onmessage = async (event) => {
    let message;
    
    // Handle text or binary (compressed) messages
    if (event.data instanceof Blob) {
      // Decompress binary message
      const arrayBuffer = await event.data.arrayBuffer();
      const decompressed = pako.inflate(arrayBuffer, { to: 'string' });
      message = JSON.parse(decompressed);
    } else {
      // Parse text message
      message = JSON.parse(event.data);
    }
    
    // Process message based on type
    switch (message.type) {
      case 'update':
        onUpdate(message.componentId, message.payload);
        break;
      case 'batch':
        for (const update of message.messages) {
          onUpdate(update.componentId, update.payload);
        }
        break;
      case 'subscription_confirmed':
        console.log(`Subscribed to ${message.componentId}`);
        break;
      case 'error':
        console.error(`Error: ${message.code} - ${message.message}`);
        break;
      case 'pong':
        // Pong received, connection is alive
        break;
    }
  };
  
  // Handle errors
  socket.onerror = (error) => {
    console.error('WebSocket error:', error);
  };
  
  // Handle disconnection
  socket.onclose = (event) => {
    console.log(`Connection closed: ${event.code} - ${event.reason}`);
    // Implement reconnection logic here
  };
  
  // Set up ping intervals
  const pingInterval = setInterval(() => {
    if (socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type: 'ping' }));
    }
  }, 30000);
  
  // Return control methods
  return {
    subscribe: (componentId) => {
      socket.send(JSON.stringify({
        type: 'subscribe',
        componentId
      }));
    },
    unsubscribe: (componentId) => {
      socket.send(JSON.stringify({
        type: 'unsubscribe',
        componentId
      }));
    },
    close: () => {
      clearInterval(pingInterval);
      socket.close();
    }
  };
};
```

### Rust Client Example

```rust
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use url::Url;
use flate2::read::GzDecoder;
use std::io::Read;

async fn connect_to_dashboard(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the WebSocket server
    let url = Url::parse(url)?;
    let (mut ws_stream, _) = connect_async(url).await?;
    
    // Subscribe to a component
    let subscribe_msg = json!({
        "type": "subscribe",
        "componentId": "system_cpu"
    });
    
    ws_stream.send(Message::Text(subscribe_msg.to_string())).await?;
    
    // Process incoming messages
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Process text message
                let value: Value = serde_json::from_str(&text)?;
                process_message(value).await?;
            },
            Ok(Message::Binary(data)) => {
                // Decompress binary message
                let mut decoder = GzDecoder::new(&data[..]);
                let mut decompressed = String::new();
                decoder.read_to_string(&mut decompressed)?;
                
                let value: Value = serde_json::from_str(&decompressed)?;
                process_message(value).await?;
            },
            Ok(Message::Close(_)) => {
                println!("Connection closed by server");
                break;
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            },
            _ => {}
        }
    }
    
    Ok(())
}

async fn process_message(message: Value) -> Result<(), Box<dyn std::error::Error>> {
    let msg_type = message["type"].as_str().unwrap_or("");
    
    match msg_type {
        "update" => {
            let component_id = message["componentId"].as_str().unwrap_or("");
            println!("Update for {}: {:?}", component_id, message["payload"]);
        },
        "batch" => {
            if let Some(messages) = message["messages"].as_array() {
                for update in messages {
                    let component_id = update["componentId"].as_str().unwrap_or("");
                    println!("Batch update for {}: {:?}", component_id, update["payload"]);
                }
            }
        },
        "subscription_confirmed" => {
            let component_id = message["componentId"].as_str().unwrap_or("");
            println!("Subscribed to {}", component_id);
        },
        "error" => {
            let code = message["code"].as_str().unwrap_or("");
            let error_msg = message["message"].as_str().unwrap_or("");
            eprintln!("Error {}: {}", code, error_msg);
        },
        "pong" => {
            // Pong received
        },
        _ => {
            println!("Unknown message type: {}", msg_type);
        }
    }
    
    Ok(())
}
```

## Change Log

| Version | Date | Description |
|---------|------|-------------|
| 1.0.0   | 2024-05-22 | Initial protocol documentation |
| 1.0.1   | 2024-05-24 | Added compression and batching details | 