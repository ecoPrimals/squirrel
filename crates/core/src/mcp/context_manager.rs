use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{error, info, instrument, warn};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::mcp::sync::state::{StateOperation, StateChange};
use crate::mcp::sync::{MCPSync, SyncConfig};
use std::sync::Arc;
use std::default::Default;
use crate::mcp::persistence::{MCPPersistence, PersistenceConfig};
use crate::mcp::monitoring::MCPMonitor;
use crate::mcp::sync::state::StateSyncManager;

/// Errors that can occur during context operations
#[derive(Debug, Error)]
pub enum ContextError {
    /// Error when a context with the specified ID is not found
    #[error("Context not found: {0}")]
    NotFound(Uuid),

    /// Error when context data is invalid or malformed
    #[error("Invalid context data: {0}")]
    InvalidData(String),

    /// Error when context fails validation against rules
    #[error("Context validation error: {0}")]
    ValidationError(String),

    /// Error when synchronization of context data fails
    #[error("Context sync error: {0}")]
    SyncError(String),

    /// Error from JSON serialization/deserialization
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

/// Context representation in the MCP system
///
/// A Context is the primary data structure in the MCP system, representing
/// a piece of contextual information that can be synchronized across instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Unique identifier for the context
    pub id: Uuid,
    /// Human-readable name for the context
    pub name: String,
    /// Primary data content of the context
    pub data: serde_json::Value,
    /// Optional metadata associated with the context
    pub metadata: Option<serde_json::Value>,
    /// Optional parent context ID, for hierarchical relationships
    pub parent_id: Option<Uuid>,
    /// Timestamp when the context was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the context was last updated
    pub updated_at: DateTime<Utc>,
    /// Optional timestamp when the context should expire
    pub expires_at: Option<DateTime<Utc>>,
}

/// Validation rules and schema for context data
///
/// Contains the JSON schema and validation rules that are applied
/// to context data to ensure validity and consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextValidation {
    /// JSON schema for validating context data structure
    pub schema: serde_json::Value,
    /// List of validation rule identifiers to apply
    pub rules: Vec<String>,
}

/// Manager for context operations and synchronization
///
/// The `ContextManager` is responsible for creating, updating, deleting, and
/// validating contexts, as well as managing their hierarchical relationships
/// and synchronization across distributed instances.
#[derive(Debug)]
pub struct ContextManager {
    /// Map of context IDs to Context instances
    contexts: Arc<RwLock<HashMap<Uuid, Context>>>,
    /// Map of parent IDs to child IDs, representing the context hierarchy
    hierarchy: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    /// Map of context types to validation rules
    validations: Arc<RwLock<HashMap<String, ContextValidation>>>,
    /// Synchronization engine for distributed context operations
    sync: Arc<MCPSync>,
}

