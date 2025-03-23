---
title: Web Interface MCP Integration
version: 1.0.0
date: 2024-03-25
status: proposed
priority: high
---

# Web Interface MCP Integration

## Overview

This document outlines the comprehensive integration between the Squirrel Web Interface and the Machine Context Protocol (MCP). The integration enables the web interface to leverage MCP capabilities for command execution, context management, and tool lifecycle management.

## Goals

1. **Bidirectional Communication**: Enable seamless two-way communication between the web interface and MCP
2. **Protocol Translation**: Translate between HTTP/WebSocket and MCP message formats
3. **Context Preservation**: Maintain context across protocol boundaries
4. **Error Propagation**: Properly propagate and handle errors between systems
5. **Performance Optimization**: Ensure efficient communication with minimal overhead
6. **Security Integration**: Enforce consistent security models across protocols

## Current State

The current MCP integration is minimal, with only a basic mock implementation:

- Mock client exists for testing purposes
- Interface defined with proper methods
- No real connection to MCP protocol
- No bidirectional communication
- Limited error handling
- No context preservation

## Components

### 1. MCP Client

The MCP Client provides a robust interface for communicating with the MCP protocol.

#### Implementation Requirements
- **Connection Management**: Establish and maintain connections to MCP server
- **Message Serialization**: Convert between Rust types and MCP messages
- **Error Handling**: Properly handle and propagate MCP errors
- **Reconnection Logic**: Automatically reconnect on connection failures
- **Security Integration**: Properly handle authentication and authorization

```rust
pub struct McpClient {
    connection: Arc<McpConnection>,
    config: McpClientConfig,
    executor: Arc<dyn Executor>,
}

impl McpClient {
    pub async fn new(config: McpClientConfig) -> Result<Self, McpError> { ... }
    
    pub async fn connect(&self) -> Result<(), McpError> { ... }
    
    pub async fn disconnect(&self) -> Result<(), McpError> { ... }
    
    pub async fn send_message(&self, message: &McpMessage) -> Result<McpResponse, McpError> { ... }
    
    pub async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
        context: Option<&McpContext>,
    ) -> Result<String, McpError> { ... }
    
    pub async fn get_command_status(
        &self,
        command_id: &str,
    ) -> Result<CommandStatusResponse, McpError> { ... }
    
    pub async fn cancel_command(
        &self,
        command_id: &str,
    ) -> Result<(), McpError> { ... }
    
    pub async fn list_available_commands(
        &self,
    ) -> Result<Vec<CommandDefinition>, McpError> { ... }
    
    pub async fn subscribe_to_events(
        &self,
        event_types: Vec<String>,
        handler: Box<dyn Fn(McpEvent) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
    ) -> Result<McpSubscription, McpError> { ... }
}
```

#### Connection Pool
```rust
pub struct McpConnectionPool {
    connections: Vec<Arc<McpConnection>>,
    config: McpPoolConfig,
}

impl McpConnectionPool {
    pub fn new(config: McpPoolConfig) -> Self { ... }
    
    pub async fn get_connection(&self) -> Result<Arc<McpConnection>, McpError> { ... }
    
    pub async fn release_connection(&self, connection: Arc<McpConnection>) { ... }
    
    pub async fn refresh_connections(&self) -> Result<(), McpError> { ... }
}
```

### 2. Message Translation Layer

The Message Translation Layer converts between HTTP/WebSocket formats and MCP protocol.

#### Implementation Requirements
- **Request Mapping**: Map HTTP requests to MCP messages
- **Response Mapping**: Map MCP responses to HTTP responses
- **WebSocket Bridge**: Bridge WebSocket and MCP events
- **Format Conversion**: Convert between JSON and MCP message format
- **Metadata Preservation**: Maintain metadata across protocol boundaries

