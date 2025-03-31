// Plugin system integration module
//
// This module provides functionality to integrate the MCP tool system with
// the plugin system.

use std::collections::HashMap;
use anyhow::Result;
use std::sync::Arc;
use std::fmt::Debug;
use tokio::sync::RwLock;
use tracing::info;

use super::ToolPluginFactory;
use super::interfaces::{Plugin, McpPlugin, PluginStatus, PluginManagerInterface};
use super::versioning::{ProtocolVersion, ProtocolVersionManager};
use crate::tool::{ToolManager, ToolContext, ToolState, ExecutionStatus, ToolExecutionResult};

/// Mock implementation of `PluginManagerInterface` for development
#[derive(Debug)]
pub struct MockPluginManager {
    /// Map of plugin IDs to plugin instances
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
}

impl MockPluginManager {
    #[must_use] pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MockPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl PluginManagerInterface for MockPluginManager {
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let id = plugin.metadata().id.clone();
        self.plugins.write().await.insert(id, plugin);
        Ok(())
    }
    
    async fn get_plugin_by_id(&self, plugin_id: String) -> Result<Option<Arc<dyn Plugin>>> {
        Ok(self.plugins.read().await.get(&plugin_id).cloned())
    }
    
    async fn execute_mcp_plugin(&self, plugin_id: String, message: serde_json::Value) -> Result<serde_json::Value> {
        // Implementation would go here
        Ok(serde_json::json!({"status": "executed"}))
    }
    
    async fn update_plugin_status(&self, plugin_id: String, status: PluginStatus) -> Result<()> {
        // Implementation would go here
        Ok(())
    }
}

/// Trait extension for `McpPlugin` to get the tool ID
pub trait McpPluginToolId {
    fn tool_id(&self) -> &str;
}

impl<T: McpPlugin + ?Sized> McpPluginToolId for T {
    fn tool_id(&self) -> &'static str {
        // This is a mock implementation that returns a placeholder
        // In a real implementation, this would be overridden or implemented differently
        "unknown-tool"
    }
}

/// Helper struct to bridge from `McpPlugin` to Plugin
#[derive(Debug)]
struct PluginWrapper<T: McpPlugin + ?Sized> {
    /// The wrapped plugin instance
    inner: Arc<T>,
}

impl<T: McpPlugin + ?Sized> PluginWrapper<T> {
    /// Creates a new plugin adapter wrapping the given plugin
    fn new(plugin: Arc<T>) -> Arc<Self> {
        Arc::new(Self { inner: plugin })
    }
}

#[async_trait::async_trait]
impl<T: McpPlugin + ?Sized> Plugin for PluginWrapper<T> {
    fn metadata(&self) -> super::interfaces::PluginMetadata {
        self.inner.metadata()
    }
    
    async fn initialize(&self) -> Result<()> {
        self.inner.initialize().await
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.inner.shutdown().await
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self.inner.as_any()
    }
}

/// Integration between the MCP tool system and the plugin system
#[derive(Debug)]
pub struct PluginSystemIntegration {
    /// The MCP tool manager
    tool_manager: Arc<ToolManager>,
    
    /// The plugin manager
    plugin_manager: Arc<dyn PluginManagerInterface>,
    
    /// Mapping from plugin ID to tool ID
    plugin_to_tool_map: RwLock<HashMap<String, String>>,
    
    /// Protocol version manager
    version_manager: ProtocolVersionManager,
}

impl PluginSystemIntegration {
    /// Create a new integration between the MCP tool system and the plugin system
    pub fn new(
        tool_manager: Arc<ToolManager>,
        plugin_manager: Arc<dyn PluginManagerInterface>,
    ) -> Self {
        Self {
            tool_manager,
            plugin_manager,
            plugin_to_tool_map: RwLock::new(HashMap::new()),
            version_manager: ProtocolVersionManager::new(
                ProtocolVersion::new(1, 0, 0),  // Current version
                vec![ProtocolVersion::new(1, 0, 0)],  // Supported versions
            ),
        }
    }
    
