# Dashboard WebSocket Protocol Documentation

## Overview

The dashboard WebSocket protocol provides real-time communication between clients and the monitoring system. This protocol enables clients to subscribe to metrics, receive updates, and interact with the dashboard in real-time.

## Connection Establishment

Clients connect to the WebSocket server using the following URL pattern:

```
ws://[hostname]:[port]/ws
```

Example:
```
ws://localhost:8765/ws
```

## Message Format

All messages use JSON format for data exchange. The general structure of a message is:

```json
{
  "type": "message_type",
  "timestamp": 1650123456789,
  ... message-specific fields ...
}
```

The `type` field determines the message type and how it should be processed.

## Client Messages

### 1. Subscribe to Component

Subscribe to receive updates for a specific component.

```json
{
  "type": "subscribe",
  "componentId": "system_cpu"
}
```

### 2. Unsubscribe from Component

Unsubscribe from a specific component to stop receiving updates.

```json
{
  "type": "unsubscribe",
  "componentId": "system_cpu"
}
```

### 3. Request Batch Update

Request a batch update for multiple components at once.

```json
{
  "type": "request_batch",
  "components": ["system_cpu", "system_memory", "network_traffic"],
  "includeHistory": true,
  "historyPoints": 10
}
```

### 4. Ping

Send a ping to keep the connection alive.

```json
{
  "type": "ping",
  "timestamp": 1650123456789
}
```

### 5. Acknowledge Alert

Acknowledge an alert by its ID.

```json
{
  "type": "acknowledge_alert",
  "alertId": "alert-123456",
  "userId": "user-789"
}
```

## Server Messages

### 1. Component Update

Single component update message.

```json
{
  "type": "update",
  "timestamp": 1650123456789,
  "componentId": "system_cpu",
  "data": {
    "usage": 45.2,
    "cores": 8,
    "temperature": 72.1
  }
}
```

### 2. Batch Update

Multiple component updates in a single message.

```json
{
  "type": "batch",
  "timestamp": 1650123456789,
  "updates": [
    {
      "componentId": "system_cpu",
      "data": {
        "usage": 45.2,
        "cores": 8,
        "temperature": 72.1
      }
    },
    {
      "componentId": "system_memory",
      "data": {
        "total": 16384,
        "used": 8192,
        "cached": 4096
      }
    }
  ]
}
```

### 3. Compressed Update

Large data responses are compressed to reduce bandwidth usage.

```json
{
  "type": "compressed",
  "timestamp": 1650123456789,
  "compressed": true,
  "compression": "gzip",
  "encoding": "base64",
  "compressed_data": "H4sIAAAAAAAA/6tWSs/PT89J9cjPTM/MKwYA6F ... (base64 encoded gzipped data)"
}
```

To handle compressed messages:
1. Extract the `compressed_data` field
2. Base64 decode the data
3. Decompress using gzip
4. Parse the resulting JSON

### 4. Pong

Response to a ping message.

```json
{
  "type": "pong",
  "timestamp": 1650123456789,
  "server_time": 1650123456790
}
```

### 5. Alert

Alert notification message.

```json
{
  "type": "alert",
  "timestamp": 1650123456789,
  "alertId": "alert-123456",
  "severity": "critical",
  "source": "system_cpu",
  "message": "CPU usage exceeded 90% for 5 minutes",
  "data": {
    "usage": 95.2,
    "threshold": 90.0,
    "duration": 300
  }
}
```

### 6. Error

Error response message.

```json
{
  "type": "error",
  "timestamp": 1650123456789,
  "code": "invalid_request",
  "message": "Invalid component ID specified"
}
```

## Message Compression

For large data responses, the server automatically compresses the message to reduce bandwidth. This typically occurs when:

1. Batch updates contain multiple components
2. History data is requested
3. The payload size exceeds a threshold (usually 10KB)

The compression process follows these steps:

1. The original JSON message is serialized to a string
2. The string is compressed using gzip
3. The compressed data is encoded using base64
4. The encoded data is included in a `compressed` message type

Clients should decompress these messages by:

1. Extracting the base64-encoded compressed data
2. Decoding from base64
3. Decompressing with gzip
4. Parsing the resulting JSON string

## Rate Limiting

To prevent server overload, the following rate limits apply:

1. Maximum 50 subscriptions per client
2. Maximum 10 requests per second per client
3. Maximum 100 connections per IP address

Exceeding these limits will result in an error message or connection termination.

## Connection Management

### Reconnection

Clients should implement reconnection logic with exponential backoff:

1. First reconnect attempt: immediate
2. Second attempt: 1 second delay
3. Third attempt: 2 seconds delay
4. Fourth attempt: 4 seconds delay
5. Maximum delay: 30 seconds

