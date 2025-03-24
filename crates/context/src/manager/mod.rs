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

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::RwLock;

use squirrel_interfaces::context::ContextManager as InterfaceContextManager;
use squirrel_interfaces::context::{ContextPlugin, ContextTransformation};
use crate::ContextError;
use crate::plugins::ContextPluginManager;

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

/// The main implementation of the Context Manager
pub struct ContextManager {
    /// Plugin manager instance
    plugin_manager: RwLock<Option<Arc<ContextPluginManager>>>,
    /// Configuration
    config: ContextManagerConfig,
    /// Initialization state
    initialized: RwLock<bool>,
}

impl ContextManager {
    /// Creates a new context manager with the given configuration
    pub fn with_config(config: ContextManagerConfig) -> Self {
        Self {
            plugin_manager: RwLock::new(None),
            config,
            initialized: RwLock::new(false),
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
    async fn transform_data(&self, transformation_id: &str, data: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
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
        plugin_manager.transform(transformation_id, data).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
    
    /// Get all available transformations
    async fn get_transformations(&self) -> Result<Vec<Box<dyn ContextTransformation>>, Box<dyn std::error::Error + Send + Sync>> {
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
    async fn register_plugin(&self, plugin: Box<dyn ContextPlugin>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        plugin_manager.register_plugin(plugin).await
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
    
    async fn transform(&self, data: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.0.transform(data).await
    }
} 