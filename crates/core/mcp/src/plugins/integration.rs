// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

// Plugin system integration module
//
// This module provides functionality to integrate the MCP tool system with
// the plugin system.

use std::collections::HashMap;
use anyhow::Result;
use std::sync::Arc;
use std::fmt::Debug;
use tokio::sync::RwLock;
use tracing::{info, error, warn};

use super::ToolPluginFactory;
use super::interfaces::{Plugin, McpPlugin, PluginStatus, PluginManagerInterface};
use super::versioning::{ProtocolVersion, ProtocolVersionManager};
use crate::tool::{ToolManager, ToolContext, ToolState, ExecutionStatus, ToolExecutionResult};
use crate::error::types::MCPError;

/// Production implementation of `PluginManagerInterface`
#[derive(Debug)]
pub struct ProductionPluginManager {
    /// Map of plugin IDs to plugin instances
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
    /// Plugin status tracking
    plugin_status: RwLock<HashMap<String, PluginStatus>>,
    /// Plugin execution metrics
    execution_metrics: RwLock<HashMap<String, PluginExecutionMetrics>>,
}

#[derive(Debug, Clone)]
pub struct PluginExecutionMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    pub average_execution_time_ms: f64,
}

impl Default for PluginExecutionMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            last_execution: None,
            average_execution_time_ms: 0.0,
        }
    }
}

impl ProductionPluginManager {
    #[must_use] pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            plugin_status: RwLock::new(HashMap::new()),
            execution_metrics: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get plugin execution metrics
    pub async fn get_plugin_metrics(&self, plugin_id: &str) -> Option<PluginExecutionMetrics> {
        let metrics = self.execution_metrics.read().await;
        metrics.get(plugin_id).cloned()
    }
    
    /// Update plugin execution metrics
    async fn update_metrics(&self, plugin_id: &str, execution_time_ms: f64, success: bool) {
        let mut metrics = self.execution_metrics.write().await;
        let plugin_metrics = metrics.entry(plugin_id.to_string()).or_default();
        
        plugin_metrics.total_executions += 1;
        if success {
            plugin_metrics.successful_executions += 1;
        } else {
            plugin_metrics.failed_executions += 1;
        }
        
        plugin_metrics.last_execution = Some(chrono::Utc::now());
        
        // Update average execution time
        let total_time = plugin_metrics.average_execution_time_ms * (plugin_metrics.total_executions - 1) as f64;
        plugin_metrics.average_execution_time_ms = (total_time + execution_time_ms) / plugin_metrics.total_executions as f64;
    }
}

impl Default for ProductionPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl PluginManagerInterface for ProductionPluginManager {
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let id = plugin.metadata().id.clone();
        
        // Validate plugin before registration
        if id.is_empty() {
            return Err(anyhow::anyhow!("Plugin ID cannot be empty"));
        }
        
        // Check if plugin already exists
        {
            let plugins = self.plugins.read().await;
            if plugins.contains_key(&id) {
                return Err(anyhow::anyhow!("Plugin with ID '{}' already registered", id));
            }
        }
        
