//! Galaxy adapter plugin implementation
//!
//! This module provides a plugin implementation for the Galaxy adapter.

use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::plugin::{Plugin, PluginMetadata};
use crate::galaxy::GalaxyPlugin;

/// Configuration for the Galaxy adapter plugin
#[derive(Debug, Clone)]
pub struct GalaxyAdapterPluginConfig {
    /// Galaxy API URL
    pub api_url: String,
    /// Galaxy API key
    pub api_key: String,
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
}

impl Default for GalaxyAdapterPluginConfig {
    fn default() -> Self {
        Self {
            api_url: "https://usegalaxy.org/api".to_string(),
            api_key: String::new(),
            timeout: Some(30),
        }
    }
}

/// Galaxy adapter plugin
#[derive(Debug)]
pub struct GalaxyAdapterPlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Galaxy adapter instance
    adapter: RwLock<Option<Arc<galaxy::adapter::GalaxyAdapter>>>,
    /// Configuration
    config: GalaxyAdapterPluginConfig,
}

impl GalaxyAdapterPlugin {
    /// Create a new Galaxy adapter plugin
    pub fn new(config: GalaxyAdapterPluginConfig) -> Self {
        let metadata = PluginMetadata::new(
            "Galaxy Adapter Plugin",
            "0.1.0",
            "Plugin for Galaxy bioinformatics platform integration",
            "DataScienceBioLab",
        )
        .with_capability("galaxy-integration")
        .with_capability("bioinformatics-tools");

        Self {
            metadata,
            adapter: RwLock::new(None),
            config,
        }
    }

    /// Get the Galaxy adapter
    pub fn adapter(&self) -> Result<Arc<galaxy::adapter::GalaxyAdapter>> {
        let adapter_guard = self.adapter.read().unwrap();
        
        match &*adapter_guard {
            Some(adapter) => Ok(Arc::clone(adapter)),
            None => {
                drop(adapter_guard);
                self.initialize_adapter()
            }
        }
    }

    /// Initialize the Galaxy adapter
    fn initialize_adapter(&self) -> Result<Arc<galaxy::adapter::GalaxyAdapter>> {
        let mut adapter_guard = self.adapter.write().unwrap();
        
        // Check again in case another thread initialized it
        if let Some(adapter) = &*adapter_guard {
            return Ok(Arc::clone(adapter));
        }
        
        // Create Galaxy configuration
        let galaxy_config = galaxy::config::GalaxyConfig::default()
            .with_api_url(&self.config.api_url)
            .with_api_key(&self.config.api_key);

        // Create the adapter
        let adapter = galaxy::adapter::GalaxyAdapter::new(galaxy_config)?;
        let adapter = Arc::new(adapter);
        
        // Store the adapter
        *adapter_guard = Some(Arc::clone(&adapter));
        
        Ok(adapter)
    }
}

#[async_trait]
impl Plugin for GalaxyAdapterPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        // Initialize the adapter
        let adapter = self.initialize_adapter()?;
        
        // Initialize MCP integration if available
        #[cfg(feature = "mcp-integration")]
        {
            let mut adapter_clone = galaxy::adapter::GalaxyAdapter::new(adapter.config().clone())?;
            adapter_clone.initialize_mcp()?;
        }
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        // Nothing special to do for shutdown
        Ok(())
    }
}

#[async_trait]
impl GalaxyPlugin for GalaxyAdapterPlugin {
    async fn connect(&self, connection_info: Value) -> Result<()> {
        // For now, just ensure the adapter is initialized
        self.adapter()?;
        Ok(())
    }
    
    async fn send_data(&self, data: Value) -> Result<Value> {
        let adapter = self.adapter()?;
        
        // Extract necessary information from the data
        let history_id = data.get("history_id")
            .and_then(|h| h.as_str())
            .unwrap_or("default");
            
        let name = data.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("uploaded_data")
            .to_string();
            
        let file_type = data.get("file_type")
            .and_then(|f| f.as_str())
            .unwrap_or("txt");
            
        let content = data.get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .as_bytes()
            .to_vec();
        
        // Upload the dataset
        let dataset = adapter.upload_dataset(&name, content, file_type, Some(history_id)).await?;
        
        // Return the dataset information
        Ok(serde_json::json!({
            "dataset_id": dataset.id,
            "name": dataset.name,
            "history_id": dataset.history_id,
            "status": dataset.status,
        }))
    }
    
    async fn receive_data(&self) -> Result<Value> {
        // In a real implementation, we would retrieve data from Galaxy
        // For now, just return mock data
        Ok(serde_json::json!({
            "datasets": [
                {
                    "id": "mock_dataset_1",
                    "name": "Example Dataset 1",
                    "history_id": "mock_history",
                    "status": "ok"
                }
            ]
        }))
    }
    
    fn get_integration_types(&self) -> Vec<String> {
        vec![
            "galaxy-core".to_string(),
            "bioinformatics-tools".to_string(),
            "data-analysis".to_string(),
        ]
    }
}

/// Factory function to create a new Galaxy adapter plugin
pub fn create_galaxy_adapter_plugin(
    api_url: impl Into<String>,
    api_key: impl Into<String>,
    timeout: Option<u64>,
) -> GalaxyAdapterPlugin {
    let config = GalaxyAdapterPluginConfig {
        api_url: api_url.into(),
        api_key: api_key.into(),
        timeout,
    };
    
    GalaxyAdapterPlugin::new(config)
} 