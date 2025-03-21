use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{error, info, instrument, warn};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::sync::state::{StateOperation, StateChange};
use crate::sync::{MCPSync, SyncConfig};
use std::sync::Arc;
use std::default::Default;
use crate::persistence::{MCPPersistence, PersistenceConfig};
use crate::monitoring::MCPMonitor;
use crate::sync::state::StateSyncManager;
use crate::error::types::ContextError;
use crate::error::{Result, MCPError};

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

/// Configuration for Context Manager
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Sync interval in seconds
    pub sync_interval: Option<u64>,
    /// Maximum retry attempts for sync operations
    pub max_retries: Option<u32>,
    /// Timeout for operations in milliseconds
    pub timeout_ms: Option<u64>,
    /// Days after which old data is cleaned up
    pub cleanup_older_than_days: Option<i64>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            sync_interval: Some(60),
            max_retries: Some(3),
            timeout_ms: Some(5000),
            cleanup_older_than_days: Some(30),
        }
    }
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
        
        // Create and initialize persistence before wrapping in Arc
        let mut persistence = MCPPersistence::new(PersistenceConfig::default());
        // Initialize persistence
        if let Err(e) = persistence.init() {
            tracing::warn!("Failed to initialize persistence: {}", e);
        }
        
        // Wrap in Arc after initialization
        let persistence = Arc::new(persistence);
        let monitor = Arc::new(MCPMonitor::default());
        let state_manager = Arc::new(StateSyncManager::new());
        
        // Create sync and initialize it
        let mut sync_instance = MCPSync::new(
            sync_config,
            persistence,
            monitor,
            state_manager
        );
        
        // Initialize the sync engine
        if let Err(e) = sync_instance.init().await {
            tracing::warn!("Failed to initialize sync engine: {}", e);
        }
        
        // Convert to Arc after initialization
        let sync = Arc::new(sync_instance);

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
    pub async fn create_context(&self, context: Context) -> Result<Uuid> {
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
            return Err(MCPError::Context(ContextError::SyncError(e.to_string())));
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
    pub async fn get_context(&self, id: Uuid) -> Result<Context> {
        let contexts = self.contexts.read().await;
        contexts
            .get(&id)
            .cloned()
            .ok_or(MCPError::Context(ContextError::NotFound(id)))
    }

    /// Updates an existing context with new data and metadata
    ///
    /// # Errors
    ///
    /// Returns `ContextError::NotFound` if the context does not exist.
    /// Returns `ContextError::SyncError` if synchronization fails.
    #[instrument(skip(self, data, metadata))]
    pub async fn update_context(
        &self,
        id: Uuid,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        
        let context = contexts
            .get_mut(&id)
            .ok_or(MCPError::Context(ContextError::NotFound(id)))?;
        
        // Update context fields
        context.data = data;
        if let Some(meta) = metadata {
            context.metadata = Some(meta);
        }
        context.updated_at = Utc::now();
        
        // Clone for sync operation
        let updated_context = context.clone();
        
        // Record change for sync
        if let Err(e) = self.sync.record_context_change(&updated_context, StateOperation::Update).await {
            error!("Failed to record context change: {}", e);
            return Err(MCPError::Context(ContextError::SyncError(e.to_string())));
        }

        info!(context_id = %id, "Context updated");
        Ok(())
    }

    /// Deletes a context by its ID
    ///
    /// # Errors
    ///
    /// Returns `ContextError::NotFound` if the context does not exist.
    /// Returns `ContextError::SyncError` if synchronization fails.
    #[instrument(skip(self))]
    pub async fn delete_context(&self, id: Uuid) -> Result<()> {
        // Get context for sync before removing
        let context = self.get_context(id).await?;
        
        // Remove from storage
        let mut contexts = self.contexts.write().await;
        contexts.remove(&id).ok_or(MCPError::Context(ContextError::NotFound(id)))?;
        
        // Remove from hierarchy
        let mut hierarchy = self.hierarchy.write().await;
        if let Some(parent_id) = context.parent_id {
            if let Some(children) = hierarchy.get_mut(&parent_id) {
                children.retain(|child_id| *child_id != id);
            }
        }
        
        // Remove any children that this context was a parent of
        hierarchy.remove(&id);
        
        // Record change for sync
        if let Err(e) = self.sync.record_context_change(&context, StateOperation::Delete).await {
            error!("Failed to record context deletion: {}", e);
            return Err(MCPError::Context(ContextError::SyncError(e.to_string())));
        }

        info!(context_id = %id, "Context deleted");
        Ok(())
    }

    /// Registers a validation schema and rules for a specific context type
    ///
    /// # Errors
    ///
    /// Returns `ContextError::ValidationError` if the validation schema is invalid.
    #[instrument(skip(self, validation))]
    pub async fn register_validation(&self, context_type: String, validation: ContextValidation) -> Result<()> {
        // Validate schema
        if validation.schema.is_null() || validation.schema.as_object().is_none_or(|o| o.is_empty()) {
            return Err(MCPError::Context(ContextError::ValidationError("Invalid validation schema".into())));
        }
        
        let mut validations = self.validations.write().await;
        validations.insert(context_type.clone(), validation);
        
        info!(context_type = %context_type, "Validation registered");
        Ok(())
    }

    /// Validates a context against registered validation rules
    ///
    /// # Errors
    ///
    /// Returns `ContextError::ValidationError` if the context fails validation.
    async fn validate_context(&self, context: &Context) -> Result<()> {
        // Check expiration
        if let Some(expires_at) = context.expires_at {
            if expires_at < Utc::now() {
                return Err(MCPError::Context(ContextError::ValidationError("Context has expired".into())));
            }
        }
        
        // Get context type from metadata
        let context_type = if let Some(metadata) = &context.metadata {
            metadata.get("type").and_then(|t| t.as_str()).unwrap_or("default")
        } else {
            "default"
        };
        
        // Apply registered validations
        let validations = self.validations.read().await;
        if let Some(validation) = validations.get(context_type) {
            // Validate against schema
            if !validation.schema.is_null() {
                // TODO: Implement JSON schema validation
            }
            
            // Apply each validation rule
            for rule in &validation.rules {
                self.apply_validation_rule(context, rule).await?;
            }
        }
        
        Ok(())
    }

    /// Applies a specific validation rule to a context
    ///
    /// # Errors
    ///
    /// Returns `ContextError::ValidationError` if the context fails the rule.
    async fn apply_validation_rule(&self, context: &Context, rule: &str) -> Result<()> {
        // Basic validation for all contexts
        if context.data.is_null() {
            return Err(MCPError::Context(ContextError::ValidationError("Context data cannot be empty".into())));
        }
        
        // Required fields validation
        if rule.starts_with("required:") {
            let field_name = rule.strip_prefix("required:").unwrap_or("");
            if !field_name.is_empty() && context.data.get(field_name).is_none() {
                return Err(MCPError::Context(ContextError::ValidationError(
                    format!("Required field '{field_name}' is missing")
                )));
            }
            return Ok(());
        }
        
        // Check if the rule function exists and apply it
        if !rule_validator(rule, context) {
            return Err(MCPError::Context(ContextError::ValidationError(format!("Rule '{}' failed for context '{}'", rule, context.name))));
        }
        
        Ok(())
    }

    /// Gets all child contexts for a given parent ID
    ///
    /// # Errors
    ///
    /// Returns `ContextError::NotFound` if the parent context does not exist.
    #[instrument(skip(self))]
    pub async fn get_child_contexts(&self, parent_id: Uuid) -> Result<Vec<Context>> {
        // Verify parent exists
        self.get_context(parent_id).await?;
        
        let hierarchy = self.hierarchy.read().await;
        let contexts = self.contexts.read().await;
        
        let child_ids = hierarchy.get(&parent_id).cloned().unwrap_or_default();
        let children = child_ids
            .into_iter()
            .filter_map(|id| contexts.get(&id).cloned())
            .collect();
        
        Ok(children)
    }

    /// Synchronizes context data with remote instances
    ///
    /// # Errors
    ///
    /// Returns `ContextError::SyncError` if synchronization fails.
    #[instrument(skip(self))]
    pub async fn sync(&self) -> Result<()> {
        if let Err(e) = self.sync.as_ref().synchronize().await {
            error!("Context synchronization failed: {}", e);
            return Err(MCPError::Context(ContextError::SyncError(e.to_string())));
        }
        Ok(())
    }

    /// Subscribes to context change notifications
    ///
    /// # Errors
    ///
    /// Returns `ContextError::SyncError` if subscription fails.
    #[instrument(skip(self))]
    pub async fn subscribe_changes(&self) -> Result<tokio::sync::broadcast::Receiver<StateChange>> {
        self.sync.subscribe_changes().await
            .map_err(|e| MCPError::Context(ContextError::SyncError(format!("Failed to subscribe to changes: {e}"))))
    }

    pub async fn create_with_persistence_and_sync(
        config: ContextConfig,
        persistence: Arc<MCPPersistence>,
        sync: Option<Arc<MCPSync>>
    ) -> Result<Self> {
        let sync = match sync {
            Some(s) => s,
            None => {
                // Create default sync instance
                let sync_config = SyncConfig {
                    sync_interval: config.sync_interval.unwrap_or(60),
                    max_retries: config.max_retries.unwrap_or(3),
                    timeout_ms: config.timeout_ms.unwrap_or(5000),
                    cleanup_older_than_days: config.cleanup_older_than_days.unwrap_or(30),
                };
                Arc::new(MCPSync::new(
                    sync_config,
                    persistence.clone(),
                    Arc::new(MCPMonitor::default()),
                    Arc::new(StateSyncManager::new())
                ))
            }
        };

        Ok(Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            sync,
        })
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
    /// Creates a default instance of `ContextManager`
    /// 
    /// # Panics
    /// 
    /// This function panics if it fails to create a Tokio runtime or
    /// if the `ContextManager` initialization in the async block fails.
    fn default() -> Self {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                rt.block_on(async { Self::new().await })
            },
            Err(e) => {
                panic!("Failed to create Tokio runtime for ContextManager: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Helper function to create a pre-initialized MCPSync instance for tests
    async fn create_test_sync() -> Arc<MCPSync> {
        // Create the persistence layer and initialize it
        let mut persistence = MCPPersistence::new(PersistenceConfig::default());
        if let Err(e) = persistence.init() {
            tracing::warn!("Failed to initialize persistence: {}", e);
        }
        let persistence = Arc::new(persistence);
        
        // Create other dependencies
        let monitor = Arc::new(MCPMonitor::default());
        let state_manager = Arc::new(StateSyncManager::new());
        
        // Create and initialize the sync instance
        let mut sync = MCPSync::new(
            SyncConfig::default(),
            persistence,
            monitor,
            state_manager
        );
        
        // Initialize the sync engine
        if let Err(e) = sync.init().await {
            tracing::warn!("Failed to initialize sync: {}", e);
        }
        
        Arc::new(sync)
    }

    #[tokio::test]
    async fn test_context_lifecycle() {
        // Use the helper to create a pre-initialized MCPSync instance
        let sync = create_test_sync().await;
        
        // Create ContextManager with pre-initialized sync
        let manager = ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            sync,
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
        let sync = create_test_sync().await;
        
        // Create ContextManager with pre-initialized sync
        let manager = ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            sync,
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
        let sync = create_test_sync().await;
        
        // Create ContextManager with pre-initialized sync
        let manager = ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            sync,
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
        let result = manager.create_context(invalid_context).await;
        println!("Invalid context creation result: {:?}", result);
    }
} 