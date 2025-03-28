---
version: 1.0.0
date: 2024-08-17
status: active
author: DataScienceBioLab
---

# MCP Implementation Patterns Guide

This guide documents recommended patterns and best practices for working with the Machine Context Protocol (MCP) implementation. Teams developing components that integrate with MCP should follow these patterns for consistent, maintainable, and efficient code.

## Table of Contents

1. [Transport Selection Patterns](#transport-selection-patterns)
2. [Message Handler Patterns](#message-handler-patterns)
3. [Error Handling Patterns](#error-handling-patterns)
4. [Security Integration Patterns](#security-integration-patterns)
5. [Testing Patterns](#testing-patterns)
6. [Performance Optimization Patterns](#performance-optimization-patterns)
7. [Cross-Platform Integration Patterns](#cross-platform-integration-patterns)

## Transport Selection Patterns

### When to Use Each Transport Type

| Transport Type | Use Cases | Advantages | Limitations |
|----------------|-----------|------------|-------------|
| **TCP** | Server-to-server communication, high-throughput systems | High performance, established protocol | Requires network configuration, firewall considerations |
| **WebSocket** | Browser-to-server, cross-platform clients | Web compatibility, firewall-friendly | Slightly higher overhead than raw TCP |
| **stdio** | Process-to-process, command-line tools | Simple setup, works with pipes | Limited to local processes |

### Transport Factory Pattern

Use a factory pattern to create the appropriate transport based on configuration:

```rust
pub struct TransportFactory;

impl TransportFactory {
    pub fn create(config: &TransportConfig) -> Box<dyn Transport> {
        match config.transport_type {
            TransportType::Tcp => Box::new(TcpTransport::new(config.tcp_config.clone())),
            TransportType::WebSocket => Box::new(WebSocketTransport::new(config.ws_config.clone())),
            TransportType::Stdio => Box::new(StdioTransport::new()),
            TransportType::Custom => {
                if let Some(factory) = &config.custom_factory {
                    factory.create_transport()
                } else {
                    panic!("Custom transport requires factory function")
                }
            }
        }
    }
}
```

### Transport Configuration Pattern

Use structured configuration with sensible defaults:

```rust
// Example usage
let config = TransportConfig::builder()
    .transport_type(TransportType::Tcp)
    .tcp_config(TcpConfig::builder()
        .remote_address("127.0.0.1:8080")
        .timeout(Duration::from_secs(30))
        .build())
    .build();

let transport = TransportFactory::create(&config);
```

## Message Handler Patterns

### Single Responsibility Handlers

Each handler should focus on a specific message type or related group:

```rust
pub struct CommandExecutionHandler {
    executor: Arc<ToolExecutor>,
}

#[async_trait]
impl MessageHandler for CommandExecutionHandler {
    async fn handle_message(&self, message: MCPMessage) -> Result<Option<MCPMessage>, MCPError> {
        // Handle only command messages
        if message.type_ != MessageType::Command {
            return Ok(None);
        }

        // Extract command from payload
        let payload = message.payload.as_object()
            .ok_or_else(|| MCPError::Protocol("Invalid payload format".into()))?;
        
        let command = payload.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::Protocol("Missing command field".into()))?;

        // Execute command and create response
        let result = self.executor.execute(command, payload).await?;
        
        Ok(Some(MCPMessage::new(MessageType::Response, result)
            .with_trace_id(message.id.0.clone())))
    }

    fn supported_message_types(&self) -> Vec<MessageType> {
        vec![MessageType::Command]
    }
}
```

### Composite Handler Pattern

Combine multiple handlers using the composite pattern:

```rust
pub struct CompositeHandler {
    handlers: Vec<Box<dyn MessageHandler>>,
}

impl CompositeHandler {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn add_handler(&mut self, handler: Box<dyn MessageHandler>) {
        self.handlers.push(handler);
    }
}

#[async_trait]
impl MessageHandler for CompositeHandler {
    async fn handle_message(&self, message: MCPMessage) -> Result<Option<MCPMessage>, MCPError> {
        for handler in &self.handlers {
            if handler.supported_message_types().contains(&message.type_) {
                if let Some(response) = handler.handle_message(message.clone()).await? {
                    return Ok(Some(response));
                }
            }
        }
        Ok(None)
    }

    fn supported_message_types(&self) -> Vec<MessageType> {
        self.handlers.iter()
            .flat_map(|h| h.supported_message_types())
            .collect()
    }
}
```

### Event-Driven Pattern

Use callbacks for event handling:

```rust
// Application code
let subscription_id = client.subscribe(
    MessageType::Event,
    Box::new(|message| Box::pin(async move {
        if let Some(event_type) = message.payload.get("event_type").and_then(|v| v.as_str()) {
            match event_type {
                "status_changed" => handle_status_change(&message).await,
                "alert" => process_alert(&message).await,
                _ => log::warn!("Unknown event type: {}", event_type),
            }
        }
    }))
).await?;
```

## Error Handling Patterns

### Context-Rich Errors

Always add context to errors as they propagate:

```rust
async fn process_command(&self, command: &str) -> Result<(), MCPError> {
    let client = self.get_client().await
        .context("Failed to initialize client during command processing")?;
    
    client.connect().await
        .context("Failed to connect to MCP server during command processing")?;
    
    client.send_command(command, self.params.clone()).await
        .context(format!("Failed to execute command '{}'", command))?;
    
    Ok(())
}
```

### Error Mapping Pattern

Implement `From` traits for clean error conversion:

```rust
impl From<std::io::Error> for TransportError {
    fn from(error: std::io::Error) -> Self {
        TransportError::IoError(error.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for TransportError {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        TransportError::ProtocolError(format!("WebSocket error: {}", error))
    }
}
```

### Retry Pattern

Use the retry mechanism for transient errors:

```rust
async fn send_with_retry(&self, message: MCPMessage) -> Result<MCPResponse, MCPError> {
    let retry_config = RetryConfig::builder()
        .max_attempts(3)
        .base_delay(Duration::from_millis(100))
        .max_delay(Duration::from_secs(1))
        .build();
    
    let retry = RetryMechanism::new(retry_config);
    
    retry.execute(|| async {
        self.client.send_message(message.clone()).await
    }).await
}
```

## Security Integration Patterns

### Security Level Selection Pattern

Choose security level based on the context:

```rust
fn select_security_level(config: &AppConfig) -> SecurityLevel {
    match config.environment {
        Environment::Development => {
            if config.enable_security {
                SecurityLevel::Basic
            } else {
                SecurityLevel::None
            }
        },
        Environment::Testing => SecurityLevel::Standard,
        Environment::Production => SecurityLevel::High,
    }
}
```

### Credential Management Pattern

Separate credential storage from usage:

```rust
pub struct CredentialProvider {
    keychain: Arc<dyn Keychain>,
}

impl CredentialProvider {
    pub async fn get_credentials(&self, purpose: &str) -> Result<Credentials, SecurityError> {
        match purpose {
            "mcp_service" => {
                let username = self.keychain.get_secret("mcp_username").await?;
                let password = self.keychain.get_secret("mcp_password").await?;
                
                Ok(Credentials::new(username, password))
            },
            // Other credential types
            _ => Err(SecurityError::Unknown(format!("Unknown credential purpose: {}", purpose))),
        }
    }
}
```

### Secure Channel Pattern

Ensure connections are secured:

```rust
async fn create_secure_client(&self, config: &Config) -> Result<Client, Error> {
    let transport = TcpTransport::new(config.tcp_config.clone());
    
    // Create security manager based on configuration
    let security_manager = match config.security_level {
        SecurityLevel::None => Arc::new(NoOpSecurityManager::new()),
        SecurityLevel::Basic => Arc::new(BasicSecurityManager::new(config.credentials.clone())),
        SecurityLevel::Standard => Arc::new(StandardSecurityManager::new(
            config.credentials.clone(),
            config.encryption_key.clone(),
        )),
        SecurityLevel::High => Arc::new(HighSecurityManager::new(
            config.credentials.clone(),
            config.encryption_key.clone(),
            config.signing_key.clone(),
        )),
    };
    
    // Wrap transport with security
    let secure_transport = SecureTransport::new(transport, security_manager);
    
    Ok(Client::new(secure_transport))
}
```

## Testing Patterns

### Mock Transport Pattern

Use mock transports for testing:

```rust
pub struct MockTransport {
    messages: Arc<Mutex<Vec<MCPMessage>>>,
    responses: Arc<Mutex<VecDeque<Result<MCPMessage, TransportError>>>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push_response(&self, response: Result<MCPMessage, TransportError>) {
        let mut responses = self.responses.lock().unwrap();
        responses.push_back(response);
    }

    pub fn get_sent_messages(&self) -> Vec<MCPMessage> {
        let messages = self.messages.lock().unwrap();
        messages.clone()
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn send_message(&self, message: MCPMessage) -> Result<(), TransportError> {
        let mut messages = self.messages.lock().unwrap();
        messages.push(message);
        Ok(())
    }

    async fn receive_message(&self) -> Result<MCPMessage, TransportError> {
        let mut responses = self.responses.lock().unwrap();
        match responses.pop_front() {
            Some(result) => result,
            None => Err(TransportError::ConnectionClosed("No more mock responses".into())),
        }
    }

    // Implement other required methods
}
```

### Test Handler Pattern

Create test-specific message handlers:

```rust
pub struct TestHandler {
    calls: Arc<AtomicUsize>,
    responses: Arc<Mutex<HashMap<MessageType, MCPMessage>>>,
}

impl TestHandler {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_response(&self, msg_type: MessageType, response: MCPMessage) {
        let mut responses = self.responses.lock().unwrap();
        responses.insert(msg_type, response);
    }

    pub fn get_call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl MessageHandler for TestHandler {
    async fn handle_message(&self, message: MCPMessage) -> Result<Option<MCPMessage>, MCPError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        
        let responses = self.responses.lock().unwrap();
        if let Some(response) = responses.get(&message.type_) {
            Ok(Some(response.clone()))
        } else {
            Ok(None)
        }
    }

    fn supported_message_types(&self) -> Vec<MessageType> {
        vec![MessageType::Command, MessageType::Event]
    }
}
```

### Integration Test Pattern

Test full communication flow:

```rust
#[tokio::test]
async fn test_client_server_communication() {
    // Setup in-memory transport pair
    let (client_transport, server_transport) = create_memory_transport_pair();
    
    // Create server with handlers
    let mut server = Server::new(server_transport);
    server.register_handler(Box::new(EchoHandler::new())).await.unwrap();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.start().await.unwrap();
    });
    
    // Create and connect client
    let client = Client::new(client_transport);
    client.connect().await.unwrap();
    
    // Send command and verify response
    let response = client.send_command(
        "echo", 
        serde_json::json!({ "message": "test message" })
    ).await.unwrap();
    
    assert_eq!(response.status, StatusCode::Ok);
    assert_eq!(
        response.payload.get("echo").and_then(|m| m.as_str()),
        Some("test message")
    );
    
    // Clean up
    client.disconnect().await.unwrap();
    server_handle.abort();
}
```

## Performance Optimization Patterns

### Connection Pooling Pattern

Reuse connections for better performance:

```rust
pub struct ConnectionPool<T: Transport> {
    connections: Arc<Mutex<Vec<Arc<T>>>>,
    config: ConnectionPoolConfig,
    factory: Box<dyn Fn() -> Result<T, TransportError> + Send + Sync>,
}

impl<T: Transport> ConnectionPool<T> {
    pub fn new(
        config: ConnectionPoolConfig,
        factory: impl Fn() -> Result<T, TransportError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            config,
            factory: Box::new(factory),
        }
    }

    pub async fn get_connection(&self) -> Result<PooledConnection<T>, TransportError> {
        let conn = {
            let mut conns = self.connections.lock().await;
            if let Some(conn) = conns.pop() {
                conn
            } else {
                let transport = (self.factory)()?;
                Arc::new(transport)
            }
        };

        Ok(PooledConnection {
            transport: conn,
            pool: self.connections.clone(),
        })
    }
}
```

### Message Batching Pattern

Batch messages when appropriate:

```rust
pub struct MessageBatch {
    messages: Vec<MCPMessage>,
    max_size: usize,
}

impl MessageBatch {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_size,
        }
    }

    pub fn add(&mut self, message: MCPMessage) -> Result<(), MCPError> {
        if self.messages.len() >= self.max_size {
            return Err(MCPError::Protocol("Batch full".into()));
        }
        self.messages.push(message);
        Ok(())
    }

    pub async fn send(&self, client: &Client) -> Result<Vec<MCPResponse>, MCPError> {
        // Create batch message
        let batch_payload = serde_json::json!({
            "messages": self.messages,
        });

        let batch_message = MCPMessage::new(
            MessageType::Batch,
            batch_payload,
        );

        // Send batch
        let response = client.send_message(batch_message).await?;
        
        // Parse batch response
        let responses = response.payload
            .get("responses")
            .and_then(|r| r.as_array())
            .ok_or_else(|| MCPError::Protocol("Invalid batch response".into()))?;
            
        // Convert to MCPResponse objects
        let mut results = Vec::new();
        for resp in responses {
            results.push(MCPResponse::from_json(resp.clone())?);
        }
        
        Ok(results)
    }
}
```

### Async Resource Management Pattern

Use proper async resource management:

```rust
pub struct ResourceManager<R> {
    resources: Arc<RwLock<HashMap<ResourceId, Arc<R>>>>,
    factory: Box<dyn Fn(ResourceId) -> Pin<Box<dyn Future<Output = Result<R, MCPError>> + Send>> + Send + Sync>,
}

impl<R: Send + Sync + 'static> ResourceManager<R> {
    pub fn new(
        factory: impl Fn(ResourceId) -> Pin<Box<dyn Future<Output = Result<R, MCPError>> + Send>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            factory: Box::new(factory),
        }
    }

    pub async fn get_resource(&self, id: ResourceId) -> Result<Arc<R>, MCPError> {
        // First check if we already have the resource
        {
            let resources = self.resources.read().await;
            if let Some(resource) = resources.get(&id) {
                return Ok(resource.clone());
            }
        }

        // Create the resource
        let resource = (self.factory)(id).await?;
        let resource_arc = Arc::new(resource);
        
        // Store it for future use
        {
            let mut resources = self.resources.write().await;
            resources.insert(id, resource_arc.clone());
        }
        
        Ok(resource_arc)
    }
}
```

## Cross-Platform Integration Patterns

### Platform Abstraction Pattern

Abstract platform-specific code:

```rust
pub trait PlatformServices: Send + Sync {
    fn create_transport(&self, config: &TransportConfig) -> Box<dyn Transport>;
    fn create_security_manager(&self, config: &SecurityConfig) -> Box<dyn SecurityManager>;
    fn create_logger(&self, config: &LogConfig) -> Box<dyn Logger>;
    // Other platform-specific services
}

pub struct PlatformFactory;

impl PlatformFactory {
    pub fn create() -> Box<dyn PlatformServices> {
        #[cfg(target_os = "windows")]
        {
            Box::new(WindowsPlatformServices::new())
        }
        
        #[cfg(target_os = "linux")]
        {
            Box::new(LinuxPlatformServices::new())
        }
        
        #[cfg(target_os = "macos")]
        {
            Box::new(MacOSPlatformServices::new())
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            Box::new(WebPlatformServices::new())
        }
    }
}
```

### Environment Detection Pattern

Configure behavior based on execution environment:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl Environment {
    pub fn current() -> Self {
        match std::env::var("MCP_ENV").as_deref() {
            Ok("development") => Environment::Development,
            Ok("testing") => Environment::Testing,
            Ok("staging") => Environment::Staging,
            Ok("production") => Environment::Production,
            _ => {
                // Default to development for safety
                #[cfg(debug_assertions)]
                {
                    Environment::Development
                }
                
                #[cfg(not(debug_assertions))]
                {
                    Environment::Production
                }
            }
        }
    }
    
    pub fn is_development(&self) -> bool {
        *self == Environment::Development
    }
    
    pub fn is_production(&self) -> bool {
        *self == Environment::Production
    }
}
```

### Feature Flags Pattern

Use feature flags for conditional compilation:

```toml
# In Cargo.toml
[features]
default = ["tcp", "websocket"]
tcp = []
websocket = ["dep:tokio-tungstenite"]
stdio = []
all-transports = ["tcp", "websocket", "stdio"]
security-high = ["dep:ring"]
```

```rust
// In code
#[cfg(feature = "tcp")]
pub use self::tcp::TcpTransport;

#[cfg(feature = "websocket")]
pub use self::websocket::WebSocketTransport;

#[cfg(feature = "stdio")]
pub use self::stdio::StdioTransport;

#[cfg(feature = "security-high")]
pub type DefaultSecurityManager = HighSecurityManager;

#[cfg(not(feature = "security-high"))]
pub type DefaultSecurityManager = StandardSecurityManager;
```

## Example Integration Scenarios

### Integrating with a Service Backend

```rust
pub struct ServiceBackend {
    client: Client,
    config: ServiceConfig,
}

impl ServiceBackend {
    pub async fn new(config: ServiceConfig) -> Result<Self, Error> {
        // Create appropriate transport
        let transport = match config.transport_type {
            TransportType::Tcp => TcpTransport::new(config.tcp_config.clone()),
            TransportType::WebSocket => WebSocketTransport::new(config.ws_config.clone()),
            _ => return Err(Error::Configuration("Unsupported transport type")),
        };
        
        // Create client with transport
        let client = Client::new(transport);
        
        // Connect to the service
        client.connect().await?;
        
        Ok(Self { client, config })
    }
    
    pub async fn execute_command(&self, name: &str, params: serde_json::Value) -> Result<serde_json::Value, Error> {
        // Send command to service
        let response = self.client.send_command(name, params).await?;
        
        // Check response status
        if response.status != StatusCode::Ok {
            return Err(Error::Service(format!(
                "Command failed: {} - {}", 
                response.status,
                response.payload.get("error").and_then(|e| e.as_str()).unwrap_or("Unknown error")
            )));
        }
        
        // Return response payload
        Ok(response.payload)
    }
}
```

### Integrating with a Web Frontend

```typescript
// TypeScript client for MCP over WebSocket
class McpClient {
    private ws: WebSocket;
    private messageHandlers: Map<string, (message: any) => void>;
    private pendingRequests: Map<string, {
        resolve: (value: any) => void,
        reject: (error: Error) => void
    }>;
    
    constructor(serverUrl: string) {
        this.ws = new WebSocket(serverUrl);
        this.messageHandlers = new Map();
        this.pendingRequests = new Map();
        
        this.ws.onmessage = (event) => this.handleMessage(JSON.parse(event.data));
        this.ws.onerror = (error) => this.handleError(error);
    }
    
    public async connect(): Promise<void> {
        if (this.ws.readyState === WebSocket.CONNECTING) {
            return new Promise((resolve, reject) => {
                this.ws.onopen = () => resolve();
                this.ws.onerror = (error) => reject(error);
            });
        } else if (this.ws.readyState === WebSocket.OPEN) {
            return Promise.resolve();
        } else {
            throw new Error("WebSocket is closed or closing");
        }
    }
    
    public async sendCommand(command: string, params: any): Promise<any> {
        await this.connect();
        
        const messageId = crypto.randomUUID();
        const message = {
            id: messageId,
            type: "Command",
            payload: {
                command,
                params
            },
            metadata: null,
            security: {
                level: "Standard"
            },
            timestamp: new Date().toISOString(),
            version: "1.0.0",
            trace_id: null
        };
        
        return new Promise((resolve, reject) => {
            this.pendingRequests.set(messageId, { resolve, reject });
            this.ws.send(JSON.stringify(message));
        });
    }
    
    public addMessageHandler(type: string, handler: (message: any) => void): void {
        this.messageHandlers.set(type, handler);
    }
    
    private handleMessage(message: any): void {
        // Handle responses to pending requests
        if (message.trace_id && this.pendingRequests.has(message.trace_id)) {
            const { resolve, reject } = this.pendingRequests.get(message.trace_id)!;
            this.pendingRequests.delete(message.trace_id);
            
            if (message.type === "Error") {
                reject(new Error(message.payload.message));
            } else {
                resolve(message.payload);
            }
            
            return;
        }
        
        // Handle messages by type
        const handler = this.messageHandlers.get(message.type);
        if (handler) {
            handler(message);
        }
    }
    
    private handleError(error: Event): void {
        // Reject all pending requests
        for (const [id, { reject }] of this.pendingRequests.entries()) {
            reject(new Error("WebSocket error"));
            this.pendingRequests.delete(id);
        }
    }
}
```

---

*Patterns guide produced by DataScienceBioLab.* 