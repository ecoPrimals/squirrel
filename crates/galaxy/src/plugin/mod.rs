/*!
 * Galaxy Plugin Architecture
 * 
 * This module defines the plugin architecture for the Galaxy adapter, allowing
 * it to be extended with custom functionality and to integrate with the wider
 * Squirrel plugin system.
 */

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::adapter::GalaxyAdapter;
use crate::error::Error;
use crate::models::tool::GalaxyTool;

/// Galaxy plugin trait - base trait for all Galaxy plugins
#[async_trait]
pub trait GalaxyPlugin: Send + Sync + Debug {
    /// Get the plugin name
    fn name(&self) -> &str;
    
    /// Get the plugin version
    fn version(&self) -> &str;
    
    /// Get the plugin description
    fn description(&self) -> &str;
    
    /// Initialize the plugin with a Galaxy adapter
    async fn initialize(&self, adapter: Arc<GalaxyAdapter>) -> Result<(), Error>;
    
    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<(), Error>;
    
    /// Check if the plugin provides a specific capability
    fn provides_capability(&self, capability: &str) -> bool;
    
    /// Get all capabilities provided by this plugin
    fn capabilities(&self) -> Vec<String>;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Galaxy tool plugin trait - extension for tool-related functionality
#[async_trait]
pub trait GalaxyToolPlugin: GalaxyPlugin {
    /// List available Galaxy tools provided by this plugin
    async fn list_tools(&self) -> Result<Vec<GalaxyTool>, Error>;
    
    /// Get a specific tool by ID
    async fn get_tool(&self, tool_id: &str) -> Result<Option<GalaxyTool>, Error>;
    
    /// Execute a tool with the given parameters
    async fn execute_tool(&self, tool_id: &str, params: HashMap<String, Value>) -> Result<String, Error>;
    
    /// Get the status of a job
    async fn get_job_status(&self, job_id: &str) -> Result<String, Error>;
    
    /// Get the results of a completed job
    async fn get_job_results(&self, job_id: &str) -> Result<Value, Error>;
}

/// Galaxy workflow plugin trait - extension for workflow-related functionality
#[async_trait]
pub trait GalaxyWorkflowPlugin: GalaxyPlugin {
    /// List available workflows
    async fn list_workflows(&self) -> Result<Vec<Value>, Error>;
    
    /// Get a specific workflow by ID
    async fn get_workflow(&self, workflow_id: &str) -> Result<Option<Value>, Error>;
    
    /// Execute a workflow with the given parameters
    async fn execute_workflow(&self, workflow_id: &str, params: Value) -> Result<String, Error>;
    
    /// Get the status of a workflow invocation
    async fn get_workflow_status(&self, invocation_id: &str) -> Result<String, Error>;
    
    /// Get the results of a completed workflow
    async fn get_workflow_results(&self, invocation_id: &str) -> Result<Value, Error>;
}

/// Galaxy dataset plugin trait - extension for dataset-related functionality
#[async_trait]
pub trait GalaxyDatasetPlugin: GalaxyPlugin {
    /// List datasets in a history
    async fn list_datasets(&self, history_id: &str) -> Result<Vec<Value>, Error>;
    
    /// Get a specific dataset by ID
    async fn get_dataset(&self, dataset_id: &str) -> Result<Option<Value>, Error>;
    
    /// Upload data to Galaxy
    async fn upload_data(&self, history_id: &str, name: &str, data: Vec<u8>, file_type: &str) -> Result<String, Error>;
    
    /// Download a dataset from Galaxy
    async fn download_dataset(&self, dataset_id: &str) -> Result<Vec<u8>, Error>;
    
    /// List dataset collections in a history
    async fn list_collections(&self, history_id: &str) -> Result<Vec<Value>, Error>;
    
    /// Get a specific dataset collection by ID
    async fn get_collection(&self, collection_id: &str) -> Result<Option<Value>, Error>;
    
    /// Create a new dataset collection from datasets
    async fn create_collection(&self, history_id: &str, name: &str, collection_type: &str, dataset_ids: Vec<String>) -> Result<String, Error>;
    
    /// Get the elements of a dataset collection
    async fn get_collection_elements(&self, collection_id: &str) -> Result<Vec<Value>, Error>;
    
    /// Delete a dataset collection
    async fn delete_collection(&self, collection_id: &str) -> Result<(), Error>;
}

/// Plugin manager for Galaxy plugins
pub struct GalaxyPluginManager {
    /// The Galaxy adapter
    adapter: Arc<GalaxyAdapter>,
    /// Registered plugins
    plugins: HashMap<String, Arc<dyn GalaxyPlugin>>,
}

impl GalaxyPluginManager {
    /// Create a new plugin manager
    pub fn new(adapter: Arc<GalaxyAdapter>) -> Self {
        Self {
            adapter,
            plugins: HashMap::new(),
        }
    }
    
    /// Register a plugin
    pub async fn register_plugin(&mut self, plugin: Arc<dyn GalaxyPlugin>) -> Result<(), Error> {
        let name = plugin.name().to_string();
        
        // Initialize the plugin
        plugin.initialize(Arc::clone(&self.adapter)).await?;
        
        // Store the plugin
        self.plugins.insert(name.clone(), plugin);
        
        Ok(())
    }
    
    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn GalaxyPlugin>> {
        self.plugins.get(name).cloned()
    }
    
    /// Get all plugins
    pub fn get_all_plugins(&self) -> Vec<Arc<dyn GalaxyPlugin>> {
        self.plugins.values().cloned().collect()
    }
    
    /// Get all plugins with a specific capability
    pub fn get_plugins_by_capability(&self, capability: &str) -> Vec<Arc<dyn GalaxyPlugin>> {
        self.plugins
            .values()
            .filter(|p| p.provides_capability(capability))
            .cloned()
            .collect()
    }
    
    /// Get all tool plugins
    pub fn get_tool_plugins(&self) -> Vec<Arc<dyn GalaxyPlugin>> {
        // Filter plugins that provide the galaxy-tool capability
        self.get_plugins_by_capability("galaxy-tool")
    }
    
    /// Shutdown all plugins
    pub async fn shutdown(&self) -> Result<(), Error> {
        for plugin in self.plugins.values() {
            plugin.shutdown().await?;
        }
        
        Ok(())
    }
}

/// Default plugin implementation
pub mod default_plugin;

/// Tool plugin implementation
pub mod tool_plugin;

/// Workflow plugin implementation
pub mod workflow_plugin;

/// Dataset plugin implementation
pub mod dataset_plugin; 