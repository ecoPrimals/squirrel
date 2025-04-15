use squirrel_mcp::error::{MCPError, Result};
use squirrel_mcp::plugins::interfaces::{Plugin, PluginMetadata, PluginStatus, PluginCapability};
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use serde_json::json;
use tokio::main;
use std::any::Any;

/// A simple resource state tracking plugin
#[derive(Debug, Clone)]
struct ResourceStatePlugin {
    metadata: PluginMetadata,
    resource_states: Arc<Mutex<serde_json::Value>>,
}

impl ResourceStatePlugin {
    fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            metadata: PluginMetadata {
                id: id.to_string(),
                name: name.to_string(),
                version: version.to_string(),
                description: "A plugin for tracking resource states".to_string(),
                status: PluginStatus::Registered,
                capabilities: vec![
                    PluginCapability::Custom("resource_state".to_string())
                ],
            },
            resource_states: Arc::new(Mutex::new(json!({}))),
        }
    }
    
    // Helper method for updating resource state (not part of the Plugin trait)
    fn update_state(&self, resource_id: &str, state: serde_json::Value) -> Result<()> {
        println!("Updating state for resource {}", resource_id);
        let mut states = self.resource_states.lock().unwrap();
        
        if states.is_object() {
            states[resource_id] = state;
        } else {
            *states = json!({
                resource_id: state
            });
        }
        
        Ok(())
    }
    
    // Helper method for getting resource state (not part of the Plugin trait)
    fn get_state(&self, resource_id: &str) -> Result<Option<serde_json::Value>> {
        let states = self.resource_states.lock().unwrap();
        
        if states.is_object() {
            if let Some(state) = states.get(resource_id) {
                return Ok(Some(state.clone()));
            }
        }
        
        Ok(None)
    }
}

#[async_trait]
impl Plugin for ResourceStatePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    async fn initialize(&self) -> anyhow::Result<()> {
        println!("Initializing resource state plugin");
        Ok(())
    }
    
    async fn shutdown(&self) -> anyhow::Result<()> {
        println!("Shutting down resource state plugin");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Example demonstrating resource state plugin
#[main]
async fn main() -> Result<()> {
    // Create the plugin
    let plugin = ResourceStatePlugin::new(
        "resource-state-plugin", 
        "Resource State Tracker", 
        "1.0.0"
    );
    
    println!("Created plugin: {} v{}", 
        plugin.metadata().name,
        plugin.metadata().version
    );
    
    // Initialize
    println!("\nInitializing plugin...");
    plugin.initialize().await.map_err(|e| MCPError::from(e.to_string()))?;
    
    // Update some resource states using our helper methods
    println!("\nUpdating resource states:");
    plugin.update_state("resource1", json!({
        "status": "active",
        "last_updated": "2023-01-01T12:00:00Z"
    }))?;
    
    plugin.update_state("resource2", json!({
        "status": "inactive",
        "last_updated": "2023-01-02T14:30:00Z"
    }))?;
    
    // Get and display resource states
    println!("\nResource states:");
    if let Some(resource1_state) = plugin.get_state("resource1")? {
        println!("  - Resource1: {}", resource1_state);
    }
    
    if let Some(resource2_state) = plugin.get_state("resource2")? {
        println!("  - Resource2: {}", resource2_state);
    }
    
    // Non-existent resource
    if let None = plugin.get_state("resource3")? {
        println!("  - Resource3: Not found");
    }
    
    // Shutdown
    println!("\nShutting down plugin...");
    plugin.shutdown().await.map_err(|e| MCPError::from(e.to_string()))?;
    
    println!("\nResource state plugin example completed!");
    Ok(())
} 