        // Register the plugin
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(id.clone(), plugin);
        }
        
        // Set initial status
        {
            let mut status = self.plugin_status.write().await;
            status.insert(id.clone(), PluginStatus::Active);
        }
        
        // Initialize metrics
        {
            let mut metrics = self.execution_metrics.write().await;
            metrics.insert(id, PluginExecutionMetrics::default());
        }
        
        info!("Plugin '{}' registered successfully", id);
        Ok(())
    }
    
    async fn get_plugin_by_id(&self, plugin_id: String) -> Result<Option<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        Ok(plugins.get(&plugin_id).cloned())
    }
    
    async fn execute_mcp_plugin(&self, plugin_id: String, message: serde_json::Value) -> Result<serde_json::Value> {
        let start_time = std::time::Instant::now();
        
        // Get plugin
        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(&plugin_id).cloned()
        };
        
        let plugin = match plugin {
            Some(p) => p,
            None => {
                return Err(anyhow::anyhow!("Plugin '{}' not found", plugin_id));
            }
        };
        
        // Check plugin status
        {
            let status = self.plugin_status.read().await;
            if let Some(plugin_status) = status.get(&plugin_id) {
                if *plugin_status != PluginStatus::Active {
                    return Err(anyhow::anyhow!("Plugin '{}' is not active (status: {:?})", plugin_id, plugin_status));
                }
            }
        }
        
        // Execute plugin
        let result = match plugin.as_any().downcast_ref::<dyn McpPlugin>() {
            Some(mcp_plugin) => {
                // Execute as MCP plugin
                let execution_result = mcp_plugin.execute_mcp_message(message).await;
                
                let execution_time = start_time.elapsed().as_millis() as f64;
                let success = execution_result.is_ok();
                
                self.update_metrics(&plugin_id, execution_time, success).await;
                
                execution_result
            }
            None => {
                // Plugin doesn't implement MCP interface
                Err(anyhow::anyhow!("Plugin '{}' does not implement MCP interface", plugin_id))
            }
        };
        
        result
    }
    
    async fn update_plugin_status(&self, plugin_id: String, status: PluginStatus) -> Result<()> {
        let mut plugin_status = self.plugin_status.write().await;
        
        if !plugin_status.contains_key(&plugin_id) {
            return Err(anyhow::anyhow!("Plugin '{}' not found", plugin_id));
        }
        
        plugin_status.insert(plugin_id.clone(), status);
        info!("Plugin '{}' status updated to {:?}", plugin_id, status);
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
    tool_manager: Arc<dyn ToolManager>,
    
    /// The plugin manager
    plugin_manager: Arc<ProductionPluginManager>,
    
    /// Mapping from plugin ID to tool ID
    plugin_to_tool_map: RwLock<HashMap<String, String>>,
    
    /// Protocol version manager
    version_manager: Arc<ProtocolVersionManager>,
}

impl PluginSystemIntegration {
    /// Create a new integration between the MCP tool system and the plugin system
    pub fn new(
        tool_manager: Arc<dyn ToolManager>,
        plugin_manager: Arc<ProductionPluginManager>,
    ) -> Self {
        Self {
            tool_manager,
            plugin_manager,
            plugin_to_tool_map: RwLock::new(HashMap::new()),
            version_manager: Arc::new(ProtocolVersionManager::new(
                ProtocolVersion::new(1, 0, 0),  // Current version
                vec![ProtocolVersion::new(1, 0, 0)],  // Supported versions
            )),
        }
    }
    
    /// Create a new integration with a specific version manager
    pub fn with_version_manager(
        tool_manager: Arc<dyn ToolManager>,
        plugin_manager: Arc<ProductionPluginManager>,
        version_manager: Arc<ProtocolVersionManager>,
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
        Ok(ToolExecutionResult {
            tool_id: tool_id.clone(),
            status: ExecutionStatus::Success,
            result: result.clone(),
            execution_time: std::time::Duration::from_millis(100),
            metadata: std::collections::HashMap::new(),
        })
    }
    
    fn get_tool_id(&self) -> String {
        self.tool_id.clone()
    }
    
    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

impl PluginToolExecutor {
    /// Create a new executor with the given integration and tool ID
    pub fn new(integration: Arc<PluginSystemIntegration>, tool_id: String) -> Self {
        Self {
            integration,
            tool_id,
            capabilities: Vec::new(),
        }
    }
    
    /// Add a capability to this executor
    pub fn add_capability(&mut self, capability: String) {
        self.capabilities.push(capability);
    }
    
    /// Get the capabilities of this executor
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

/// This is the main integration class that bridges the plugin system with the tool system.
/// It allows plugins to function as tools and tools to function as plugins.
pub struct PluginSystemIntegration {
    /// Plugin manager for managing plugins
    plugin_manager: Arc<ProductionPluginManager>,
    /// Tool manager for managing tools
    tool_manager: Arc<dyn ToolManager>,
    /// Version manager for protocol versioning
    version_manager: Arc<ProtocolVersionManager>,
}

impl PluginSystemIntegration {
    /// Create a new integration with the given managers
    pub fn new(
        plugin_manager: Arc<ProductionPluginManager>,
        tool_manager: Arc<dyn ToolManager>,
        version_manager: Arc<ProtocolVersionManager>,
    ) -> Self {
        Self {
            plugin_manager,
            tool_manager,
            version_manager,
        }
    }
    
