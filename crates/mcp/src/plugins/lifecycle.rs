// Plugin lifecycle hooks for MCP
//
// This module provides lifecycle hooks for integrating MCP tools with the plugin system.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{debug, info, error, warn};

use crate::tool::{Tool, ToolLifecycleHook, ToolError};
use crate::plugins::types::PluginLifecycleStep;
use crate::plugins::interfaces::{PluginManagerInterface, Plugin, PluginStatus};
use crate::error::{MCPError, Result as MCPResult, PluginError};

/// A lifecycle hook that synchronizes tool state changes to plugin status
#[derive(Debug)]
pub struct PluginLifecycleHook {
    /// The plugin system integration
    integration: Arc<dyn PluginManagerInterface>,
    
    /// List of tool IDs to monitor
    monitored_tools: RwLock<HashMap<String, bool>>,
}

impl PluginLifecycleHook {
    /// Create a new plugin lifecycle hook
    pub fn new(integration: Arc<dyn PluginManagerInterface>) -> Self {
        Self {
            integration,
            monitored_tools: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add a tool to the monitored list
    pub async fn monitor_tool(&self, tool_id: &str) {
        let mut tools = self.monitored_tools.write().await;
        tools.insert(tool_id.to_string(), true);
    }
    
    /// Remove a tool from the monitored list
    pub async fn unmonitor_tool(&self, tool_id: &str) {
        let mut tools = self.monitored_tools.write().await;
        tools.remove(tool_id);
    }
    
    /// Check if a tool is being monitored
    pub async fn is_monitored(&self, tool_id: &str) -> bool {
        let tools = self.monitored_tools.read().await;
        tools.get(tool_id).copied().unwrap_or(false)
    }

    /// Converts this trait object to Any for downcasting
    /// 
    /// This allows callers to downcast the trait object to a concrete type
    /// for accessing implementation-specific functionality.
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl ToolLifecycleHook for PluginLifecycleHook {
    async fn on_register(&self, tool: &Tool) -> std::result::Result<(), ToolError> {
        self.monitor_tool(&tool.id).await;
        debug!("Tool registered and monitored: {}", tool.id);
        Ok(())
    }

    async fn on_unregister(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.unmonitor_tool(tool_id).await;
        debug!("Tool unregistered and unmonitored: {}", tool_id);
        Ok(())
    }

    async fn on_activate(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        if self.is_monitored(tool_id).await {
            debug!("Tool activated: {}", tool_id);
            // Could update plugin status here if needed
        }
        Ok(())
    }

    async fn on_deactivate(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        if self.is_monitored(tool_id).await {
            debug!("Tool deactivated: {}", tool_id);
            // Could update plugin status here if needed
        }
        Ok(())
    }

    async fn on_error(&self, tool_id: &str, error: &ToolError) -> std::result::Result<(), ToolError> {
        if self.is_monitored(tool_id).await {
            info!("Tool error: {} - {:?}", tool_id, error);
            // Could update plugin status to error state if needed
        }
        Ok(())
    }

    async fn pre_start(&self, _tool_id: &str) -> std::result::Result<(), ToolError> { Ok(()) }
    async fn post_start(&self, _tool_id: &str) -> std::result::Result<(), ToolError> { Ok(()) }
    async fn pre_stop(&self, _tool_id: &str) -> std::result::Result<(), ToolError> { Ok(()) }
    async fn post_stop(&self, _tool_id: &str) -> std::result::Result<(), ToolError> { Ok(()) }
    async fn on_pause(&self, _tool_id: &str) -> std::result::Result<(), ToolError> { Ok(()) }
    async fn on_resume(&self, _tool_id: &str) -> std::result::Result<(), ToolError> { Ok(()) }
    async fn on_update(&self, _tool: &Tool) -> std::result::Result<(), ToolError> { Ok(()) }
    async fn on_cleanup(&self, _tool_id: &str) -> std::result::Result<(), ToolError> { Ok(()) }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A composite lifecycle hook that combines the plugin lifecycle hook with another hook
#[derive(Debug)]
pub struct CompositePluginLifecycleHook<T: ToolLifecycleHook> {
    /// The plugin lifecycle hook
    plugin_hook: PluginLifecycleHook,
    /// The base lifecycle hook
    base_hook: T,
}

impl<T: ToolLifecycleHook> CompositePluginLifecycleHook<T> {
    /// Create a new composite lifecycle hook
    pub const fn new(base_hook: T, plugin_hook: PluginLifecycleHook) -> Self {
        Self {
            plugin_hook,
            base_hook,
        }
    }
}

#[async_trait]
impl<T: ToolLifecycleHook + Send + Sync + 'static> ToolLifecycleHook for CompositePluginLifecycleHook<T> {
    async fn on_register(&self, tool: &Tool) -> std::result::Result<(), ToolError> {
        self.base_hook.on_register(tool).await?;
        self.plugin_hook.on_register(tool).await?;
        Ok(())
    }

    async fn on_unregister(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.on_unregister(tool_id).await?;
        self.plugin_hook.on_unregister(tool_id).await?;
        Ok(())
    }

    async fn on_activate(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.on_activate(tool_id).await?;
        self.plugin_hook.on_activate(tool_id).await?;
        Ok(())
    }

    async fn on_deactivate(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.on_deactivate(tool_id).await?;
        self.plugin_hook.on_deactivate(tool_id).await?;
        Ok(())
    }

    async fn on_error(&self, tool_id: &str, error: &ToolError) -> std::result::Result<(), ToolError> {
        self.base_hook.on_error(tool_id, error).await?;
        self.plugin_hook.on_error(tool_id, error).await?;
        Ok(())
    }

    async fn pre_start(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.pre_start(tool_id).await?;
        self.plugin_hook.pre_start(tool_id).await?;
        Ok(())
    }

    async fn post_start(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.post_start(tool_id).await?;
        self.plugin_hook.post_start(tool_id).await?;
        Ok(())
    }

    async fn pre_stop(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.pre_stop(tool_id).await?;
        self.plugin_hook.pre_stop(tool_id).await?;
        Ok(())
    }

    async fn post_stop(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.post_stop(tool_id).await?;
        self.plugin_hook.post_stop(tool_id).await?;
        Ok(())
    }

    async fn on_pause(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.on_pause(tool_id).await?;
        self.plugin_hook.on_pause(tool_id).await?;
        Ok(())
    }

    async fn on_resume(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.on_resume(tool_id).await?;
        self.plugin_hook.on_resume(tool_id).await?;
        Ok(())
    }

    async fn on_update(&self, tool: &Tool) -> std::result::Result<(), ToolError> {
        self.base_hook.on_update(tool).await?;
        self.plugin_hook.on_update(tool).await?;
        Ok(())
    }

    async fn on_cleanup(&self, tool_id: &str) -> std::result::Result<(), ToolError> {
        self.base_hook.on_cleanup(tool_id).await?;
        self.plugin_hook.on_cleanup(tool_id).await?;
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Arc;
    use uuid::Uuid;
    use crate::plugins::interfaces::{PluginStatus, Plugin};

    // Mock plugin manager for testing
    #[derive(Debug)]
    struct MockPluginManager;

    #[async_trait]
    impl PluginManagerInterface for MockPluginManager {
        async fn register_plugin(&self, _plugin: Arc<dyn Plugin>) -> Result<()> {
            Ok(())
        }

        async fn get_plugin_by_id(&self, _plugin_id: Uuid) -> Result<Option<Arc<dyn Plugin>>> {
            Ok(None)
        }

        async fn execute_mcp_plugin(&self, _plugin_id: Uuid, _message: serde_json::Value) -> Result<serde_json::Value> {
            Ok(json!({"result": "mock result"}))
        }

        async fn update_plugin_status(&self, _plugin_id: Uuid, _status: PluginStatus) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_plugin_lifecycle_hook() -> Result<(), ToolError> {
        // Create a mock plugin manager
        let plugin_manager = Arc::new(MockPluginManager);
        
        // Create the lifecycle hook
        let hook = PluginLifecycleHook::new(plugin_manager);
        
        // Create a test tool
        let tool = Tool::builder()
            .id("test-tool")
            .name("Test Tool")
            .version("1.0.0")
            .description("A test tool")
            .security_level(1)
            .build();
        
        // Test on_register
        hook.on_register(&tool).await?;
        assert!(hook.is_monitored("test-tool").await);
        
        // Test on_unregister
        hook.on_unregister("test-tool").await?;
        assert!(!hook.is_monitored("test-tool").await);
        
        Ok(())
    }
} 