# MCP Plugin System Architecture

## Overview

The MCP Plugin System Integration provides bidirectional interoperability between the MCP Tool System and the Unified Plugin System. This allows:

1. MCP tools to be exposed as plugins
2. Plugins to be used as MCP tools

## Architecture Components

### Core Components

#### 1. Adapter Module (`adapter.rs`)

The adapter module provides a bridge from MCP tools to plugins:

- **ToolPluginAdapter**: Adapts an existing MCP tool to implement the Plugin and McpPlugin traits
- **ToolPluginFactory**: Creates plugin adapters for tools on demand

#### 2. Integration Module (`integration.rs`)

The integration module provides overall integration between the two systems:

- **PluginSystemIntegration**: Main integration class for bidirectional interaction
- **PluginToolExecutor**: Allows executing plugins through the MCP tool interface

#### 3. Lifecycle Module (`lifecycle.rs`)

The lifecycle module synchronizes state between tools and plugins:

- **PluginLifecycleHook**: Responds to tool lifecycle events and propagates them to the plugin system
- **CompositePluginLifecycleHook**: Combines multiple lifecycle hooks for comprehensive event handling

#### 4. Discovery Module (`discovery.rs`)

The discovery module enables discovery and registration of plugins as tools:

- **PluginProxyExecutor**: Implements the ToolExecutor interface for plugins
- **PluginDiscoveryManager**: Discovers and registers plugins as tools

### Supporting Components

#### 5. Examples Module (`examples.rs`)

Provides examples for using the plugin system integration:

- Setup examples
- Registration examples
- Execution examples
- Complete workflow examples

#### 6. Tests Module (`tests/mod.rs`)

Contains integration tests for the plugin system:

- Tool-to-plugin flow tests
- Plugin-to-tool flow tests
- Bidirectional integration tests
- Lifecycle event propagation tests

## Integration Flows

### Tool-to-Plugin Flow

1. An MCP Tool is registered with the ToolManager
2. The PluginSystemIntegration creates a ToolPluginAdapter for the tool
3. The adapter is registered with the PluginManager
4. The plugin can be executed through the PluginManager interface

### Plugin-to-Tool Flow

1. A Plugin is registered with the PluginManager
2. The PluginDiscoveryManager creates a PluginProxyExecutor for the plugin
3. A Tool is created and registered with the ToolManager
4. The tool can be executed through the ToolManager interface

## State Synchronization

- Tool state changes (active, paused, etc.) are propagated to plugins via the PluginLifecycleHook
- Plugin state changes (during initialization or execution) are propagated through the PluginProxyExecutor

## Message Format

- Tools and plugins communicate using a standardized JSON message format
- Messages include:
  - capability: The functionality being requested
  - parameters: Input parameters for the capability
  - request_id: A unique identifier for the request

## Example Usage

```rust
// Set up the necessary components
let tool_manager = Arc::new(ToolManager::builder().build());
let plugin_manager = Arc::new(PluginManager::new());

// Create the integration components
let integration = PluginSystemIntegration::new(
    tool_manager.clone(),
    plugin_manager.clone()
);

let discovery_manager = PluginDiscoveryManager::new(
    tool_manager.clone(),
    plugin_manager.clone()
);

// Register a tool as a plugin
let plugin_id = integration.register_tool_as_plugin("my-tool-id").await?;

// Register a plugin as a tool
let tool_id = discovery_manager.discover_and_register_all_plugins().await?;

// Execute a tool as a plugin
let result = plugin_manager.execute_plugin::<McpPlugin>(plugin_id, |plugin| async move {
    plugin.handle_message(message).await
}).await?;

// Execute a plugin as a tool
let result = tool_manager.execute_tool(
    &tool_id,
    "capability",
    parameters,
    None
).await?;
```

## Error Handling

- All components use anyhow::Result for consistent error handling
- Errors are propagated appropriately between systems
- Detailed error messages are provided for troubleshooting

## Testing Strategy

- Unit tests for individual components
- Integration tests for end-to-end flows
- Examples that demonstrate complete usage scenarios 