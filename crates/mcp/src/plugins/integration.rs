// MCP plugin system integration
//
// This module provides integration between the MCP tool system and the unified plugin system,
// allowing for bidirectional interoperability.

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::tool::{Tool, ToolManager, ToolState, ToolExecutor, ToolContext, ToolExecutionResult, ExecutionStatus};
use super::{ToolPluginAdapter, ToolPluginFactory};
use squirrel_plugins::plugin::{Plugin, PluginMetadata, PluginStatus};
use squirrel_plugins::manager::PluginManager;
use squirrel_plugins::mcp::McpPlugin;

/// Integrates MCP tool system with the unified plugin system
pub struct PluginSystemIntegration {
    /// Tool manager reference
    tool_manager: Arc<ToolManager>,
    
    /// Plugin manager reference
    plugin_manager: Arc<PluginManager>,
    
    /// Mapping of tool IDs to plugin IDs
    tool_to_plugin_map: RwLock<std::collections::HashMap<String, Uuid>>,
    
    /// Mapping of plugin IDs to tool IDs
    plugin_to_tool_map: RwLock<std::collections::HashMap<Uuid, String>>,
}

impl PluginSystemIntegration {
    /// Create a new plugin system integration
    pub fn new(tool_manager: Arc<ToolManager>, plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            tool_manager,
            plugin_manager,
            tool_to_plugin_map: RwLock::new(std::collections::HashMap::new()),
            plugin_to_tool_map: RwLock::new(std::collections::HashMap::new()),
        }
    }
    
    /// Register all active tools as plugins
    pub async fn register_tools_as_plugins(&self) -> Result<()> {
        let factory = ToolPluginFactory::new(self.tool_manager.clone());
        let adapters = factory.create_plugin_adapters().await?;
        
        for adapter in adapters {
            let tool_id = adapter.tool_id().to_string();
            let plugin_id = adapter.metadata().id;
            
            // Register the plugin with the plugin manager
            self.plugin_manager.register_plugin(Arc::new(adapter) as Arc<dyn McpPlugin>).await?;
            
            // Update the mappings
            {
                let mut tool_to_plugin = self.tool_to_plugin_map.write().await;
                let mut plugin_to_tool = self.plugin_to_tool_map.write().await;
                
                tool_to_plugin.insert(tool_id.clone(), plugin_id);
                plugin_to_tool.insert(plugin_id, tool_id);
            }
            
            info!("Registered tool '{}' as plugin '{}'", tool_id, plugin_id);
        }
        
        Ok(())
    }
    
    /// Register a specific tool as a plugin
    pub async fn register_tool_as_plugin(&self, tool_id: &str) -> Result<Uuid> {
        // Check if already registered
        {
            let tool_to_plugin = self.tool_to_plugin_map.read().await;
            if let Some(plugin_id) = tool_to_plugin.get(tool_id) {
                return Ok(*plugin_id);
            }
        }
        
        let factory = ToolPluginFactory::new(self.tool_manager.clone());
        let adapter = factory.create_plugin_adapter(tool_id).await?;
        
        let plugin_id = adapter.metadata().id;
        
        // Register the plugin with the plugin manager
        self.plugin_manager.register_plugin(Arc::new(adapter) as Arc<dyn McpPlugin>).await?;
        
        // Update the mappings
        {
            let mut tool_to_plugin = self.tool_to_plugin_map.write().await;
            let mut plugin_to_tool = self.plugin_to_tool_map.write().await;
            
            tool_to_plugin.insert(tool_id.to_string(), plugin_id);
            plugin_to_tool.insert(plugin_id, tool_id.to_string());
        }
        
        info!("Registered tool '{}' as plugin '{}'", tool_id, plugin_id);
        
        Ok(plugin_id)
    }
    
    /// Handle tool state changes to update plugin state
    pub async fn handle_tool_state_change(&self, tool_id: &str, state: ToolState) -> Result<()> {
        let plugin_id = {
            let tool_to_plugin = self.tool_to_plugin_map.read().await;
            match tool_to_plugin.get(tool_id) {
                Some(plugin_id) => *plugin_id,
                None => return Ok(()) // Tool is not registered as a plugin
            }
        };
        
        // Translate tool state to plugin status
        let status = match state {
            ToolState::Active | ToolState::Started => PluginStatus::Active,
            ToolState::Inactive | ToolState::Stopped | ToolState::Paused => PluginStatus::Inactive,
            ToolState::Error => PluginStatus::Failed,
            _ => return Ok(()), // Other states don't map cleanly to plugin status
        };
        
        // Update plugin status
        self.plugin_manager.update_plugin_status(plugin_id, status).await?;
        
        Ok(())
    }
    
    /// Create a plugin executor that can access plugins from the tool system
    pub fn create_plugin_executor(&self) -> Arc<PluginToolExecutor> {
        Arc::new(PluginToolExecutor::new(
            self.plugin_manager.clone(),
            self.plugin_to_tool_map.clone(),
        ))
    }
}

