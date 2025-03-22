---
title: Real-Time Monitoring Pattern
version: 1.0.0
last_updated: 2024-04-01
status: active
---

# Real-Time Monitoring Pattern

## Context

This pattern addresses the implementation of real-time monitoring systems that require immediate feedback and visualization of system metrics, health status, and alerts. It is particularly relevant for:

- Dashboard implementations requiring live updates
- Systems where timely notification of issues is critical
- Monitoring UIs that need to display constantly changing metrics
- Applications with distributed components requiring central monitoring

## Problem

Traditional polling-based monitoring approaches have limitations:

1. High latency between state changes and visibility
2. Inefficient resource usage due to frequent polling
3. Difficulty scaling with many clients
4. Challenge in maintaining consistent state across multiple clients
5. Complex reconnection and recovery mechanisms

## Solution

Implement a WebSocket-based real-time monitoring system with these key components:

### 1. Event-Driven Architecture

- Use a broadcast channel for distributing updates to multiple subscribers
- Implement a publish-subscribe model for selective monitoring
- Process events asynchronously to avoid blocking

### 2. WebSocket Server Implementation

```rust
// Create a WebSocket handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

// Handle each WebSocket connection
async fn handle_socket(socket: WebSocket, state: ServerState) {
    // Split for concurrent send/receive
    let (sender, receiver) = socket.split();
    
    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();
    
    // Handle incoming messages (subscriptions)
    let recv_task = tokio::spawn(handle_incoming(receiver, state.clone()));
    
    // Send updates to client
    let send_task = tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            if let Err(_) = sender.send(Message::Text(update)).await {
                break; // Client disconnected
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = recv_task => {},
        _ = send_task => {},
    }
}
```

### 3. Client Subscription Model

```rust
// Process client subscription requests
async fn handle_incoming(
    mut receiver: SplitStream<WebSocket>,
    state: ServerState
) {
    // Track subscriptions for this client
    let subscriptions = Arc::new(Mutex::new(HashSet::new()));
    
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(sub) = serde_json::from_str::<Subscription>(&text) {
                match sub.action {
                    "subscribe" => {
                        subscriptions.lock().await.insert(sub.topic);
                    },
                    "unsubscribe" => {
                        subscriptions.lock().await.remove(&sub.topic);
                    },
                    _ => {} // Ignore unknown actions
                }
            }
        }
    }
}
```

### 4. Update Broadcasting

```rust
// Broadcast updates to all relevant subscribers
pub async fn broadcast_update(state: &ServerState, topic: &str, payload: impl Serialize) {
    let payload = match serde_json::to_string(&payload) {
        Ok(p) => p,
        Err(_) => return,
    };
    
    let update = json!({
        "topic": topic,
        "timestamp": chrono::Utc::now(),
        "payload": serde_json::from_str::<Value>(&payload).unwrap_or(Value::Null),
    }).to_string();
    
    let _ = state.tx.send(update);
}
```

### 5. Reconnection Management

- Implement client-side reconnection with exponential backoff
- Maintain session state to resume after reconnection
- Provide heartbeat mechanism to detect disconnections

```typescript
// Client-side reconnection example (TypeScript)
class MonitoringClient {
    private ws: WebSocket | null = null;
    private reconnectAttempts = 0;
    private subscriptions = new Set<string>();
    
    constructor(private url: string) {}
    
    connect() {
        this.ws = new WebSocket(this.url);
        
        this.ws.onopen = () => {
            console.log('Connected to monitoring server');
            this.reconnectAttempts = 0;
            
            // Resubscribe to previous topics
            this.subscriptions.forEach(topic => {
                this.subscribe(topic);
            });
        };
        
        this.ws.onclose = () => {
            this.handleDisconnection();
        };
        
        this.ws.onerror = () => {
            this.handleDisconnection();
        };
    }
    
    private handleDisconnection() {
        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
        this.reconnectAttempts++;
        
        console.log(`Disconnected. Reconnecting in ${delay}ms...`);
        setTimeout(() => this.connect(), delay);
    }
    
    subscribe(topic: string) {
        this.subscriptions.add(topic);
        if (this.ws?.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({
                action: 'subscribe',
                topic
            }));
        }
    }
    
    unsubscribe(topic: string) {
        this.subscriptions.delete(topic);
        if (this.ws?.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({
                action: 'unsubscribe',
                topic
            }));
        }
    }
}
```

