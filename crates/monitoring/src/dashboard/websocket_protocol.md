# WebSocket Protocol Documentation

## Overview

This document describes the WebSocket communication protocol used by the monitoring dashboard system. The WebSocket server provides real-time updates for dashboard components, health status, metrics, and alerts. This protocol enables bidirectional communication between clients and the server, allowing for efficient real-time data exchange.

## Connection Establishment

### WebSocket Endpoint

- **URL Path**: `/ws`
- **Protocol**: `ws://` or `wss://` (for secure connections)
- **Default Port**: 8080 (configurable via `DashboardConfig.server.port`)

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

### Standard Message Structure

All messages follow this general structure:
```json
{
  "type": "message_type",
  "timestamp": 1653472392000,
  "payload": { ... },
  "compressed": false
}
```

Where:
- `type`: Indicates the message type (e.g., "update", "subscribe")
- `timestamp`: Unix timestamp in milliseconds when the message was created
- `payload`: Message-specific data (structure varies by message type)
- `compressed`: Boolean indicating if the payload is compressed

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
  "componentId": "system_cpu",
  "timestamp": 1653472392000
}
```

### Subscribe to Multiple Components

Subscribe to multiple components in a single request:

```json
{
  "type": "subscribe_batch",
  "componentIds": ["system_cpu", "system_memory", "network_throughput"],
  "timestamp": 1653472392000
}
```

### Unsubscribe from Component

Unsubscribe from a component to stop receiving updates:

```json
{
  "type": "unsubscribe",
  "componentId": "system_cpu",
  "timestamp": 1653472392000
}
```

### Request Component Data

Request the current data for a component:

```json
{
  "type": "request_data",
  "componentId": "system_cpu",
  "timestamp": 1653472392000
}
```

### Request Components List

Request the list of available components:

```json
{
  "type": "list_components",
  "timestamp": 1653472392000
}
```

### Ping

Send a ping to check connection health:

```json
{
  "type": "ping",
  "timestamp": 1653472392000
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
      "timestamp": 1653472392000
    },
    {
      "type": "update",
      "componentId": "system_memory",
      "payload": {
        "used": 8590000000,
        "total": 16000000000
      },
      "timestamp": 1653472392000
    }
  ],
  "timestamp": 1653472392000,
  "compressed": false
}
```

### Components List

Response to a list_components request:

```json
{
  "type": "components_list",
  "components": [
    {
      "id": "system_cpu",
      "name": "CPU Usage",
      "type": "graph",
      "config": { ... }
    },
    {
      "id": "system_memory",
      "name": "Memory Usage",
      "type": "gauge",
      "config": { ... }
    }
  ],
  "timestamp": 1653472392000
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
  "details": { ... },
  "timestamp": 1653472392000
}
```

Common error codes:
- `invalid_request`: The request format is invalid
- `not_found`: The requested component was not found
- `rate_limit_exceeded`: Client has exceeded rate limits
- `unauthorized`: Client is not authorized to access the requested component
- `internal_error`: An internal server error occurred
- `subscription_error`: Failed to subscribe to component(s)
- `unsubscription_error`: Failed to unsubscribe from component(s)
- `invalid_component`: Component ID is invalid or unknown
- `decompression_error`: Failed to decompress a binary message

## Compression and Batching

### Message Compression

For efficient communication, the server compresses large messages before sending. The compression is applied based on configuration settings:

- `compression_enabled`: Whether compression is enabled
- `min_compression_size`: Minimum message size (in bytes) for compression to be applied
- `compression_level`: Compression level (0-9, where 9 is maximum compression)

When receiving a binary message, clients should decompress it using Gzip decompression. Example (JavaScript):

```javascript
async function decompressMessage(binaryData) {
  // Decompress using pako (gzip implementation for JavaScript)
  const decompressedData = pako.inflate(binaryData, { to: 'string' });
  return JSON.parse(decompressedData);
}
```

### Message Batching

To reduce the number of messages for high-frequency updates, the server batches multiple component updates into a single message:

- `batching_enabled`: Whether batching is enabled
- `max_messages`: Maximum number of messages per batch
- `max_interval_ms`: Maximum batch interval in milliseconds

Batching logic:
1. Server collects updates within the batching interval
2. If the number of updates exceeds `max_messages` or the interval elapses, a batch is sent
3. Each batch has a unique `batch_id` for correlation

## Reconnection Strategies

### Client Reconnection Algorithm

Clients should implement a robust reconnection strategy with exponential backoff:

```javascript
class WebSocketClient {
  constructor(url) {
    this.url = url;
    this.socket = null;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 10;
    this.baseReconnectDelay = 100; // ms
    this.maxReconnectDelay = 30000; // 30 seconds
    this.subscriptions = new Set();
    this.connect();
  }

  connect() {
    this.socket = new WebSocket(this.url);
    
    this.socket.onopen = () => {
      console.log('Connected to WebSocket server');
      this.reconnectAttempts = 0;
      // Resubscribe to previous subscriptions
      this.resubscribe();
    };
    
    this.socket.onclose = (event) => {
      if (!event.wasClean) {
        this.scheduleReconnect();
      }
    };
    
    this.socket.onerror = () => {
      this.socket.close();
    };
    
    this.socket.onmessage = (event) => {
      this.handleMessage(event.data);
    };
  }
  
  scheduleReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Maximum reconnection attempts reached');
      return;
    }
    
    // Calculate delay with exponential backoff
    const delay = Math.min(
      this.baseReconnectDelay * Math.pow(2, this.reconnectAttempts),
      this.maxReconnectDelay
    );
    
    console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts + 1})`);
    
    setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, delay);
  }
  
  resubscribe() {
    // Resubscribe to all previous subscriptions
    for (const componentId of this.subscriptions) {
      this.subscribe(componentId);
    }
  }
  
  subscribe(componentId) {
    this.subscriptions.add(componentId);
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({
        type: 'subscribe',
        componentId,
        timestamp: Date.now()
      }));
    }
  }
  
  handleMessage(data) {
    // Check if the message is binary (compressed)
    if (data instanceof Blob) {
      // Handle binary message
      data.arrayBuffer().then(buffer => {
        const uint8Array = new Uint8Array(buffer);
        return this.decompressMessage(uint8Array);
      }).then(message => {
        this.processMessage(message);
      }).catch(error => {
        console.error('Error decompressing message:', error);
      });
    } else {
      // Handle text message
      try {
        const message = JSON.parse(data);
        this.processMessage(message);
      } catch (error) {
        console.error('Error parsing message:', error);
      }
    }
  }
  
  async decompressMessage(compressedData) {
    // Use pako or other gzip implementation
    const decompressedData = pako.inflate(compressedData, { to: 'string' });
    return JSON.parse(decompressedData);
  }
  
  processMessage(message) {
    switch (message.type) {
      case 'update':
        this.handleUpdate(message);
        break;
      case 'batch':
        this.handleBatch(message);
        break;
      case 'subscription_confirmed':
        console.log(`Subscription confirmed: ${message.componentId}`);
        break;
      case 'error':
        console.error(`WebSocket error: ${message.code} - ${message.message}`);
        break;
      // Handle other message types
    }
  }
  
  // Other methods for handling specific message types
}
```

### Server Reconnection Handling

The server handles client reconnection as follows:

1. When a client disconnects, the server:
   - Marks the client's subscriptions as inactive
   - Keeps the subscription data for a configurable period (default: 5 minutes)
   - Periodically cleans up inactive subscriptions that exceed the retention period

2. When a client reconnects:
   - A new connection is established with a new client ID
   - The client must resubscribe to components
   - If the client resubscribes within the retention period, the server sends the latest data immediately

## Performance Considerations

### Client-Side Optimization

1. **Selective Subscriptions**: Subscribe only to components that are currently visible
2. **Unsubscribe When Not Needed**: Unsubscribe from components when they're no longer visible
3. **Batch Processing**: Process batch updates efficiently, avoiding UI blocking
4. **Throttle UI Updates**: Limit the frequency of UI updates for high-frequency data

Example of visibility-based subscription management:

```javascript
class DashboardComponent {
  constructor(componentId, client) {
    this.componentId = componentId;
    this.client = client;
    this.visible = false;
    this.observer = new IntersectionObserver(this.handleVisibilityChange.bind(this));
    
    // Start observing the DOM element
    const element = document.getElementById(componentId);
    if (element) {
      this.observer.observe(element);
    }
  }
  
  handleVisibilityChange(entries) {
    const [entry] = entries;
    const isVisible = entry.isIntersecting;
    
    if (isVisible && !this.visible) {
      // Component became visible, subscribe
      this.client.subscribe(this.componentId);
      this.visible = true;
    } else if (!isVisible && this.visible) {
      // Component is no longer visible, unsubscribe
      this.client.unsubscribe(this.componentId);
      this.visible = false;
    }
  }
  
  // Clean up when component is destroyed
  destroy() {
    this.observer.disconnect();
    if (this.visible) {
      this.client.unsubscribe(this.componentId);
    }
  }
}
```

### Server-Side Optimization

1. **Message Batching**: The server batches multiple updates to reduce protocol overhead
2. **Message Compression**: The server compresses large messages to reduce bandwidth usage
3. **Subscription Management**: The server tracks active subscriptions to avoid sending unnecessary updates
4. **Rate Limiting**: The server implements rate limiting to prevent abuse

## Error Handling

### Connection Errors

If the WebSocket connection is lost, clients should implement a reconnection strategy with exponential backoff as described in the Reconnection Strategies section.

### Protocol Errors

When receiving an error message, clients should handle it based on the error code:

- For transient errors (like rate limiting), retry with backoff
- For permanent errors (like unauthorized), fix the issue before retrying
- For internal errors, log the error and retry with backoff

Example error handling:

