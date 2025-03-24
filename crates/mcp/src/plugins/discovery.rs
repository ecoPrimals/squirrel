// Plugin discovery and registration module
//
// This module provides functionality for discovering and registering plugins
// from the unified plugin system as tools in the MCP system.

use std::sync::Arc;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use serde_json::json;

use crate::tool::{Tool, ToolManager, ToolContext, ToolExecutionResult, ToolExecutor, ExecutionStatus};
use crate::tool::executor::BasicToolExecutor;
// Use local interfaces instead of squirrel-plugins
use crate::plugins::interfaces::{Plugin, PluginMetadata, McpPlugin, PluginManagerInterface};

/// A tool executor that delegates execution to a plugin
pub struct PluginProxyExecutor {
    /// The plugin ID
    plugin_id: Uuid,
    
    /// The tool ID (derived from plugin ID)
    tool_id: String,
    
    /// The plugin manager
    plugin_manager: Arc<dyn PluginManagerInterface>,
    
    /// The plugin's capabilities
    capabilities: Vec<String>,
}

impl PluginProxyExecutor {
    /// Create a new plugin proxy executor
    pub fn new(plugin_id: Uuid, capabilities: Vec<String>, plugin_manager: Arc<dyn PluginManagerInterface>) -> Self {
        Self {
            plugin_id,
            tool_id: format!("plugin-{}", plugin_id),
            plugin_manager,
            capabilities,
        }
    }

    /// Execute the capability with the given parameters
    async fn execute(&self, context: ToolContext) -> Result<ToolExecutionResult, crate::tool::ToolError> {
        let start_time = std::time::Instant::now();
        
        // Build a message to send to the plugin
        let message = serde_json::json!({
            "capability": context.capability,
            "parameters": context.parameters,
            "request_id": context.request_id
        });
        
        // Execute the plugin via the plugin manager
        let result = match self.plugin_manager.execute_mcp_plugin(self.plugin_id, message).await {
            Ok(response) => {
                // Convert the plugin response to a tool execution result
                let output = response.get("result").cloned();
                let error = response.get("error").and_then(|e| e.as_str()).map(String::from);
                let success = response.get("success").and_then(|s| s.as_bool()).unwrap_or(true);
                
                ToolExecutionResult {
                    tool_id: self.tool_id.clone(),
                    capability: context.capability,
                    request_id: context.request_id,
                    status: if success { ExecutionStatus::Success } else { ExecutionStatus::Failure },
                    output,
                    error_message: error,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                }
            },
            Err(err) => {
                // Return an error result
                ToolExecutionResult {
                    tool_id: self.tool_id.clone(),
                    capability: context.capability,
                    request_id: context.request_id,
                    status: ExecutionStatus::Failure,
                    output: None,
                    error_message: Some(format!("Plugin execution failed: {}", err)),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    timestamp: chrono::Utc::now(),
                }
            }
        };
        
        Ok(result)
    }
}

impl std::fmt::Debug for PluginProxyExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginProxyExecutor")
            .field("plugin_id", &self.plugin_id)
            .field("tool_id", &self.tool_id)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

#[async_trait]
impl ToolExecutor for PluginProxyExecutor {
    async fn execute(&self, context: ToolContext) -> Result<ToolExecutionResult, crate::tool::ToolError> {
        self.execute(context).await
    }
    
    fn get_tool_id(&self) -> String {
        self.tool_id.clone()
    }
    
    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

/// Manager for discovering and registering plugins as tools
pub struct PluginDiscoveryManager {
    /// Tool manager instance
    tool_manager: Arc<ToolManager>,
    /// Plugin manager instance
    plugin_manager: Arc<dyn PluginManagerInterface>,
    
    /// The mapping of plugin IDs to tool IDs
    registered_plugins: tokio::sync::RwLock<std::collections::HashMap<Uuid, String>>,
}

impl PluginDiscoveryManager {
    /// Create a new plugin discovery manager
    pub fn new(tool_manager: Arc<ToolManager>, plugin_manager: Arc<dyn PluginManagerInterface>) -> Self {
        Self {
            tool_manager,
            plugin_manager,
            registered_plugins: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
    
    // NOTE: Most of these methods would need to be implemented once we have a proper
    // plugin manager interface. For now, this is a simplified version to avoid build errors.
    
    /// Check if a plugin is registered as a tool
    pub async fn is_plugin_registered(&self, plugin_id: Uuid) -> bool {
        let registered = self.registered_plugins.read().await;
        registered.contains_key(&plugin_id)
    }
    
    /// Get the tool ID for a registered plugin
    pub async fn get_tool_id_for_plugin(&self, plugin_id: Uuid) -> Option<String> {
        let registered = self.registered_plugins.read().await;
        registered.get(&plugin_id).cloned()
    }
    
    /// Unregister a plugin from the tool system
    pub async fn unregister_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let tool_id = {
            let mut registered = self.registered_plugins.write().await;
            registered.remove(&plugin_id)
        };
        
        if let Some(tool_id) = tool_id {
            self.tool_manager.unregister_tool(&tool_id).await
                .map_err(|e| anyhow!("Failed to unregister tool: {}", e))?;
            
            info!("Unregistered plugin '{}' (tool '{}')", plugin_id, tool_id);
            Ok(())
        } else {
            Err(anyhow!("Plugin not registered: {}", plugin_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::interfaces::PluginStatus;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_plugin_proxy_executor() {
        // This is a basic test of the structure only since we can't easily mock
        // the full plugin manager functionality now
        let plugin_id = Uuid::new_v4();
        
        // Create a mock plugin manager
        #[derive(Debug)]
        struct MockPluginManager {}
        
        #[async_trait]
        impl PluginManagerInterface for MockPluginManager {
            async fn register_plugin(&self, _plugin: Arc<dyn Plugin>) -> Result<()> {
                Err(anyhow!("Not implemented for test"))
            }
            
            async fn get_plugin_by_id(&self, _plugin_id: Uuid) -> Result<Option<Arc<dyn Plugin>>> {
                Err(anyhow!("Not implemented for test"))
            }
            
            async fn execute_mcp_plugin(&self, _id: Uuid, message: Value) -> Result<Value> {
                // Return a success response for testing purposes
                Ok(json!({
                    "success": true,
                    "result": {
                        "data": "Test result data",
                        "input": message
                    },
                    "message": "Executed successfully in test"
                }))
            }
            
            async fn update_plugin_status(&self, _id: Uuid, _status: PluginStatus) -> Result<()> {
                Err(anyhow!("Not implemented for test"))
            }
        }
        
        let plugin_manager = Arc::new(MockPluginManager {});
        
        // Create the executor
        let executor = PluginProxyExecutor::new(
            plugin_id, 
            vec!["test".to_string()], 
            plugin_manager
        );
        
        // Basic checks
        assert_eq!(executor.get_tool_id(), format!("plugin-{}", plugin_id));
        assert_eq!(executor.get_capabilities(), vec!["test".to_string()]);
        
        // Test execution with the mock
        let context = ToolContext {
            tool_id: executor.get_tool_id(),
            capability: "test".to_string(),
            parameters: std::collections::HashMap::new(),
            request_id: "test-request".to_string(),
            security_token: None,
            session_id: None,
            timestamp: chrono::Utc::now(),
        };
        
        let result = executor.execute(context).await.unwrap();
        assert_eq!(result.status, ExecutionStatus::Success);
        assert!(result.output.is_some());
    }
} 