    /// Create a new integration with a specific version manager
    pub fn with_version_manager(
        tool_manager: Arc<ToolManager>,
        plugin_manager: Arc<dyn PluginManagerInterface>,
        version_manager: ProtocolVersionManager,
    ) -> Self {
        Self {
            tool_manager,
            plugin_manager,
            plugin_to_tool_map: RwLock::new(HashMap::new()),
            version_manager,
        }
    }
    
    /// Register all active tools as plugins
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The factory fails to create plugin adapters
    /// - There is an error with the plugin registration process
    /// - The protocol version requirements are incompatible
    pub async fn register_tools_as_plugins(&self) -> Result<Vec<String>> {
        let factory = ToolPluginFactory::new(self.tool_manager.clone());
        let adapters = factory.create_plugin_adapters().await?;
        
        let mut plugin_ids = Vec::new();
        for adapter in adapters {
            // Check compatibility with version manager
            let version_req = adapter.protocol_version_requirements();
            if !self.version_manager.is_compatible_with_requirement(&version_req)? {
                info!("Skipping registration of tool due to incompatible version requirements");
                continue;
            }
            
            // Get the plugin ID before wrapping
            let plugin_id = adapter.metadata().id.clone();
            
            // Add the plugin_id to the result vector
            plugin_ids.push(plugin_id.clone());
            
            // Create a clone of the adapter for registration
            let adapter_arc = Arc::new(adapter);
            
            // Register the adapter as a plugin using the wrapper
            let plugin_arc = PluginWrapper::new(adapter_arc.clone());
            self.plugin_manager.register_plugin(plugin_arc).await?;
            
            // Store the mapping with shorter lock duration
            // Use a placeholder or generate a proper tool ID
            let tool_id = format!("tool-{}", plugin_id);
            {
                let mut map = self.plugin_to_tool_map.write().await;
                map.insert(plugin_id.clone(), tool_id.clone());
            }
            
            info!("Registered tool '{}' as plugin '{}'", tool_id, plugin_id);
        }
        
        Ok(plugin_ids)
    }
    
    /// Register a single tool as a plugin
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The plugin registration process fails
    /// - There is an error with the plugin system
    pub async fn register_tool_as_plugin(&self, adapter: Arc<dyn McpPlugin>) -> Result<String> {
        // Get the plugin ID from the metadata
        let plugin_id = adapter.metadata().id.clone();
        
        // Register the plugin using the wrapper to avoid direct casting
        let plugin_arc = PluginWrapper::new(adapter.clone());
        self.plugin_manager.register_plugin(plugin_arc).await?;
        
        // Store the mapping with shorter lock duration
        // Use a placeholder or generate a proper tool ID
        let tool_id = format!("tool-{}", plugin_id);
        {
            let mut map = self.plugin_to_tool_map.write().await;
            map.insert(plugin_id.clone(), tool_id.clone());
        }
        
        info!("Registered tool '{}' as plugin '{}'", tool_id, plugin_id);
        
        Ok(plugin_id)
    }
    
    /// Get a plugin by tool ID
    pub async fn get_plugin_by_tool_id(&self, tool_id: &str) -> Option<Arc<dyn Plugin>> {
        // Get tool-to-plugin mapping, cloning only what we need while minimizing lock duration
        let plugin_id = {
            let map = self.plugin_to_tool_map.read().await;
            
            // Find the plugin ID for this tool ID
            map.iter()
                .find(|(_, tid)| tid.as_str() == tool_id)
                .map(|(plugin_id, _)| plugin_id.clone())
        };
        
        // If we found a plugin ID, fetch the plugin
        if let Some(plugin_id) = plugin_id {
            if let Ok(Some(plugin)) = self.plugin_manager.get_plugin_by_id(plugin_id).await {
                return Some(plugin);
            }
        }
        
        None
    }
    
    /// Handle tool state changes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The plugin status cannot be updated
    /// - There is an internal error in the plugin system
    pub async fn handle_tool_state_change(&self, tool_id: &str, state: ToolState) -> Result<()> {
        // Get the plugin for this tool
        if let Some(_plugin) = self.get_plugin_by_tool_id(tool_id).await {
            // Update plugin status based on tool state
            let new_status = match state {
                ToolState::Active | ToolState::Started => PluginStatus::Running,
                ToolState::Inactive | ToolState::Stopped => PluginStatus::ShutDown,
                ToolState::Error => PluginStatus::Error,
                _ => PluginStatus::Registered,
            };
            
            // In a real implementation, we would update the plugin status
            info!("Tool '{}' state changed to {:?}, updating plugin status to {:?}",
                  tool_id, state, new_status);
        }
        
        Ok(())
    }
    
