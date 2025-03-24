use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use serde_json::Value;
use async_trait::async_trait;

use galaxy::plugin::{GalaxyPlugin, GalaxyPluginManager};

struct ExamplePlugin {
    name: String,
    version: String,
    description: String,
}

impl ExamplePlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Example Galaxy Plugin".to_string(),
        }
    }
}

impl std::fmt::Debug for ExamplePlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExamplePlugin")
            .field("name", &self.name)
            .field("version", &self.version)
            .finish()
    }
}

#[async_trait]
impl GalaxyPlugin for ExamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn initialize(&self, _adapter: Arc<galaxy::adapter::GalaxyAdapter>) -> Result<(), galaxy::error::Error> {
        println!("Initializing plugin: {}", self.name);
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), galaxy::error::Error> {
        println!("Shutting down plugin: {}", self.name);
        Ok(())
    }
    
    fn provides_capability(&self, capability: &str) -> bool {
        capability == "example"
    }
    
    fn capabilities(&self) -> Vec<String> {
        vec!["example".to_string()]
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Galaxy Plugin Example");
    
    // Create a Galaxy adapter
    let adapter = galaxy::create_adapter()?;
    let adapter = Arc::new(adapter);
    
    // Create a plugin manager
    let mut manager = galaxy::create_plugin_manager(adapter.clone());
    
    // Create and register a plugin
    let plugin = Arc::new(ExamplePlugin::new("example.plugin"));
    manager.register_plugin(plugin.clone()).await?;
    
    // Get the plugin by name
    let retrieved_plugin = manager.get_plugin("example.plugin").expect("Plugin not found");
    println!("Retrieved plugin: {:?}", retrieved_plugin);
    
    println!("Example completed successfully");
    Ok(())
} 