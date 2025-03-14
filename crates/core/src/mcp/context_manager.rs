use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{error, info, instrument, warn};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Context not found: {0}")]
    NotFound(Uuid),

    #[error("Invalid context data: {0}")]
    InvalidData(String),

    #[error("Context validation error: {0}")]
    ValidationError(String),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: Uuid,
    pub name: String,
    pub data: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextValidation {
    pub schema: serde_json::Value,
    pub rules: Vec<String>,
}

#[derive(Debug)]
pub struct ContextManager {
    contexts: RwLock<HashMap<Uuid, Context>>,
    validations: RwLock<HashMap<String, ContextValidation>>,
    hierarchy: RwLock<HashMap<Uuid, Vec<Uuid>>>,
}

impl Clone for ContextManager {
    fn clone(&self) -> Self {
        Self {
            contexts: RwLock::new(HashMap::new()),
            validations: RwLock::new(HashMap::new()),
            hierarchy: RwLock::new(HashMap::new()),
        }
    }
}

impl ContextManager {
    #[instrument]
    pub fn new() -> Self {
        info!("Initializing MCP context manager");
        
        Self {
            contexts: RwLock::new(HashMap::new()),
            validations: RwLock::new(HashMap::new()),
            hierarchy: RwLock::new(HashMap::new()),
        }
    }

    #[instrument(skip(self, context))]
    pub async fn create_context(&self, context: Context) -> Result<Uuid, ContextError> {
        // Validate context data
        self.validate_context(&context).await?;

        let context_id = context.id;
        
        // Update hierarchy if parent exists
        if let Some(parent_id) = context.parent_id {
            let mut hierarchy = self.hierarchy.write().await;
            hierarchy
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(context_id);
        }

        // Store context
        let mut contexts = self.contexts.write().await;
        contexts.insert(context_id, context);

        info!(context_id = %context_id, "Context created");
        Ok(context_id)
    }

    #[instrument(skip(self))]
    pub async fn get_context(&self, id: Uuid) -> Result<Context, ContextError> {
        let contexts = self.contexts.read().await;
        contexts
            .get(&id)
            .cloned()
            .ok_or(ContextError::NotFound(id))
    }

    #[instrument(skip(self, data))]
    pub async fn update_context(
        &self,
        id: Uuid,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), ContextError> {
        let mut contexts = self.contexts.write().await;
        
        let context = contexts
            .get_mut(&id)
            .ok_or(ContextError::NotFound(id))?;

        context.data = data;
        context.metadata = metadata;
        context.updated_at = Utc::now();

        info!(context_id = %id, "Context updated");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete_context(&self, id: Uuid) -> Result<(), ContextError> {
        // Remove from contexts
        let mut contexts = self.contexts.write().await;
        contexts.remove(&id).ok_or(ContextError::NotFound(id))?;

        // Remove from hierarchy
        let mut hierarchy = self.hierarchy.write().await;
        if let Some(children) = hierarchy.remove(&id) {
            // Recursively delete child contexts
            for child_id in children {
                self.delete_context(child_id).await?;
            }
        }

        info!(context_id = %id, "Context deleted");
        Ok(())
    }

    #[instrument(skip(self, validation))]
    pub async fn register_validation(
        &self,
        context_type: String,
        validation: ContextValidation,
    ) -> Result<(), ContextError> {
        let mut validations = self.validations.write().await;
        validations.insert(context_type, validation);
        
        info!(context_type = %context_type, "Context validation registered");
        Ok(())
    }

    #[instrument(skip(self, context))]
    async fn validate_context(&self, context: &Context) -> Result<(), ContextError> {
        let validations = self.validations.read().await;
        
        if let Some(validation) = validations.get(&context.name) {
            // Validate against JSON schema
            if let Err(e) = jsonschema::validate(&validation.schema, &context.data) {
                return Err(ContextError::ValidationError(e.to_string()));
            }

            // Apply custom validation rules
            for rule in &validation.rules {
                self.apply_validation_rule(context, rule).await?;
            }
        }

        Ok(())
    }

    #[instrument(skip(self, context))]
    async fn apply_validation_rule(&self, context: &Context, rule: &str) -> Result<(), ContextError> {
        // Implement custom validation rules
        match rule {
            "required_fields" => {
                // Example validation
                if context.data.as_object().map_or(true, |obj| obj.is_empty()) {
                    return Err(ContextError::ValidationError("Context data cannot be empty".into()));
                }
            }
            "expiration_check" => {
                if let Some(expires_at) = context.expires_at {
                    if expires_at < Utc::now() {
                        return Err(ContextError::ValidationError("Context has expired".into()));
                    }
                }
            }
            _ => {
                warn!(rule = %rule, "Unknown validation rule");
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_child_contexts(&self, parent_id: Uuid) -> Result<Vec<Context>, ContextError> {
        let hierarchy = self.hierarchy.read().await;
        let contexts = self.contexts.read().await;

        Ok(hierarchy
            .get(&parent_id)
            .map(|children| {
                children
                    .iter()
                    .filter_map(|id| contexts.get(id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() {
        let manager = ContextManager::new();
        let context = Context {
            id: Uuid::new_v4(),
            name: "test_context".to_string(),
            data: serde_json::json!({"key": "value"}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        assert!(manager.create_context(context).await.is_ok());
    }

    #[tokio::test]
    async fn test_context_update() {
        let manager = ContextManager::new();
        let context_id = Uuid::new_v4();
        let context = Context {
            id: context_id,
            name: "test_context".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        manager.create_context(context).await.unwrap();
        let new_data = serde_json::json!({"updated": "value"});
        assert!(manager.update_context(context_id, new_data, None).await.is_ok());
    }

    #[tokio::test]
    async fn test_context_hierarchy() {
        let manager = ContextManager::new();
        let parent_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();

        let parent = Context {
            id: parent_id,
            name: "parent".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        let child = Context {
            id: child_id,
            name: "child".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: Some(parent_id),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        manager.create_context(parent).await.unwrap();
        manager.create_context(child).await.unwrap();

        let children = manager.get_child_contexts(parent_id).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child_id);
    }
} 