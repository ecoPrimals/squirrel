//! Universal Tool Executor and Management
//!
//! This module contains the UniversalToolExecutor and tool management
//! functionality for the AI coordinator system.

use std::collections::HashMap;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, debug, warn, error, instrument};

use crate::error::types::Result;
use super::types::{
    ToolExecutorConfig, ToolExecution, ToolResult, AISession, AIRequirements
};

/// Universal Tool Executor - can execute tools through ANY AI system
pub struct UniversalToolExecutor {
    /// AI coordinator for routing
    coordinator: Arc<super::AICoordinator>,
    
    /// Tool registry
    tools: Arc<RwLock<HashMap<String, Arc<dyn UniversalTool>>>>,
    
    /// Execution history
    history: Arc<RwLock<Vec<ToolExecution>>>,
    
    /// Configuration
    config: ToolExecutorConfig,
}

/// Universal tool trait - works with any AI system
#[async_trait::async_trait]
pub trait UniversalTool: Send + Sync {
    /// Tool name
    fn name(&self) -> &str;
    
    /// Tool description
    fn description(&self) -> &str;
    
    /// Tool parameters schema
    fn parameters_schema(&self) -> serde_json::Value;
    
    /// Execute tool with AI assistance
    async fn execute(&self, 
        params: serde_json::Value, 
        ai_context: &AISession
    ) -> Result<ToolResult>;
    
    /// AI requirements for this tool
    fn ai_requirements(&self) -> AIRequirements;
}

impl UniversalToolExecutor {
    /// Create a new tool executor
    pub async fn new(coordinator: Arc<super::AICoordinator>, config: ToolExecutorConfig) -> Result<Self> {
        Ok(Self {
            coordinator,
            tools: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            config,
        })
    }
    
    /// Register a tool
    pub async fn register_tool(&self, tool: Arc<dyn UniversalTool>) -> Result<()> {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name().to_string(), tool);
        Ok(())
    }
    
    /// Execute a tool
    #[instrument(skip(self, params))]
    pub async fn execute_tool(&self, 
        tool_name: &str, 
        params: serde_json::Value, 
        session_id: &str
    ) -> Result<ToolResult> {
        let start_time = std::time::Instant::now();
        
        // Get the tool
        let tool = {
            let tools = self.tools.read().await;
            tools.get(tool_name)
                .ok_or_else(|| crate::error::types::MCPError::Configuration(
                    format!("Tool '{}' not found", tool_name)
                ))?
                .clone()
        };
        
        // Get AI session (mock for now)
        let ai_session = self.get_or_create_session(session_id).await?;
        
        // Execute the tool
        let result = tool.execute(params.clone(), &ai_session).await?;
        
        // Record execution
        let execution = ToolExecution {
            id: Uuid::new_v4().to_string(),
            tool_name: tool_name.to_string(),
            ai_model: "mock".to_string(), // TODO: Get actual model from session
            parameters: params,
            result: result.clone(),
            duration: start_time.elapsed(),
            timestamp: chrono::Utc::now(),
        };
        
        // Store in history
        let mut history = self.history.write().await;
        history.push(execution);
        
        // Limit history size
        if history.len() > 1000 {
            history.drain(0..500);
        }
        
        Ok(result)
    }
    
    /// Get or create AI session
    async fn get_or_create_session(&self, session_id: &str) -> Result<AISession> {
        // Mock session for now
        Ok(AISession {
            id: session_id.to_string(),
            active_models: vec!["mock".to_string()],
            history: Vec::new(),
            preferences: super::types::UserPreferences {
                preferred_providers: Vec::new(),
                privacy_level: super::types::PrivacyLevel::Public,
                cost_sensitivity: super::types::CostSensitivity::Medium,
                quality_preference: super::types::QualityPreference::Balanced,
                language: None,
                timezone: None,
                theme: None,
            },
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
        })
    }
    
    /// Get tool definitions
    pub async fn get_tools(&self) -> Result<Vec<serde_json::Value>> {
        let tools = self.tools.read().await;
        let mut definitions = Vec::new();
        
        for tool in tools.values() {
            let definition = serde_json::json!({
                "name": tool.name(),
                "description": tool.description(),
                "parameters": tool.parameters_schema(),
                "ai_requirements": tool.ai_requirements()
            });
            definitions.push(definition);
        }
        
        Ok(definitions)
    }
    
    /// Get execution history
    pub async fn get_history(&self) -> Result<Vec<ToolExecution>> {
        let history = self.history.read().await;
        Ok(history.clone())
    }
    
    /// Clear execution history
    pub async fn clear_history(&self) -> Result<()> {
        let mut history = self.history.write().await;
        history.clear();
        Ok(())
    }
    
    /// Get tool by name
    pub async fn get_tool(&self, name: &str) -> Result<Option<Arc<dyn UniversalTool>>> {
        let tools = self.tools.read().await;
        Ok(tools.get(name).cloned())
    }
    
    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<String>> {
        let tools = self.tools.read().await;
        Ok(tools.keys().cloned().collect())
    }
    
    /// Remove a tool
    pub async fn remove_tool(&self, name: &str) -> Result<bool> {
        let mut tools = self.tools.write().await;
        Ok(tools.remove(name).is_some())
    }
}

/// Plugin Manager Interface
pub trait PluginManagerInterface: Send + Sync {
    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String>;
    
    /// Execute plugin
    fn execute_plugin(&self, name: &str, params: serde_json::Value) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + '_>>;
}

/// Tool Manager Interface
pub trait ToolManagerInterface: Send + Sync {
    /// Get tool definitions
    fn get_tools(&self) -> Vec<String>;
    
    /// Execute tool
    fn execute_tool(&self, name: &str, params: serde_json::Value) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + '_>>;
}

/// Plugin Manager Implementation
pub struct PluginManager {
    plugins: HashMap<String, String>,
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
}

impl PluginManagerInterface for PluginManager {
    fn get_capabilities(&self) -> Vec<String> {
        vec!["mock".to_string()]
    }
    
    fn execute_plugin(&self, _name: &str, _params: serde_json::Value) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + '_>> {
        Box::pin(async move {
            Ok(serde_json::json!({"result": "mock plugin execution"}))
        })
    }
}

/// Tool Manager Implementation
pub struct ToolManager {
    tools: HashMap<String, String>,
}

impl ToolManager {
    /// Create new tool manager
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
}

impl ToolManagerInterface for ToolManager {
    fn get_tools(&self) -> Vec<String> {
        vec!["mock".to_string()]
    }
    
    fn execute_tool(&self, _name: &str, _params: serde_json::Value) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + '_>> {
        Box::pin(async move {
            Ok(serde_json::json!({"result": "mock tool execution"}))
        })
    }
}

/// Example tool implementation
pub struct ExampleTool {
    name: String,
    description: String,
}

impl ExampleTool {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

#[async_trait::async_trait]
impl UniversalTool for ExampleTool {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input for the example tool"
                }
            },
            "required": ["input"]
        })
    }
    
    async fn execute(&self, 
        params: serde_json::Value, 
        _ai_context: &AISession
    ) -> Result<ToolResult> {
        let input = params.get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        
        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "output": format!("Processed: {}", input),
                "tool": self.name
            }),
            error: None,
            ai_analysis: Some("Tool executed successfully".to_string()),
        })
    }
    
    fn ai_requirements(&self) -> AIRequirements {
        AIRequirements {
            min_model_size: Some("small".to_string()),
            required_capabilities: vec!["text-generation".to_string()],
            max_cost: Some(0.01),
            max_latency: Some(std::time::Duration::from_secs(30)),
        }
    }
} 