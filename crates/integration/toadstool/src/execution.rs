//! Execution request types for Toadstool integration

use crate::{sandbox::SandboxPolicy, ExecutionEnvironment, ResourceLimits};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Request to execute a plugin via Toadstool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique identifier for this execution
    pub execution_id: Uuid,

    /// Plugin identifier
    pub plugin_id: String,

    /// Plugin code (base64 encoded for JSON transport)
    pub code: String,

    /// Execution environment configuration
    pub environment: ExecutionEnvironment,

    /// MCP context information
    pub mcp_context: Option<McpContext>,

    /// Priority of execution (0 = lowest, 10 = highest)
    pub priority: u8,

    /// Execution timeout in milliseconds  
    pub timeout: Option<u64>,

    /// Execution metadata
    pub metadata: HashMap<String, String>,
}

/// MCP-specific context for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    /// MCP session identifier
    pub session_id: String,

    /// Active agent information
    pub agent_info: AgentInfo,

    /// Available MCP tools
    pub available_tools: Vec<McpTool>,

    /// Context variables
    pub context_vars: HashMap<String, serde_json::Value>,

    /// Parent conversation/thread ID
    pub conversation_id: Option<String>,
}

/// Information about the AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent identifier
    pub agent_id: String,

    /// Agent name
    pub name: String,

    /// Agent version
    pub version: String,

    /// Agent capabilities
    pub capabilities: Vec<String>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,

    /// Tool description  
    pub description: String,

    /// Input schema
    pub input_schema: serde_json::Value,

    /// Tool metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ExecutionRequest {
    /// Create a new execution request
    pub fn new(plugin_id: String, code: String) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            plugin_id,
            code,
            environment: ExecutionEnvironment::default(),
            mcp_context: None,
            priority: 5,           // Default priority
            timeout: Some(30_000), // 30 second default timeout
            metadata: HashMap::new(),
        }
    }

    /// Set the execution environment
    pub fn with_environment(mut self, environment: ExecutionEnvironment) -> Self {
        self.environment = environment;
        self
    }

    /// Set MCP context
    pub fn with_mcp_context(mut self, context: McpContext) -> Self {
        self.mcp_context = Some(context);
        self
    }

    /// Set execution priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10); // Cap at 10
        self
    }

    /// Set execution timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Create WASM execution environment
    pub fn wasm_environment(
        resource_limits: ResourceLimits,
        sandbox_policy: SandboxPolicy,
    ) -> ExecutionEnvironment {
        ExecutionEnvironment {
            environment_type: "wasm".to_string(),
            resource_limits,
            security_policy: sandbox_policy,
            env_vars: HashMap::new(),
        }
    }

    /// Create native execution environment
    pub fn native_environment(
        resource_limits: ResourceLimits,
        sandbox_policy: SandboxPolicy,
    ) -> ExecutionEnvironment {
        ExecutionEnvironment {
            environment_type: "native".to_string(),
            resource_limits,
            security_policy: sandbox_policy,
            env_vars: HashMap::new(),
        }
    }

    /// Create container execution environment
    pub fn container_environment(
        resource_limits: ResourceLimits,
        sandbox_policy: SandboxPolicy,
    ) -> ExecutionEnvironment {
        ExecutionEnvironment {
            environment_type: "container".to_string(),
            resource_limits,
            security_policy: sandbox_policy,
            env_vars: HashMap::new(),
        }
    }
}

impl Default for ExecutionEnvironment {
    fn default() -> Self {
        Self {
            environment_type: "wasm".to_string(),
            resource_limits: ResourceLimits::default(),
            security_policy: SandboxPolicy::default(),
            env_vars: HashMap::new(),
        }
    }
}

impl McpContext {
    /// Create a new MCP context
    pub fn new(session_id: String, agent_info: AgentInfo) -> Self {
        Self {
            session_id,
            agent_info,
            available_tools: Vec::new(),
            context_vars: HashMap::new(),
            conversation_id: None,
        }
    }

    /// Add an available tool
    pub fn add_tool(mut self, tool: McpTool) -> Self {
        self.available_tools.push(tool);
        self
    }

    /// Add a context variable
    pub fn add_context_var(mut self, key: String, value: serde_json::Value) -> Self {
        self.context_vars.insert(key, value);
        self
    }

    /// Set conversation ID
    pub fn with_conversation(mut self, conversation_id: String) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }
}

impl AgentInfo {
    /// Create new agent info
    pub fn new(agent_id: String, name: String, version: String) -> Self {
        Self {
            agent_id,
            name,
            version,
            capabilities: Vec::new(),
        }
    }

    /// Add a capability
    pub fn add_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }
}

impl McpTool {
    /// Create a new MCP tool
    pub fn new(name: String, description: String, input_schema: serde_json::Value) -> Self {
        Self {
            name,
            description,
            input_schema,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}
