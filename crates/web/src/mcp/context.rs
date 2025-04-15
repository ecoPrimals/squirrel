use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// MCP Context for message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    pub user_id: String,
    pub request_id: String,
    pub timestamp: chrono::DateTime<Utc>,
    // Additional context fields for improved context preservation
    pub session_id: Option<String>,
    pub source: Option<String>,
    pub correlation_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Context updates structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContextUpdates {
    pub metadata: Option<serde_json::Value>,
    pub correlation_id: Option<String>,
}

/// Expanded context manager for improved context preservation
pub struct ContextManager {
    // Cache for context storage
    context_cache: Arc<RwLock<HashMap<String, McpContext>>>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self {
            context_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new context
    pub async fn create_context(
        &self,
        user_id: String,
        session_id: Option<String>,
        source: Option<String>,
        correlation_id: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> McpContext {
        let context = McpContext {
            user_id,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            session_id,
            source,
            correlation_id,
            metadata,
        };
        
        // Store in cache
        let request_id = context.request_id.clone();
        self.context_cache.write().await.insert(request_id, context.clone());
        
        context
    }
    
    /// Get context by request ID
    pub async fn get_context(&self, request_id: &str) -> Option<McpContext> {
        self.context_cache.read().await.get(request_id).cloned()
    }
    
    /// Update context
    pub async fn update_context(
        &self,
        request_id: &str,
        updates: McpContextUpdates,
    ) -> Option<McpContext> {
        let mut contexts = self.context_cache.write().await;
        
        if let Some(context) = contexts.get_mut(request_id) {
            // Apply updates
            if let Some(metadata) = updates.metadata {
                context.metadata = Some(metadata);
            }
            
            if let Some(correlation_id) = updates.correlation_id {
                context.correlation_id = Some(correlation_id);
            }
            
            return Some(context.clone());
        }
        
        None
    }
    
    /// Remove context
    pub async fn remove_context(&self, request_id: &str) -> Option<McpContext> {
        self.context_cache.write().await.remove(request_id)
    }
} 