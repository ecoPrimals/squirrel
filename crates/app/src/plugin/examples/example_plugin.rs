//! Example plugin implementation
//!
//! This file demonstrates how to create a plugin using the Squirrel plugin system.
//! It implements a simple command plugin that provides a few example commands.

use crate::error::Result;
use crate::plugin::{
    Plugin, PluginMetadata, PluginState,
    types::{CommandPlugin, CommandPluginBuilder},
};
use async_trait::async_trait;
use futures::future::BoxFuture;
use serde_json::{json, Value};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::commands_crate::CommandRegistry;

/// Create a simple command plugin using the builder pattern
#[must_use] pub fn create_example_command_plugin() -> Box<dyn CommandPlugin> {
    CommandPluginBuilder::new(PluginMetadata {
        id: Uuid::new_v4(),
        name: "example-commands".to_string(),
        version: "0.1.0".to_string(),
        description: "Example command plugin".to_string(),
        author: "Squirrel Team".to_string(),
        dependencies: vec![],
        capabilities: vec!["command".to_string()],
    })
    .with_command("hello", "Say hello to someone")
    .with_command("echo", "Echo back the input")
    .build()
}

/// Advanced example plugin implementation
#[derive(Debug)]
pub struct AdvancedExamplePlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Internal counter
    counter: RwLock<u32>,
    /// Plugin state
    state: RwLock<Option<PluginState>>,
    /// Available commands
    commands: Vec<String>,
    /// Command registry
    registry: Arc<CommandRegistry>,
}

impl AdvancedExamplePlugin {
    /// Create a new advanced example plugin
    #[must_use] pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: "advanced-example".to_string(),
                version: "0.1.0".to_string(),
                description: "Advanced example plugin with custom implementation".to_string(),
                author: "Squirrel Team".to_string(),
                dependencies: vec![],
                capabilities: vec!["command".to_string()],
            },
            counter: RwLock::new(0),
            state: RwLock::new(None),
            commands: vec![
                "increment".to_string(),
                "decrement".to_string(),
                "get_count".to_string(),
                "reset".to_string(),
            ],
            registry: Arc::new(CommandRegistry::new()),
        }
    }
    
    /// Create a new advanced example plugin with custom metadata
    #[must_use] pub fn with_metadata(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            counter: RwLock::new(0),
            state: RwLock::new(None),
            commands: vec![
                "increment".to_string(),
                "decrement".to_string(),
                "get_count".to_string(),
                "reset".to_string(),
            ],
            registry: Arc::new(CommandRegistry::new()),
        }
    }
    
    /// Increment the counter
    async fn increment(&self, amount: u32) -> u32 {
        let mut counter = self.counter.write().await;
        *counter += amount;
        *counter
    }
    
    /// Decrement the counter
    async fn decrement(&self, amount: u32) -> u32 {
        let mut counter = self.counter.write().await;
        *counter = counter.saturating_sub(amount);
        *counter
    }
    
    /// Get the current count
    async fn get_count(&self) -> u32 {
        let counter = self.counter.read().await;
        *counter
    }
    
    /// Reset the counter
    async fn reset(&self) -> u32 {
        let mut counter = self.counter.write().await;
        *counter = 0;
        *counter
    }
}

impl Default for AdvancedExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AdvancedExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Load state if available
            if let Ok(Some(state)) = self.get_state().await {
                if let Some(count) = state.data.get("count").and_then(serde_json::Value::as_u64) {
                    let mut counter = self.counter.write().await;
                    *counter = count as u32;
                }
            }
            
            Ok(())
        })
    }
    
    fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Save state
            let count = self.get_count().await;
            let state = PluginState {
                plugin_id: self.metadata.id,
                data: json!({
                    "count": count,
                }),
                last_modified: chrono::Utc::now(),
            };
            
            self.set_state(state).await?;
            
            Ok(())
        })
    }
    
    fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>> {
        Box::pin(async move {
            let state = self.state.read().await;
            Ok(state.clone())
        })
    }
    
    fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let mut guard = self.state.write().await;
            *guard = Some(state);
            Ok(())
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(Self {
            metadata: self.metadata.clone(),
            counter: RwLock::new(0),
            state: RwLock::new(None),
            commands: self.commands.clone(),
            registry: self.registry.clone(),
        })
    }
}

#[async_trait]
impl CommandPlugin for AdvancedExamplePlugin {
    async fn execute_command(&self, command: &str, args: Value) -> Result<Value> {
        match command {
            "increment" => {
                let amount = args.get("amount").and_then(Value::as_u64).unwrap_or(1) as u32;
                let new_count = self.increment(amount).await;
                Ok(json!({
                    "count": new_count,
                    "message": format!("Incremented counter by {amount} to {new_count}")
                }))
            }
            "decrement" => {
                let amount = args.get("amount").and_then(Value::as_u64).unwrap_or(1) as u32;
                let new_count = self.decrement(amount).await;
                Ok(json!({
                    "count": new_count,
                    "message": format!("Decremented counter by {amount} to {new_count}")
                }))
            }
            "get_count" => {
                let count = self.get_count().await;
                Ok(json!({
                    "count": count,
                    "message": format!("Current count is {count}")
                }))
            }
            "reset" => {
                let new_count = self.reset().await;
                Ok(json!({
                    "count": new_count,
                    "message": "Counter reset to 0"
                }))
            }
            _ => {
                Ok(json!({
                    "error": format!("Unknown command: {command}")
                }))
            }
        }
    }
    
    async fn get_commands(&self) -> Result<Vec<String>> {
        Ok(self.commands.clone())
    }
    
    fn get_command_help(&self, command: &str) -> Option<String> {
        match command {
            "increment" => Some("Increment the counter. Args: amount (default: 1)".to_string()),
            "decrement" => Some("Decrement the counter. Args: amount (default: 1)".to_string()),
            "get_count" => Some("Get the current count.".to_string()),
            "reset" => Some("Reset the counter to 0.".to_string()),
            _ => None,
        }
    }
    
    fn list_commands(&self) -> Vec<String> {
        self.commands.clone()
    }
    
    fn registry(&self) -> Arc<CommandRegistry> {
        self.registry.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_builder_plugin() {
        let plugin = create_example_command_plugin();
        
        // Test metadata
        assert_eq!(plugin.metadata().name, "example-commands");
        
        // Test commands
        let commands = plugin.list_commands();
        assert!(commands.contains(&"hello".to_string()));
        assert!(commands.contains(&"echo".to_string()));
    }
    
    #[tokio::test]
    async fn test_advanced_plugin() {
        let plugin = AdvancedExamplePlugin::new();
        
        // Test initialization
        plugin.initialize().await.unwrap();
        
        // Test command execution
        let result = plugin.execute_command("increment", json!({"amount": 5})).await.unwrap();
        assert_eq!(result.get("count").unwrap(), 5);
        
        let result = plugin.execute_command("get_count", json!({})).await.unwrap();
        assert_eq!(result.get("count").unwrap(), 5);
        
        let result = plugin.execute_command("decrement", json!({"amount": 2})).await.unwrap();
        assert_eq!(result.get("count").unwrap(), 3);
        
        let result = plugin.execute_command("reset", json!({})).await.unwrap();
        assert_eq!(result.get("count").unwrap(), 0);
        
        // Test shutdown
        plugin.shutdown().await.unwrap();
        
        // Check state
        let state = plugin.get_state().await.unwrap().unwrap();
        assert_eq!(state.data.get("count").unwrap(), 0);
    }
} 