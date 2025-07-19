//! Example demonstrating the usage of context plugins

use std::fmt::Debug;
use std::sync::Arc;
use serde_json::json;
use async_trait::async_trait;
use anyhow::Result;

use squirrel_interfaces::plugins::{Plugin, PluginMetadata};
use squirrel_interfaces::context::{
    ContextPlugin, 
    ContextTransformation,
    ContextAdapterPlugin,
    ContextManager
};

use squirrel_context::{
    create_manager_with_config,
    ContextManagerConfig
};

// Example of a simple context plugin
#[derive(Debug)]
struct ExamplePlugin {
    metadata: PluginMetadata
}

impl ExamplePlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "example.plugin",
                "1.0.0",
                "A simple example plugin for demonstration",
                "DataScienceBioLab"
            ).with_capability("context")
        }
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

#[async_trait]
impl ContextPlugin for ExamplePlugin {
    async fn get_transformations(&self) -> Vec<Arc<dyn ContextTransformation>> {
        vec![Arc::new(ExampleTransformation)]
    }
    
    async fn get_adapters(&self) -> Vec<Arc<dyn ContextAdapterPlugin>> {
        vec![]
    }
}

// Example of a simple transformation
#[derive(Debug)]
struct ExampleTransformation;

#[async_trait]
impl ContextTransformation for ExampleTransformation {
    fn get_id(&self) -> &str {
        "example.transform"
    }
    
    fn get_name(&self) -> &str {
        "Example Transformation"
    }
    
    fn get_description(&self) -> &str {
        "A simple example transformation that adds an 'example' field to the output"
    }
    
    async fn transform(&self, data: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Create a result object with the original data and metadata
        let result = json!({
            "result": data,
            "transformation": self.get_id(),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "example": "This is from the example transformation"
        });
        
        Ok(result)
    }
}

#[tokio::main]
async fn main() {
    // Create a context manager with plugins enabled
    let config = ContextManagerConfig {
        enable_plugins: true,
        ..Default::default()
    };
    
    let manager = create_manager_with_config(config);
    
    // Initialize the manager
    if let Err(e) = manager.initialize().await {
        eprintln!("Failed to initialize manager: {}", e);
        return;
    }
    
    // Register our example plugin
    if let Err(e) = manager.register_plugin(Box::new(ExamplePlugin::new())).await {
        eprintln!("Failed to register plugin: {}", e);
        return;
    }
    
    // Create some test data
    let test_data = json!({
        "query": "What is the meaning of life?",
        "context": {
            "source": "example"
        }
    });
    
    // Transform the data using our example transformation
    match manager.transform_data("example.transform", test_data).await {
        Ok(result) => {
            // Print the result
            match serde_json::to_string_pretty(&result) {
                Ok(json_str) => println!("Transformation result: {}", json_str),
                Err(e) => eprintln!("Failed to format result: {}", e),
            }
        },
        Err(e) => eprintln!("Transformation failed: {}", e),
    }
} 