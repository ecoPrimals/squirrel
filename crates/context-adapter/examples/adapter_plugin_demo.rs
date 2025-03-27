use std::sync::Arc;
use serde_json::json;

use squirrel_context_adapter::adapter::{
    ContextAdapterConfig, 
    create_context_adapter_with_plugins
};
use squirrel_context::plugins::ContextPluginManager;
use squirrel_interfaces::context::{
    ContextPlugin, 
    ContextTransformation, 
    ContextAdapterPlugin,
    AdapterMetadata
};
use squirrel_interfaces::plugins::{Plugin, PluginMetadata};

// Custom plugin implementation for demonstration
#[derive(Debug)]
struct DemoPlugin {
    metadata: PluginMetadata,
    transformations: Vec<Arc<dyn ContextTransformation>>,
    adapters: Vec<Arc<dyn ContextAdapterPlugin>>,
}

impl DemoPlugin {
    fn new() -> Self {
        let metadata = PluginMetadata::new(
            "Demo Plugin",
            "1.0.0",
            "A demonstration plugin for context adapters",
            "DataScienceBioLab",
        )
        .with_capability("context.transform")
        .with_capability("context.format");

        // Create demo transformations
        let demo_transformation = Arc::new(DemoTransformation {
            id: "demo.transform".to_string(),
            name: "Demo Transformation".to_string(),
            description: "A demonstration transformation".to_string(),
        });

        // Create demo adapters
        let demo_adapter = Arc::new(DemoAdapter {
            metadata: AdapterMetadata {
                id: "demo.adapter".to_string(),
                name: "Demo Adapter".to_string(),
                description: "A demonstration adapter".to_string(),
                source_format: "json".to_string(),
                target_format: "mcp".to_string(),
            },
            plugin_metadata: PluginMetadata {
                id: "demo.adapter".to_string(),
                name: "Demo Adapter Plugin".to_string(),
                description: "Plugin for demo adapter".to_string(),
                version: "1.0.0".to_string(),
                author: "Squirrel Demo".to_string(),
                capabilities: Vec::new(),
            },
        });

        Self {
            metadata,
            transformations: vec![demo_transformation as Arc<dyn ContextTransformation>],
            adapters: vec![demo_adapter as Arc<dyn ContextAdapterPlugin>],
        }
    }
}

#[async_trait::async_trait]
impl Plugin for DemoPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        println!("Initializing Demo Plugin");
        Ok(())
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        println!("Shutting down Demo Plugin");
        Ok(())
    }
}

#[async_trait::async_trait]
impl ContextPlugin for DemoPlugin {
    async fn get_transformations(&self) -> Vec<Arc<dyn ContextTransformation>> {
        println!("Getting transformations from Demo Plugin");
        self.transformations.clone()
    }

    async fn get_adapters(&self) -> Vec<Arc<dyn ContextAdapterPlugin>> {
        println!("Getting adapters from Demo Plugin");
        self.adapters.clone()
    }
}

// Demo transformation implementation
#[derive(Debug)]
struct DemoTransformation {
    id: String,
    name: String,
    description: String,
}

#[async_trait::async_trait]
impl ContextTransformation for DemoTransformation {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    async fn transform(&self, data: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        println!("Transforming data with: {}", self.name);
        
        // Add some metadata to the data
        let result = json!({
            "original": data,
            "metadata": {
                "transformation": self.id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });
        
        Ok(result)
    }
}

// Demo adapter implementation
#[derive(Debug)]
struct DemoAdapter {
    metadata: AdapterMetadata,
    plugin_metadata: PluginMetadata,
}

#[async_trait::async_trait]
impl ContextAdapterPlugin for DemoAdapter {
    async fn get_metadata(&self) -> AdapterMetadata {
        self.metadata.clone()
    }

    async fn convert(&self, data: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        println!("Converting data with: {}", self.metadata.name);
        
        // Add MCP wrapper structure
        let result = json!({
            "version": "1.0",
            "format": "mcp",
            "message": {
                "content": data,
                "converted_by": self.metadata.id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });
        
        Ok(result)
    }
}

#[async_trait::async_trait]
impl Plugin for DemoAdapter {
    fn metadata(&self) -> &PluginMetadata {
        &self.plugin_metadata
    }
    
    async fn initialize(&self) -> anyhow::Result<()> {
        println!("Initializing Demo Adapter");
        Ok(())
    }
    
    async fn shutdown(&self) -> anyhow::Result<()> {
        println!("Shutting down Demo Adapter");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable tracing for logs
    tracing_subscriber::fmt::init();
    
    // Create a plugin manager
    let plugin_manager = Arc::new(ContextPluginManager::new());
    
    // Create and register our demo plugin
    let demo_plugin = Box::new(DemoPlugin::new());
    plugin_manager.register_plugin(demo_plugin).await?;
    
    // Create configuration with plugins enabled
    let config = ContextAdapterConfig {
        max_contexts: 100,
        ttl_seconds: 3600,
        enable_auto_cleanup: true,
        enable_plugins: true,
    };
    
    // Create adapter with plugin support
    println!("Creating context adapter with plugin support");
    let adapter = create_context_adapter_with_plugins(config, plugin_manager);
    
    // Initialize plugins
    println!("Initializing plugins");
    adapter.initialize_plugins().await?;
    
    // Create sample data
    let sample_data = json!({
        "user": "example_user",
        "action": "update",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": {
            "field1": "value1",
            "field2": 42,
            "field3": true
        }
    });
    
    // Create a context
    println!("\nCreating context with sample data");
    adapter.create_context("demo-context".to_string(), sample_data.clone()).await?;
    
    // List all contexts
    println!("\nListing all contexts:");
    let contexts = adapter.list_contexts().await?;
    for context in contexts {
        println!("  - {}: {}", context.id, context.data);
    }
    
    // Get available transformations
    println!("\nAvailable transformations:");
    let transformations = adapter.get_transformations().await?;
    for transformation in transformations {
        println!("  - {}", transformation);
    }
    
    // Get available adapters
    println!("\nAvailable adapters:");
    let adapters = adapter.get_adapters().await?;
    for adapter_metadata in adapters {
        println!("  - {}: {} to {}", 
            adapter_metadata.id, 
            adapter_metadata.source_format,
            adapter_metadata.target_format
        );
    }
    
    // Transform the data
    println!("\nTransforming data with 'demo.transform'");
    let transformed_data = adapter.transform_data("demo.transform", sample_data.clone()).await?;
    println!("Transformed data: {}", serde_json::to_string_pretty(&transformed_data)?);
    
    // Convert the data
    println!("\nConverting data with 'demo.adapter'");
    let converted_data = adapter.convert_data("demo.adapter", sample_data.clone()).await?;
    println!("Converted data: {}", serde_json::to_string_pretty(&converted_data)?);
    
    println!("\nExample completed successfully");
    Ok(())
} 