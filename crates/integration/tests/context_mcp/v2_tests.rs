use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::json;
use squirrel_integration::{
    ContextMcpAdapter, ContextMcpAdapterConfig, ContextManagerV2,
    ContextManagerCallbacks, types::SquirrelContext,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Simple in-memory implementation of ContextManagerV2 for testing
#[derive(Debug, Default)]
struct TestContextManagerV2 {
    /// In-memory storage for contexts
    contexts: Arc<Mutex<HashMap<String, SquirrelContext>>>,
    
    /// Flag to track if callbacks were registered
    callbacks_registered: Arc<Mutex<bool>>,
    
    /// MCP service callback
    mcp_service: Option<Box<dyn Fn(&str) -> Result<String> + Send + Sync>>,
    
    /// Log event callback
    log_event: Option<Box<dyn Fn(&str, &str) -> Result<()> + Send + Sync>>,
}

impl TestContextManagerV2 {
    /// Check if callbacks were registered
    fn were_callbacks_registered(&self) -> bool {
        *self.callbacks_registered.lock().unwrap()
    }
}

#[async_trait]
impl ContextManagerV2 for TestContextManagerV2 {
    async fn create_context(
        &self,
        id: &str,
        name: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        // Use the mcp_service callback if available to log operation
        if let Some(mcp_service) = &self.mcp_service {
            let _ = mcp_service(&format!("Creating context: {}", id));
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
        // Get the context
        let contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get(id) {
            Ok(context.clone())
        } else {
            Err(anyhow!("Context not found: {}", id))
        }
    }
    
    async fn update_context(
        &self,
        id: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        // Get the context
        let mut contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get_mut(id) {
            context.data = data;
            if let Some(meta) = metadata {
                context.metadata = meta;
            }
            Ok(())
        } else {
            Err(anyhow!("Context not found: {}", id))
        }
    }
    
    async fn delete_context(&self, id: &str) -> Result<()> {
        // Remove the context
        let mut contexts = self.contexts.lock().unwrap();
        if contexts.remove(id).is_some() {
            Ok(())
        } else {
            Err(anyhow!("Context not found: {}", id))
        }
    }
    
    async fn list_contexts(&self) -> Result<Vec<SquirrelContext>> {
        // Get all contexts
        let contexts = self.contexts.lock().unwrap();
        Ok(contexts.values().cloned().collect())
    }
    
    fn register_callbacks(&mut self, callbacks: ContextManagerCallbacks) {
        // Mark callbacks as registered
        let mut registered = self.callbacks_registered.lock().unwrap();
        *registered = true;
        
        // Store the callbacks we need
        self.mcp_service = callbacks.mcp_service;
        self.log_event = callbacks.log_event;
    }
}

#[tokio::test]
async fn test_context_manager_v2_callbacks() {
    // Create a context manager
    let context_manager = TestContextManagerV2::default();
    
    // Create an adapter with the context manager
    let adapter = ContextMcpAdapter::with_config_v2(
        ContextMcpAdapterConfig::default(),
        context_manager.clone(),
    ).await.expect("Failed to create adapter");
    
    // Verify callbacks were registered
    assert!(context_manager.were_callbacks_registered(), "Callbacks should be registered");
}

#[tokio::test]
async fn test_context_manager_v2_crud() {
    // Create a context manager
    let context_manager = TestContextManagerV2::default();
    
    // Create an adapter with the context manager
    let adapter = ContextMcpAdapter::with_config_v2(
        ContextMcpAdapterConfig::default(),
        context_manager,
    ).await.expect("Failed to create adapter");
    
    // Initialize the adapter
    adapter.initialize().await.expect("Failed to initialize adapter");
    
    // Create a test context
    let context_id = "test-context-v2";
    let context_name = "Test Context V2";
    let context_data = json!({
        "description": "This is a test context",
        "tags": ["test", "v2"],
    });
    
    adapter.create_context(
        context_id,
        context_name,
        context_data.clone(),
        None,
    ).await.expect("Failed to create context");
    
    // Get the context
    let context = adapter.get_context(context_id).await.expect("Failed to get context");
    
    // Verify the context
    assert_eq!(context.id, context_id);
    assert_eq!(context.name, context_name);
    assert_eq!(context.data, context_data);
    
    // Update the context
    let updated_data = json!({
        "description": "This is an updated test context",
        "tags": ["test", "v2", "updated"],
        "updated": true,
    });
    
    adapter.update_context(
        context_id,
        updated_data.clone(),
        None,
    ).await.expect("Failed to update context");
    
    // Get the updated context
    let updated_context = adapter.get_context(context_id).await.expect("Failed to get updated context");
    
    // Verify the updated context
    assert_eq!(updated_context.id, context_id);
    assert_eq!(updated_context.name, context_name);
    assert_eq!(updated_context.data, updated_data);
    
    // List contexts
    let contexts = adapter.list_contexts().await.expect("Failed to list contexts");
    
    // Verify the contexts
    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0].id, context_id);
    
    // Delete the context
    adapter.delete_context(context_id).await.expect("Failed to delete context");
    
    // Verify the context is deleted
    let result = adapter.get_context(context_id).await;
    assert!(result.is_err(), "Context should be deleted");
} 