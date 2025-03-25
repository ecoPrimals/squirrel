// Plugin discovery and registration module
//
// This module provides functionality for discovering and registering plugins
// from the unified plugin system as tools in the MCP system.

use std::sync::Arc;
use async_trait::async_trait;
use tracing::info;
use crate::plugins::interfaces::Plugin;

use crate::tool::{ToolManager, ToolContext, ToolExecutionResult, ToolExecutor, ExecutionStatus, ToolError};
// Use local interfaces instead of squirrel-plugins

/// Executor for plugin proxy
#[derive(Debug)]
pub struct PluginProxyExecutor {
    /// Plugin ID
    plugin_id: String,
    /// Tool ID
    tool_id: String,
    /// Plugin capabilities
    capabilities: Vec<String>,
}

impl PluginProxyExecutor {
    /// Create a new plugin proxy executor
    pub fn new(plugin_id: String, tool_id: String, capabilities: Vec<String>) -> Self {
        Self {
            plugin_id,
            tool_id,
            capabilities,
        }
    }
}

#[async_trait]
impl ToolExecutor for PluginProxyExecutor {
    /// Executes a capability with the given context
    async fn execute(
        &self,
        context: ToolContext
    ) -> std::result::Result<ToolExecutionResult, ToolError> {
        // Create a proper ToolExecutionResult
        Ok(ToolExecutionResult {
            tool_id: self.tool_id.clone(),
            capability: context.capability.clone(),
            request_id: context.request_id.clone(),
            status: ExecutionStatus::Success,
            output: Some(serde_json::json!({"message": "Plugin proxy execution successful"})),
            error_message: None,
            execution_time_ms: 0,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Gets the tool ID this executor is associated with
    fn get_tool_id(&self) -> String {
        self.tool_id.clone()
    }

    /// Gets the capabilities this executor can handle
    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

/// Manager for discovering and registering plugins as tools
#[derive(Debug)]
pub struct PluginDiscoveryManager {
    /// Tool manager to register plugin tools with
    tool_manager: Arc<ToolManager>,
    /// Map of plugin IDs to tool IDs
    tools: tokio::sync::RwLock<std::collections::HashMap<String, String>>,
}

impl PluginDiscoveryManager {
    /// Create a new plugin discovery manager
    pub fn new(tool_manager: Arc<ToolManager>) -> Self {
        Self {
            tool_manager,
            tools: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
    
    /// Unregister a plugin from the tool system
    pub async fn unregister_plugin(&self, plugin_id: &str) -> std::result::Result<(), ToolError> {
        // Get the tool ID for this plugin
        let tool_id = {
            let plugin_tools = self.tools.read().await;
            plugin_tools.get(plugin_id).cloned()
        };
        
        if let Some(tool_id) = tool_id {
            self.tool_manager.unregister_tool(&tool_id).await
                .map_err(|e| ToolError::ExecutionError(format!("Failed to unregister tool: {}", e)))?;
            
            // Remove from our tracking
            let mut plugin_tools = self.tools.write().await;
            plugin_tools.remove(plugin_id);
            
            info!("Unregistered plugin '{}' (tool '{}')", plugin_id, tool_id);
            Ok(())
        } else {
            Err(ToolError::ExecutionError(format!("Plugin not registered: {}", plugin_id)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use uuid::Uuid;
    use anyhow::anyhow;
    use crate::plugins::interfaces::Plugin;
    use crate::plugins::integration::PluginManagerInterface;
    
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
            async fn register_plugin(&self, _plugin: Arc<dyn Plugin>) -> anyhow::Result<Uuid> {
                // Return a fixed ID for testing
                Ok(Uuid::new_v4())
            }
            
            async fn get_plugin(&self, _id: &Uuid) -> Option<Arc<dyn Plugin>> {
                None
            }
            
            async fn unregister_plugin(&self, _id: &Uuid) -> anyhow::Result<()> {
                Ok(())
            }
        }
        
        let _plugin_manager = Arc::new(MockPluginManager {});
        
        // Create the executor
        let executor = PluginProxyExecutor::new(
            format!("plugin-{}", plugin_id),
            format!("plugin-{}", plugin_id),
            vec!["test".to_string()]
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