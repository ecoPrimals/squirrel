use anyhow::Result;
use serde_json::json;
use squirrel_context::{
    create_default_manager
};
use squirrel_interfaces::context::ContextManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a context manager
    let manager = create_default_manager();
    
    // Initialize the manager
    if let Err(e) = manager.initialize().await {
        eprintln!("Failed to initialize manager: {:?}", e);
        anyhow::bail!("Manager initialization failed");
    }
    println!("Manager initialized successfully");
    
    // Create a test plugin
    #[derive(Debug)]
    struct TestPlugin {
        id: String,
        name: String
    }
    
    impl TestPlugin {
        fn new(id: &str, name: &str) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string()
            }
        }
    }
    
    use async_trait::async_trait;
    use squirrel_interfaces::plugins::{Plugin, PluginMetadata};
    use squirrel_interfaces::context::{ContextPlugin, ContextTransformation, ContextAdapterPlugin};
    use std::sync::Arc;
    
    #[async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            // This would typically be a field in the struct
            // but for simplicity we're creating it on demand
            static mut METADATA: Option<PluginMetadata> = None;
            unsafe {
                if METADATA.is_none() {
                    METADATA = Some(PluginMetadata::new(
                        "test.basic",
                        "1.0.0",
                        "A basic test plugin",
                        "DataScienceBioLab"
                    ).with_capability("context"));
                }
                METADATA.as_ref().unwrap()
            }
        }
    }
    
    #[async_trait]
    impl ContextPlugin for TestPlugin {
        async fn get_transformations(&self) -> Vec<Arc<dyn ContextTransformation>> {
            vec![Arc::new(BasicTransformation::new())]
        }
        
        async fn get_adapters(&self) -> Vec<Arc<dyn ContextAdapterPlugin>> {
            vec![]
        }
    }
    
    // Define a simple transformation
    #[derive(Debug)]
    struct BasicTransformation;
    
    impl BasicTransformation {
        fn new() -> Self {
            Self
        }
    }
    
    #[async_trait]
    impl ContextTransformation for BasicTransformation {
        fn get_id(&self) -> &str {
            "basic.transform"
        }
        
        fn get_name(&self) -> &str {
            "Basic Transformation"
        }
        
        fn get_description(&self) -> &str {
            "A simple transformation for the basic example"
        }
        
        async fn transform(&self, data: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            // Add a timestamp and wrap the data
            let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(n) => n.as_secs().to_string(),
                Err(_) => "unknown".to_string(),
            };
            
            let result = json!({
                "original": data,
                "timestamp": timestamp,
                "transformation": "basic.transform"
            });
            
            Ok(result)
        }
    }
    
    // Register the plugin
    let plugin = Box::new(TestPlugin::new("test.basic", "Basic Test Plugin"));
    if let Err(e) = manager.register_plugin(plugin).await {
        eprintln!("Failed to register plugin: {:?}", e);
        anyhow::bail!("Plugin registration failed");
    }
    println!("Plugin registered successfully");
    
    // Create test data
    let test_data = json!({
        "query": "What is the capital of France?",
        "context": {
            "user": "example_user",
            "session": "12345"
        }
    });
    
    // Transform the data
    match manager.transform_data("basic.transform", test_data.clone()).await {
        Ok(transformed) => {
            println!("Transformation result: {}", serde_json::to_string_pretty(&transformed)?);
        },
        Err(e) => {
            eprintln!("Transformation failed: {:?}", e);
            anyhow::bail!("Data transformation failed");
        }
    }
    
    Ok(())
} 