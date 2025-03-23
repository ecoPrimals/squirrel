# MCP Plugin System Specification

## Overview
The MCP plugin system enables extension of the Machine Context Protocol functionality. It is implemented and maintained by the MCP Team (src/mcp). The system now supports bidirectional integration between MCP tools and the unified plugin system.

## MCP Plugin Types

### Protocol Extensions
- **Purpose**: Extend MCP protocol capabilities
- **Team**: MCP Team (src/mcp)
- **Responsibilities**:
  - Protocol version management
  - Message type extensions
  - Protocol validation
  - Protocol security
  - Protocol compatibility

### Message Type Extensions
- **Purpose**: Add new message types
- **Team**: MCP Team (src/mcp)
- **Responsibilities**:
  - Message type definition
  - Message validation
  - Message serialization
  - Message deserialization
  - Message routing

### Security Protocol Extensions
- **Purpose**: Extend security features
- **Team**: MCP Team (src/mcp)
- **Responsibilities**:
  - Authentication extensions
  - Authorization extensions
  - Encryption extensions
  - Key management
  - Security monitoring

### Tool Protocol Extensions
- **Purpose**: Extend tool communication
- **Team**: MCP Team (src/mcp)
- **Responsibilities**:
  - Tool registration
  - Tool discovery
  - Tool communication
  - Tool state management
  - Tool error handling

### State Protocol Extensions
- **Purpose**: Extend state management
- **Team**: MCP Team (src/mcp)
- **Responsibilities**:
  - State synchronization
  - State persistence
  - State validation
  - State recovery
  - State monitoring

## Implementation Status

### Core Features - 75% Complete
- [x] Basic MCP plugin interface defined
- [x] Message handling interface implemented
- [x] Protocol extension points established
- [x] Message type system implemented
- [x] Bidirectional tools/plugins integration
- [x] Plugin discovery and registration
- [x] Lifecycle event management
- [ ] Protocol versioning system
- [ ] Advanced security features

### Tool-Plugin Bidirectional Integration - 100% Complete
- [x] Tool to Plugin adaptation (ToolPluginAdapter)
- [x] Plugin to Tool adaptation (PluginProxyExecutor)
- [x] Automatic tool-to-plugin registration
- [x] Automatic plugin-to-tool discovery
- [x] State synchronization
- [x] Lifecycle event propagation
- [x] Error handling and recovery

### Security Features - 40% Complete
- [x] Basic security interface
- [x] Authentication framework
- [x] Tool-level security integration
- [ ] Advanced authorization system
- [ ] Encryption framework
- [ ] Key management

### Tool Integration - 85% Complete
- [x] Tool interface fully defined
- [x] Tool registration and discovery
- [x] Plugin integration with tools
- [x] Tool state management
- [x] Lifecycle hooks for tools
- [x] Error handling
- [ ] Advanced tool capabilities

### State Management - 70% Complete
- [x] State interface fully implemented
- [x] State persistence
- [x] State synchronization
- [x] Lifecycle event management
- [ ] Advanced state recovery
- [ ] Comprehensive state monitoring

## Implementation Details

### Plugin Interface
```rust
#[async_trait]
pub trait McpPlugin: Plugin {
    /// Handle MCP message
    async fn handle_message(&self, message: Value) -> Result<Value>;
    
    /// Validate message schema
    fn validate_message_schema(&self, message: &Value) -> Result<()>;
}
```

### Tool-Plugin Adaptation

#### ToolPluginAdapter
```rust
pub struct ToolPluginAdapter {
    /// The plugin ID
    plugin_id: Uuid,
    
    /// The tool manager
    tool_manager: Arc<ToolManager>,
    
    /// The tool ID
    tool_id: String,
}

impl ToolPluginAdapter {
    /// Create a new tool plugin adapter
    pub fn new(tool_id: String, tool_manager: Arc<ToolManager>) -> Self;
}

#[async_trait]
impl Plugin for ToolPluginAdapter {
    fn metadata(&self) -> PluginMetadata;
    
    async fn initialize(&self) -> Result<()>;
    
    async fn shutdown(&self) -> Result<()>;
}

#[async_trait]
impl McpPlugin for ToolPluginAdapter {
    async fn handle_message(&self, message: Value) -> Result<Value>;
    
    fn validate_message_schema(&self, message: &Value) -> Result<()>;
}
```

