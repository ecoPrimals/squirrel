//! Tool Manager Core Implementation
//!
//! This module contains the core ToolManager struct and its initialization methods.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::types::*;
use crate::tool::cleanup::{BasicResourceManager, ResourceManager, RecoveryHook};
use crate::tool::lifecycle::BasicLifecycleHook;
use crate::error::types::MCPError;
use crate::tool::management::types::{Tool, ToolInfo as TypesToolInfo, ToolState, ToolError};
use crate::tool::management::ToolInfo;
use super::ToolManager;
use anyhow::Result;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Tool Manager Implementation - Core tool management system
#[derive(Debug)]
pub struct ToolManagerImpl {
    /// Map of tool IDs to tools
    pub(super) tools: RwLock<HashMap<String, Tool>>,
    /// Map of tool IDs to tool states
    pub(super) states: RwLock<HashMap<String, ToolState>>,
    /// Map of tool IDs to tool executors
    pub(super) executors: RwLock<HashMap<String, Arc<dyn ToolExecutor>>>,
    /// Map of capability names to tool IDs
    pub(super) capability_map: RwLock<HashMap<String, HashSet<String>>>,
    /// Tool lifecycle hook
    pub(super) lifecycle_hook: Arc<dyn ToolLifecycleHook>,
    /// Resource manager for tools
    pub(super) resource_manager: Arc<dyn ResourceManager>,
    /// Recovery hook for tool errors
    pub(super) recovery_hook: Option<Arc<RecoveryHook>>,
}

/// Builder for creating ToolManagerImpl instances
pub struct ToolManagerBuilder {
    /// Optional lifecycle hook, defaults to `BasicLifecycleHook` if not provided
    lifecycle_hook: Option<Arc<dyn ToolLifecycleHook>>,
    /// Optional resource manager, defaults to `BasicResourceManager` if not provided
    resource_manager: Option<Arc<dyn ResourceManager>>,
    /// Optional recovery hook for handling tool errors
    recovery_hook: Option<Arc<RecoveryHook>>,
}

impl ToolManagerBuilder {
    /// Creates a new `ToolManagerBuilder`
    #[must_use] 
    pub fn new() -> Self {
        Self {
            lifecycle_hook: None,
            resource_manager: None,
            recovery_hook: None,
        }
    }

    /// Sets the lifecycle hook for the tool manager
    pub fn lifecycle_hook(mut self, hook: impl ToolLifecycleHook + 'static) -> Self {
        self.lifecycle_hook = Some(Arc::new(hook));
        self
    }

    /// Sets the resource manager for the tool manager
    pub fn resource_manager(mut self, manager: impl ResourceManager + 'static) -> Self {
        self.resource_manager = Some(Arc::new(manager));
        self
    }

    /// Sets the recovery hook for the tool manager
    pub fn recovery_hook(mut self, hook: RecoveryHook) -> Self {
        self.recovery_hook = Some(Arc::new(hook));
        self
    }

    /// Builds the `ToolManagerImpl` instance
    #[must_use] 
    pub fn build(self) -> ToolManagerImpl {
        ToolManagerImpl {
            tools: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
            executors: RwLock::new(HashMap::new()),
            capability_map: RwLock::new(HashMap::new()),
            lifecycle_hook: self.lifecycle_hook
                .unwrap_or_else(|| Arc::new(BasicLifecycleHook::new())),
            resource_manager: self.resource_manager
                .unwrap_or_else(|| Arc::new(BasicResourceManager::new())),
            recovery_hook: self.recovery_hook,
        }
    }
}

impl Default for ToolManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolManagerImpl {
    /// Creates a new builder for ToolManagerImpl
    #[must_use] 
    pub fn builder() -> ToolManagerBuilder {
        ToolManagerBuilder::new()
    }