## Implementation Guidelines

### 1. Server-Side Architecture

- Use a dedicated broadcast channel for distributing updates
- Implement proper error handling for disconnections
- Use separate tasks for receiving and sending messages
- Provide authentication and authorization for sensitive data
- Implement rate limiting to prevent abuse

```rust
pub struct MonitoringServer {
    // Configuration
    config: ServerConfig,
    // State shared across all connections
    state: ServerState,
    // Server handle for graceful shutdown
    server_handle: Option<JoinHandle<()>>,
}

impl MonitoringServer {
    pub async fn start(&mut self) -> Result<()> {
        // Create routes
        let router = Router::new()
            .route("/ws", get(self.websocket_handler))
            .with_state(self.state.clone());
        
        // Start server
        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.port));
        let server = axum::Server::bind(&addr)
            .serve(router.into_make_service());
            
        // Store handle for shutdown
        let handle = tokio::spawn(server);
        self.server_handle = Some(handle);
        
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
        Ok(())
    }
}
```

### 2. Message Format Standardization

Define a consistent message format for all monitoring events:

```json
{
    "topic": "system.metrics.cpu",
    "timestamp": "2024-04-01T12:34:56Z",
    "payload": {
        "usage": 0.67,
        "cores": 8,
        "processes": 120
    }
}
```

### 3. Performance Optimization

- Implement message batching for high-frequency updates
- Use binary formats (e.g., MessagePack, CBOR) for efficiency
- Implement server-side throttling for high-volume metrics
- Use compression for large payloads

```rust
// Message batching example
async fn batch_updates(updates: &mut Vec<Update>, tx: &Sender<String>) {
    if updates.is_empty() {
        return;
    }
    
    // Create batch message
    let batch = json!({
        "topic": "batch.update",
        "timestamp": chrono::Utc::now(),
        "payload": {
            "updates": updates
        }
    }).to_string();
    
    // Send batch and clear the queue
    let _ = tx.send(batch);
    updates.clear();
}
```

### 4. Testing Strategy

- Implement load testing with multiple simulated clients
- Test reconnection scenarios and error recovery
- Verify message delivery guarantees
- Test backpressure handling

## Tradeoffs

### Advantages

1. Near real-time updates with low latency
2. Efficient resource usage compared to polling
3. Better user experience with responsive dashboards
4. Reduced network traffic for frequent updates
5. Support for bidirectional communication

### Disadvantages

1. Increased server complexity
2. Challenges with connection management at scale
3. Additional client-side reconnection handling
4. State synchronization complexity
5. Potential memory leaks with improper connection handling

## Related Patterns

- [Async Programming Pattern](async-programming.md) - For handling concurrent WebSocket connections
- [Resource Management Pattern](resource-management.md) - For proper connection handling
- [Error Handling Pattern](error-handling.md) - For managing WebSocket errors and recovery
- [Adapter Implementation Guide](adapter-implementation-guide.md) - For creating monitoring adapters

## Example

A complete example of a WebSocket-based real-time monitoring dashboard:

1. Server implementation with Axum
2. Client implementation with TypeScript and React
3. Integration with metrics collection system
4. Real-time chart updates
5. Subscription management
6. Reconnection handling
7. Error recovery

## References

- [WebSocket Protocol RFC 6455](https://tools.ietf.org/html/rfc6455)
- [Axum WebSocket Documentation](https://docs.rs/axum/latest/axum/extract/ws/index.html)
- [Tokio Broadcast Channel](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)
- [React WebSocket Hooks](https://github.com/robtaussig/react-use-websocket)

<version>1.0.0</version> 