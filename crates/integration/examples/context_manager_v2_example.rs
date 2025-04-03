use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;
use squirrel_integration::{
    ContextMcpAdapter, ContextMcpAdapterConfig, ContextManagerV2,
    ContextManagerCallbacks, types::SquirrelContext,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Example implementation of ContextManagerV2 for improved thread safety
#[derive(Debug, Default)]
struct SimpleContextManagerV2 {
    /// In-memory storage for contexts
    contexts: Arc<Mutex<HashMap<String, SquirrelContext>>>,
    
    /// MCP service callback
    mcp_service: Option<Box<dyn Fn(&str) -> Result<String> + Send + Sync>>,
    
    /// Log event callback
    log_event: Option<Box<dyn Fn(&str, &str) -> Result<()> + Send + Sync>>,
}

#[async_trait]
impl ContextManagerV2 for SimpleContextManagerV2 {
    async fn create_context(
        &self,
        id: &str,
        name: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        // Log the operation using callback if available
        if let Some(log) = &self.log_event {
            log("create_context", &format!("Creating context: {}", id))?;
        }
        
        let context = SquirrelContext {
            id: id.to_string(),
            name: name.to_string(),
            data,
            metadata: metadata.unwrap_or_else(|| json!({})),
        };
        
        // Store the context
        let mut contexts = self.contexts.lock().unwrap();
        contexts.insert(id.to_string(), context);
        
        Ok(())
    }
    
    async fn with_context(&self, id: &str) -> Result<SquirrelContext> {
        // Log the operation using callback if available
        if let Some(log) = &self.log_event {
            log("with_context", &format!("Retrieving context: {}", id))?;
        }
        
        // Get the context
        let contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get(id) {
            Ok(context.clone())
        } else {
            Err(anyhow::anyhow!("Context not found: {}", id))
        }
    }
    
    async fn update_context(
        &self,
        id: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        // Log the operation using callback if available
        if let Some(log) = &self.log_event {
            log("update_context", &format!("Updating context: {}", id))?;
        }
        
        // Get the context
        let mut contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get_mut(id) {
            context.data = data;
            if let Some(meta) = metadata {
                context.metadata = meta;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Context not found: {}", id))
        }
    }
    
    async fn delete_context(&self, id: &str) -> Result<()> {
        // Log the operation using callback if available
        if let Some(log) = &self.log_event {
            log("delete_context", &format!("Deleting context: {}", id))?;
        }
        
        // Remove the context
        let mut contexts = self.contexts.lock().unwrap();
        if contexts.remove(id).is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Context not found: {}", id))
        }
    }
    
    async fn list_contexts(&self) -> Result<Vec<SquirrelContext>> {
        // Log the operation using callback if available
        if let Some(log) = &self.log_event {
            log("list_contexts", "Listing all contexts")?;
        }
        
        // Get all contexts
        let contexts = self.contexts.lock().unwrap();
        Ok(contexts.values().cloned().collect())
    }
    
    fn register_callbacks(&mut self, callbacks: ContextManagerCallbacks) {
        // Store the callbacks we need
        self.mcp_service = callbacks.mcp_service;
        self.log_event = callbacks.log_event;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new context manager
    let context_manager = SimpleContextManagerV2::default();
    
    // Create a new adapter with the context manager
    let adapter = ContextMcpAdapter::with_config_v2(
        ContextMcpAdapterConfig::default(),
        context_manager,
    ).await?;
    
    // Initialize the adapter
    adapter.initialize().await?;
    
    // Create a new context
    let context_id = "example-context";
    adapter.create_context(
        context_id,
        "Example Context",
        json!({
            "description": "This is an example context using the V2 trait",
            "tags": ["example", "v2", "thread-safe"],
        }),
        Some(json!({
            "created_at": chrono::Utc::now().to_rfc3339(),
            "version": "1.0.0",
        })),
    ).await?;
    
    // Get the context
    let context = adapter.get_context(context_id).await?;
    println!("Context: {:#?}", context);
    
    // Update the context
    adapter.update_context(
        context_id,
        json!({
            "description": "This is an updated example context",
            "tags": ["example", "v2", "thread-safe", "updated"],
            "updated": true,
        }),
        None,
    ).await?;
    
    // Get the updated context
    let updated_context = adapter.get_context(context_id).await?;
    println!("Updated Context: {:#?}", updated_context);
    
    // List all contexts
    let contexts = adapter.list_contexts().await?;
    println!("All Contexts: {:#?}", contexts);
    
    // Delete the context
    adapter.delete_context(context_id).await?;
    println!("Context deleted successfully");
    
    Ok(())
} 