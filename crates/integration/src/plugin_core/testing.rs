//! Mock implementations for testing the PluginCoreAdapter
//!
//! This module provides mock implementations to test the adapter.

use std::sync::{Arc, RwLock};
use std::any::Any;
use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result as AnyhowResult;

use squirrel_plugins::{
    Plugin, PluginMetadata, PluginStatus
};

/// A mock plugin for testing
#[derive(Debug)]
pub struct MockPlugin {
    /// Metadata for the plugin
    pub metadata: PluginMetadata,
    /// Current status of the plugin
    status: RwLock<PluginStatus>,
    /// Whether initialization should fail
    fail_init: bool,
    /// Whether shutdown should fail
    fail_shutdown: bool,
}

impl MockPlugin {
    /// Creates a new mock plugin
    pub fn new(id: Uuid, name: &str, version: &str) -> Self {
        Self {
            metadata: PluginMetadata {
                id,
                name: name.to_string(),
                version: version.to_string(),
                author: "Test Author".to_string(),
                description: "Test Plugin".to_string(),
                capabilities: vec![],
                dependencies: vec![],
            },
            status: RwLock::new(PluginStatus::Registered),
            fail_init: false,
            fail_shutdown: false,
        }
    }
    
    /// Creates a new mock plugin with a random ID
    pub fn new_random(name: &str, version: &str) -> Self {
        Self::new(Uuid::new_v4(), name, version)
    }
    
    /// Sets whether initialization should fail
    pub fn with_init_failure(mut self, fail: bool) -> Self {
        self.fail_init = fail;
        self
    }
    
    /// Sets whether shutdown should fail
    pub fn with_shutdown_failure(mut self, fail: bool) -> Self {
        self.fail_shutdown = fail;
        self
    }
}

#[async_trait]
impl Plugin for MockPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> AnyhowResult<()> {
        if self.fail_init {
            return Err(anyhow::anyhow!("Mock initialization failure"));
        }
        let mut status = self.status.write().unwrap();
        *status = PluginStatus::Initialized;
        Ok(())
    }
    
    async fn shutdown(&self) -> AnyhowResult<()> {
        if self.fail_shutdown {
            return Err(anyhow::anyhow!("Mock shutdown failure"));
        }
        let mut status = self.status.write().unwrap();
        *status = PluginStatus::Registered; // No Shutdown status in the enum
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Creates a test plugin with the given ID
pub fn create_test_plugin(id: Uuid) -> Arc<MockPlugin> {
    Arc::new(MockPlugin::new(
        id,
        "Test Plugin",
        "1.0.0",
    ))
}

/// Creates a test plugin with a random ID
pub fn create_random_test_plugin() -> Arc<MockPlugin> {
    Arc::new(MockPlugin::new_random(
        "Test Plugin",
        "1.0.0",
    ))
}

/// Create multiple test plugins for testing
pub fn create_mock_plugin_db(count: usize) -> Vec<Arc<MockPlugin>> {
    let mut plugins = Vec::with_capacity(count);
    
    for i in 0..count {
        plugins.push(Arc::new(MockPlugin::new_random(
            &format!("Test Plugin {}", i),
            "1.0.0",
        )));
    }
    
    plugins
} 