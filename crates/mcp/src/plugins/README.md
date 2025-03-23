# MCP Plugin System Integration

This module provides a bidirectional bridge between the MCP tool system and the unified plugin system in `squirrel_plugins`. It enables interoperability by allowing:

1. MCP tools to function as plugins
2. Unified plugins to be used as MCP tools

## Architecture Overview

The integration consists of several key components:

### 1. Tool to Plugin Adaptation

- **ToolPluginAdapter**: Adapts an existing MCP tool to the plugin interface
- **ToolPluginFactory**: Creates plugin adapters for tools

### 2. Plugin to Tool Adaptation

- **PluginProxyExecutor**: Implements the tool executor interface for plugins
- **PluginDiscoveryManager**: Discovers and registers plugins as tools

### 3. State Synchronization

- **PluginLifecycleHook**: Synchronizes tool lifecycle events with plugin state
- **CompositePluginLifecycleHook**: Combines multiple hooks for comprehensive event handling

### 4. Integration Management

- **PluginSystemIntegration**: Manages the overall integration between tools and plugins
- **PluginToolExecutor**: Allows executing plugins through the MCP tool interface

## Usage Examples

See the `examples.rs` module for complete usage examples. Here's a quick overview:

### Setting Up the Plugin System

```rust
let (tool_manager, plugin_manager, integration, discovery_manager) = 
    setup_plugin_system().await?;
```

### Registering a Tool as a Plugin

```rust
let plugin_id = integration.register_tool_as_plugin("my-tool-id").await?;
```

### Registering a Plugin as a Tool

```rust
let tool_id = discovery_manager.register_plugin_as_tool(plugin).await?;
```

### Execute a Tool as a Plugin

```rust
let result = plugin_manager.execute_plugin::<McpPluginType>(plugin_id, |plugin| async move {
    plugin.handle_message(message).await
}).await?;
```

### Execute a Plugin as a Tool

```rust
let result = tool_manager.execute_tool(
    tool_id,
    capability,
    parameters,
    None
).await?;
```

## Testing

The integration includes unit tests for each component, as well as integration tests demonstrating the end-to-end functionality.

## Integration with MCP System

The plugin system integration is exposed through the MCP prelude, making it easy to use in applications:

```rust
use mcp::prelude::{
    ToolPluginAdapter, 
    ToolPluginFactory,
    PluginSystemIntegration, 
    PluginToolExecutor,
    PluginLifecycleHook,
    PluginDiscoveryManager
};
``` 