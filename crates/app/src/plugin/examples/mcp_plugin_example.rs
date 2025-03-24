//! Example MCP plugin implementation
//!
//! This file demonstrates how to create an MCP plugin using the Squirrel plugin system.
//! It implements an MCP plugin that provides context enrichment and custom protocol extensions.

use crate::error::Result;
use crate::plugin::{
    Plugin, PluginMetadata, PluginState,
    types::{McpPlugin, McpPluginImpl, McpPluginBuilder},
};
use async_trait::async_trait;
use futures::future::BoxFuture;
use serde_json::{json, Value};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::collections::HashMap;

/// Create a simple MCP plugin using the builder pattern
#[must_use] pub fn create_example_mcp_plugin() -> Arc<McpPluginImpl> {
    McpPluginBuilder::new(PluginMetadata {
        id: Uuid::new_v4(),
        name: "example-mcp".to_string(),
        version: "0.1.0".to_string(),
        description: "Example MCP plugin".to_string(),
        author: "Squirrel Team".to_string(),
        dependencies: vec![],
        capabilities: vec!["mcp".to_string()],
    })
    .with_extension("context-enhancer")
    .with_extension("code-metrics")
    .build()
}

/// Advanced MCP plugin implementation with custom logic
pub struct AdvancedMcpPlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Plugin state
    state: RwLock<Option<PluginState>>,
    /// Supported protocol extensions
    extensions: Vec<String>,
    /// Message handlers
    handlers: RwLock<HashMap<String, Box<dyn Fn(Value) -> BoxFuture<'static, Result<Value>> + Send + Sync>>>,
    /// Context cache
    context_cache: RwLock<HashMap<String, Value>>,
}

impl Clone for AdvancedMcpPlugin {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            state: RwLock::new(None), // Create a new RwLock for the clone
            extensions: self.extensions.clone(),
            handlers: RwLock::new(HashMap::new()), // Create a new empty handlers map
            context_cache: RwLock::new(HashMap::new()), // Create a new empty context cache
        }
    }
}

impl std::fmt::Debug for AdvancedMcpPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdvancedMcpPlugin")
            .field("metadata", &self.metadata)
            .field("extensions", &self.extensions)
            .field("state", &self.state)
            .field("handlers", &format!("<{} handlers>", self.extensions.len()))
            .field("context_cache", &"<context cache>".to_string())
            .finish()
    }
}

