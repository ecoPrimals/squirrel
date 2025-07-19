use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use crate::mcp::error::{MCPError, ToolError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<Capability>,
    pub security_level: SecurityLevel,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub return_type: ReturnType,
    pub required_permissions: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_: ParameterType,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnType {
    pub type_: ParameterType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ToolState {
    pub status: ToolStatus,
    pub last_used: DateTime<Utc>,
    pub usage_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolStatus {
    Active,
    Inactive,
    Error,
    Maintenance,
}

pub struct ToolManager {
    tools: RwLock<HashMap<String, Tool>>,
    states: RwLock<HashMap<String, ToolState>>,
    capabilities: RwLock<HashMap<String, HashSet<String>>>, // capability -> tool IDs
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_tool(&self, tool: Tool) -> Result<()> {
        // Validate tool
        self.validate_tool(&tool)?;

        // Update capabilities index
        let mut capabilities = self.capabilities.write().await;
        for capability in &tool.capabilities {
            capabilities.entry(capability.name.clone())
                .or_insert_with(HashSet::new)
                .insert(tool.id.clone());
        }

        // Initialize tool state
        let state = ToolState {
            status: ToolStatus::Active,
            last_used: Utc::now(),
            usage_count: 0,
            error_count: 0,
        };

        // Store tool and state
        let mut tools = self.tools.write().await;
        let mut states = self.states.write().await;
        tools.insert(tool.id.clone(), tool);
        states.insert(tool.id.clone(), state);

        Ok(())
    }

    pub async fn unregister_tool(&self, tool_id: &str) -> Result<()> {
        // Remove tool capabilities from index
        let mut capabilities = self.capabilities.write().await;
        if let Some(tool) = self.get_tool(tool_id).await? {
            for capability in &tool.capabilities {
                if let Some(tools) = capabilities.get_mut(&capability.name) {
                    tools.remove(tool_id);
                }
            }
        }

        // Remove tool and state
        let mut tools = self.tools.write().await;
        let mut states = self.states.write().await;
        tools.remove(tool_id)
            .ok_or_else(|| MCPError::Tool(ToolError::RegistrationFailed(
                format!("Tool not found: {}", tool_id)
            )))?;
        states.remove(tool_id);

        Ok(())
    }

    pub async fn get_tool(&self, tool_id: &str) -> Result<Option<Tool>> {
        let tools = self.tools.read().await;
        Ok(tools.get(tool_id).cloned())
    }

    pub async fn get_tool_state(&self, tool_id: &str) -> Result<Option<ToolState>> {
        let states = self.states.read().await;
        Ok(states.get(tool_id).cloned())
    }

    pub async fn update_tool_state(&self, tool_id: &str, status: ToolStatus) -> Result<()> {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tool_id) {
            state.status = status;
            state.last_used = Utc::now();
        } else {
            return Err(MCPError::Tool(ToolError::LifecycleError(
                format!("Tool not found: {}", tool_id)
            )).into());
        }
        Ok(())
    }

    pub async fn find_tools_by_capability(&self, capability: &str) -> Result<HashSet<String>> {
        let capabilities = self.capabilities.read().await;
        Ok(capabilities.get(capability)
            .cloned()
            .unwrap_or_default())
    }

    pub async fn validate_capability(&self, tool_id: &str, capability: &str) -> Result<()> {
        let tools = self.tools.read().await;
        let tool = tools.get(tool_id)
            .ok_or_else(|| MCPError::Tool(ToolError::ValidationFailed(
                format!("Tool not found: {}", tool_id)
            )))?;

        if !tool.capabilities.iter().any(|c| c.name == capability) {
            return Err(MCPError::Tool(ToolError::ValidationFailed(
                format!("Tool {} does not have capability {}", tool_id, capability)
            )).into());
        }

        Ok(())
    }

    fn validate_tool(&self, tool: &Tool) -> Result<()> {
        // Validate basic fields
        if tool.id.is_empty() {
            return Err(MCPError::Tool(ToolError::ValidationFailed(
                "Tool ID cannot be empty".to_string()
            )).into());
        }

        if tool.name.is_empty() {
            return Err(MCPError::Tool(ToolError::ValidationFailed(
                "Tool name cannot be empty".to_string()
            )).into());
        }

        if tool.version.is_empty() {
            return Err(MCPError::Tool(ToolError::ValidationFailed(
                "Tool version cannot be empty".to_string()
            )).into());
        }

        // Validate capabilities
        if tool.capabilities.is_empty() {
            return Err(MCPError::Tool(ToolError::ValidationFailed(
                "Tool must have at least one capability".to_string()
            )).into());
        }

        for capability in &tool.capabilities {
            if capability.name.is_empty() {
                return Err(MCPError::Tool(ToolError::ValidationFailed(
                    "Capability name cannot be empty".to_string()
                )).into());
            }

            // Validate parameters
            for param in &capability.parameters {
                if param.name.is_empty() {
                    return Err(MCPError::Tool(ToolError::ValidationFailed(
                        "Parameter name cannot be empty".to_string()
                    )).into());
                }
            }
        }

        Ok(())
    }

    pub async fn increment_usage(&self, tool_id: &str) -> Result<()> {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tool_id) {
            state.usage_count += 1;
            state.last_used = Utc::now();
        }
        Ok(())
    }

    pub async fn increment_error(&self, tool_id: &str) -> Result<()> {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tool_id) {
            state.error_count += 1;
            if state.error_count > 10 {
                state.status = ToolStatus::Error;
            }
        }
        Ok(())
    }

    pub async fn get_active_tools(&self) -> Result<Vec<Tool>> {
        let tools = self.tools.read().await;
        let states = self.states.read().await;
        
        Ok(tools.values()
            .filter(|tool| {
                states.get(&tool.id)
                    .map(|state| state.status == ToolStatus::Active)
                    .unwrap_or(false)
            })
            .cloned()
            .collect())
    }

    pub async fn get_tools_by_security_level(&self, level: SecurityLevel) -> Result<Vec<Tool>> {
        let tools = self.tools.read().await;
        Ok(tools.values()
            .filter(|tool| tool.security_level == level)
            .cloned()
            .collect())
    }
} 