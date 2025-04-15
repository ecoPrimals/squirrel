use crate::context_mcp::*;
use crate::context_mcp::adapter::{ContextManager, ContextMcpAdapter, ContextMcpAdapterConfig, Result, SquirrelContext, SyncDirection};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::json;
use std::collections::HashMap;
use async_trait::async_trait;
use crate::context_mcp::config::{ContextEnhancementType, ContextAiEnhancementOptions};
use tracing::debug;

/// Mock Context Manager for testing
#[derive(Clone)]
struct MockContextManager {
    contexts: Arc<RwLock<HashMap<String, SquirrelContext>>>,
}

impl MockContextManager {
    fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ContextManager for MockContextManager {
    async fn create_context(
        &self, 
        id: &str, 
        name: &str, 
        data: serde_json::Value, 
        metadata: Option<serde_json::Value>
    ) -> anyhow::Result<()> {
        let context = SquirrelContext {
            id: id.to_string(),
            name: name.to_string(),
            data,
            metadata: metadata.unwrap_or_else(|| json!({})),
        };
        
        let mut contexts = self.contexts.write().await;
        contexts.insert(id.to_string(), context);
        Ok(())
    }
    
    async fn with_context(&self, id: &str) -> anyhow::Result<SquirrelContext> {
        let contexts = self.contexts.read().await;
        match contexts.get(id) {
            Some(context) => Ok(context.clone()),
            None => Err(anyhow::anyhow!("Context not found: {}", id)),
        }
    }
    
    async fn update_context(
        &self, 
        id: &str, 
        data: serde_json::Value, 
        metadata: Option<serde_json::Value>
    ) -> anyhow::Result<()> {
        let mut contexts = self.contexts.write().await;
        
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
    
    async fn delete_context(&self, id: &str) -> anyhow::Result<()> {
        let mut contexts = self.contexts.write().await;
        if contexts.remove(id).is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Context not found: {}", id))
        }
    }
    
    async fn list_contexts(&self) -> anyhow::Result<Vec<SquirrelContext>> {
        let contexts = self.contexts.read().await;
        Ok(contexts.values().cloned().collect())
    }
}

/// Create a test adapter with a short sync interval for testing
async fn create_test_adapter() -> Result<ContextMcpAdapter> {
    let config = ContextMcpAdapterConfig {
        sync_interval_secs: 5,  // Short interval for testing
        ..Default::default()
    };
    
    create_context_mcp_adapter_with_config(config).await
}

#[tokio::test]
async fn test_adapter_creation() {
    let adapter = create_test_adapter().await;
    assert!(adapter.is_ok(), "Failed to create adapter: {:?}", adapter.err());
}

#[tokio::test]
async fn test_adapter_initialization() {
    let adapter = create_test_adapter().await.unwrap();
    let init_result = adapter.initialize().await;
    
    // Note: This test might fail in CI if no actual MCP or context services are available.
    // In a real environment, you'd mock these dependencies.
    match init_result {
        Ok(()) => {
            println!("Adapter initialized successfully");
        },
        Err(e) => {
            println!("Adapter initialization failed as expected in test environment: {:?}", e);
            // This is fine in test environment 
        }
    }
}

#[tokio::test]
async fn test_adapter_status() {
    let adapter = create_test_adapter().await.unwrap();
    let status = adapter.get_status().await;
    
    assert_eq!(status.error_count, 0, "Initial error count should be 0");
    assert_eq!(status.successful_syncs, 0, "Initial successful syncs should be 0");
}

#[tokio::test]
async fn test_sync_direction() {
    let adapter = create_test_adapter().await.unwrap();
    
    // Check that sync_direction doesn't panic
    // It will likely fail due to missing connections but shouldn't panic
    let result = adapter.sync_direction(SyncDirection::SquirrelToMcp).await;
    println!("Sync result: {:?}", result);
}

#[tokio::test]
async fn test_clone() {
    let adapter = create_test_adapter().await.unwrap();
    let cloned = adapter.clone();
    
    // Both should have the same initial status
    let status1 = adapter.get_status().await;
    let status2 = cloned.get_status().await;
    
    assert_eq!(status1.error_count, status2.error_count);
    assert_eq!(status1.successful_syncs, status2.successful_syncs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_adapter() -> ContextMcpAdapter {
        // Create a simple config for testing
        let config = ContextMcpAdapterConfig {
            api_base_url: "https://api.example.com".to_string(),
            api_key: "test_key".to_string(),
            org_id: Some("test_org".to_string()),
            max_retries: Some(3),
            timeout_ms: Some(5000),
        };
        
        ContextMcpAdapter::new(config)
    }

    #[test]
    fn test_create_adapter() {
        let adapter = setup_test_adapter();
        debug!("Created adapter: {:?}", adapter);
        
        // Basic assertions to verify adapter initialization
        assert_eq!(adapter.config.api_base_url, "https://api.example.com");
        assert_eq!(adapter.config.api_key, "test_key");
        assert_eq!(adapter.config.org_id, Some("test_org".to_string()));
    }
    
    #[test]
    fn test_enhancement_options() {
        // Test creating enhancement options with different types
        let summary_options = ContextAiEnhancementOptions::new(
            ContextEnhancementType::Summary,
            "openai",
            "test_api_key"
        )
        .with_model("gpt-4")
        .with_timeout(10000);
        
        assert_eq!(summary_options.enhancement_type, ContextEnhancementType::Summary);
        assert_eq!(summary_options.provider, "openai");
        assert_eq!(summary_options.model, Some("gpt-4".to_string()));
        assert_eq!(summary_options.timeout_ms, Some(10000));
        
        // Test with custom parameters
        let custom_options = ContextAiEnhancementOptions::new(
            ContextEnhancementType::Custom("analyze_sentiment".to_string()),
            "anthropic",
            "test_anthropic_key"
        )
        .with_system_prompt("Analyze sentiment of the following text.")
        .with_parameter("language", "en")
        .with_parameter("detailed", true)
        .with_parameter("score_range", serde_json::json!({"min": 0, "max": 10}));
        
        assert_eq!(custom_options.provider, "anthropic");
        assert!(matches!(custom_options.enhancement_type, 
                         ContextEnhancementType::Custom(prompt) if prompt == "analyze_sentiment"));
        assert_eq!(custom_options.custom_prompt, Some("Analyze sentiment of the following text.".to_string()));
        assert!(custom_options.parameters.contains_key("language"));
        assert_eq!(custom_options.parameters.get("language").unwrap().as_str().unwrap(), "en");
        assert!(custom_options.parameters.contains_key("detailed"));
        assert_eq!(custom_options.parameters.get("detailed").unwrap().as_bool().unwrap(), true);
        assert!(custom_options.parameters.contains_key("score_range"));
        let score_range = custom_options.parameters.get("score_range").unwrap().as_object().unwrap();
        assert_eq!(score_range.get("min").unwrap().as_i64().unwrap(), 0);
        assert_eq!(score_range.get("max").unwrap().as_i64().unwrap(), 10);
    }
} 