```rust
pub struct MessageTranslator {
    config: TranslatorConfig,
}

impl MessageTranslator {
    pub fn new(config: TranslatorConfig) -> Self { ... }
    
    pub fn http_request_to_mcp(
        &self,
        request: &HttpRequest,
    ) -> Result<McpMessage, TranslationError> { ... }
    
    pub fn mcp_response_to_http(
        &self,
        response: &McpResponse,
    ) -> Result<HttpResponse, TranslationError> { ... }
    
    pub fn websocket_message_to_mcp(
        &self,
        message: &WebSocketMessage,
    ) -> Result<McpMessage, TranslationError> { ... }
    
    pub fn mcp_event_to_websocket(
        &self,
        event: &McpEvent,
    ) -> Result<WebSocketMessage, TranslationError> { ... }
}
```

### 3. Context Management

The Context Management system preserves context across protocol boundaries.

#### Implementation Requirements
- **Context Capture**: Capture relevant context from HTTP requests
- **Context Propagation**: Propagate context to MCP protocol
- **Context Enrichment**: Enrich context with additional metadata
- **Context Storage**: Store context for long-running operations
- **Context Retrieval**: Retrieve context for asynchronous responses

```rust
pub struct ContextManager {
    storage: Arc<dyn ContextStorage>,
}

impl ContextManager {
    pub fn new(storage: Arc<dyn ContextStorage>) -> Self { ... }
    
    pub async fn create_context(
        &self,
        request: &HttpRequest,
        user_id: &str,
    ) -> Result<McpContext, ContextError> { ... }
    
    pub async fn store_context(
        &self,
        context_id: &str,
        context: &McpContext,
    ) -> Result<(), ContextError> { ... }
    
    pub async fn retrieve_context(
        &self,
        context_id: &str,
    ) -> Result<McpContext, ContextError> { ... }
    
    pub async fn update_context(
        &self,
        context_id: &str,
        updates: &ContextUpdates,
    ) -> Result<McpContext, ContextError> { ... }
    
    pub async fn delete_context(
        &self,
        context_id: &str,
    ) -> Result<(), ContextError> { ... }
}
```

### 4. Event Bridging

The Event Bridging system connects MCP events to WebSocket events.

#### Implementation Requirements
- **Event Subscription**: Subscribe to relevant MCP events
- **Event Transformation**: Transform MCP events to WebSocket messages
- **Client Management**: Track WebSocket clients and their subscriptions
- **Event Filtering**: Filter events based on client subscriptions
- **Delivery Confirmation**: Confirm event delivery when required

```rust
pub struct EventBridge {
    mcp_client: Arc<dyn McpCommandClient>,
    websocket_manager: Arc<WebSocketConnectionManager>,
    translator: Arc<MessageTranslator>,
}

impl EventBridge {
    pub fn new(
        mcp_client: Arc<dyn McpCommandClient>,
        websocket_manager: Arc<WebSocketConnectionManager>,
        translator: Arc<MessageTranslator>,
    ) -> Self { ... }
    
    pub async fn start(&self) -> Result<(), BridgeError> { ... }
    
    pub async fn stop(&self) -> Result<(), BridgeError> { ... }
    
    async fn handle_mcp_event(&self, event: McpEvent) { ... }
    
    async fn handle_websocket_message(&self, client_id: &str, message: WebSocketMessage) { ... }
    
    async fn map_subscriptions(&self, client_id: &str, mcp_subscriptions: &[String]) -> Result<(), BridgeError> { ... }
}
```

### 5. Error Handling

The Error Handling system properly propagates and transforms errors between systems.

#### Implementation Requirements
- **Error Mapping**: Map MCP errors to HTTP/WebSocket errors
- **Error Details**: Preserve error details across protocols
- **Error Recovery**: Implement recovery strategies for common errors
- **Error Reporting**: Report errors through appropriate channels
- **Error Documentation**: Document error codes and recovery steps

```rust
pub enum McpError {
    ConnectionError(String),
    AuthenticationError(String),
    CommandError(String),
    ContextError(String),
    MessageError(String),
    TimeoutError(String),
    InternalError(String),
}

impl From<McpError> for ApiError {
    fn from(error: McpError) -> Self {
        match error {
            McpError::ConnectionError(msg) => ApiError::ServiceUnavailable(msg),
            McpError::AuthenticationError(msg) => ApiError::Unauthorized(msg),
            McpError::CommandError(msg) => ApiError::BadRequest(msg),
            McpError::ContextError(msg) => ApiError::BadRequest(msg),
            McpError::MessageError(msg) => ApiError::BadRequest(msg),
            McpError::TimeoutError(msg) => ApiError::RequestTimeout(msg),
            McpError::InternalError(msg) => ApiError::InternalServerError(msg),
        }
    }
}
```

