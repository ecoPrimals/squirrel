// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin Tests module
//!
//! This module provides test utilities for plugins.

use crate::commands::registry::CommandRegistry;
use crate::plugins::error::PluginError;
use crate::plugins::manager::PluginManager;
use crate::plugins::plugin::Plugin;
use async_trait::async_trait;
use squirrel_commands::Command;
use std::sync::Arc;

/// Test plugin implementation
pub struct TestPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
}

impl TestPlugin {
    /// Create a new test plugin
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}

#[async_trait]
impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> Option<&str> {
        Some("A test plugin")
    }

    async fn initialize(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn register_commands(&self, _registry: &CommandRegistry) -> Result<(), PluginError> {
        Ok(())
    }

    fn commands(&self) -> Vec<Arc<dyn Command>> {
        Vec::new()
    }

    async fn execute(&self, _args: &[String]) -> Result<String, PluginError> {
        Ok("Test command executed".to_string())
    }

    async fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Plugin manager test utilities
pub struct PluginManagerTester {
    /// Plugin manager instance
    manager: PluginManager,
}

impl PluginManagerTester {
    /// Create a new plugin manager tester
    pub fn new() -> Self {
        Self {
            manager: PluginManager::new(),
        }
    }

    /// Test plugin loading
    pub fn test_plugin_loading(&self) -> Result<(), PluginError> {
        // Test plugin loading functionality
        Ok(())
    }

    /// Test plugin security
    pub fn test_plugin_security(&self) -> Result<(), PluginError> {
        // Test plugin security functionality
        // This is a placeholder for future security testing
        Ok(())
    }

    /// Test plugin execution
    pub async fn test_plugin_execution(&mut self) -> Result<(), PluginError> {
        // Test plugin execution functionality
        let plugin = TestPlugin::new("test-plugin".to_string(), "1.0.0".to_string());

        // Test execution directly
        let result = plugin.execute(&[]).await?;
        assert_eq!(result, "Test command executed");

        Ok(())
    }
}

/// Test utilities for plugin system
pub struct PluginTestUtils;

impl PluginTestUtils {
    /// Create a test plugin manager
    pub fn create_test_manager() -> PluginManager {
        PluginManager::new()
    }

    /// Verify plugin functionality
    pub async fn verify_plugin_functionality(plugin: &dyn Plugin) -> Result<(), PluginError> {
        // Test plugin initialization
        plugin.initialize().await?;

        // Test plugin execution
        let result = plugin.execute(&[]).await?;
        assert!(!result.is_empty());

        // Test plugin cleanup
        plugin.cleanup().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = TestPlugin::new("test".to_string(), "1.0.0".to_string());
        assert_eq!(plugin.name(), "test");
        assert_eq!(plugin.version(), "1.0.0");
    }

    #[tokio::test]
    async fn test_plugin_manager_tester() {
        let mut tester = PluginManagerTester::new();
        tester.test_plugin_loading().unwrap();
        tester.test_plugin_security().unwrap();
        tester.test_plugin_execution().await.unwrap();
    }

    #[tokio::test]
    async fn test_plugin_utils() {
        let plugin = TestPlugin::new("test".to_string(), "1.0.0".to_string());
        PluginTestUtils::verify_plugin_functionality(&plugin)
            .await
            .unwrap();
    }
}
