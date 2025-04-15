//! Mock Plugin Database for E2E Testing
//!
//! This module provides a mock plugin database for end-to-end testing and stress testing.

use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs::{self, File, create_dir_all};
use std::io::Write;
use std::any::Any;
use anyhow::Result;
use uuid::Uuid;
use async_trait::async_trait;
use serde_json::json;

use squirrel_plugins::{Plugin, PluginMetadata, PluginStatus};

/// A mock plugin implementation for testing
#[derive(Debug)]
pub struct MockPlugin {
    /// Metadata for the plugin
    metadata: PluginMetadata,
    /// Current status of the plugin
    status: PluginStatus,
    /// The capabilities this plugin has
    capabilities: Vec<String>,
    /// Whether the plugin should fail on initialize
    fail_on_initialize: bool,
    /// Whether the plugin should fail on shutdown
    fail_on_shutdown: bool,
    /// Resource usage simulation (memory in MB)
    memory_usage: u64,
    /// CPU usage simulation (percentage)
    cpu_usage: u8,
}

impl MockPlugin {
    /// Create a new mock plugin
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version: version.to_string(),
                description: format!("Mock plugin {}", name),
                author: "Testing Framework".to_string(),
                capabilities: vec![],
                dependencies: vec![],
            },
            status: PluginStatus::Registered,
            capabilities: vec![],
            fail_on_initialize: false,
            fail_on_shutdown: false,
            memory_usage: 10,
            cpu_usage: 5,
        }
    }
    
    /// Configure the plugin to fail on initialize
    pub fn fail_on_initialize(mut self) -> Self {
        self.fail_on_initialize = true;
        self
    }
    
    /// Configure the plugin to fail on shutdown
    pub fn fail_on_shutdown(mut self) -> Self {
        self.fail_on_shutdown = true;
        self
    }
    
    /// Add capabilities to the plugin
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities.clone();
        self.metadata.capabilities = capabilities;
        self
    }
    
    /// Set simulated resource usage
    pub fn with_resource_usage(mut self, memory_mb: u64, cpu_percent: u8) -> Self {
        self.memory_usage = memory_mb;
        self.cpu_usage = cpu_percent.min(100);
        self
    }
}

#[async_trait]
impl Plugin for MockPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn as_any(&self) -> &(dyn Any + 'static) {
        self
    }
    
    async fn initialize(&self) -> Result<()> {
        if self.fail_on_initialize {
            return Err(anyhow::anyhow!("Plugin initialization failed (simulated failure)"));
        }
        
        // Simulate resource usage
        let _memory = vec![0u8; (self.memory_usage * 1024 * 1024) as usize];
        
        // Simulate CPU usage
        if self.cpu_usage > 0 {
            let start = std::time::Instant::now();
            let mut counter = 0;
            while start.elapsed().as_millis() < (self.cpu_usage as u128 * 10) {
                counter += 1;
                // Prevent optimization from removing the counter
                if counter > u64::MAX - 10 {
                    counter = 0;
                }
            }
        }
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        if self.fail_on_shutdown {
            return Err(anyhow::anyhow!("Plugin shutdown failed (simulated failure)"));
        }
        
        Ok(())
    }
}

/// A collection of mock plugins for testing
pub struct MockPluginDatabase {
    plugins: Vec<Arc<MockPlugin>>,
    database_dir: PathBuf,
}