#### PluginDiscoveryManager
```rust
pub struct PluginDiscoveryManager {
    /// The tool manager
    tool_manager: Arc<ToolManager>,
    
    /// The plugin manager
    plugin_manager: Arc<PluginManager>,
    
    /// The mapping of plugin IDs to tool IDs
    registered_plugins: RwLock<HashMap<Uuid, String>>,
}

impl PluginDiscoveryManager {
    /// Create a new plugin discovery manager
    pub fn new(tool_manager: Arc<ToolManager>, plugin_manager: Arc<PluginManager>) -> Self;
    
    /// Discover and register all MCP plugins as tools
    pub async fn discover_and_register_all_plugins(&self) -> Result<Vec<String>>;
    
    /// Register a specific plugin as a tool
    pub async fn register_plugin_as_tool(&self, plugin: Arc<dyn McpPlugin>) -> Result<String>;
    
    /// Check if a plugin is registered as a tool
    pub async fn is_plugin_registered(&self, plugin_id: Uuid) -> bool;
    
    /// Get the tool ID for a registered plugin
    pub async fn get_tool_id_for_plugin(&self, plugin_id: Uuid) -> Option<String>;
    
    /// Unregister a plugin from the tool system
    pub async fn unregister_plugin(&self, plugin_id: Uuid) -> Result<()>;
}
```

### Lifecycle Management

#### PluginLifecycleHook
```rust
pub struct PluginLifecycleHook {
    /// The plugin system integration
    integration: Arc<PluginSystemIntegration>,
    
    /// List of tool IDs to monitor
    monitored_tools: RwLock<Vec<String>>,
}

impl PluginLifecycleHook {
    /// Create a new plugin lifecycle hook
    pub fn new(integration: Arc<PluginSystemIntegration>) -> Self;
    
    /// Add a tool to be monitored for state changes
    pub async fn add_monitored_tool(&self, tool_id: String);
    
    /// Remove a tool from being monitored
    pub async fn remove_monitored_tool(&self, tool_id: &str);
    
    /// Check if a tool is being monitored
    pub async fn is_monitored(&self, tool_id: &str) -> bool;
}

#[async_trait]
impl LifecycleHook for PluginLifecycleHook {
    async fn on_event(&self, event: &LifecycleEvent) -> Result<(), MCPError>;
}
```

#### CompositePluginLifecycleHook
```rust
pub struct CompositePluginLifecycleHook<T: LifecycleHook> {
    /// The plugin lifecycle hook
    plugin_hook: Arc<PluginLifecycleHook>,
    
    /// The base lifecycle hook
    base_hook: Arc<T>,
}

impl<T: LifecycleHook> CompositePluginLifecycleHook<T> {
    /// Create a new composite lifecycle hook
    pub fn new(base_hook: Arc<T>, plugin_hook: Arc<PluginLifecycleHook>) -> Self;
}

#[async_trait]
impl<T: LifecycleHook + Send + Sync> LifecycleHook for CompositePluginLifecycleHook<T> {
    async fn on_event(&self, event: &LifecycleEvent) -> Result<(), MCPError>;
}
```

### Integration System

```rust
pub struct PluginSystemIntegration {
    /// The tool manager
    tool_manager: Arc<ToolManager>,
    
    /// The plugin manager
    plugin_manager: Arc<PluginManager>,
    
    /// The mapping of tool IDs to plugin IDs
    tool_plugins: RwLock<HashMap<String, Uuid>>,
}

impl PluginSystemIntegration {
    /// Create a new plugin system integration
    pub fn new(tool_manager: Arc<ToolManager>, plugin_manager: Arc<PluginManager>) -> Self;
    
    /// Register an active tool as a plugin
    pub async fn register_tool_as_plugin(&self, tool_id: &str) -> Result<Uuid>;
    
    /// Register all active tools as plugins
    pub async fn register_all_active_tools_as_plugins(&self) -> Result<Vec<Uuid>>;
    
    /// Handle a tool state change
    pub async fn handle_tool_state_change(&self, tool_id: &str, state: ToolState) -> Result<()>;
    
    /// Unregister a tool plugin
    pub async fn unregister_tool_plugin(&self, tool_id: &str) -> Result<()>;
}
```