## Security Integration

### Authentication and Authorization

The integration must enforce consistent security controls across protocols.

#### Implementation Requirements
- **Token Translation**: Translate JWT tokens to MCP authentication
- **Role Mapping**: Map web roles to MCP permissions
- **Scope Validation**: Validate operation scopes against user permissions
- **Audit Logging**: Log security-relevant events across protocol boundaries
- **Request Attribution**: Ensure all MCP requests are properly attributed

```rust
pub struct SecurityBridge {
    token_validator: Arc<dyn TokenValidator>,
    role_mapper: Arc<dyn RoleMapper>,
    audit_logger: Arc<dyn AuditLogger>,
}

impl SecurityBridge {
    pub fn new(
        token_validator: Arc<dyn TokenValidator>,
        role_mapper: Arc<dyn RoleMapper>,
        audit_logger: Arc<dyn AuditLogger>,
    ) -> Self { ... }
    
    pub async fn authenticate_mcp_request(
        &self,
        http_request: &HttpRequest,
        mcp_message: &mut McpMessage,
    ) -> Result<(), SecurityError> { ... }
    
    pub async fn authorize_command(
        &self,
        user: &AuthClaims,
        command: &str,
    ) -> Result<(), SecurityError> { ... }
    
    pub async fn create_mcp_security_context(
        &self,
        user: &AuthClaims,
    ) -> Result<McpSecurityContext, SecurityError> { ... }
    
    pub async fn log_operation(
        &self,
        user: &AuthClaims,
        operation: &str,
        details: Option<&serde_json::Value>,
    ) -> Result<(), SecurityError> { ... }
}
```

## Integration Patterns

### 1. Command Execution Flow

The Command Execution Flow handles the full lifecycle of command execution.

#### Flow Diagram
```
┌──────────┐     ┌─────────────┐     ┌───────────┐     ┌─────────────┐
│  Client  │─────►  Web API    │─────►  MCP      │─────►  Command    │
│          │     │  Controller │     │  Client   │     │  Executor   │
└──────────┘     └─────────────┘     └───────────┘     └─────────────┘
      │                │                  │                  │
      │   HTTP POST    │                  │                  │
      │───────────────►│                  │                  │
      │                │  Create Context  │                  │
      │                │──────────────────┘                  │
      │                │                                     │
      │                │       Execute Command               │
      │                │────────────────────────────────────►│
      │                │                                     │
      │                │      Command Accepted               │
      │                │◄────────────────────────────────────┘
      │   201 Created  │                                     │
      │◄───────────────│                                     │
      │                │                                     │
      │                │                                     │
      │   WebSocket    │       Status Updates                │
      │◄ ─ ─ ─ ─ ─ ─ ─ ┼ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┘
      │                │                                     │
```

#### Implementation Requirements
1. Client sends HTTP POST to `/api/commands` with command and parameters
2. Web API controller validates request and user permissions
3. MCP Client creates execution context from HTTP request
4. MCP Client sends command execution request to MCP
5. Command ID is returned to client
6. Client receives status updates via WebSocket
7. Client can query command status via `/api/commands/:id`

### 2. Event Subscription Flow

The Event Subscription Flow enables real-time event notifications.

