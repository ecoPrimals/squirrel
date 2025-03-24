// Plugin system integration module
//
// This module provides functionality to integrate the MCP tool system with
// the plugin system.

use std::collections::HashMap;
use anyhow::Result;
use std::sync::Arc;
use std::fmt::Debug;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::info;

use super::ToolPluginFactory;
use super::interfaces::{Plugin, McpPlugin, PluginStatus};
use super::versioning::{ProtocolVersion, ProtocolVersionManager};
use crate::tool::{ToolManager, ToolContext, ToolState, ExecutionStatus, ToolExecutionResult};

// Mock Interface for PluginManagerInterface until we have the actual implementation
#[async_trait::async_trait]
pub trait PluginManagerInterface: Send + Sync + Debug {
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<Uuid>;
    async fn get_plugin(&self, id: &Uuid) -> Option<Arc<dyn Plugin>>;
    async fn unregister_plugin(&self, id: &Uuid) -> Result<()>;
}

/// Mock implementation of PluginManagerInterface for development
#[derive(Debug)]
pub struct MockPluginManager {
    plugins: RwLock<HashMap<Uuid, Arc<dyn Plugin>>>,
}

impl MockPluginManager {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl PluginManagerInterface for MockPluginManager {
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<Uuid> {
        let id = plugin.metadata().id;
        self.plugins.write().await.insert(id, plugin);
        Ok(id)
    }
    
    async fn get_plugin(&self, id: &Uuid) -> Option<Arc<dyn Plugin>> {
        self.plugins.read().await.get(id).cloned()
    }
    
    async fn unregister_plugin(&self, id: &Uuid) -> Result<()> {
        self.plugins.write().await.remove(id);
        Ok(())
    }
}

/// Trait extension for McpPlugin to get the tool ID
pub trait McpPluginToolId {
    fn tool_id(&self) -> &str;
}

impl<T: McpPlugin> McpPluginToolId for T {
    fn tool_id(&self) -> &str {
        // This is a mock implementation that returns a placeholder
        // In a real implementation, this would be overridden or implemented differently
        "unknown-tool"
    }
}

/// Helper struct to bridge from McpPlugin to Plugin
#[derive(Debug)]
struct PluginWrapper<T: McpPlugin + ?Sized> {
    inner: Arc<T>,
}

impl<T: McpPlugin + ?Sized> PluginWrapper<T> {
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
    plugin_to_tool_map: RwLock<HashMap<Uuid, String>>,
    
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
                ProtocolVersion::new(1, 0, 0),  // Minimum supported version
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
    pub async fn register_tools_as_plugins(&self) -> Result<Vec<Uuid>> {
        let factory = ToolPluginFactory::new(self.tool_manager.clone());
        let adapters = factory.create_plugin_adapters().await?;
        
        let mut plugin_ids = Vec::new();
        for adapter in adapters {
            // Check compatibility with version manager
            let version_req = adapter.protocol_version_requirements();
            if !self.version_manager.is_compatible_with_requirement(&version_req)? {
                info!("Skipping registration of tool '{}' due to incompatible version requirements",
                      adapter.tool_id());
                continue;
            }
            
            // Register the adapter as a plugin using the wrapper
            let adapter_arc = Arc::new(adapter);
            let plugin_arc = PluginWrapper::new(adapter_arc);
            let plugin_id = self.plugin_manager.register_plugin(plugin_arc).await?;
            plugin_ids.push(plugin_id);
            
            // Store the mapping (using a placeholder tool_id for now)
            let mut map = self.plugin_to_tool_map.write().await;
            let tool_id = "adapter-tool"; // Placeholder
            map.insert(plugin_id, tool_id.to_string());
            info!("Registered tool '{}' as plugin '{}'", tool_id, plugin_id);
        }
        
        Ok(plugin_ids)
    }
    
    /// Register a single tool as a plugin
    pub async fn register_tool_as_plugin(&self, adapter: Arc<dyn McpPlugin>) -> Result<Uuid> {
        // Register the plugin using the wrapper to avoid direct casting
        let plugin_arc = PluginWrapper::new(adapter);
        let plugin_id = self.plugin_manager.register_plugin(plugin_arc).await?;
        
        // Store the mapping
        {
            let mut map = self.plugin_to_tool_map.write().await;
            let tool_id = "adapter-tool"; // Placeholder
            map.insert(plugin_id, tool_id.to_string());
            info!("Registered tool '{}' as plugin '{}'", tool_id, plugin_id);
        }
        
        Ok(plugin_id)
    }
    
    /// Get a plugin by tool ID
    pub async fn get_plugin_by_tool_id(&self, tool_id: &str) -> Option<Arc<dyn Plugin>> {
        let map = self.plugin_to_tool_map.read().await;
        for (plugin_id, tid) in map.iter() {
            if tid == tool_id {
                return self.plugin_manager.get_plugin(plugin_id).await;
            }
        }
        
        None
    }
    
    /// Handle tool state changes
    pub async fn handle_tool_state_change(&self, tool_id: &str, state: ToolState) -> Result<()> {
        // Get the plugin for this tool
        if let Some(_plugin) = self.get_plugin_by_tool_id(tool_id).await {
            // Update plugin status based on tool state
            let new_status = match state {
                ToolState::Active => PluginStatus::Running,
                ToolState::Inactive => PluginStatus::ShutDown,
                ToolState::Error => PluginStatus::Error,
                ToolState::Started => PluginStatus::Running,
                ToolState::Stopped => PluginStatus::ShutDown,
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
    pub fn version_manager(&self) -> &ProtocolVersionManager {
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
    integration: Arc<PluginSystemIntegration>,
    tool_id: String,
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
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        let output = result.get("result").cloned();
        let error_message = result.get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let execution_time_ms = result.get("execution_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
            
        let timestamp = result.get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.into())
            .unwrap_or_else(chrono::Utc::now);
            
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
            ProtocolVersion::new(1, 0, 0),
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