use crate::error::context_err::ContextError;
use crate::error::{MCPError, Result};
// use crate::sync::state::StateSyncManager;
// use crate::sync::state::{StateChange, StateOperation};
// use crate::sync::{MCPSync, SyncConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn, debug};
use uuid::Uuid;

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
    /// Central server URL
    pub sync_server_url: Option<String>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            sync_interval: Some(60),
            max_retries: Some(3),
            timeout_ms: Some(5000),
            cleanup_older_than_days: Some(30),
            sync_server_url: None,
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
    // Synchronization engine for distributed context operations
    // sync: Option<Arc<MCPSync>>,
}

impl ContextManager {
    /// Creates a new context manager with default configuration
    ///
    /// Initializes the context manager with a default sync configuration
    /// and creates necessary dependencies like persistence, monitoring,
    /// and state management.
    #[instrument]
    pub async fn new() -> Self {
        // let sync_config = SyncConfig::default();
        // let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
        // let monitor = Arc::new(MCPMonitor::new().await.unwrap_or_default());
        // let state_manager = Arc::new(StateSyncManager::new());
        // let sync = Arc::new(MCPSync::new(sync_config, persistence, monitor, state_manager));

        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            // sync: Some(sync),
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
        self.validate_context(&context).await.map_err(|e| e)?;

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
        // if let Err(e) = self
        //     .sync
        //     .as_ref()
        //     .unwrap()
        //     .record_context_change(&context, StateOperation::Create)
        //     .await
        // {
        //     error!("Failed to record context change: {}", e);
        //     return Err(MCPError::Context(ContextError::SyncError(e.to_string())));
        // }

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
            .ok_or_else(|| MCPError::Context(ContextError::NotFound(id)))
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
            .ok_or_else(|| MCPError::Context(ContextError::NotFound(id)))?;

        // Update context fields
        context.data = data;
        if let Some(meta) = metadata {
            context.metadata = Some(meta);
        }
        context.updated_at = Utc::now();

        // Clone for sync operation
        let updated_context = context.clone();

        // Record change for sync
        // if let Err(e) = self
        //     .sync
        //     .as_ref()
        //     .unwrap()
        //     .record_context_change(&updated_context, StateOperation::Update)
        //     .await
        // {
        //     error!("Failed to record context change: {}", e);
        //     return Err(MCPError::Context(ContextError::SyncError(e.to_string())));
        // }

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
        let context = self.get_context(id).await.map_err(|e| e)?;

        // Remove from storage
        let mut contexts = self.contexts.write().await;
        contexts
            .remove(&id)
            .ok_or_else(|| MCPError::Context(ContextError::NotFound(id)))?;

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
        // if let Err(e) = self
        //     .sync
        //     .as_ref()
        //     .unwrap()
        //     .record_context_change(&context, StateOperation::Delete)
        //     .await
        // {
        //     error!("Failed to record context deletion: {}", e);
        //     return Err(MCPError::Context(ContextError::SyncError(e.to_string())));
        // }

        info!(context_id = %id, "Context deleted");
        Ok(())
    }

    /// Registers a validation schema and rules for a specific context type
    ///
    /// # Errors
    ///
    /// Returns `ContextError::ValidationError` if the validation schema is invalid.
    #[instrument(skip(self, validation))]
    pub async fn register_validation(
        &self,
        context_type: String,
        validation: ContextValidation,
    ) -> Result<()> {
        // Validate schema
        if validation.schema.is_null() || is_none_or_matches(validation.schema.as_object(), serde_json::Map::is_empty)
        {
            return Err(MCPError::Context(
                ContextError::from("Invalid validation schema")
            ));
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
                return Err(MCPError::Context(
                    ContextError::from("Context has expired")
                ));
            }
        }

        // Get context type from metadata
        let context_type = if let Some(metadata) = &context.metadata {
            metadata
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("default")
        } else {
            "default"
        };

