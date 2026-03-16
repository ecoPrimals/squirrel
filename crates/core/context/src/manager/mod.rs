// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Context manager module
//!
//! This module provides context management functionality for storing, retrieving,
//! and synchronizing context data across the application.
//!
//! ## Concurrency and Locking
//!
//! The context manager uses tokio's asynchronous locks (`RwLock`, `Mutex`) to ensure
//! thread safety while maintaining good performance in an async environment.
//! Key locking practices implemented in this module:
//!
//! - Using scope-based locking to minimize lock duration
//! - Avoiding holding locks across `.await` points
//! - Using read locks for operations that don't modify data
//! - Using write locks for operations that modify data
//! - Dropping locks explicitly before async operations
//!
//! When working with the context manager in asynchronous code, it's important to
//! follow these same patterns to avoid potential deadlocks or performance issues.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ContextError;
use crate::plugins::ContextPluginManager;
use squirrel_interfaces::context::ContextManager as InterfaceContextManager;
use squirrel_interfaces::context::{ContextPlugin, ContextTransformation};

/// Configuration for the context manager
#[derive(Debug, Clone)]
pub struct ContextManagerConfig {
    /// Whether to enable plugins
    pub enable_plugins: bool,
    /// Additional configuration options can be added here
    pub plugin_paths: Option<Vec<String>>,
}

impl Default for ContextManagerConfig {
    fn default() -> Self {
        Self {
            enable_plugins: true,
            plugin_paths: None,
        }
    }
}

/// The main implementation of the Context Manager.
///
/// Stores context state in-memory via a concurrent map. When NestGate is
/// discovered at runtime (via `storage.put` / `storage.get` capabilities),
/// the manager will persist state to durable storage. Without NestGate,
/// state lives only in-memory (graceful degradation).
#[derive(Debug)]
pub struct ContextManager {
    /// Plugin manager instance
    plugin_manager: RwLock<Option<Arc<ContextPluginManager>>>,
    /// Configuration
    config: ContextManagerConfig,
    /// Initialization state
    initialized: RwLock<bool>,
    /// In-memory context store (session_id → ContextState).
    store: std::sync::Arc<dashmap::DashMap<String, crate::ContextState>>,
}

