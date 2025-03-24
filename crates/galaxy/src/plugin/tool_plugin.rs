/*!
 * Galaxy Tool Plugin Implementation
 * 
 * This module provides a plugin implementation for Galaxy tools functionality.
 */

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;

use crate::adapter::GalaxyAdapter;
use crate::error::Error;
use crate::models::tool::GalaxyTool;
use crate::plugin::{GalaxyPlugin, GalaxyToolPlugin};
use crate::plugin::default_plugin::DefaultGalaxyPlugin;

/// A Galaxy tool plugin
#[derive(Debug)]
pub struct GalaxyToolPluginImpl {
    /// Base plugin implementation
    base: DefaultGalaxyPlugin,
    /// Custom tool list
    custom_tools: Vec<GalaxyTool>,
}

impl GalaxyToolPluginImpl {
    /// Create a new tool plugin
    pub fn new(name: &str, version: &str, description: &str) -> Self {
        let base = DefaultGalaxyPlugin::new(name, version, description)
            .with_capability("galaxy-tool");
        
        Self {
            base,
            custom_tools: Vec::new(),
        }
    }
    
    /// Add a custom tool to this plugin
    pub fn with_tool(mut self, tool: GalaxyTool) -> Self {
        self.custom_tools.push(tool);
        self
    }
}

#[async_trait]
impl GalaxyPlugin for GalaxyToolPluginImpl {
    fn name(&self) -> &str {
        self.base.name()
    }
    
    fn version(&self) -> &str {
        self.base.version()
    }
    
    fn description(&self) -> &str {
        self.base.description()
    }
    
    async fn initialize(&self, adapter: Arc<GalaxyAdapter>) -> Result<(), Error> {
        self.base.initialize(adapter).await
    }
    
    async fn shutdown(&self) -> Result<(), Error> {
        self.base.shutdown().await
    }
    
    fn provides_capability(&self, capability: &str) -> bool {
        self.base.provides_capability(capability)
    }
    
    fn capabilities(&self) -> Vec<String> {
        self.base.capabilities()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl GalaxyToolPlugin for GalaxyToolPluginImpl {
    async fn list_tools(&self) -> Result<Vec<GalaxyTool>, Error> {
        let _adapter = self.base.adapter()?;
        
        // First, get standard tools from the adapter
        let mut tools = Vec::new();
        
        // For simplicity, we'll just use our custom tools for now, but in a real
        // implementation we'd merge with tools from the Galaxy instance
        tools.extend(self.custom_tools.clone());
        
        Ok(tools)
    }
    
    async fn get_tool(&self, tool_id: &str) -> Result<Option<GalaxyTool>, Error> {
        // Check custom tools first
        for tool in &self.custom_tools {
            if tool.id == tool_id {
                return Ok(Some(tool.clone()));
            }
        }
        
        // If not found in custom tools, delegate to the adapter
        let _adapter = self.base.adapter()?;
        
        // This would be a real implementation in a complete system
        // For now, we just return None
        Ok(None)
    }
    
    async fn execute_tool(&self, tool_id: &str, _params: HashMap<String, Value>) -> Result<String, Error> {
        // Find the tool first
        let _tool = match self.get_tool(tool_id).await? {
            Some(tool) => tool,
            None => return Err(Error::NotFound(format!("Tool not found: {}", tool_id))),
        };
        
        // Get the adapter
        let _adapter = self.base.adapter()?;
        
        // In a real implementation, we'd execute the tool via the adapter
        // For now, we just return a mock job ID
        let job_id = format!("job-{}-{}", tool_id, uuid::Uuid::new_v4());
        
        Ok(job_id)
    }
    
    async fn get_job_status(&self, _job_id: &str) -> Result<String, Error> {
        // Get the adapter
        let _adapter = self.base.adapter()?;
        
        // In a real implementation, we'd check the job status via the adapter
        // For now, we just return a mock status
        Ok("running".to_string())
    }
    
    async fn get_job_results(&self, job_id: &str) -> Result<Value, Error> {
        // Get the adapter
        let _adapter = self.base.adapter()?;
        
        // In a real implementation, we'd get the job results via the adapter
        // For now, we just return mock results
        Ok(serde_json::json!({
            "job_id": job_id,
            "status": "complete",
            "results": {
                "output_data": "Mock output data",
                "statistics": {
                    "runtime": 10.5,
                    "memory": "2GB"
                }
            }
        }))
    }
}

/// Factory function to create a new Galaxy tool plugin
pub fn create_tool_plugin(name: &str, version: &str, description: &str) -> GalaxyToolPluginImpl {
    GalaxyToolPluginImpl::new(name, version, description)
} 