// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use crate::error::{MCPError, tool::ToolError};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

/// Registered MCP tool definition including capabilities and security metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Unique tool identifier used in the registry.
    pub id: String,
    /// Human-readable tool name.
    pub name: String,
    /// Semantic version string for the tool implementation.
    pub version: String,
    /// Short description of what the tool does.
    pub description: String,
    /// Capabilities exposed by this tool.
    pub capabilities: Vec<Capability>,
    /// Sensitivity tier enforced for this tool.
    pub security_level: SecurityLevel,
    /// Arbitrary key-value metadata for discovery or policy.
    pub metadata: HashMap<String, String>,
}

/// Single capability exposed by a tool, including schema and permission requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name used for routing and discovery.
    pub name: String,
    /// Human-readable description of the capability.
    pub description: String,
    /// Input parameters accepted by this capability.
    pub parameters: Vec<Parameter>,
    /// Declared return shape for this capability.
    pub return_type: ReturnType,
    /// Permission names required before invoking this capability.
    pub required_permissions: HashSet<String>,
}

/// Parameter schema entry for a tool capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name as exposed to callers.
    pub name: String,
    /// JSON-like type of the parameter.
    pub type_: ParameterType,
    /// Documentation for the parameter.
    pub description: String,
    /// Whether the parameter must be supplied.
    pub required: bool,
    /// Default JSON value when omitted, if any.
    pub default_value: Option<serde_json::Value>,
}

/// JSON-like scalar and container types for parameter and return typing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// String value.
    String,
    /// Numeric value.
    Number,
    /// Boolean value.
    Boolean,
    /// Array of values.
    Array,
    /// Object with string keys.
    Object,
}

/// Declared return type for a capability invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnType {
    /// JSON-like type of the return value.
    pub type_: ParameterType,
    /// Human-readable description of the return value.
    pub description: String,
}

/// Coarse security tier applied to tools and workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Low-sensitivity data or operations.
    Low,
    /// Moderate-sensitivity data or operations.
    Medium,
    /// High-sensitivity data or operations.
    High,
    /// Highest sensitivity; strictest controls.
    Critical,
}

/// Runtime bookkeeping for a registered tool.
#[derive(Debug, Clone)]
pub struct ToolState {
    /// Current lifecycle state of the tool.
    pub status: ToolStatus,
    /// Last time the tool was used or touched.
    pub last_used: DateTime<Utc>,
    /// Number of successful invocations recorded.
    pub usage_count: u64,
    /// Number of failed invocations recorded.
    pub error_count: u64,
}

/// Lifecycle state of a tool in the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    /// Tool is available for invocation.
    Active,
    /// Tool is registered but not currently used.
    Inactive,
    /// Tool is in an error state after repeated failures.
    Error,
    /// Tool is temporarily unavailable for maintenance.
    Maintenance,
}

/// Registry of tools, their states, and capability-to-tool indexes.
pub struct ToolManager {
    tools: RwLock<HashMap<String, Tool>>,
    states: RwLock<HashMap<String, ToolState>>,
    capabilities: RwLock<HashMap<String, HashSet<String>>>, // capability -> tool IDs
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolManager {
    /// Creates an empty tool manager with no registered tools.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a validated tool, indexes its capabilities, and initializes state.
    pub async fn register_tool(&self, tool: Tool) -> Result<()> {
        // Validate tool
        self.validate_tool(&tool)?;

        // Update capabilities index
        let mut capabilities = self.capabilities.write().await;
        for capability in &tool.capabilities {
            capabilities
                .entry(capability.name.clone())
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
        let tool_id = tool.id.clone();
        tools.insert(tool_id.clone(), tool);
        states.insert(tool_id, state);

        Ok(())
    }

    /// Removes a tool and its capability index entries.
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
        tools.remove(tool_id).ok_or_else(|| {
            MCPError::Tool(ToolError::RegistrationFailed(format!(
                "Tool not found: {tool_id}"
            )))
        })?;
        states.remove(tool_id);

        Ok(())
    }

    /// Returns a clone of the tool definition for the given id, if present.
    pub async fn get_tool(&self, tool_id: &str) -> Result<Option<Tool>> {
        let tools = self.tools.read().await;
        Ok(tools.get(tool_id).cloned())
    }

    /// Returns runtime state for the tool, if it exists.
    pub async fn get_tool_state(&self, tool_id: &str) -> Result<Option<ToolState>> {
        let states = self.states.read().await;
        Ok(states.get(tool_id).cloned())
    }

    /// Updates the tool status and refreshes last-used time.
    pub async fn update_tool_state(&self, tool_id: &str, status: ToolStatus) -> Result<()> {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tool_id) {
            state.status = status;
            state.last_used = Utc::now();
        } else {
            return Err(MCPError::Tool(ToolError::LifecycleError(format!(
                "Tool not found: {tool_id}"
            )))
            .into());
        }
        Ok(())
    }

