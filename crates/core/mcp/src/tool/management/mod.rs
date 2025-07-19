//! Tool management module for MCP
//!
//! Core tool management functionality that remains in Squirrel MCP.
//! Complex tool lifecycle management has been moved to ToadStool.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::Result;

// Core modules that remain in MCP
pub mod types;
pub mod builder;
pub mod manager;
pub mod operations;
pub mod state_management;
pub mod execution;

// Re-export public types for backward compatibility
pub use types::*;
pub use builder::*;
pub use manager::*;

/// Core tool management interface (simplified for MCP core)
#[async_trait]
pub trait ToolManager: Send + Sync {
    /// Register a new tool
    async fn register_tool(&self, tool: types::Tool) -> Result<(), crate::tool::management::types::ToolError>;
    
    /// Execute a tool with given parameters
    async fn execute_tool(&self, tool_name: &str, parameters: serde_json::Value) -> Result<serde_json::Value>;
    
    /// Get basic tool information
    async fn get_tool(&self, id: &str) -> Result<Option<ToolInfo>>;
    
    /// List available tools
    async fn list_tools(&self) -> Result<Vec<ToolInfo>>;
    
    /// Unregister a tool
    async fn unregister_tool(&self, tool_id: &str) -> Result<(), crate::tool::management::types::ToolError>;
    
    /// Recover a tool
    async fn recover_tool(&self, tool_id: &str) -> Result<(), crate::tool::management::types::ToolError>;
}

/// Basic tool information for MCP core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
}

/// Simple tool manager implementation for core MCP functionality
#[derive(Debug)]
pub struct CoreToolManager {
    tools: Arc<tokio::sync::RwLock<HashMap<String, ToolInfo>>>,
}

impl CoreToolManager {
    /// Creates a new instance of the core tool manager
    /// 
    /// Initializes the tool manager with an empty tool registry.
    /// 
    /// # Returns
    /// 
    /// A new `CoreToolManager` instance
    pub fn new() -> Self {
        Self {
            tools: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ToolManager for CoreToolManager {
    async fn register_tool(&self, tool: types::Tool) -> Result<(), crate::tool::management::types::ToolError> {
        let tool_info = ToolInfo {
            id: tool.id.clone(),
            name: tool.name.clone(),
            description: tool.description.clone(),
            version: tool.version.clone(),
        };
        
        let mut tools = self.tools.write().await;
        if tools.contains_key(&tool.id) {
            return Err(crate::tool::management::types::ToolError::AlreadyRegistered(tool.id));
        }
        
        tools.insert(tool.id.clone(), tool_info);
        Ok(())
    }
    
    async fn execute_tool(&self, _tool_name: &str, _parameters: serde_json::Value) -> Result<serde_json::Value> {
        // Simplified implementation for core MCP
        Ok(serde_json::Value::Null)
    }
    
    async fn get_tool(&self, id: &str) -> Result<Option<ToolInfo>> {
        let tools = self.tools.read().await;
        Ok(tools.get(id).cloned())
    }
    
    async fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        let tools = self.tools.read().await;
        Ok(tools.values().cloned().collect())
    }
    
    async fn unregister_tool(&self, tool_id: &str) -> Result<(), crate::tool::management::types::ToolError> {
        let mut tools = self.tools.write().await;
        tools.remove(tool_id);
        Ok(())
    }
    
    async fn recover_tool(&self, _tool_id: &str) -> Result<(), crate::tool::management::types::ToolError> {
        // Simplified implementation for core MCP
        Ok(())
    }
} 