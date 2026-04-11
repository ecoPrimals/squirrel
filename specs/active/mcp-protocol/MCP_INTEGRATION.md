# Machine Context Protocol (MCP) Integration

> **Note (April 2026):** WebSocket transport was removed from Squirrel in v0.1.0-alpha.47 (Tower Atomic pattern — WebSocket provided by Songbird service mesh). WebSocket references below are historical.

## Overview

The Machine Context Protocol (MCP) is a core component of the Squirrel platform, allowing communication between various components and services. The Squirrel CLI integrates with MCP to provide command-line access to MCP functionality.

## Current Status

### Components

| Component | Status | Notes |
|-----------|--------|-------|
| MCP Command | ✅ Complete | Full command implementation with server, client, publish, and subscribe functionality |
| MCP Server | ✅ Complete | Full implementation with request/response handling, subscriptions, topic-based notifications, and command registry integration |
| MCP Client | ✅ Complete | Client implementation with interactive mode, command execution, and subscription management |
| Protocol | ✅ Complete | Message structures, serialization, pub/sub support, and error handling |
| Command Registry | ✅ Complete | Integration with the CLI command registry for remote command execution |

### Implementation Details

#### MCP Command

The `mcp` command provides the main interface for interacting with the MCP protocol. It includes:

- ✅ Server subcommand for starting a local MCP server
- ✅ Client subcommand for connecting to an MCP server
- ✅ Subscribe subcommand for subscribing to topics
- ✅ Publish subcommand for publishing notifications
- ✅ Status subcommand for checking server status
- ✅ Protocol subcommand for viewing protocol documentation

#### MCP Server

The server component handles incoming MCP connections. It includes:

- ✅ TCP-based server implementation
- ✅ Command request processing
- ✅ Response handling
- ✅ Support for custom command handlers
- ✅ Topic-based subscription system
- ✅ Notification routing to subscribers
- ✅ Multi-client support
- ✅ Graceful shutdown
- ✅ Subscription cleanup on client disconnect
- ✅ Command registry integration for remote execution

#### MCP Client

The client component connects to MCP servers. It includes:

- ✅ TCP-based client implementation
- ✅ Command execution
- ✅ Response handling
- ✅ Topic subscription with callbacks
- ✅ Notification publishing
- ✅ Interactive mode with subscribe/publish commands
- ✅ Connection management
- ✅ Error handling
- ✅ Async notification handling
- ✅ Remote command execution support

#### Protocol

The protocol defines the message format and serialization. It includes:

- ✅ Message structure definition
- ✅ JSON serialization
- ✅ Message types (request, response, notification, error)
- ✅ Topic-based subscription model
- ✅ Error handling
- ✅ Unit tests for message parsing
- ✅ Command argument serialization

## Implementation Plan

### Phase 1: Core Protocol (Completed)

- ✅ Define message structures
- ✅ Implement serialization/deserialization
- ✅ Create basic server implementation
- ✅ Create basic client implementation
- ✅ Add command structure

### Phase 2: Command Integration (Completed)

- ✅ Implement MCP command with subcommands
- ✅ Add server subcommand functionality
- ✅ Add client subcommand functionality
- ✅ Implement proper error handling
- ✅ Add interactive mode for client
- ✅ Add subscribe and publish subcommands
- ✅ Integrate with command registry for server-side execution

### Phase 3: Advanced Features (In Progress)

- ✅ Add subscription support for notifications
- ✅ Implement topic-based routing
- ✅ Add subscription management in clients
- ✅ Implement notification callbacks
- ✅ Add command registry integration
- ❌ Implement authentication and encryption
- ❌ Add session management
- ❌ Implement server persistence
- ❌ Create admin interface for server management

### Phase 4: Ecosystem Integration (Planned)

- ❌ Integrate with plugin system
- ❌ Add MCP service discovery
- ❌ Create SDKs for various languages
- 🔄 Implement comprehensive testing
- 🔄 Add detailed documentation

## Next Steps

1. Enhance security:
   - Implement authentication mechanisms for MCP connections
   - Add TLS support for encrypted communication
   - Create access control for remote command execution

2. Expand command registry integration:
   - Add persistent server instance management
   - Implement command history tracking via MCP
   - Add more detailed command execution responses
   - Create cross-instance command coordination

3. Add authentication and security:
   - Implement authentication mechanisms
   - Add TLS support for encrypted communication
   - Create access control for commands

4. Testing and documentation:
   - Expand unit test coverage
   - Create integration tests for client-server communication
   - Document protocol specification in detail
   - Add examples and tutorials

## Requirements

### MCP Command

The CLI provides an `mcp` command with the following subcommands:

- `server`: Start an MCP server
  - Options:
    - `--host`: Server host (default: 127.0.0.1)
    - `--port`: Server port (default: 8778)
    - `--action`: Server action (start, stop, status)
  
- `client`: Connect to an MCP server
  - Options:
    - `--host`: Server host (default: 127.0.0.1)
    - `--port`: Server port (default: 8778)
    - `--timeout`: Connection timeout (default: 5s)
    - `--interactive`: Start interactive mode
    - `command`: Command to execute
    - `args`: Command arguments

- `subscribe`: Subscribe to a topic on an MCP server
  - Options:
    - `--host`: Server host (default: 127.0.0.1)
    - `--port`: Server port (default: 8778)
    - `--timeout`: Connection timeout (default: 5s)
    - `--wait`: Wait for notifications
    - `--count`: Number of notifications to wait for
    - `topic`: Topic to subscribe to

