// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Plugin manager for context transformations and adapters
use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;

use crate::ContextError;
use squirrel_interfaces::context::{
    AdapterMetadata, ContextAdapterPlugin, ContextPlugin, ContextTransformation,
};

/// Plugin manager for managing context plugins and transformations
#[derive(Debug)]
pub struct ContextPluginManager {
    /// Collection of registered context plugins
    plugins: RwLock<Vec<Box<dyn ContextPlugin>>>,
    /// Collection of available context transformations from plugins
    transformations: RwLock<Vec<Arc<dyn ContextTransformation>>>,
    /// Map of adapter IDs to context adapter plugins
    adapters: RwLock<HashMap<String, Arc<dyn ContextAdapterPlugin>>>,
}

impl ContextPluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(Vec::new()),
            transformations: RwLock::new(Vec::new()),
            adapters: RwLock::new(HashMap::new()),
        }
    }

    /// Register a plugin
    pub async fn register_plugin(
        &self,
        plugin: Box<dyn ContextPlugin>,
    ) -> Result<(), ContextError> {
        // Get the plugin's transformations
        let transformations = plugin.get_transformations().await;

        // Get the plugin's adapters
        let adapters = plugin.get_adapters().await;

        // Register transformations
        {
            let mut transformations_lock = self.transformations.write().await;
            for transformation in transformations {
                transformations_lock.push(transformation);
            }
        }

        // Register adapters
        {
            let mut adapters_lock = self.adapters.write().await;
            for adapter in adapters {
                let metadata = adapter.get_metadata().await;
                adapters_lock.insert(metadata.id.clone(), adapter);
            }
        }

        // Store the plugin
        self.plugins.write().await.push(plugin);

        Ok(())
    }

    /// Load plugins from a directory path
    pub async fn load_plugins_from_path(&self, _path: &str) -> Result<(), ContextError> {
        // This is a placeholder for dynamic plugin loading
        // We'll need to implement this when we have actual plugins to load
        #[cfg(feature = "with-plugins")]
        {
            // Load plugins from the path using libloading or similar
            // This would be implemented in a separate module when the feature is enabled
            // For now, we'll just return Ok since this is a placeholder
        }

        Ok(())
    }

    /// Get all registered transformations
    pub async fn get_transformations(&self) -> Vec<Arc<dyn ContextTransformation>> {
        self.transformations.read().await.clone()
    }

    /// Get all registered adapters
    pub async fn get_adapters(&self) -> HashMap<String, Arc<dyn ContextAdapterPlugin>> {
        self.adapters.read().await.clone()
    }

    /// Transform data using a registered transformation
    pub async fn transform(
        &self,
        transformation_id: &str,
        data: Value,
    ) -> Result<Value, ContextError> {
        // Find the transformation
        let transformations = self.transformations.read().await;
        let transformation = transformations
            .iter()
            .find(|t| t.get_id() == transformation_id)
            .ok_or(ContextError::TransformationNotFound(
                transformation_id.to_string(),
            ))?;

        // Apply the transformation
        transformation.transform(data).await.map_err(|e| {
            ContextError::TransformationFailed(transformation_id.to_string(), e.to_string())
        })
    }

    /// Get adapter by ID
    pub async fn get_adapter(&self, adapter_id: &str) -> Option<Arc<dyn ContextAdapterPlugin>> {
        self.adapters.read().await.get(adapter_id).cloned()
    }

    /// Get adapter metadata
    pub async fn get_adapter_metadata(
        &self,
        adapter_id: &str,
    ) -> Result<AdapterMetadata, ContextError> {
        let adapter = self
            .get_adapter(adapter_id)
            .await
            .ok_or(ContextError::AdapterNotFound(adapter_id.to_string()))?;

        Ok(adapter.get_metadata().await)
    }
}

impl Default for ContextPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default plugin manager with built-in plugins
#[cfg(feature = "with-plugins")]
pub async fn create_default_plugin_manager() -> Result<Arc<ContextPluginManager>, ContextError> {
    let manager = Arc::new(ContextPluginManager::new());

    // Register built-in plugins when the feature is enabled
    // This would be implemented when we have actual plugins

    Ok(manager)
}
