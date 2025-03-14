use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::{MCPError, Result};
use crate::mcp::types::{ProtocolVersion, ProtocolState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub max_context_size: u64,
    pub context_timeout_ms: u64,
    pub cleanup_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
    pub metadata: serde_json::Value,
}

pub struct MCPContext {
    config: Arc<RwLock<ContextConfig>>,
    contexts: Arc<RwLock<std::collections::HashMap<String, ContextData>>>,
}

impl MCPContext {
    pub fn new(config: ContextConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            contexts: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn create_context(&self, id: String, data: serde_json::Value) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if contexts.contains_key(&id) {
            return Err(MCPError::Protocol(format!("Context already exists with id: {}", id)));
        }

        let context = ContextData {
            id: id.clone(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            data,
            metadata: serde_json::Value::Null,
        };

        contexts.insert(id, context);
        Ok(())
    }

    pub async fn get_context(&self, id: &str) -> Result<ContextData> {
        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(id)
            .ok_or_else(|| MCPError::Protocol(format!("No context found with id: {}", id)))?;
        
        context.last_accessed = chrono::Utc::now();
        Ok(context.clone())
    }

    pub async fn update_context(&self, id: &str, data: serde_json::Value) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(id)
            .ok_or_else(|| MCPError::Protocol(format!("No context found with id: {}", id)))?;
        
        context.data = data;
        context.last_accessed = chrono::Utc::now();
        Ok(())
    }

    pub async fn delete_context(&self, id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts.remove(id)
            .ok_or_else(|| MCPError::Protocol(format!("No context found with id: {}", id)))?;
        Ok(())
    }

    pub async fn update_config(&self, config: ContextConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    pub async fn get_config(&self) -> Result<ContextConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }

    pub async fn list_contexts(&self) -> Result<Vec<ContextData>> {
        let contexts = self.contexts.read().await;
        Ok(contexts.values().cloned().collect())
    }

    pub async fn cleanup_expired_contexts(&self) -> Result<()> {
        let config = self.config.read().await;
        let mut contexts = self.contexts.write().await;
        let now = chrono::Utc::now();
        
        contexts.retain(|_, context| {
            (now - context.last_accessed).num_milliseconds() < config.context_timeout_ms as i64
        });
        
        Ok(())
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_context_size: 1024 * 1024 * 10, // 10MB
            context_timeout_ms: 3600000, // 1 hour
            cleanup_interval_ms: 300000, // 5 minutes
        }
    }
}

impl Default for MCPContext {
    fn default() -> Self {
        Self::new(ContextConfig::default())
    }
} 