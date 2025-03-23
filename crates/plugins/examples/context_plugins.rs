/*!
 * # Context Plugin System Example
 * 
 * This example demonstrates how to use the Context and Context Adapter plugins
 * in the Squirrel plugin system. It showcases:
 * 
 * - Creating and initializing plugins
 * - Listing available transformations and adapters
 * - Using transformations to process context data
 * - Using adapters to convert between different formats
 * - Validating data formats
 */

use tokio;
use anyhow::Result;
use serde_json::json;

use squirrel_plugins::context::{create_context_plugin};
use squirrel_plugins::context_adapter::{create_context_adapter_plugin};

/// Example of using the context and context adapter plugins
#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Squirrel Context Plugin System Example ===");

    // Create context plugin
    let context_plugin = create_context_plugin();
    println!("\n--- Context Plugin Info ---");
    println!("Plugin: {}", context_plugin.metadata().name);
    println!("Description: {}", context_plugin.metadata().description);
    println!("Capabilities: {:?}", context_plugin.metadata().capabilities);

    // Initialize the plugin
    context_plugin.initialize().await?;
    
    // List transformations
    let transformations = context_plugin.get_transformations();
    println!("\n--- Available Transformations ---");
    for transform in &transformations {
        println!("ID: {}", transform.id);
        println!("Name: {}", transform.name);
        println!("Description: {}", transform.description);
    }
    
    // Use a transformation
    if !transformations.is_empty() {
        let transform_id = &transformations[0].id;
        println!("\n--- Using Transformation '{}' ---", transform_id);
        
        let input_data = json!({
            "data": {
                "context_key": "context_value",
                "nested": {
                    "item1": 1,
                    "item2": "two"
                }
            }
        });
        
        println!("Input: {}", input_data);
        let result = context_plugin.transform(transform_id, input_data.clone()).await?;
        println!("Output: {}", result);
    }
    
    // Create context adapter plugin
    let adapter_plugin = create_context_adapter_plugin();
    println!("\n--- Context Adapter Plugin Info ---");
    println!("Plugin: {}", adapter_plugin.metadata().name);
    println!("Description: {}", adapter_plugin.metadata().description);
    println!("Capabilities: {:?}", adapter_plugin.metadata().capabilities);
    
    // Initialize the plugin
    adapter_plugin.initialize().await?;
    
    // List adapters
    let adapters = adapter_plugin.get_adapters();
    println!("\n--- Available Adapters ---");
    for adapter in &adapters {
        println!("ID: {}", adapter.id);
        println!("Name: {}", adapter.name);
        println!("Formats: {} -> {}", adapter.source_format, adapter.target_format);
    }
    
    // Use an adapter
    if !adapters.is_empty() {
        // Find the JSON to MCP adapter
        let adapter_id = "json.to.mcp";
        println!("\n--- Using Adapter '{}' ---", adapter_id);
        
        let input_data = json!({
            "command": "process",
            "data": {
                "value": 42,
                "text": "Hello, world!"
            }
        });
        
        println!("Input: {}", input_data);
        // Clone the input_data before passing it to the convert method
        let result = adapter_plugin.convert(adapter_id, input_data.clone()).await?;
        println!("Output: {}", result);
        
        // Validate format
        let validation = adapter_plugin.validate_format("json", &input_data)?;
        println!("Input format validation: {}", validation);
    }
    
    // Shutdown plugins
    adapter_plugin.shutdown().await?;
    context_plugin.shutdown().await?;
    
    println!("\n=== Example Completed Successfully ===");
    Ok(())
} 