#### Flow Diagram
```
┌──────────┐     ┌─────────────┐     ┌───────────┐     ┌─────────────┐
│  Client  │─────►  WebSocket  │─────►  Event    │─────►  MCP        │
│          │     │  Server     │     │  Bridge   │     │  Client     │
└──────────┘     └─────────────┘     └───────────┘     └─────────────┘
      │                │                  │                  │
      │   Connect      │                  │                  │
      │───────────────►│                  │                  │
      │                │───────────────────►                  │
      │                │                  │  Subscribe        │
      │                │                  │─────────────────►│
      │                │                  │                  │
      │  Subscribe Cmd │                  │                  │
      │───────────────►│                  │                  │
      │                │───────────────────►                  │
      │                │                  │                  │
      │                │                  │    MCP Event     │
      │                │                  │◄─────────────────┘
      │                │  WebSocket Event │                  │
      │                │◄───────────────────                  │
      │  Event Message │                  │                  │
      │◄───────────────│                  │                  │
      │                │                  │                  │
```

#### Implementation Requirements
1. Client establishes WebSocket connection
2. Client sends subscription message for specific event types
3. Event Bridge maps the subscription to MCP event types
4. Event Bridge subscribes to MCP events
5. When MCP events occur, they are translated to WebSocket messages
6. WebSocket server sends events to subscribed clients

## Command Mapping

The integration will support the following command mappings between HTTP API and MCP:

| API Endpoint | HTTP Method | MCP Command | Description |
|--------------|-------------|-------------|-------------|
| `/api/commands` | POST | `execute_command` | Execute a command |
| `/api/commands/:id` | GET | `get_command_status` | Get command status |
| `/api/commands/:id/cancel` | POST | `cancel_command` | Cancel a command |
| `/api/commands/available` | GET | `list_available_commands` | List available commands |

## Event Mapping

The integration will map the following MCP events to WebSocket events:

| MCP Event | WebSocket Event | Channel | Description |
|-----------|----------------|---------|-------------|
| `command_status_update` | `command-status` | `command:{id}` | Command status update |
| `command_progress` | `command-progress` | `command:{id}` | Command progress update |
| `command_result` | `command-result` | `command:{id}` | Command result |
| `command_error` | `command-error` | `command:{id}` | Command error |
| `command_list_update` | `command-list-update` | `command:list` | Available command list update |

## Error Codes

The integration will standardize error codes across protocols:

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `MCP_CONNECTION_ERROR` | 503 | Unable to connect to MCP server |
| `MCP_AUTHENTICATION_ERROR` | 401 | Authentication failed with MCP |
| `MCP_COMMAND_NOT_FOUND` | 404 | Command not found |
| `MCP_COMMAND_FAILED` | 400 | Command execution failed |
| `MCP_COMMAND_TIMEOUT` | 408 | Command execution timed out |
| `MCP_CONTEXT_ERROR` | 400 | Context error |
| `MCP_AUTHORIZATION_ERROR` | 403 | User not authorized for command |
| `MCP_INTERNAL_ERROR` | 500 | Internal MCP error |

## Implementation Plan

### Phase 1: Core Integration (2 weeks)
1. Implement real MCP client
2. Create message translation layer
3. Set up basic context management
4. Implement error mapping
5. Create security integration

### Phase 2: Event System (2 weeks)
1. Implement event bridge
2. Create WebSocket event mapping
3. Add subscription management
4. Implement event filtering
5. Add event delivery confirmation

### Phase 3: Context and Security (1 week)
1. Enhance context management
2. Add role-based security
3. Implement audit logging
4. Create context storage
5. Add context synchronization

### Phase 4: Testing and Optimization (1 week)
1. Create integration tests
2. Add performance benchmarks
3. Optimize message handling
4. Implement connection pooling
5. Add monitoring and metrics

## Dependencies
- MCP Protocol Crate (`crates/mcp`)
- WebSocket Manager (`crates/web/src/websocket`)
- Authentication System (`crates/web/src/auth`)
- Command API (`crates/web/src/handlers/commands`)

## Success Criteria
1. All command API endpoints fully functional with MCP integration
2. Real-time event updates via WebSocket
3. Proper error propagation and recovery
4. Context preservation across protocol boundaries
5. Consistent security enforcement
6. Performance within specified targets (response time < 200ms)

## Conclusion

The enhanced MCP integration will provide a robust, bidirectional communication channel between the Squirrel Web Interface and the MCP protocol. By implementing proper message translation, context preservation, and event bridging, the web interface will be able to fully leverage MCP capabilities while maintaining a clean separation of concerns. 