Upon reconnection, clients should re-subscribe to all components they were previously subscribed to.

### Keepalive

To keep the connection alive, clients should:

1. Send a ping message every 15-30 seconds
2. Monitor for pong responses
3. Implement reconnection if no response is received

## Usage Examples

### JavaScript Client Example

```javascript
// Connect to WebSocket server
const ws = new WebSocket('ws://localhost:8765/ws');

// Set up event handlers
ws.onopen = () => {
  console.log('Connected to WebSocket server');
  
  // Subscribe to components
  ws.send(JSON.stringify({
    type: 'subscribe',
    componentId: 'system_cpu'
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  switch (message.type) {
    case 'update':
      console.log(`Update for ${message.componentId}:`, message.data);
      break;
      
    case 'compressed':
      // Handle compressed message
      const compressedData = message.compressed_data;
      const decodedData = atob(compressedData); // Base64 decode
      
      // Use pako or similar library for decompression
      const decompressed = pako.ungzip(decodedData);
      const decompressedStr = new TextDecoder().decode(decompressed);
      const decompressedJson = JSON.parse(decompressedStr);
      
      console.log('Decompressed data:', decompressedJson);
      break;
      
    case 'batch':
      console.log('Batch update with', message.updates.length, 'items');
      break;
      
    case 'error':
      console.error('Error:', message.message);
      break;
  }
};

// Implement reconnection
ws.onclose = () => {
  console.log('Connection closed, reconnecting...');
  setTimeout(() => {
    // Reconnect logic here
  }, 1000);
};

// Implement keepalive
setInterval(() => {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({
      type: 'ping',
      timestamp: Date.now()
    }));
  }
}, 15000);
```

### Rust Client Example

```rust
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use serde_json::json;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use flate2::read::GzDecoder;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to WebSocket server
    let url = Url::parse("ws://localhost:8765/ws")?;
    let (mut ws_stream, _) = connect_async(url).await?;
    
    // Subscribe to a component
    let subscribe_msg = json!({
        "type": "subscribe",
        "componentId": "system_cpu"
    }).to_string();
    
    ws_stream.send(Message::Text(subscribe_msg)).await?;
    
    // Process incoming messages
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let parsed: serde_json::Value = serde_json::from_str(&text)?;
                
                match parsed["type"].as_str() {
                    Some("update") => {
                        println!("Update for {}: {:?}", 
                            parsed["componentId"], parsed["data"]);
                    },
                    Some("compressed") => {
                        // Handle compressed message
                        if let Some(compressed_data) = parsed["compressed_data"].as_str() {
                            // Decode base64
                            let data = BASE64.decode(compressed_data)?;
                            
                            // Decompress
                            let mut decoder = GzDecoder::new(&data[..]);
                            let mut decompressed = String::new();
                            decoder.read_to_string(&mut decompressed)?;
                            
                            // Parse decompressed JSON
                            let decompressed_json: serde_json::Value = 
                                serde_json::from_str(&decompressed)?;
                            
                            println!("Decompressed data: {:?}", decompressed_json);
                        }
                    },
                    Some("batch") => {
                        if let Some(updates) = parsed["updates"].as_array() {
                            println!("Batch update with {} items", updates.len());
                        }
                    },
                    Some("error") => {
                        println!("Error: {}", parsed["message"]);
                    },
                    _ => {
                        println!("Unknown message type: {}", text);
                    }
                }
            },
            Ok(Message::Close(..)) => {
                println!("WebSocket closed");
                break;
            },
            _ => {}
        }
    }
    
    Ok(())
}
```

## Performance Considerations

1. **Message Size**: Large messages are automatically compressed to reduce bandwidth.
2. **Subscription Management**: Only subscribe to components you need to minimize traffic.
3. **Batch Requests**: Use batch requests instead of multiple individual requests.
4. **Connection Pooling**: Reuse WebSocket connections when possible.
5. **Error Handling**: Implement proper error handling and reconnection logic.

## Security

1. The WebSocket server supports TLS encryption (wss:// protocol).
2. Authentication can be implemented using bearer tokens in the request headers.
3. All data is validated on both client and server sides.
4. Messages are rate-limited to prevent abuse.

## Troubleshooting

Common issues and solutions:

1. **Connection Refused**: Check that the server is running and the port is correct.
2. **Authentication Failed**: Verify your authentication token.
3. **Rate Limited**: Reduce the frequency of your requests.
4. **Large Message Failure**: Ensure your client supports message decompression.
5. **Disconnections**: Implement robust reconnection logic.

## Further Resources

- [Dashboard API Documentation](./dashboard_api.md)
- [Monitoring System Overview](./system_overview.md)
- [Alert Management](./alert_management.md)
- [Component Registry](./component_registry.md) 