- `publish`: Publish a notification to a topic
  - Options:
    - `--host`: Server host (default: 127.0.0.1)
    - `--port`: Server port (default: 8778)
    - `--timeout`: Connection timeout (default: 5s)
    - `topic`: Topic to publish to
    - `payload`: JSON payload to send

- `status`: Check MCP server status

- `protocol`: Show MCP protocol information

### Plugin Integration

Plugins should be able to extend MCP functionality by:

1. Registering custom MCP message handlers
2. Adding custom MCP protocol extensions
3. Providing custom MCP client/server implementations

## Integration with Core CLI

The MCP functionality is integrated with the core CLI through:

1. **Command Registry**: The MCP command is registered with the command registry
2. **Plugin System**: Plugins will be able to extend MCP functionality
3. **Formatter System**: MCP output uses the CLI's formatter system for consistent output

## Technical Specifications

### MCP Server

The MCP server:
- Uses TCP for communication
- Supports JSON message format
- Implements topic-based subscription
- Provides notification routing
- Handles client disconnects gracefully
- Supports custom command handlers
- Integrates with the command registry
- Enables remote command execution

### MCP Client

The MCP client:
- Supports multiple connection profiles
- Provides both interactive and non-interactive modes
- Implements subscription management
- Supports notification callbacks
- Implements retry and timeout logic
- Handles notifications asynchronously
- Executes commands remotely

### Protocol Management

The protocol includes:
- JSON message format
- Topic-based subscription model
- Request/response pattern
- Notification support
- Error handling
- Extensible message structure
- Command argument serialization

## Subscription System

The MCP implementation includes a complete subscription system:

1. **Topic-Based Model**: Clients can subscribe to specific topics
2. **Callback Mechanism**: Notifications trigger registered callbacks
3. **Subscription Management**: Clients can subscribe and unsubscribe from topics
4. **Notification Routing**: Server routes notifications to appropriate subscribers
5. **Cleanup on Disconnect**: Subscriptions are cleaned up when clients disconnect
6. **Interactive Support**: Interactive mode supports subscription commands

The subscription system enables publish/subscribe patterns for real-time data and event notifications between components, providing a foundation for event-driven architecture in the Squirrel platform.

## Command Registry Integration

The MCP implementation now includes complete command registry integration:

1. **Remote Execution**: Commands registered in the CLI can be executed remotely
2. **Argument Passing**: Command arguments can be passed via MCP messages
3. **Output Capture**: Command output is captured and returned in responses
4. **Error Handling**: Errors during command execution are properly reported
5. **Registry Sharing**: The server can share its command registry with clients
6. **Testing Support**: Unit tests verify the command registry integration

This integration enables remote control of CLI functionality, allowing components to execute commands on the server from client applications, creating a foundation for distributed command execution in the Squirrel platform.

## Future Considerations

### Security Enhancements

For production use, the MCP implementation should be enhanced with strong security features:

1. **Authentication**:
   - Implement user/client authentication mechanisms
   - Support multiple authentication methods (API keys, tokens, certificates)
   - Create a permission model for command execution
   - Add secure credential storage and management

2. **Encryption and TLS**:
   - Implement TLS for all MCP communications
   - Support certificate validation and management
   - Enable secure key exchange
   - Add support for encrypted payloads

3. **Access Control**:
   - Implement fine-grained access control for commands
   - Create role-based permissions for notification topics
   - Develop audit logging for security events
   - Implement rate limiting to prevent abuse

### Performance Optimizations

For high-throughput scenarios, performance optimizations should be considered:

1. **Connection Pooling**:
   - Implement connection pooling for clients
   - Add persistent connection management
   - Optimize reconnection strategies

2. **Message Batching**:
   - Support batch processing of commands
   - Implement efficient message queuing
   - Add compression for large payloads

3. **Scalability**:
   - Design for horizontal scaling with multiple server instances
   - Add load balancing capabilities
   - Implement distributed topic routing

### Advanced Features

Additional features to consider for future versions:

1. **Stateful Sessions**:
   - Implement persistent client sessions
   - Add support for session recovery
   - Create context-aware command execution

2. **Extended Subscription Patterns**:
   - Add wildcard support for topic subscriptions
   - Implement hierarchical topic structures
   - Support message filtering based on content
   - Add Quality of Service (QoS) levels

3. **Cross-Platform Support**:
   - Create language-specific client libraries
   - Ensure compatibility with different operating systems
   - Support resource-constrained environments

### Integration with External Systems

Extending MCP's reach to other technologies:

1. **Protocol Bridges**:
   - Create bridges to other protocols (MQTT, WebSockets, etc.)
   - Support protocol translation and message mapping
   - Enable multi-protocol server capabilities

2. **Cloud Integration**:
   - Add support for cloud-based message brokers
   - Implement event integration with cloud platforms
   - Create cloud-native deployment options

3. **Monitoring and Observability**:
   - Implement comprehensive metrics collection
   - Add distributed tracing support
   - Create dashboards for MCP server monitoring
   - Support anomaly detection and alerting

### Development Experience

Enhancing the developer experience:

1. **Testing Tools**:
   - Create MCP-specific testing framework
   - Implement mock servers for testing client code
   - Add conformance test suites for protocol implementation

2. **Documentation**:
   - Provide comprehensive API documentation
   - Create tutorials and examples for common use cases
   - Document protocol details for third-party implementations

3. **Debugging Support**:
   - Add detailed logging and tracing options
   - Create visualization tools for message flows
   - Implement interactive debugging capabilities 