impl ContextManager {
    /// Creates a new context manager with default configuration
    ///
    /// Initializes the context manager with a default sync configuration
    /// and creates necessary dependencies like persistence, monitoring,
    /// and state management.
    #[instrument]
    pub async fn new() -> Self {
        let sync_config = SyncConfig::default();
        
        // Create required dependencies
        let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
        let monitor = Arc::new(MCPMonitor::default());
        let state_manager = Arc::new(StateSyncManager::new());
        
        let sync = Arc::new(MCPSync::new(
            sync_config,
            persistence,
            monitor,
            state_manager
        ));

        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            sync,
        }
    }

    /// Creates a new context in the system
    ///
    /// Validates the context data according to registered validation rules,
    /// updates the context hierarchy if a parent is specified, and triggers
    /// synchronization of the context across distributed instances.
    ///
    /// # Errors
    ///
    /// Returns `ContextError::ValidationError` if the context fails validation.
    /// Returns `ContextError::SyncError` if synchronization fails.
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
        contexts.insert(context_id, context.clone());

        // Record change for sync
        if let Err(e) = self.sync.record_context_change(&context, StateOperation::Create).await {
            error!("Failed to record context change: {}", e);
            return Err(ContextError::SyncError(e.to_string()));
        }

        info!(context_id = %context_id, "Context created");
        Ok(context_id)
    }

    /// Retrieves a context by its ID
    ///
    /// # Errors
    ///
    /// Returns `ContextError::NotFound` if the context does not exist.
    #[instrument(skip(self))]
    pub async fn get_context(&self, id: Uuid) -> Result<Context, ContextError> {
        let contexts = self.contexts.read().await;
        contexts
            .get(&id)
            .cloned()
            .ok_or(ContextError::NotFound(id))
    }

    /// Updates an existing context's data and metadata
    ///
    /// Updates the context with new data and metadata and triggers
    /// synchronization of the updated context.
    ///
    /// # Errors
    ///
    /// Returns `ContextError::NotFound` if the context does not exist.
    /// Returns `ContextError::SyncError` if synchronization fails.
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

        // Record change for sync
        if let Err(e) = self.sync.record_context_change(context, StateOperation::Update).await {
            error!("Failed to record context update: {}", e);
            return Err(ContextError::SyncError(e.to_string()));
        }

        info!(context_id = %id, "Context updated");
        Ok(())
    }

    /// Deletes a context and all its children
    ///
    /// Recursively deletes the context and all its children from the system
    /// and triggers synchronization of these deletions.
    ///
    /// # Errors
    ///
    /// Returns `ContextError::NotFound` if the context does not exist.
    /// Returns `ContextError::SyncError` if synchronization fails.
    #[instrument(skip(self))]
    pub async fn delete_context(&self, id: Uuid) -> Result<(), ContextError> {
        // Get context for sync before removal
        let context = self.get_context(id).await?;

        // Remove from contexts
        let mut contexts = self.contexts.write().await;
        contexts.remove(&id).ok_or(ContextError::NotFound(id))?;

        // Collect children to delete iteratively
        let mut children_to_delete = Vec::new();
        
        // Remove from hierarchy
        let mut hierarchy = self.hierarchy.write().await;
        if let Some(children) = hierarchy.remove(&id) {
            // Add children to the list to delete
            children_to_delete.extend(children);
        }
        
        // Release locks before deleting children
        drop(contexts);
        drop(hierarchy);

        // Delete children iteratively
        while let Some(child_id) = children_to_delete.pop() {
            // Get child context
            let child_context = match self.get_context(child_id).await {
                Ok(ctx) => ctx,
                Err(e) => {
                    error!("Failed to get child context {}: {}", child_id, e);
                    continue;
                }
            };
            
            // Remove child from contexts
            let mut contexts = self.contexts.write().await;
            contexts.remove(&child_id);
            drop(contexts);
            
            // Remove child from hierarchy and collect its children
            let mut hierarchy = self.hierarchy.write().await;
            if let Some(grandchildren) = hierarchy.remove(&child_id) {
                // Add grandchildren to the list to delete
                children_to_delete.extend(grandchildren);
            }
            drop(hierarchy);
            
            // Record deletion for sync
            if let Err(e) = self.sync.record_context_change(&child_context, StateOperation::Delete).await {
                error!("Failed to record child context deletion: {}", e);
            }
        }

        // Record change for sync
        if let Err(e) = self.sync.record_context_change(&context, StateOperation::Delete).await {
            error!("Failed to record context deletion: {}", e);
            return Err(ContextError::SyncError(e.to_string()));
        }

        info!(context_id = %id, "Context deleted");
        Ok(())
    }

    /// Registers validation rules for a specific context type
    ///
    /// # Errors
    ///
    /// This operation currently does not return errors, but the result
    /// type is maintained for future extensibility.
    #[instrument(skip(self))]
    pub async fn register_validation(&self, context_type: String, validation: ContextValidation) -> Result<(), ContextError> {
        let mut validations = self.validations.write().await;
        // Clone the context_type for use in the log after insert
        let context_type_clone = context_type.clone();
        validations.insert(context_type, validation);
        
        info!(context_type = %context_type_clone, "Context validation registered");
        Ok(())
    }

    #[instrument(skip(self, context))]
    async fn validate_context(&self, context: &Context) -> Result<(), ContextError> {
        let validations = self.validations.read().await;
        
        if let Some(validation) = validations.get(&context.name) {
            for rule in &validation.rules {
                // Apply each validation rule
                self.apply_validation_rule(context, rule).await?;
            }
            
            // Comment out or remove the jsonschema validation for now
            // Uncomment and add proper dependency when needed:
            // if let Err(e) = jsonschema::validate(&validation.schema, &context.data) {
            //     return Err(ContextError::ValidationError(format!("Schema validation failed: {}", e)));
            // }
        }
        
        Ok(())
    }

    #[instrument(skip(self, context))]
    async fn apply_validation_rule(&self, context: &Context, rule: &str) -> Result<(), ContextError> {
        // Implement custom validation rules
        match rule {
            "required_fields" => {
                // Example validation
                if context.data.as_object().is_none_or(serde_json::Map::is_empty) {
                    return Err(ContextError::ValidationError("Context data cannot be empty".into()));
                }
                
                // Check for required fields based on the schema
                let validations = self.validations.read().await;
                if let Some(validation) = validations.get(&context.name) {
                    if let Some(schema) = validation.schema.as_object() {
                        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
                            if let Some(_properties) = schema.get("properties").and_then(|p| p.as_object()) {
                                for req in required {
                                    if let Some(field_name) = req.as_str() {
                                        if !context.data.as_object().is_some_and(|obj| obj.contains_key(field_name)) {
                                            return Err(ContextError::ValidationError(
                                                format!("Required field '{field_name}' is missing")
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
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
                // Fall back to simple rule validator for other rules
                if !rule_validator(rule, context) {
                    return Err(ContextError::ValidationError(format!("Rule '{}' failed for context '{}'", rule, context.name)));
                }
            }
        }

        Ok(())
    }

    /// Retrieves all child contexts for a given parent ID
    ///
    /// # Errors
    ///
    /// Returns `ContextError::NotFound` if the parent context does not exist.
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

    /// Synchronizes context data with other instances
    ///
    /// Triggers a synchronization operation that sends and receives
    /// context changes to and from other distributed instances.
    ///
    /// # Errors
    ///
    /// Returns `ContextError::SyncError` if synchronization fails.
    #[instrument(skip(self))]
    pub async fn sync(&self) -> Result<(), ContextError> {
        if let Err(e) = self.sync.sync().await {
            error!("Failed to sync contexts: {}", e);
            return Err(ContextError::SyncError(e.to_string()));
        }
        Ok(())
    }

    /// Subscribes to context change notifications
    ///
    /// Returns a receiver that will be notified of all context changes,
    /// allowing reactive handling of context updates.
    #[instrument(skip(self))]
    pub async fn subscribe_changes(&self) -> tokio::sync::broadcast::Receiver<StateChange> {
        self.sync.subscribe_changes().await.expect("Failed to subscribe to changes")
    }
}

/// Validates if a rule applies to a context
///
/// This function checks whether a given rule applies to a specific context
/// based on its properties.
///
/// # Arguments
/// * `rule` - The rule string to validate
/// * `context` - The context to validate against
///
/// # Returns
/// `true` if the rule applies to the context, `false` otherwise
fn rule_validator(rule: &str, context: &Context) -> bool {
    // This is a placeholder implementation - add real validation logic as needed
    match rule {
        "has_id" => context.id != Uuid::nil(),
        "has_name" => !context.name.is_empty(),
        "has_data" => !context.data.is_null(),
        _ => {
            warn!("Unknown validation rule: {}", rule);
            true // Default to passing for unknown rules
        }
    }
}

// Add a Default implementation for ContextManager
impl Default for ContextManager {
    fn default() -> Self {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { Self::new().await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_lifecycle() {
        // Use the helper to create a pre-initialized MCPSync instance
        let sync_config = SyncConfig::default();
        let sync = crate::mcp::sync::create_mcp_sync(sync_config).await.expect("Failed to create MCPSync");
        
        // Create ContextManager with pre-initialized sync
        let manager = ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            sync: Arc::new(sync),
        };
        
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

        // Create
        let id = manager.create_context(context.clone()).await.unwrap();
        assert_eq!(id, context.id);

        // Get
        let retrieved = manager.get_context(id).await.unwrap();
        assert_eq!(retrieved.id, context.id);

        // Update
        let new_data = serde_json::json!({"key": "new_value"});
        assert!(manager.update_context(id, new_data.clone(), None).await.is_ok());

        // Verify update
        let updated = manager.get_context(id).await.unwrap();
        assert_eq!(updated.data, new_data);

        // Delete
        assert!(manager.delete_context(id).await.is_ok());

        // Verify deletion
        assert!(manager.get_context(id).await.is_err());
    }

    #[tokio::test]
    async fn test_context_hierarchy() {
        // Use the helper to create a pre-initialized MCPSync instance
        let sync_config = SyncConfig::default();
        let sync = crate::mcp::sync::create_mcp_sync(sync_config).await.expect("Failed to create MCPSync");
        
        // Create ContextManager with pre-initialized sync
        let manager = ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            sync: Arc::new(sync),
        };
        
        let parent_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();

        // Create parent
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
        assert!(manager.create_context(parent).await.is_ok());

        // Create child
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
        assert!(manager.create_context(child).await.is_ok());

        // Get children
        let children = manager.get_child_contexts(parent_id).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child_id);
    }

    #[tokio::test]
    async fn test_context_validation() {
        // Use the helper to create a pre-initialized MCPSync instance
        let sync_config = SyncConfig::default();
        let sync = crate::mcp::sync::create_mcp_sync(sync_config).await.expect("Failed to create MCPSync");
        
        // Create ContextManager with pre-initialized sync
        let manager = ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            sync: Arc::new(sync),
        };
        
        // Register validation
        let validation = ContextValidation {
            schema: serde_json::json!({
                "type": "object",
                "required": ["key"],
                "properties": {
                    "key": {"type": "string"}
                }
            }),
            rules: vec!["required_fields".to_string()],
        };
        assert!(manager.register_validation("test".to_string(), validation).await.is_ok());

        // Test valid context
        let valid_context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({"key": "value"}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        assert!(manager.create_context(valid_context).await.is_ok());

        // Test invalid context
        let invalid_context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        assert!(manager.create_context(invalid_context).await.is_err());
    }
} 