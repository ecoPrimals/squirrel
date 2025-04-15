# Squirrel Integration

This crate provides integration components between various subsystems of the Squirrel platform, leveraging the adapter pattern for loose coupling.

## Main Components

### Context-MCP Integration

The Context-MCP integration provides bidirectional synchronization between the Squirrel context system and the MCP (Machine Context Protocol) system.

```rust
use squirrel_integration::{
    create_context_mcp_adapter,
    ContextMcpAdapterConfig
};

// Create adapter with default config
let adapter = create_context_mcp_adapter().await?;

// Or with custom config
let config = ContextMcpAdapterConfig::default();
let adapter = create_context_mcp_adapter_with_config(config).await?;
```

### AI Agent Integration

The AI Agent integration enables AI-powered capabilities within the Squirrel platform, connecting to various LLM providers.

```rust
use squirrel_integration::{
    create_ai_agent_adapter,
    AIAgentConfig
};

// Create with custom config
let config = AIAgentConfig::new("openai", "api-key-here")
    .with_model("gpt-4o")
    .with_timeout(30000);
    
let ai_adapter = create_ai_agent_adapter_with_config(config);
```

### MCP-AI Tools Integration

The MCP-AI Tools integration provides AI tools and capabilities to the MCP system, enabling advanced AI-powered features.

```rust
use squirrel_integration::{
    create_mcp_adapter,
    create_mcp_ai_tools_adapter_with_config,
    McpAiToolsConfig
};

// Create MCP adapter
let mcp_adapter = create_mcp_adapter()?;

// Create AI tools adapter
let config = McpAiToolsConfig::new("openai")
    .with_timeout(15000);
    
let ai_tools_adapter = create_mcp_ai_tools_adapter_with_config(
    mcp_adapter.clone(),
    config
)?;
```

### Context-AI Tools Integration

The Context-AI Tools integration adds AI-powered enhancement capabilities to the Context-MCP adapter, enabling advanced context analysis and insights.

```rust
use squirrel_integration::{
    create_context_mcp_adapter,
    ContextEnhancementType,
    ContextAiEnhancementOptions
};

// Create adapter
let adapter = create_context_mcp_adapter().await?;

// Define enhancement options
let options = ContextAiEnhancementOptions::new(
    ContextEnhancementType::Insights,
    "openai",
    "api-key-here"
)
.with_model("gpt-4o")
.with_timeout(15000);

// Apply enhancements to a context
adapter.apply_ai_enhancements("context-id", options).await?;
```

#### Unified Interface for AI Context Enhancement

For easier use, the Context-MCP adapter also provides a unified interface with convenience methods:

```rust
use squirrel_integration::{
    create_context_mcp_adapter,
    ContextEnhancementType,
};

// Create adapter
let adapter = create_context_mcp_adapter().await?;

// Convenience methods for common enhancement types
adapter.enhance_with_insights("context-id", "openai", "api-key", Some("gpt-4o")).await?;
adapter.enhance_with_summary("context-id", "openai", "api-key", Some("gpt-4o")).await?;
adapter.enhance_with_recommendations("context-id", "openai", "api-key", Some("gpt-4o")).await?;

// Full unified interface with custom parameters
let mut params = serde_json::Map::new();
params.insert("temperature".to_string(), serde_json::json!(0.8));

adapter.enhance_context(
    "context-id",
    ContextEnhancementType::TrendAnalysis,
    "openai",
    "api-key",
    Some("gpt-4o".to_string()),
    Some(25000), // timeout
    Some("Custom system prompt".to_string()),
    Some(params),
).await?;
```

## Available Enhancement Types

The following AI enhancement types are supported:

- `ContextEnhancementType::Summarize` - Create a concise summary of the context data
- `ContextEnhancementType::Insights` - Extract key insights from the context data
- `ContextEnhancementType::TrendAnalysis` - Analyze trends in the context data
- `ContextEnhancementType::Recommendations` - Generate recommendations based on the context
- `ContextEnhancementType::AnomalyDetection` - Detect anomalies in the context data
- `ContextEnhancementType::Custom(String)` - Custom enhancement with specific instructions

## Examples

See the `examples/` directory for practical demonstrations of how to use these integration components:

- `context_mcp_use_case.rs` - Basic Context-MCP integration
- `ai_agent_example.rs` - AI Agent integration example
- `mcp_ai_tools_example.rs` - MCP-AI Tools integration example
- `context_ai_tools.rs` - Context-AI Tools integration using the options API
- `ai_context_enhancement.rs` - Context-AI Tools integration using the unified interface

To run the examples:

```bash
export OPENAI_API_KEY=your-api-key-here
cargo run --example ai_context_enhancement
```

## Environment Variables

For examples and tests that use AI capabilities, you'll need to set appropriate API keys:

```bash
export OPENAI_API_KEY=your-api-key-here
```

## Available Integrations

### Plugin-Core Integration

The Plugin-Core integration bridges the Plugin system with Core components, enabling seamless interaction while maintaining loose coupling.

#### Key Features

- **Adapter Pattern**: Clean separation between components 
- **Lifecycle Management**: Control plugin lifecycle through core
- **Configuration Options**: Flexible configuration for plugin loading and security
- **Error Handling**: Clean error boundaries

#### Usage Example

```rust
use squirrel_integration::plugin_core::{PluginCoreAdapter, PluginCoreConfig};

async fn example() -> anyhow::Result<()> {
    // Create a custom configuration
    let config = PluginCoreConfig {
        auto_initialize_plugins: true,
        require_core_registration: false,
        plugin_directory: "./plugins".to_string(),
        verify_signatures: false,
    };
    
    // Create and initialize the adapter
    let mut adapter = PluginCoreAdapter::with_config(config);
    adapter.initialize().await?;
    
    // Load plugins from the configured directory
    let plugin_ids = adapter.load_plugins().await?;
    println!("Loaded {} plugins", plugin_ids.len());
    
    // Get plugin statuses
    for id in plugin_ids {
        let status = adapter.get_plugin_status(id).await?;
        println!("Plugin {}: {:?}", id, status);
    }
    
    // Get core status
    let core_status = adapter.get_core_status().await?;
    println!("Core status: {}", core_status.status);
    
    // Shutdown all plugins when done
    adapter.shutdown_all_plugins().await?;
    
    Ok(())
}
```

#### Using with Dependency Injection

The adapter can also be used with dependency injection:

```rust
use std::sync::Arc;
use squirrel_integration::plugin_core::PluginCoreAdapter;

async fn process_plugins(adapter: Arc<PluginCoreAdapter>) -> anyhow::Result<()> {
    // Use the adapter
    let plugins = adapter.get_all_plugins().await?;
    for plugin in plugins {
        // Process each plugin
        println!("Processing plugin: {}", plugin.metadata().name);
    }
    
    Ok(())
}
```

## Testing

The integration components come with comprehensive testing support:

```rust
# Run the tests
cargo test -p squirrel-integration
```

## Adding New Integrations

To add a new integration:

1. Create a new module in `src/`
2. Implement the adapter pattern for your integration
3. Add proper error handling
4. Include comprehensive tests

## Documentation

For more detailed documentation, see:

- [Plugin-Core Integration Spec](../specs/integration/plugin-core-integration.md)
- [Integration Progress Update](../specs/integration/PROGRESS_UPDATE.md)
- [Adapter Implementation Guide](../specs/patterns/adapter-implementation-guide.md) 