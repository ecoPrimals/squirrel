// MCP plugin adaptation layer
//
// This module provides an adaptation layer between the MCP tool system and the new unified 
// plugin system. It allows existing MCP tools to be used as plugins in the new system.

use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{debug, error};

// Use local interfaces instead of squirrel-plugins
use crate::plugins::interfaces::{Plugin, PluginMetadata, McpPlugin, PluginStatus, PluginCapability};
use crate::tool::{Tool, ToolManager};
use super::versioning::VersionRequirement;

/// An adapter that converts an MCP Tool to a Plugin
pub struct ToolPluginAdapter {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// The wrapped tool manager
    tool_manager: Arc<ToolManager>,
    
    /// The tool ID in the tool manager
    tool_id: String,
    
    /// Protocol version requirements
    version_requirements: VersionRequirement,
}

impl ToolPluginAdapter {
    /// Create a new tool plugin adapter
    pub fn new(tool_id: String, tool_manager: Arc<ToolManager>) -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4().to_string(),
                name: "Tool Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Adapter for tool plugins".to_string(),
                status: PluginStatus::Registered,
                capabilities: vec![PluginCapability::Tool],
            },
            tool_manager,
            tool_id,
            version_requirements: VersionRequirement::new(">=1.0.0, <2.0.0"),
        }
    }
    
    /// Create a new tool plugin adapter with specific version requirements
    pub fn with_version_requirements(
        tool_id: String, 
        tool_manager: Arc<ToolManager>,
        version_requirements: VersionRequirement
    ) -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4().to_string(),
                name: "Tool Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Adapter for tool plugins with version requirements".to_string(),
                status: PluginStatus::Registered,
                capabilities: vec![PluginCapability::Tool],
            },
            tool_manager,
            tool_id,
            version_requirements,
        }
    }
    
    /// Get the tool ID
    #[must_use] pub fn tool_id(&self) -> &str {
        &self.tool_id
    }
    
    /// Get the tool from the manager
    async fn get_tool(&self) -> Result<Tool> {
        self.tool_manager.get_tool(&self.tool_id).await
            .map_or_else(
                || Err(anyhow!("Tool not found: {}", self.tool_id)),
                |tool| Ok(tool)
            )
    }
}

impl std::fmt::Debug for ToolPluginAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolPluginAdapter")
            .field("tool_id", &self.tool_id)
            .field("plugin_id", &self.metadata.id)
            .field("plugin_name", &self.metadata.name)
            .field("metadata", &self.metadata)
            .field("version_requirements", &self.version_requirements.requirement)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl Plugin for ToolPluginAdapter {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    async fn initialize(&self) -> Result<()> {
        // Try to get the tool to verify it exists
        let _ = self.get_tool().await?;
        debug!("Initialized tool adapter for {}", self.tool_id);
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down tool adapter for {}", self.tool_id);
        Ok(())
    }
}

#[async_trait]
impl McpPlugin for ToolPluginAdapter {
    async fn handle_message(&self, message: Value) -> Result<Value> {
        // Extract capability and parameters
        let capability = message.get("capability")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing capability in message"))?;
            
        let parameters = message.get("parameters")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
            
        let request_id = message.get("request_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Check if the version is present and compatible
        if let Some(version_value) = message.get("protocol_version") {
            // In a real implementation, check version compatibility
            debug!("Message protocol version: {:?}", version_value);
        }
            
        // Execute the tool
        debug!("Executing tool {} with capability {}", self.tool_id, capability);
        let result = self.tool_manager.execute_tool(
            &self.tool_id,
            capability,
            parameters,
            Some(request_id),
        ).await;
        
        match result {
            Ok(execution_result) => {
                // Use debug formatting for status since ExecutionStatus doesn't implement Display
                let status = format!("{:?}", execution_result.status);
                let output = execution_result.output;
                
                // Create response
                let response = serde_json::json!({
                    "success": matches!(execution_result.status, crate::tool::ExecutionStatus::Success),
                    "status": status,
                    "result": output,
                    "execution_time_ms": execution_result.execution_time_ms,
                    "timestamp": execution_result.timestamp.to_rfc3339(),
                });
                
                Ok(response)
            },
            Err(e) => {
                error!("Error executing tool {}: {}", self.tool_id, e);
                
                // Create error response
                let response = serde_json::json!({
                    "success": false,
                    "status": "error",
                    "error": e.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                
                Ok(response)
            }
        }
    }
    
    fn validate_message_schema(&self, message: &Value) -> Result<()> {
        // Check that the message has the required fields
        if !message.is_object() {
            return Err(anyhow!("Message must be an object"));
        }
        
        if message.get("capability").is_none() {
            return Err(anyhow!("Message must contain 'capability' field"));
        }
        
        // Parameters are optional, but if present, must be an object
        if let Some(params) = message.get("parameters") {
            if !params.is_object() && !params.is_null() {
                return Err(anyhow!("'parameters' field must be an object or null"));
            }
        }
        
        Ok(())
    }
    
    fn protocol_version_requirements(&self) -> VersionRequirement {
        self.version_requirements.clone()
    }
}

/// Factory for creating tool plugin adapters
pub struct ToolPluginFactory {
    /// The tool manager
    tool_manager: Arc<ToolManager>,
}

impl ToolPluginFactory {
    /// Create a new tool plugin factory
    pub const fn new(tool_manager: Arc<ToolManager>) -> Self {
        Self {
            tool_manager,
        }
    }
    
    /// Create a plugin adapter for a specific tool
    pub async fn create_plugin_adapter(&self, tool_id: &str) -> Result<ToolPluginAdapter> {
        // Verify that the tool exists
        if self.tool_manager.get_tool(tool_id).await.is_none() {
            return Err(anyhow!("Tool not found: {}", tool_id));
        }
        
        let adapter = ToolPluginAdapter::new(tool_id.to_string(), self.tool_manager.clone());
        
        Ok(adapter)
    }
    
    /// Create plugin adapters for all active tools
    pub async fn create_plugin_adapters(&self) -> Result<Vec<ToolPluginAdapter>> {
        let mut adapters = Vec::new();
        
        // Get all tools and create adapters for them
        // Since we don't have get_active_tools(), we'll get all tools instead
        let all_tools = self.tool_manager.get_all_tools().await;
        for tool in all_tools {
            let adapter = ToolPluginAdapter::new(tool.id.clone(), self.tool_manager.clone());
            adapters.push(adapter);
        }
        
        Ok(adapters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Simplified test for adapter functionality
    #[test]
    fn test_tool_plugin_adapter_version_requirements() {
        // Create a standard ToolManager instance for testing
        let tool_manager = Arc::new(crate::tool::ToolManager::new());
        let version_req = VersionRequirement::new(">=2.0.0, <3.0.0");
        let adapter = ToolPluginAdapter::with_version_requirements(
            "test-tool".to_string(), 
            tool_manager,
            version_req.clone()
        );
        
        // Test version requirements
        let adapter_req = adapter.protocol_version_requirements();
        assert_eq!(adapter_req.requirement, ">=2.0.0, <3.0.0");
    }
} 