## Security Model

### Plugin Security
- Tool security level inheritance for plugins
- Message authentication via HMAC
- Authorization via permission levels
- Protocol-level encryption (future)
- Key management (future)
- Security logging and monitoring

### Protocol Validation
- Message schema validation
- Protocol compatibility checking
- Protocol version validation
- Security validation
- Rate limiting

### Protocol Monitoring
- Performance metrics collection
- Security event monitoring
- Health checking
- Error tracking
- Usage analytics

## Performance Requirements

### Message Processing
- Message handling: < 10ms
- Message validation: < 5ms
- State operations: < 5ms
- Protocol operations: < 10ms
- Error handling: < 5ms

### Resource Usage
- Memory: < 100MB per protocol
- CPU: < 5% per protocol
- Network: < 1MB/s per protocol
- Storage: < 50MB per protocol

## Error Handling

### Error Types
```rust
pub enum ProtocolError {
    /// Protocol implementation error
    ProtocolError(String),
    /// Message processing error
    MessageError(String),
    /// State management error
    StateError(String),
    /// Security error
    SecurityError(String),
    /// Validation error
    ValidationError(String),
    /// Resource limit exceeded
    ResourceError(String),
    /// Plugin error
    PluginError(String),
}
```

### Recovery Strategies
- Circuit breaker for failed protocols
- Automatic retry with backoff
- Fallback to default handlers
- State recovery from snapshots
- Error reporting and logging
- Graceful degradation of service

## Testing Requirements

### Unit Tests
- Protocol interface tests
- Message handling tests
- State management tests
- Security tests
- Validation tests
- Tool-plugin adaptation tests
- Lifecycle hook tests

### Integration Tests
- Protocol manager tests
- Message routing tests
- State synchronization tests
- Security integration tests
- Performance tests
- Tool-to-plugin flow tests
- Plugin-to-tool flow tests
- Bidirectional integration tests
- Lifecycle event propagation tests

## Documentation Requirements

### API Documentation
- Protocol documentation
- Message documentation
- State documentation
- Security documentation
- Tool-plugin integration documentation
- Lifecycle management documentation

## Setup and Usage Examples

### Complete System Setup
```rust
// Set up the plugin system
let (tool_manager, plugin_manager, integration, discovery_manager) = 
    setup_plugin_system().await?;

// Register a sample tool
let sample_tool = Tool::builder()
    .id("sample-tool")
    .name("Sample Tool")
    .version("1.0.0")
    .description("A sample tool for testing")
    .build();

// Register the tool with a basic executor
let executor = BasicToolExecutor::new(
    "sample-tool", 
    vec!["sample".to_string()],
    |context| async move {
        Ok(serde_json::json!({
            "result": "Hello from sample tool!",
            "input": context.parameters
        }))
    }
);

tool_manager.register_tool(sample_tool, executor).await?;

// Register the tool as a plugin
let plugin_id = integration.register_tool_as_plugin("sample-tool").await?;

// Execute the tool as a plugin
let plugin_result = plugin_manager.execute_plugin::<McpPlugin>(plugin_id, |plugin| async move {
    plugin.handle_message(message).await
}).await?;

// Now go in the reverse direction
// Discover and register plugins as tools
let tool_ids = discovery_manager.discover_and_register_all_plugins().await?;
```

## Next Steps

1. **Enhanced Security**: Add additional security checks for plugin-to-tool conversions
2. **Configuration Options**: Allow configuring aspects of the integration 
3. **Monitoring and Metrics**: Add monitoring and metrics collection for the integration
4. **Plugin Versioning**: Add support for versioning and compatibility checking
5. **Extended Testing**: Add more comprehensive integration tests and performance tests

## Success Criteria

### Functional Requirements
- All protocol types functional
- Message handling robust
- Security model effective
- Performance requirements met
- Testing complete

### Non-Functional Requirements
- Response times met
- Resource limits respected
- Security requirements satisfied
- Documentation complete
- Community feedback positive 