    /// Returns tool ids that advertise the given capability name.
    pub async fn find_tools_by_capability(&self, capability: &str) -> Result<HashSet<String>> {
        let capabilities = self.capabilities.read().await;
        Ok(capabilities.get(capability).cloned().unwrap_or_default())
    }

    /// Ensures the tool exists and lists the given capability.
    pub async fn validate_capability(&self, tool_id: &str, capability: &str) -> Result<()> {
        let tools = self.tools.read().await;
        let tool = tools.get(tool_id).ok_or_else(|| {
            MCPError::Tool(ToolError::ValidationFailed(format!(
                "Tool not found: {tool_id}"
            )))
        })?;

        if !tool.capabilities.iter().any(|c| c.name == capability) {
            return Err(MCPError::Tool(ToolError::ValidationFailed(format!(
                "Tool {tool_id} does not have capability {capability}"
            )))
            .into());
        }

        Ok(())
    }

    #[expect(
        clippy::unused_self,
        reason = "method will use self when validation checks security context"
    )]
    fn validate_tool(&self, tool: &Tool) -> Result<()> {
        // Validate basic fields
        if tool.id.is_empty() {
            return Err(MCPError::Tool(ToolError::ValidationFailed(
                "Tool ID cannot be empty".to_string(),
            ))
            .into());
        }

        if tool.name.is_empty() {
            return Err(MCPError::Tool(ToolError::ValidationFailed(
                "Tool name cannot be empty".to_string(),
            ))
            .into());
        }

        if tool.version.is_empty() {
            return Err(MCPError::Tool(ToolError::ValidationFailed(
                "Tool version cannot be empty".to_string(),
            ))
            .into());
        }

        // Validate capabilities
        if tool.capabilities.is_empty() {
            return Err(MCPError::Tool(ToolError::ValidationFailed(
                "Tool must have at least one capability".to_string(),
            ))
            .into());
        }

        for capability in &tool.capabilities {
            if capability.name.is_empty() {
                return Err(MCPError::Tool(ToolError::ValidationFailed(
                    "Capability name cannot be empty".to_string(),
                ))
                .into());
            }

            // Validate parameters
            for param in &capability.parameters {
                if param.name.is_empty() {
                    return Err(MCPError::Tool(ToolError::ValidationFailed(
                        "Parameter name cannot be empty".to_string(),
                    ))
                    .into());
                }
            }
        }

        Ok(())
    }

    /// Increments usage count and updates last-used time for a tool.
    pub async fn increment_usage(&self, tool_id: &str) -> Result<()> {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(tool_id) {
            state.usage_count += 1;
            state.last_used = Utc::now();
        }
        Ok(())
    }

    /// Increments error count and may mark the tool as errored after a threshold.
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

    /// Returns all tools whose state is active.
    pub async fn get_active_tools(&self) -> Result<Vec<Tool>> {
        let tools = self.tools.read().await;
        let states = self.states.read().await;

        Ok(tools
            .values()
            .filter(|tool| {
                states
                    .get(&tool.id)
                    .is_some_and(|state| state.status == ToolStatus::Active)
            })
            .cloned()
            .collect())
    }

    /// Returns tools whose declared security level matches the filter.
    pub async fn get_tools_by_security_level(&self, level: SecurityLevel) -> Result<Vec<Tool>> {
        let tools = self.tools.read().await;
        Ok(tools
            .values()
            .filter(|tool| tool.security_level == level)
            .cloned()
            .collect())
    }
}
