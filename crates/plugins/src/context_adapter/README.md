# Context Adapter Plugin

The Context Adapter Plugin provides functionality for adapting context data between different formats in the Squirrel system. This plugin implements the core Plugin trait for lifecycle management and provides specialized format conversion operations.

## Features

- Format conversion between different context formats (e.g., JSON to MCP)
- Format validation for supported formats
- Format compatibility checking
- Support for custom adapters
- Factory functions for easy plugin creation

## Usage

### Basic Usage

```rust
use squirrel_plugins::context_adapter::{create_context_adapter_plugin};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create the default context adapter plugin
    let adapter_plugin = create_context_adapter_plugin();
    
    // Initialize the plugin
    adapter_plugin.initialize().await?;
    
    // List available adapters
    let adapters = adapter_plugin.get_adapters();
    for adapter in &adapters {
        println!("ID: {}", adapter.id);
        println!("Name: {}", adapter.name);
        println!("Formats: {} -> {}", adapter.source_format, adapter.target_format);
    }
    
    // Use an adapter
    let adapter_id = "json.to.mcp";
    let input_data = json!({
        "command": "process",
        "data": {
            "text": "Hello, world!",
            "value": 42
        }
    });
    
    let result = adapter_plugin.convert(adapter_id, input_data.clone()).await?;
    println!("Converted data: {}", result);
    
    // Validate format
    let validation = adapter_plugin.validate_format("json", &input_data)?;
    println!("Input format validation: {}", validation);
    
    // Check compatibility
    let compatibility = adapter_plugin.check_compatibility("json", "mcp");
    println!("Format compatibility: {}", compatibility);
    
    // Shutdown the plugin
    adapter_plugin.shutdown().await?;
    
    Ok(())
}
```

### Creating a Custom Plugin

You can create a custom context adapter plugin with your own adapters:

```rust
use squirrel_plugins::context_adapter::{
    create_custom_context_adapter_plugin, AdapterMetadata
};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Define custom adapters
    let adapters = vec![
        AdapterMetadata {
            id: "custom.format.adapter".to_string(),
            name: "Custom Format Adapter".to_string(),
            description: "A custom format adapter".to_string(),
            source_format: "custom".to_string(),
            target_format: "standard".to_string(),
        }
    ];
    
    // Create a custom context adapter plugin
    let custom_plugin = create_custom_context_adapter_plugin(
        "My Context Adapter Plugin",
        "A custom context format adapter plugin",
        adapters
    );
    
    // Use the custom plugin...
    
    Ok(())
}
```

## API

### Main Trait

```rust
pub trait ContextAdapterPlugin: Plugin {
    fn get_adapters(&self) -> Vec<AdapterMetadata>;
    async fn convert(&self, adapter_id: &str, data: Value) -> Result<Value>;
    fn validate_format(&self, format: &str, data: &Value) -> Result<bool>;
    fn check_compatibility(&self, source_format: &str, target_format: &str) -> bool;
    fn supports_adapter(&self, adapter_id: &str) -> bool;
    fn get_capabilities(&self) -> Vec<String>;
}
```

### Factory Functions

```rust
pub fn create_context_adapter_plugin() -> Arc<dyn ContextAdapterPlugin>

pub fn create_custom_context_adapter_plugin(
    name: impl Into<String>,
    description: impl Into<String>,
    adapters: Vec<AdapterMetadata>,
) -> Arc<dyn ContextAdapterPlugin>
```

## Integration with Plugin System

The Context Adapter Plugin fully integrates with the Squirrel Plugin System:

1. Implements the core `Plugin` trait for lifecycle management
2. Provides plugin metadata and capabilities
3. Can be registered with the plugin registry
4. Works with the plugin manager for coordination

## Working with Machine Context Protocol (MCP)

This plugin is specifically designed to work with the Machine Context Protocol (MCP), providing adapters for conversion between JSON and MCP formats. This enables seamless integration of different data sources with the Squirrel machine context system.

## Migration Notes

This plugin has been migrated from the standalone `squirrel-context-adapter` crate as part of the plugin architecture consolidation effort. It maintains the core functionality while adapting to the plugin system's architecture.

Key changes:
- Implemented the `Plugin` trait for lifecycle management
- Added metadata and capability support
- Standardized API to match the plugin system
- Added factory functions for easier instantiation
- Enhanced format validation and compatibility checking 