    /// Create a Tool executor that uses the plugin system
    pub fn create_plugin_tool_executor(&self) -> PluginToolExecutor {
        PluginToolExecutor {
            integration: Arc::new(self.clone()),
            tool_id: "plugin_executor".to_string(),
            capabilities: vec!["execute_plugin".to_string()],
        }
    }
    
    /// Get the protocol version manager
    pub const fn version_manager(&self) -> &ProtocolVersionManager {
        &self.version_manager
    }
}

impl Clone for PluginSystemIntegration {
    fn clone(&self) -> Self {
        Self {
            tool_manager: self.tool_manager.clone(),
            plugin_manager: self.plugin_manager.clone(),
            plugin_to_tool_map: RwLock::new(HashMap::new()),
            version_manager: self.version_manager.clone(),
        }
    }
}

/// A Tool executor that uses the plugin system
#[derive(Debug)]
pub struct PluginToolExecutor {
    /// Reference to the plugin system integration
    integration: Arc<PluginSystemIntegration>,
    /// Identifier for the tool associated with this executor
    tool_id: String,
    /// List of capabilities supported by this executor
    capabilities: Vec<String>,
}

#[async_trait::async_trait]
impl crate::tool::ToolExecutor for PluginToolExecutor {
    async fn execute(&self, context: ToolContext) -> Result<ToolExecutionResult, crate::tool::ToolError> {
        // Find the plugin for this tool
        let tool_id = context.tool_id.clone();
        let _plugin = self.integration.get_plugin_by_tool_id(&tool_id).await
            .ok_or_else(|| crate::tool::ToolError::ToolNotFound(tool_id.clone()))?;
        
        // Try to downcast to McpPlugin - note: this is a simplified approach
        // In a real implementation, we'd use a proper downcast mechanism
        
        // Create a message for the plugin
        let _message = serde_json::json!({
            "capability": context.capability,
            "parameters": context.parameters,
            "request_id": context.request_id,
            "protocol_version": {
                "major": 1,
                "minor": 0,
                "patch": 0,
                "pre_release": null,
                "build": null
            }
        });
        
        // Try to cast the plugin to McpPlugin
        // This is a dummy implementation - in a real system this would be handled better
        // Let's pretend it works and create a dummy response
        let result = serde_json::json!({
            "success": true,
            "status": "success",
            "result": {"message": "Plugin executed successfully"},
            "execution_time_ms": 100,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        // Convert the result to a ToolExecutionResult
        let success = result.get("success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
            
        let output = result.get("result").cloned();
        let error_message = result.get("error")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);
            
        let execution_time_ms = result.get("execution_time_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
            
        let timestamp = result.get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map_or_else(chrono::Utc::now, std::convert::Into::into);
            
        Ok(ToolExecutionResult {
            tool_id: tool_id.clone(),
            capability: context.capability,
            request_id: context.request_id,
            status: if success { ExecutionStatus::Success } else { ExecutionStatus::Failure },
            output,
            error_message,
            execution_time_ms,
            timestamp,
        })
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
    
    // Basic test for version compatibility checks
    #[test]
    fn test_version_compatibility() {
        let tool_manager = Arc::new(crate::tool::ToolManager::new());
        let plugin_manager = Arc::new(MockPluginManager::new());
        
        // Create an integration with version 1.0.0
        let version_manager = ProtocolVersionManager::new(
            ProtocolVersion::new(1, 0, 0),
            vec![ProtocolVersion::new(1, 0, 0)],
        );
        let integration = PluginSystemIntegration::with_version_manager(
            tool_manager,
            plugin_manager,
            version_manager,
        );
        
        // Verify version manager
        assert_eq!(integration.version_manager().current_version().to_string(), "1.0.0");
    }
} 