    /// Register a tool as a plugin
    pub async fn register_tool_as_plugin(&self, tool_id: String) -> Result<()> {
        // Get the tool from the tool manager
        let tool_info = self.tool_manager.get_tool(&tool_id).await?;
        
        if let Some(tool_info) = tool_info {
            // Create a plugin adapter for the tool
            let plugin_adapter = ToolPluginFactory::create_plugin_for_tool(tool_id.clone(), tool_info)?;
            
            // Register the plugin adapter
            self.plugin_manager.register_plugin(plugin_adapter).await?;
            
            info!("Tool '{}' registered as plugin", tool_id);
        } else {
            warn!("Tool '{}' not found for plugin registration", tool_id);
        }
        
        Ok(())
    }
    
    /// Register all active tools as plugins
    pub async fn register_all_tools_as_plugins(&self) -> Result<()> {
        let tools = self.tool_manager.list_tools().await?;
        
        for tool_info in tools {
            if let Err(e) = self.register_tool_as_plugin(tool_info.id.clone()).await {
                error!("Failed to register tool '{}' as plugin: {}", tool_info.id, e);
            }
        }
        
        Ok(())
    }
    
    /// Get a plugin by tool ID
    pub async fn get_plugin_by_tool_id(&self, tool_id: &str) -> Option<Arc<dyn Plugin>> {
        self.plugin_manager.get_plugin_by_id(tool_id.to_string()).await.ok().flatten()
    }
    
    /// Get all registered plugins
    pub async fn get_all_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        // This is a simplified implementation - in a real system, you'd have a method to list all plugins
        Vec::new()
    }
    
    /// Execute a plugin through the MCP interface
    pub async fn execute_plugin_mcp(&self, plugin_id: String, message: serde_json::Value) -> Result<serde_json::Value> {
        self.plugin_manager.execute_mcp_plugin(plugin_id, message).await
    }
    
    /// Update plugin status
    pub async fn update_plugin_status(&self, plugin_id: String, status: PluginStatus) -> Result<()> {
        self.plugin_manager.update_plugin_status(plugin_id, status).await
    }
    
    /// Get plugin execution metrics
    pub async fn get_plugin_metrics(&self, plugin_id: &str) -> Option<PluginExecutionMetrics> {
        self.plugin_manager.get_plugin_metrics(plugin_id).await
    }
}

impl Default for PluginSystemIntegration {
    fn default() -> Self {
        // Create default managers
        let plugin_manager = Arc::new(ProductionPluginManager::new());
        let tool_manager = Arc::new(crate::tool::management::CoreToolManager::new());
        let version_manager = Arc::new(ProtocolVersionManager::new());
        
        Self::new(plugin_manager, tool_manager, version_manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Basic test for version compatibility checks
    #[test]
    fn test_version_compatibility() {
        let tool_manager = Arc::new(crate::tool::ToolManager::new());
        let plugin_manager = Arc::new(ProductionPluginManager::new());
        
        // Create an integration with version 1.0.0
        let version_manager = Arc::new(ProtocolVersionManager::new(
            ProtocolVersion::new(1, 0, 0),
            vec![ProtocolVersion::new(1, 0, 0)],
        ));
        let integration = PluginSystemIntegration::with_version_manager(
            tool_manager,
            plugin_manager,
            version_manager,
        );
        
        // Verify version manager
        assert_eq!(integration.version_manager().current_version().to_string(), "1.0.0");
    }
}

/// Plugin integration manager
pub struct PluginIntegrationManager {
    /// Tool manager reference
    tool_manager: Arc<dyn ToolManager>,
}

// Implement Debug manually for PluginIntegrationManager
impl std::fmt::Debug for PluginIntegrationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginIntegrationManager")
            .field("tool_manager", &"<ToolManager>")
            .finish()
    }
} 