```javascript
function handleError(error) {
  switch (error.code) {
    case 'rate_limit_exceeded':
      // Wait and retry with backoff
      setTimeout(() => {
        // Retry operation
      }, 5000);
      break;
      
    case 'unauthorized':
      // Refresh authentication token
      refreshToken().then(() => {
        // Retry with new token
      });
      break;
      
    case 'not_found':
      // Component no longer exists, remove from UI
      removeComponent(error.componentId);
      break;
      
    case 'internal_error':
      // Log error and retry with backoff
      console.error('Internal server error:', error.message);
      setTimeout(() => {
        // Retry operation
      }, 10000);
      break;
      
    default:
      // Handle other error types
      console.error('WebSocket error:', error);
  }
}
```

## Security Considerations

### Authentication and Authorization

- Use HTTPS and WSS for secure connections
- Keep authentication tokens secure and use appropriate expiration
- Validate all input from clients to prevent injection attacks
- Implement proper token refresh mechanisms to maintain session security

Authentication token refresh example:

```javascript
class SecureWebSocketClient extends WebSocketClient {
  constructor(url, authProvider) {
    super(url);
    this.authProvider = authProvider;
    this.setupTokenRefresh();
  }
  
  async getAuthenticatedUrl() {
    const token = await this.authProvider.getToken();
    const url = new URL(this.url);
    url.searchParams.set('token', token);
    return url.toString();
  }
  
  async connect() {
    const authenticatedUrl = await this.getAuthenticatedUrl();
    this.socket = new WebSocket(authenticatedUrl);
    // Rest of connection setup
  }
  
  setupTokenRefresh() {
    // Set up token refresh before expiration
    this.authProvider.onTokenExpiring(() => {
      // Get a new token
      this.authProvider.refreshToken().then(() => {
        // Reconnect with new token
        this.socket.close();
      });
    });
  }
}
```

### Rate Limiting

The server implements rate limiting to prevent abuse:

- Connection rate limiting: Limits the number of connection attempts per IP
- Message rate limiting: Limits the number of messages per client
- Subscription rate limiting: Limits the number of subscriptions per client

Rate limits can be configured in the server configuration:

```rust
// Server configuration
let config = DashboardConfig {
    rate_limiting: Some(RateLimitConfig {
        connections_per_minute: 60,       // 60 connections per minute per IP
        messages_per_minute: 600,         // 600 messages per minute per client
        max_subscriptions_per_client: 50, // 50 subscriptions per client
    }),
    // Other configuration
};
```

### Data Privacy

- Sensitive data should be masked or encrypted
- Use the data masking configuration to automatically mask sensitive fields
- Implement proper access control to restrict access to sensitive data

Data masking configuration example:

```rust
// Data masking configuration
let config = DashboardConfig {
    data_masking: Some(DataMaskingConfig {
        mask_sensitive_fields: true,
        sensitive_field_patterns: vec![
            "password".to_string(),
            "token".to_string(),
            "key".to_string(),
            "secret".to_string(),
            "credential".to_string(),
        ],
        mask_character: "*".to_string(),
    }),
    // Other configuration
};
```

## Testing and Validation

To ensure reliable WebSocket communication, thorough testing should be performed:

1. **Connection Testing**: Test connection establishment, authentication, and disconnection
2. **Message Exchange Testing**: Test all message types and verify correct handling
3. **Reconnection Testing**: Test reconnection scenarios with network interruptions
4. **Performance Testing**: Test with high message rates and many concurrent clients
5. **Security Testing**: Test authentication, authorization, and data privacy features

The monitoring crate includes comprehensive test utilities for WebSocket testing:

```rust
// Example of WebSocket connection testing
#[tokio::test]
async fn test_websocket_connection() -> Result<()> {
    // Set up test server
    let config = create_test_config();
    let server = DashboardServer::new(config);
    server.start().await?;
    
    // Connect to WebSocket server
    let url = format!("ws://{}:{}/ws", config.host, config.port);
    let (ws_stream, _) = connect_async(url).await?;
    
    // Test subscription
    let (mut sender, mut receiver) = ws_stream.split();
    
    // Send subscription request
    let subscribe_msg = json!({
        "type": "subscribe",
        "componentId": "test_component",
        "timestamp": Utc::now().timestamp_millis(),
    }).to_string();
    
    sender.send(Message::Text(subscribe_msg)).await?;
    
    // Wait for confirmation
    let response = receiver.next().await.unwrap()?;
    
    // Verify response
    if let Message::Text(text) = response {
        let json: Value = serde_json::from_str(&text)?;
        assert_eq!(json["type"], "subscription_confirmed");
        assert_eq!(json["componentId"], "test_component");
    } else {
        panic!("Expected text message");
    }
    
    // Clean up
    server.stop().await?;
    
    Ok(())
}
```

## Version History

| Version | Date       | Changes                                                   |
|---------|------------|-----------------------------------------------------------|
| 1.0.0   | 2023-03-01 | Initial protocol specification                            |
| 1.1.0   | 2023-06-15 | Added batch updates and compression                       |
| 1.2.0   | 2023-09-22 | Added support for multiple component subscription         |
| 1.3.0   | 2024-02-10 | Enhanced reconnection handling and error codes            |
| 2.0.0   | 2024-06-20 | Comprehensive update with client/server implementation examples, performance optimizations, and improved error handling | 