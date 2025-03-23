# Context Plugin

The Context Plugin provides functionality for transforming and validating context data in the Squirrel system. This plugin implements the core Plugin trait for lifecycle management and provides specialized context operations.

## Features

- Context data transformation through a defined interface
- Schema-based validation of context data
- Support for custom transformations
- Factory functions for easy plugin creation

## Usage

### Basic Usage

```rust
use squirrel_plugins::context::{create_context_plugin};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create the default context plugin
    let context_plugin = create_context_plugin();
    
    // Initialize the plugin
    context_plugin.initialize().await?;
    
    // List available transformations
    let transformations = context_plugin.get_transformations();
    for transform in &transformations {
        println!("ID: {}", transform.id);
        println!("Name: {}", transform.name);
    }
    
    // Use a transformation
    let transform_id = &transformations[0].id;
    let input_data = json!({
        "data": {
            "context_key": "context_value",
            "nested": { "item1": 1 }
        }
    });
    
    let result = context_plugin.transform(transform_id, input_data).await?;
    println!("Transformed data: {}", result);
    
    // Shutdown the plugin
    context_plugin.shutdown().await?;
    
    Ok(())
}
```

### Creating a Custom Plugin

You can create a custom context plugin with your own transformations:

```rust
use squirrel_plugins::context::{
    create_custom_context_plugin, ContextTransformation
};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Define custom transformations
    let transformations = vec![
        ContextTransformation {
            id: "my.custom.transform".to_string(),
            name: "My Custom Transformation".to_string(),
            description: "A custom context transformation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "custom_data": { "type": "object" }
                }
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "result": { "type": "object" }
                }
            }),
        }
    ];
    
    // Create a custom context plugin
    let custom_plugin = create_custom_context_plugin(
        "My Context Plugin",
        "A custom context transformation plugin",
        transformations
    );
    
    // Use the custom plugin...
    
    Ok(())
}
```

## API

### Main Trait

```rust
pub trait ContextPlugin: Plugin {
    fn get_transformations(&self) -> Vec<ContextTransformation>;
    async fn transform(&self, transformation_id: &str, data: Value) -> Result<Value>;
    fn validate(&self, schema: &Value, data: &Value) -> Result<bool>;
    fn supports_transformation(&self, transformation_id: &str) -> bool;
    fn get_capabilities(&self) -> Vec<String>;
}
```

### Factory Functions

```rust
pub fn create_context_plugin() -> Arc<dyn ContextPlugin>

pub fn create_custom_context_plugin(
    name: impl Into<String>,
    description: impl Into<String>,
    transformations: Vec<ContextTransformation>,
) -> Arc<dyn ContextPlugin>
```

## Integration with Plugin System

The Context Plugin fully integrates with the Squirrel Plugin System:

1. Implements the core `Plugin` trait for lifecycle management
2. Provides plugin metadata and capabilities
3. Can be registered with the plugin registry
4. Works with the plugin manager for coordination

## Migration Notes

This plugin has been migrated from the standalone `squirrel-context` crate as part of the plugin architecture consolidation effort. It maintains the core functionality while adapting to the plugin system's architecture.

Key changes:
- Implemented the `Plugin` trait for lifecycle management
- Added metadata and capability support
- Standardized API to match the plugin system
- Added factory functions for easier instantiation 