impl Default for AdvancedMcpPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvancedMcpPlugin {
    /// Create a new advanced MCP plugin
    #[must_use] pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: "advanced-mcp-plugin".to_string(),
                version: "0.1.0".to_string(),
                description: "Advanced MCP plugin with context enrichment and protocol extensions".to_string(),
                author: "Squirrel Team".to_string(),
                dependencies: vec![],
                capabilities: vec!["mcp".to_string()],
            },
            state: RwLock::new(None),
            context_cache: RwLock::new(HashMap::new()),
            handlers: RwLock::new(HashMap::new()),
            extensions: vec![
                "context-enrichment".to_string(),
                "code-intelligence".to_string(),
                "memory-management".to_string(),
            ],
        }
    }
    
    /// Create a new advanced MCP plugin with custom metadata
    #[must_use] pub fn with_metadata(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            state: RwLock::new(None),
            context_cache: RwLock::new(HashMap::new()),
            handlers: RwLock::new(HashMap::new()),
            extensions: vec![
                "context-enrichment".to_string(),
                "code-intelligence".to_string(),
                "memory-management".to_string(),
            ],
        }
    }
    
    /// Register message handlers
    pub async fn register_handlers(&self) {
        // Handler for context enrichment messages
        self.register_handler("context-enrich", |message| {
            Box::pin(async move {
                // Extract file path from message
                let file_path = message.get("file_path").and_then(Value::as_str).unwrap_or("");
                
                // For this example, we'll just return some mock enriched context
                let enriched_context = json!({
                    "file_path": file_path,
                    "language": "rust",
                    "metrics": {
                        "lines_of_code": 142,
                        "complexity": 32,
                        "functions": 12,
                        "classes": 2
                    },
                    "imports": [
                        "std::sync::Arc",
                        "tokio::sync::RwLock",
                        "serde_json::Value"
                    ],
                    "dependencies": [
                        "tokio",
                        "serde_json",
                        "uuid"
                    ]
                });
                
                Ok(json!({
                    "status": "success",
                    "context": enriched_context
                }))
            })
        }).await;
        
        // Handler for code intelligence messages
        self.register_handler("code-intelligence", |message| {
            Box::pin(async move {
                // Extract code from message
                let code = message.get("code").and_then(Value::as_str).unwrap_or("");
                
                // For this example, we'll return mock intelligence data
                Ok(json!({
                    "status": "success",
                    "intelligence": {
                        "suggestions": [
                            {
                                "line": 12,
                                "suggestion": "Consider using Arc<RwLock<T>> for thread-safe access",
                                "confidence": 0.92
                            },
                            {
                                "line": 24,
                                "suggestion": "Use tokio::spawn for better concurrency",
                                "confidence": 0.85
                            }
                        ],
                        "summary": "The code appears to be a Rust implementation of a thread-safe cache.",
                        "complexity_analysis": "Medium complexity with potential for optimization."
                    }
                }))
            })
        }).await;
        
        // Handler for memory management messages
        self.register_handler("memory-management", |message| {
            Box::pin(async move {
                let action = message.get("action").and_then(Value::as_str).unwrap_or("");
                
                match action {
                    "analyze" => {
                        Ok(json!({
                            "status": "success",
                            "memory_analysis": {
                                "total_allocated": "4.2 MB",
                                "peak_usage": "12.8 MB",
                                "recommendation": "Consider using Arc instead of multiple clones."
                            }
                        }))
                    },
                    "optimize" => {
                        Ok(json!({
                            "status": "success",
                            "optimization_result": {
                                "before": "12.8 MB",
                                "after": "8.3 MB",
                                "improvement": "35%",
                                "changes_made": [
                                    "Replaced redundant clones with Arc",
                                    "Optimized string allocations",
                                    "Reduced unnecessary Box usage"
                                ]
                            }
                        }))
                    },
                    _ => Ok(json!({
                        "status": "error",
                        "error": "Unknown action"
                    }))
                }
            })
        }).await;
    }
    
    /// Register a message handler
    pub async fn register_handler<F, Fut>(&self, message_type: &str, handler: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value>> + Send + 'static,
    {
        let mut handlers = self.handlers.write().await;
        
        // Convert the handler to a BoxFuture-returning closure
        let boxed_handler = Box::new(move |value: Value| -> BoxFuture<'static, Result<Value>> {
            Box::pin(handler(value))
        });
        
        handlers.insert(message_type.to_string(), boxed_handler);
    }
    
    /// Update context cache
    pub async fn update_context(&self, key: &str, value: Value) -> Result<()> {
        let mut cache = self.context_cache.write().await;
        cache.insert(key.to_string(), value);
        Ok(())
    }
    
    /// Get context from cache
    pub async fn get_context(&self, key: &str) -> Option<Value> {
        let cache = self.context_cache.read().await;
        cache.get(key).cloned()
    }
    
    /// Clear context cache
    pub async fn clear_context(&self) -> Result<()> {
        let mut cache = self.context_cache.write().await;
        cache.clear();
        Ok(())
    }
}

