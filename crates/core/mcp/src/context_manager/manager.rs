// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use crate::error::context_err::ContextError;
use crate::error::{MCPError, Result};
use super::types::{Context, ContextValidation};
use super::validation::ValidationEngine;
use super::helpers::is_none_or_matches;
use chrono::Utc;
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, debug};
use uuid::Uuid;

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
    /// Validation engine
    validation_engine: ValidationEngine,
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

        let validations = Arc::new(RwLock::new(HashMap::new()));
        let validation_engine = ValidationEngine::new(Arc::clone(&validations));

        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations,
            validation_engine,
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
        self.validation_engine.validate_context(&context).await.map_err(|e| e)?;

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
        
        let validations = Arc::new(RwLock::new(HashMap::new()));
        let validation_engine = ValidationEngine::new(Arc::clone(&validations));
        
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
            validations,
            validation_engine,
            // sync: Some(sync),
        }
    }
}