/// Tool executor that forwards execution to plugins
pub struct PluginToolExecutor {
    /// Plugin manager reference
    plugin_manager: Arc<PluginManager>,
    
    /// Mapping of plugin IDs to tool IDs
    plugin_to_tool_map: RwLock<std::collections::HashMap<Uuid, String>>,
    
    /// Cached tool ID
    tool_id: String,
    
    /// Capabilities this executor can handle
    capabilities: Vec<String>,
}

impl PluginToolExecutor {
    /// Create a new plugin tool executor
    pub fn new(
        plugin_manager: Arc<PluginManager>,
        plugin_to_tool_map: RwLock<std::collections::HashMap<Uuid, String>>,
    ) -> Self {
        Self {
            plugin_manager,
            plugin_to_tool_map,
            tool_id: "plugin_executor".to_string(),
            capabilities: vec!["execute_plugin".to_string()],
        }
    }
}

impl std::fmt::Debug for PluginToolExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginToolExecutor")
            .field("tool_id", &self.tool_id)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

#[async_trait]
impl ToolExecutor for PluginToolExecutor {
    async fn execute(&self, context: ToolContext) -> Result<ToolExecutionResult, crate::tool::ToolError> {
        // Extract plugin ID and other parameters
        let plugin_id_str = context.parameters.get("plugin_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::tool::ToolError::ExecutionError(
                "Missing 'plugin_id' parameter".to_string()))?;
            
        let plugin_id = plugin_id_str.parse::<Uuid>()
            .map_err(|e| crate::tool::ToolError::ExecutionError(
                format!("Invalid plugin ID: {}", e)))?;
                
        let capability = context.parameters.get("capability")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::tool::ToolError::ExecutionError(
                "Missing 'capability' parameter".to_string()))?;
                
        let params = context.parameters.get("parameters")
            .cloned()
            .unwrap_or(serde_json::json!({}));
            
        // Build the message to send to the plugin
        let message = serde_json::json!({
            "capability": capability,
            "parameters": params,
            "request_id": context.request_id,
        });
        
        // Execute the plugin
        let start_time = std::time::Instant::now();
        
        let result = self.plugin_manager.execute_plugin::<McpPlugin>(plugin_id, |plugin| async move {
            plugin.handle_message(message).await
        }).await;
        
        let duration = start_time.elapsed();
        
        // Convert the result to a tool execution result
        match result {
            Ok(output) => {
                let status = if output.get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false) {
                    ExecutionStatus::Success
                } else {
                    ExecutionStatus::Failure
                };
                
                Ok(ToolExecutionResult {
                    tool_id: self.tool_id.clone(),
                    capability: context.capability.clone(),
                    request_id: context.request_id.clone(),
                    status,
                    output: Some(output),
                    error_message: None,
                    execution_time_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                })
            },
            Err(e) => {
                Ok(ToolExecutionResult {
                    tool_id: self.tool_id.clone(),
                    capability: context.capability.clone(),
                    request_id: context.request_id.clone(),
                    status: ExecutionStatus::Failure,
                    output: None,
                    error_message: Some(e.to_string()),
                    execution_time_ms: duration.as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                })
            }
        }
    }
    
    fn get_tool_id(&self) -> String {
        self.tool_id.clone()
    }
    
    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests will be added during implementation
} 