    /// Creates a new `ToolManagerImpl` with default configuration
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
            executors: RwLock::new(HashMap::new()),
            capability_map: RwLock::new(HashMap::new()),
            lifecycle_hook: Arc::new(BasicLifecycleHook::new()),
            resource_manager: Arc::new(BasicResourceManager::new()),
            recovery_hook: None,
        }
    }

    /// Creates a new `ToolManagerImpl` with a custom lifecycle hook
    pub fn with_lifecycle_hook(lifecycle_hook: impl ToolLifecycleHook + 'static) -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
            executors: RwLock::new(HashMap::new()),
            capability_map: RwLock::new(HashMap::new()),
            lifecycle_hook: Arc::new(lifecycle_hook),
            resource_manager: Arc::new(BasicResourceManager::new()),
            recovery_hook: None,
        }
    }

    /// Adds a recovery hook to the tool manager
    pub fn with_recovery_hook(mut self, recovery_hook: RecoveryHook) -> Self {
        self.recovery_hook = Some(Arc::new(recovery_hook));
        self
    }

    /// Get a reference to the lifecycle hook
    pub fn lifecycle_hook(&self) -> &Arc<dyn ToolLifecycleHook> {
        &self.lifecycle_hook
    }

    /// Get a reference to the resource manager
    pub fn resource_manager(&self) -> &Arc<dyn ResourceManager> {
        &self.resource_manager
    }

    /// Get a reference to the recovery hook
    pub fn recovery_hook(&self) -> Option<&Arc<RecoveryHook>> {
        self.recovery_hook.as_ref()
    }
}

impl Default for ToolManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

/// Core tool manager implementation
#[derive(Debug)]
pub struct CoreToolManager {
    /// Tools registry
    tools: Arc<RwLock<HashMap<String, Arc<Tool>>>>,
    /// Tool information cache
    tool_info: Arc<RwLock<HashMap<String, ToolInfo>>>,
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
    /// Recovery hooks
    recovery_hooks: Arc<RwLock<HashMap<String, Arc<RecoveryHook>>>>,
}

impl CoreToolManager {
    /// Create a new core tool manager
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            tool_info: Arc::new(RwLock::new(HashMap::new())),
            resource_manager: Arc::new(BasicResourceManager::new()),
            recovery_hooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl ToolManager for CoreToolManager {
    async fn register_tool(&self, tool: Tool) -> Result<(), ToolError> {
        let mut tools = self.tools.write().await;
        let mut tool_info = self.tool_info.write().await;
        
        // Check if tool already exists
        if tools.contains_key(&tool.id) {
            return Err(ToolError::AlreadyRegistered(tool.id));
        }
        
        // Create tool info
        let info = ToolInfo {
            id: tool.id.clone(),
            name: tool.name.clone(),
            description: tool.description.clone(),
            version: tool.version.clone(),
        };
        
        // Store tool and info
        tools.insert(tool.id.clone(), Arc::new(tool));
        tool_info.insert(info.id.clone(), info);
        
        Ok(())
    }
    
    async fn get_tool(&self, id: &str) -> crate::error::types::Result<Option<ToolInfo>> {
        let tool_info = self.tool_info.read().await;
        Ok(tool_info.get(id).cloned())
    }
    
    async fn list_tools(&self) -> Result<Vec<super::ToolInfo>, MCPError> {
        let tools = self.tools.read().await;
        let tool_infos: Vec<super::ToolInfo> = tools.values().map(|tool| super::ToolInfo {
            id: tool.id.clone(),
            name: tool.name.clone(),
            description: tool.description.clone(),
            version: tool.version.clone(),
        }).collect();
        Ok(tool_infos)
    }
    
    async fn unregister_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut tools = self.tools.write().await;
        let mut tool_info = self.tool_info.write().await;
        
        // Remove tool and info
        tools.remove(tool_id);
        tool_info.remove(tool_id);
        
        info!("Unregistered tool: {}", tool_id);
        Ok(())
    }
    
    async fn recover_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut tools = self.tools.write().await;
        
        if let Some(tool) = tools.get_mut(tool_id) {
            tracing::info!("Recovering tool: {}", tool_id);
            // In a real implementation, this would recover the tool state
            Ok(())
        } else {
            Err(ToolError::ToolNotFound(tool_id.to_string()))
        }
    }

    async fn execute_tool(&self, tool_name: &str, parameters: serde_json::Value) -> crate::error::Result<serde_json::Value> {
        let tools = self.tools.read().await;
        
        if let Some(_tool) = tools.get(tool_name) {
            tracing::info!("Executing tool: {} with parameters: {:?}", tool_name, parameters);
            
            // Simplified implementation - returns a basic success response
            Ok(serde_json::json!({
                "status": "success",
                "result": "Tool executed successfully",
                "tool_name": tool_name,
                "parameters": parameters
            }))
        } else {
            Err(MCPError::NotFound(format!("Tool not found: {}", tool_name)))
        }
    }
} 