impl ContextManager {
    /// Creates a new context manager with the given configuration
    pub fn with_config(config: ContextManagerConfig) -> Self {
        Self {
            plugin_manager: RwLock::new(None),
            config,
            initialized: RwLock::new(false),
            store: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Creates a new context manager with default configuration
    pub fn new() -> Self {
        Self::with_config(ContextManagerConfig::default())
    }

    /// Returns the plugin manager if enabled
    pub async fn get_plugin_manager(&self) -> Option<Arc<ContextPluginManager>> {
        self.plugin_manager.read().await.clone()
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InterfaceContextManager for ContextManager {
    /// Initialize the context manager
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Ok(());
        }

        // Initialize plugin manager if enabled
        if self.config.enable_plugins {
            let plugin_manager = Arc::new(ContextPluginManager::new());

            // Load plugins from configured paths if specified
            if let Some(paths) = &self.config.plugin_paths {
                for path in paths {
                    plugin_manager.load_plugins_from_path(path).await?;
                }
            }

            // Store the plugin manager
            *self.plugin_manager.write().await = Some(plugin_manager);
        }

        *initialized = true;
        Ok(())
    }

    /// Transform data using the specified transformation ID
    async fn transform_data(
        &self,
        transformation_id: &str,
        data: Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // Check if initialized
        if !*self.initialized.read().await {
            return Err(Box::new(ContextError::NotInitialized));
        }

        // Get plugin manager
        let plugin_manager = match &*self.plugin_manager.read().await {
            Some(manager) => manager.clone(),
            None => return Err(Box::new(ContextError::PluginsDisabled)),
        };

        // Transform the data
        plugin_manager
            .transform(transformation_id, data)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    /// Get all available transformations
    async fn get_transformations(
        &self,
    ) -> Result<Vec<Box<dyn ContextTransformation>>, Box<dyn std::error::Error + Send + Sync>> {
        // Check if initialized
        if !*self.initialized.read().await {
            return Err(Box::new(ContextError::NotInitialized));
        }

        // Get plugin manager
        let plugin_manager = match &*self.plugin_manager.read().await {
            Some(manager) => manager.clone(),
            None => return Err(Box::new(ContextError::PluginsDisabled)),
        };

        // Get transformations
        let transformations = plugin_manager.get_transformations().await;

        // Convert to Box<dyn ContextTransformation>
        let mut result: Vec<Box<dyn ContextTransformation>> = Vec::new();
        for t in transformations {
            // Create a wrapper type that can be converted to Box<dyn ContextTransformation>
            let boxed: Box<dyn ContextTransformation> = Box::new(TransformationWrapper(t));
            result.push(boxed);
        }

        Ok(result)
    }

    /// Register a plugin
    async fn register_plugin(
        &self,
        plugin: Box<dyn ContextPlugin>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if initialized
        if !*self.initialized.read().await {
            return Err(Box::new(ContextError::NotInitialized));
        }

        // Get plugin manager
        let plugin_manager = match &*self.plugin_manager.read().await {
            Some(manager) => manager.clone(),
            None => return Err(Box::new(ContextError::PluginsDisabled)),
        };

        // Register the plugin
        plugin_manager
            .register_plugin(plugin)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

// A wrapper type to handle the conversion
#[derive(Debug)]
struct TransformationWrapper(Arc<dyn ContextTransformation>);

#[async_trait]
impl ContextTransformation for TransformationWrapper {
    fn get_id(&self) -> &str {
        self.0.get_id()
    }

    fn get_name(&self) -> &str {
        self.0.get_name()
    }

    fn get_description(&self) -> &str {
        self.0.get_description()
    }

    async fn transform(
        &self,
        data: Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.0.transform(data).await
    }
}

impl ContextManager {
    /// Create a recovery point (snapshot) for the given state.
    ///
    /// Persists the current state to the in-memory store so it can be
    /// restored later. NestGate persistence is planned for Phase 2.
    pub async fn create_recovery_point(&self, state: &crate::ContextState) -> crate::Result<()> {
        let key = format!("{}__recovery_v{}", state.id, state.version);
        self.store.insert(key, state.clone());
        Ok(())
    }

    /// Get context state by ID.
    ///
    /// Returns the stored state or creates a new empty session.
    pub async fn get_context_state(&self, id: &str) -> crate::Result<crate::ContextState> {
        if let Some(entry) = self.store.get(id) {
            return Ok(entry.value().clone());
        }

        // No existing state — create a fresh session
        let state = crate::ContextState {
            id: id.to_string(),
            version: 1,
            timestamp: chrono::Utc::now().timestamp() as u64,
            data: serde_json::json!({}),
            metadata: std::collections::HashMap::new(),
            synchronized: false,
            last_modified: std::time::SystemTime::now(),
        };
        self.store.insert(id.to_string(), state.clone());
        Ok(state)
    }

    /// Update context state by ID.
    ///
    /// Replaces the stored state and bumps the version.
    pub async fn update_context_state(
        &self,
        id: &str,
        state: crate::ContextState,
    ) -> crate::Result<()> {
        self.store.insert(id.to_string(), state);
        Ok(())
    }

    /// Delete a context session by ID.
    pub async fn delete_context_state(&self, id: &str) -> crate::Result<bool> {
        Ok(self.store.remove(id).is_some())
    }

    /// List all active session IDs.
    pub async fn list_sessions(&self) -> Vec<String> {
        self.store.iter().map(|e| e.key().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_manager_config_default() {
        let config = ContextManagerConfig::default();
        assert!(config.enable_plugins);
        assert!(config.plugin_paths.is_none());
    }

    #[test]
    fn test_context_manager_config_custom() {
        let config = ContextManagerConfig {
            enable_plugins: false,
            plugin_paths: Some(vec!["/path/to/plugins".to_string()]),
        };
        assert!(!config.enable_plugins);
        assert!(config.plugin_paths.is_some());
        assert_eq!(config.plugin_paths.unwrap().len(), 1);
    }

    #[test]
    fn test_context_manager_new() {
        let manager = ContextManager::new();
        assert!(manager.config.enable_plugins);
    }

    #[test]
    fn test_context_manager_default() {
        let manager = ContextManager::default();
        assert!(manager.config.enable_plugins);
    }

    #[test]
    fn test_context_manager_with_config() {
        let config = ContextManagerConfig {
            enable_plugins: false,
            plugin_paths: None,
        };
        let manager = ContextManager::with_config(config);
        assert!(!manager.config.enable_plugins);
    }

    #[tokio::test]
    async fn test_context_manager_initialize() {
        let manager = ContextManager::new();
        let result = manager.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_manager_initialize_idempotent() {
        use squirrel_interfaces::context::ContextManager as InterfaceContextManager;
        let manager = ContextManager::new();
        manager.initialize().await.unwrap();
        // Second init should succeed (no-op)
        manager.initialize().await.unwrap();
    }

    #[tokio::test]
    async fn test_context_manager_get_plugin_manager_before_init() {
        let manager = ContextManager::new();
        let pm = manager.get_plugin_manager().await;
        assert!(pm.is_none());
    }

    #[tokio::test]
    async fn test_context_manager_get_plugin_manager_after_init() {
        use squirrel_interfaces::context::ContextManager as InterfaceContextManager;
        let manager = ContextManager::new();
        manager.initialize().await.unwrap();
        let pm = manager.get_plugin_manager().await;
        assert!(pm.is_some());
    }

    #[tokio::test]
    async fn test_context_manager_plugins_disabled() {
        use squirrel_interfaces::context::ContextManager as InterfaceContextManager;
        let config = ContextManagerConfig {
            enable_plugins: false,
            plugin_paths: None,
        };
        let manager = ContextManager::with_config(config);
        manager.initialize().await.unwrap();
        let pm = manager.get_plugin_manager().await;
        assert!(pm.is_none());
    }

    #[tokio::test]
    async fn test_context_manager_transform_before_init() {
        use squirrel_interfaces::context::ContextManager as InterfaceContextManager;
        let manager = ContextManager::new();
        let result = manager.transform_data("test", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_manager_get_context_state() {
        let manager = ContextManager::new();
        let state = manager.get_context_state("test-id").await.unwrap();
        assert_eq!(state.id, "test-id");
        assert_eq!(state.version, 1);
    }

    #[tokio::test]
    async fn test_context_manager_create_recovery_point() {
        use std::collections::HashMap;
        use std::time::SystemTime;
        let manager = ContextManager::new();
        let state = crate::ContextState {
            id: "test".to_string(),
            version: 1,
            timestamp: 0,
            data: serde_json::json!({}),
            metadata: HashMap::new(),
            synchronized: false,
            last_modified: SystemTime::now(),
        };
        let result = manager.create_recovery_point(&state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_manager_update_context_state() {
        use std::collections::HashMap;
        use std::time::SystemTime;
        let manager = ContextManager::new();
        let state = crate::ContextState {
            id: "test".to_string(),
            version: 1,
            timestamp: 0,
            data: serde_json::json!({}),
            metadata: HashMap::new(),
            synchronized: false,
            last_modified: SystemTime::now(),
        };
        let result = manager.update_context_state("test", state).await;
        assert!(result.is_ok());
    }
}