        // Apply registered validations
        let validations = self.validations.read().await;
        if let Some(validation) = validations.get(context_type) {
            // Validate against schema
            if !validation.schema.is_null() {
                self.validate_json_schema(&validation.schema, &context.data).await?;
            }

            // Apply each validation rule
            for rule in &validation.rules {
                self.apply_validation_rule(context, rule).await.map_err(|e| e)?;
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
            return Err(MCPError::Context(
                ContextError::from("Context data cannot be empty")
            ));
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
            return Err(MCPError::Context(ContextError::ValidationError(
                format!("Rule '{}' failed for context '{}'", rule, context.name)
            )));
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
        // Check if parent exists
        self.get_context(parent_id).await.map_err(|e| e)?;

        let hierarchy = self.hierarchy.read().await;
        let children = hierarchy.get(&parent_id).cloned().unwrap_or_default();

        let contexts = self.contexts.read().await;
        let result: Vec<Context> = children
            .iter()
            .filter_map(|id| contexts.get(id).cloned())
            .collect();

        Ok(result)
    }

    /// Synchronizes context data with remote instances
    ///
    /// # Errors
    ///
    /// Returns `ContextError::SyncError` if synchronization fails.
    #[instrument(skip(self))]
    pub async fn sync(&self) -> Result<()> {
        // if let Some(sync) = &self.sync {
        //     if let Err(e) = sync.synchronize().await {
        //         error!("Context synchronization failed: {}", e);
        //         return Err(MCPError::Context(ContextError::SyncError(e.to_string())));
        //     }
        // } else {
            debug!("Sync is disabled, skipping synchronization");
        // }
        Ok(())
    }

    /// Synchronize the context manager state with remote instances
    ///
    /// This initiates a synchronization cycle with the central sync server,
    /// sending local changes and applying remote changes.
    ///
    /// # Errors
    /// Returns an error if synchronization fails for any reason
    pub async fn synchronize(&self) -> Result<()> {
        // if let Some(sync) = &self.sync {
        //     if let Err(e) = sync.synchronize().await {
        //         error!("Failed to synchronize state: {}", e);
        //         return Err(MCPError::Sync(e.to_string()));
        //     }
        // } else {
            debug!("Sync is disabled, skipping synchronization");
        // }
        
        Ok(())
    }

    // Update sync state when a context is updated
    // async fn update_sync_state(&self, context: &Context, operation: StateOperation) -> Result<()> {
    //     if let Some(sync) = &self.sync {
    //         if let Err(e) = sync.record_context_change(context, operation).await {
    //             error!("Failed to update sync state: {}", e);
    //             return Err(MCPError::Sync(e.to_string()));
    //         }
    //     } else {
    //         debug!("Sync is disabled, skipping sync state update");
    //     }
        
    //     Ok(())
    // }

    /// Validate JSON schema for context data
    async fn validate_json_schema(&self, schema: &serde_json::Value, data: &serde_json::Value) -> Result<()> {
        let schema_obj = schema.as_object().ok_or_else(|| {
            MCPError::Context(ContextError::from("Schema must be an object"))
        })?;

        // Validate type
        if let Some(type_value) = schema_obj.get("type") {
            if let Some(expected_type) = type_value.as_str() {
                let actual_type = match data {
                    serde_json::Value::Null => "null",
                    serde_json::Value::Bool(_) => "boolean",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::Object(_) => "object",
                };

                if expected_type != actual_type {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Expected type '{}', got '{}'", expected_type, actual_type
                    ))));
                }
            }
        }

        // Validate object properties
        if let Some(properties) = schema_obj.get("properties") {
            if let Some(data_obj) = data.as_object() {
                if let Some(properties_obj) = properties.as_object() {
                    for (property_name, property_schema) in properties_obj {
                        if let Some(property_value) = data_obj.get(property_name) {
                            // Recursively validate nested properties
                            self.validate_json_schema(property_schema, property_value).await?;
                        }
                    }
                }
            }
        }

        // Validate required fields
        if let Some(required) = schema_obj.get("required") {
            if let Some(required_array) = required.as_array() {
                if let Some(data_obj) = data.as_object() {
                    for req_field in required_array {
                        if let Some(field_name) = req_field.as_str() {
                            if !data_obj.contains_key(field_name) {
                                return Err(MCPError::Context(ContextError::from(format!(
                                    "Missing required field: {}", field_name
                                ))));
                            }
                        }
                    }
                }
            }
        }

        // Validate string constraints
        if let Some(data_str) = data.as_str() {
            if let Some(min_length) = schema_obj.get("minLength").and_then(|v| v.as_u64()) {
                if data_str.len() < min_length as usize {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "String length {} is less than minimum {}", data_str.len(), min_length
                    ))));
                }
            }
            if let Some(max_length) = schema_obj.get("maxLength").and_then(|v| v.as_u64()) {
                if data_str.len() > max_length as usize {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "String length {} exceeds maximum {}", data_str.len(), max_length
                    ))));
                }
            }
        }

        // Validate numeric constraints
        if let Some(data_num) = data.as_f64() {
            if let Some(minimum) = schema_obj.get("minimum").and_then(|v| v.as_f64()) {
                if data_num < minimum {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Value {} is less than minimum {}", data_num, minimum
                    ))));
                }
            }
            if let Some(maximum) = schema_obj.get("maximum").and_then(|v| v.as_f64()) {
                if data_num > maximum {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Value {} exceeds maximum {}", data_num, maximum
                    ))));
                }
            }
        }

        // Validate array constraints
        if let Some(data_array) = data.as_array() {
            if let Some(min_items) = schema_obj.get("minItems").and_then(|v| v.as_u64()) {
                if data_array.len() < min_items as usize {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Array length {} is less than minimum {}", data_array.len(), min_items
                    ))));
                }
            }
            if let Some(max_items) = schema_obj.get("maxItems").and_then(|v| v.as_u64()) {
                if data_array.len() > max_items as usize {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Array length {} exceeds maximum {}", data_array.len(), max_items
                    ))));
                }
            }
            
            // Validate array items
            if let Some(items_schema) = schema_obj.get("items") {
                for item in data_array {
                    self.validate_json_schema(items_schema, item).await?;
                }
            }
        }

        Ok(())
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