impl Plugin for AdvancedMcpPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Register message handlers during initialization
            self.register_handlers().await;
            
            // Initialize context cache
            let _ = self.update_context("plugin_start_time", json!(chrono::Utc::now().to_rfc3339())).await;
            
            // Load state if available
            if let Ok(Some(state)) = self.get_state().await {
                if let Some(cache_data) = state.data.get("context_cache") {
                    if let Some(cache_map) = cache_data.as_object() {
                        let mut cache = self.context_cache.write().await;
                        for (key, value) in cache_map {
                            cache.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            
            Ok(())
        })
    }
    
    fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Save context cache to state
            let cache = self.context_cache.read().await;
            let mut cache_data = serde_json::Map::new();
            
            for (key, value) in cache.iter() {
                cache_data.insert(key.clone(), value.clone());
            }
            
            // Create state with context cache
            let state = PluginState {
                plugin_id: self.metadata.id,
                data: json!({
                    "context_cache": cache_data,
                    "shutdown_time": chrono::Utc::now().to_rfc3339(),
                }),
                last_modified: chrono::Utc::now(),
            };
            
            // Save state
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
            state: RwLock::new(None),
            extensions: self.extensions.clone(),
            handlers: RwLock::new(HashMap::new()),
            context_cache: RwLock::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl McpPlugin for AdvancedMcpPlugin {
    async fn handle_message(&self, message: Value) -> Result<Value> {
        // Extract the message type
        let message_type = match message.get("type").and_then(Value::as_str) {
            Some(t) => t,
            None => return Ok(json!({"error": "Missing message type"})),
        };
        
        // Find the handler for this message type
        let handlers = self.handlers.read().await;
        
        if let Some(handler) = handlers.get(message_type) {
            // Call the handler
            handler(message).await
        } else {
            // No handler found
            Ok(json!({
                "status": "error",
                "error": format!("No handler for message type: {}", message_type)
            }))
        }
    }
    
    fn get_protocol_extensions(&self) -> Vec<String> {
        self.extensions.clone()
    }
    
    fn get_message_handlers(&self) -> Vec<String> {
        vec![
            "context-enrich".to_string(),
            "code-intelligence".to_string(),
            "memory-management".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_builder_plugin() {
        let plugin = create_example_mcp_plugin();
        
        // Test metadata
        assert_eq!(plugin.metadata().name, "example-mcp");
        
        // Test extensions
        let extensions = plugin.get_protocol_extensions();
        assert_eq!(extensions.len(), 2);
        assert!(extensions.contains(&"context-enhancer".to_string()));
        assert!(extensions.contains(&"code-metrics".to_string()));
        
        // Register a handler - note: this requires mutable access which we can't get through an Arc
        // Register handlers in the builder pattern instead
    }
    
    #[tokio::test]
    async fn test_advanced_mcp_plugin() {
        let plugin = AdvancedMcpPlugin::new();
        
        // Initialize plugin
        plugin.initialize().await.unwrap();
        
        // Test context enrichment
        let enrich_request = json!({
            "type": "context-enrich",
            "file_path": "/path/to/src/main.rs"
        });
        
        let result = plugin.handle_message(enrich_request).await.unwrap();
        assert_eq!(result.get("status").unwrap(), "success");
        assert!(result.get("context").is_some());
        
        // Test code intelligence
        let intelligence_request = json!({
            "type": "code-intelligence",
            "code": "fn main() { println!(\"Hello world\"); }"
        });
        
        let result = plugin.handle_message(intelligence_request).await.unwrap();
        assert_eq!(result.get("status").unwrap(), "success");
        assert!(result.get("intelligence").is_some());
        
        // Test memory management
        let memory_request = json!({
            "type": "memory-management",
            "action": "analyze"
        });
        
        let result = plugin.handle_message(memory_request).await.unwrap();
        assert_eq!(result.get("status").unwrap(), "success");
        assert!(result.get("memory_analysis").is_some());
        
        // Test context cache
        plugin.update_context("test_key", json!("test_value")).await.unwrap();
        let context = plugin.get_context("test_key").await.unwrap();
        assert_eq!(context, json!("test_value"));
        
        // Test shutdown
        plugin.shutdown().await.unwrap();
        
        // Check that state was saved
        let state = plugin.get_state().await.unwrap().unwrap();
        assert!(state.data.get("context_cache").is_some());
        assert!(state.data.get("shutdown_time").is_some());
    }
} 