impl MockPluginDatabase {
    /// Create a new mock plugin database
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            database_dir: PathBuf::from("mock_plugins"),
        }
    }
    
    /// Set the database directory
    pub fn with_database_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.database_dir = dir.as_ref().to_path_buf();
        self
    }
    
    /// Add a plugin to the database
    pub fn add_plugin(mut self, plugin: MockPlugin) -> Self {
        self.plugins.push(Arc::new(plugin));
        self
    }
    
    /// Add multiple standard plugins for stress testing
    pub fn add_standard_plugins(mut self, count: usize) -> Self {
        for i in 0..count {
            let plugin = MockPlugin::new(&format!("standard-plugin-{}", i), "1.0.0")
                .with_capabilities(vec!["standard".to_string()])
                .with_resource_usage(5, 2); // Low resource usage
            
            self.plugins.push(Arc::new(plugin));
        }
        self
    }
    
    /// Add plugins with different resource profiles for testing
    pub fn add_varying_plugins(mut self) -> Self {
        // High memory plugin
        let high_memory = MockPlugin::new("high-memory-plugin", "1.0.0")
            .with_resource_usage(100, 5)
            .with_capabilities(vec!["memory-intensive".to_string()]);
        
        // High CPU plugin
        let high_cpu = MockPlugin::new("high-cpu-plugin", "1.0.0")
            .with_resource_usage(10, 60)
            .with_capabilities(vec!["cpu-intensive".to_string()]);
        
        // Balanced plugin
        let balanced = MockPlugin::new("balanced-plugin", "1.0.0")
            .with_resource_usage(30, 30)
            .with_capabilities(vec!["balanced".to_string()]);
        
        // Failing plugin
        let failing = MockPlugin::new("failing-plugin", "1.0.0")
            .fail_on_initialize()
            .with_capabilities(vec!["unstable".to_string()]);
        
        self.plugins.push(Arc::new(high_memory));
        self.plugins.push(Arc::new(high_cpu));
        self.plugins.push(Arc::new(balanced));
        self.plugins.push(Arc::new(failing));
        
        self
    }
    
    /// Get all plugins
    pub fn get_plugins(&self) -> Vec<Arc<MockPlugin>> {
        self.plugins.clone()
    }
    
    /// Get a specific plugin by index
    pub fn get_plugin(&self, index: usize) -> Option<Arc<MockPlugin>> {
        self.plugins.get(index).cloned()
    }
    
    /// Generate plugin files on disk for testing loading mechanisms
    pub fn generate_files(&self) -> Result<()> {
        // Create the directory if it doesn't exist
        create_dir_all(&self.database_dir)?;
        
        // Create a plugin manifest
        let mut manifest = File::create(self.database_dir.join("plugins.json"))?;
        
        let plugins_info: Vec<_> = self.plugins.iter()
            .map(|p| {
                let metadata = p.metadata();
                json!({
                    "id": metadata.id.to_string(),
                    "name": metadata.name,
                    "version": metadata.version,
                    "description": metadata.description,
                    "author": metadata.author,
                    "capabilities": metadata.capabilities
                })
            })
            .collect();
        
        let manifest_json = json!({
            "plugins": plugins_info,
            "generated": chrono::Local::now().to_rfc3339()
        });
        
        write!(manifest, "{}", serde_json::to_string_pretty(&manifest_json)?)?;
        
        // Create subdirectories for each plugin
        for plugin in &self.plugins {
            let metadata = plugin.metadata();
            let plugin_dir = self.database_dir.join(&metadata.name);
            create_dir_all(&plugin_dir)?;
            
            // Create plugin.json
            let mut plugin_json = File::create(plugin_dir.join("plugin.json"))?;
            let plugin_info = json!({
                "id": metadata.id.to_string(),
                "name": metadata.name,
                "version": metadata.version,
                "description": metadata.description,
                "author": metadata.author,
                "capabilities": metadata.capabilities,
                "resources": {
                    "memory": plugin.memory_usage,
                    "cpu": plugin.cpu_usage
                }
            });
            
            write!(plugin_json, "{}", serde_json::to_string_pretty(&plugin_info)?)?;
            
            // Create sample resource files
            let mut readme = File::create(plugin_dir.join("README.md"))?;
            write!(readme, "# {}\n\n{}\n\nAuthor: {}\n", 
                metadata.name, metadata.description, metadata.author)?;
        }
        
        Ok(())
    }
    
    /// Clean up the generated files
    pub fn cleanup(&self) -> Result<()> {
        if self.database_dir.exists() {
            fs::remove_dir_all(&self.database_dir)?;
        }
        
        Ok(())
    }
}

impl Default for MockPluginDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a mock plugin database with standard test plugins
pub fn create_test_database() -> MockPluginDatabase {
    MockPluginDatabase::new()
        .with_database_dir("test_plugins")
        .add_standard_plugins(5)
        .add_varying_plugins()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_plugin_initialization() {
        let plugin = MockPlugin::new("test-plugin", "1.0.0");
        assert_eq!(plugin.metadata().name, "test-plugin");
        
        // Test successful initialization
        assert!(plugin.initialize().await.is_ok());
        
        // Test failed initialization
        let failing_plugin = MockPlugin::new("failing-plugin", "1.0.0").fail_on_initialize();
        assert!(failing_plugin.initialize().await.is_err());
    }
    
    #[tokio::test]
    async fn test_plugin_database() {
        let db = create_test_database();
        
        // Verify database has the expected plugins
        let plugins = db.get_plugins();
        assert!(plugins.len() > 5); // 5 standard + varying plugins
        
        // Test cleanup (not actually deleting in tests)
        let result = db.cleanup();
        assert!(result.is_ok());
    }
} 