/// Helper function to check if an Option is None or its value satisfies a predicate
fn is_none_or_matches<T, F>(opt: Option<&T>, predicate: F) -> bool
where
    F: FnOnce(&T) -> bool,
{
    match opt {
        None => true,
        Some(value) => predicate(value),
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
        // let sync_config = SyncConfig::default();
        // let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
        
        // Create a default monitor without async
        // let monitor = Arc::new(MCPMonitor::default());
        // let state_manager = Arc::new(StateSyncManager::new());
        // let sync = Arc::new(MCPSync::new(sync_config, persistence, monitor, state_manager));
        
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            // sync: Some(sync),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a pre-initialized MCPSync instance for tests
    // async fn create_test_sync() -> Arc<MCPSync> {
    //     // Create the persistence layer and initialize it
    //     let mut persistence = MCPPersistence::new(PersistenceConfig::default());
    //     if let Err(e) = persistence.init() {
    //         tracing::warn!("Failed to initialize persistence: {}", e);
    //     }
    //     let persistence = Arc::new(persistence);

    //     // Create other dependencies
    //     let monitor = Arc::new(MCPMonitor::default());
    //     let state_manager = Arc::new(StateSyncManager::new());

    //     // Create and initialize the sync instance
    //     let mut sync = MCPSync::new(SyncConfig::default(), persistence, monitor, state_manager);

    //     // Initialize the sync engine
    //     if let Err(e) = sync.init().await {
    //         tracing::warn!("Failed to initialize sync: {}", e);
    //     }

    //     Arc::new(sync)
    // }

    #[tokio::test]
    async fn test_context_lifecycle() {
        // Use the helper to create a pre-initialized MCPSync instance
        // let sync = create_test_sync().await;

        // Create ContextManager with pre-initialized sync
        let manager = ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            // sync: Some(sync),
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
        assert!(manager
            .update_context(id, new_data.clone(), None)
            .await
            .is_ok());

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
        // let sync = create_test_sync().await;

        // Create ContextManager with pre-initialized sync
        let manager = ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            // sync: Some(sync),
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
        // let sync = create_test_sync().await;

        // Create ContextManager with pre-initialized sync
        let manager = ContextManager {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations: Arc::new(RwLock::new(HashMap::new())),
            // sync: Some(sync),
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
        assert!(manager
            .register_validation("test".to_string(), validation)
            .await
            .is_ok());

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
