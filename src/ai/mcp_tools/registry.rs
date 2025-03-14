use crate::ai::mcp_tools::{
    context::MachineContext,
    types::MCPError,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use std::time::SystemTime;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use crate::ai::mcp_tools::types::{
    ToolState, ToolEvent, ToolConfig,
    ResourceLimits, SecurityRequirements,
};

/// Tool capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapability {
    pub name: String,
    pub description: String,
    pub version: String,
    pub parameters: Vec<ToolParameter>,
    pub return_type: String,
    pub security_level: String,
}

/// Tool parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub parameter_type: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Tool registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistration {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<ToolCapability>,
    pub metadata: HashMap<String, String>,
}

/// Tool instance
#[derive(Debug)]
struct Tool {
    config: ToolConfig,
    state: ToolState,
    active_requests: HashMap<String, SystemTime>,
}

/// Registry service for managing MCP tools
pub struct RegistryService {
    tools: Arc<RwLock<HashMap<String, Tool>>>,
    event_tx: broadcast::Sender<ToolEvent>,
    event_rx: broadcast::Receiver<ToolEvent>,
}

impl RegistryService {
    /// Create a new registry service
    pub fn new() -> Self {
        let (event_tx, event_rx) = broadcast::channel(100);
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx,
        }
    }

    /// Register a new tool
    pub async fn register_tool(&mut self, config: ToolConfig) -> Result<(), MCPError> {
        let tool_id = config.id.clone();
        info!(tool_id = %tool_id, "Registering tool");

        // Validate tool configuration
        self.validate_tool_config(&config)?;

        // Check dependencies
        self.check_dependencies(&config)?;

        // Create tool instance
        let tool = Tool {
            config,
            state: ToolState::Registered,
            active_requests: HashMap::new(),
        };

        // Add tool to registry
        let mut tools = self.tools.write()
            .map_err(|_| MCPError::ToolError("Failed to acquire lock".to_string()))?;
        tools.insert(tool_id.clone(), tool);

        // Emit registration event
        self.emit_event(ToolEvent::Registered {
            tool_id,
            timestamp: SystemTime::now(),
        })?;

        Ok(())
    }

    /// Initialize a tool
    pub async fn initialize_tool(&mut self, tool_id: &str) -> Result<(), MCPError> {
        info!(tool_id = %tool_id, "Initializing tool");

        // Get tool
        let mut tools = self.tools.write()
            .map_err(|_| MCPError::ToolError("Failed to acquire lock".to_string()))?;
        let tool = tools.get_mut(tool_id)
            .ok_or_else(|| MCPError::ToolError(format!("Tool not found: {}", tool_id)))?;

        // Check if tool can be initialized
        if tool.state != ToolState::Registered {
            return Err(MCPError::ToolError(format!("Tool {} is not in registered state", tool_id)));
        }

        // Update tool state
        tool.state = ToolState::Initializing;

        // Emit initialization started event
        self.emit_event(ToolEvent::InitializationStarted {
            tool_id: tool_id.to_string(),
            timestamp: SystemTime::now(),
        })?;

        // TODO: Perform actual initialization (e.g., load resources, connect to services)

        // Update tool state
        tool.state = ToolState::Ready;

        // Emit initialization completed event
        self.emit_event(ToolEvent::InitializationCompleted {
            tool_id: tool_id.to_string(),
            timestamp: SystemTime::now(),
        })?;

        Ok(())
    }

    /// Start processing a request
    pub async fn start_processing(&mut self, tool_id: &str, request_id: &str) -> Result<(), MCPError> {
        info!(tool_id = %tool_id, request_id = %request_id, "Starting request processing");

        // Get tool
        let mut tools = self.tools.write()
            .map_err(|_| MCPError::ToolError("Failed to acquire lock".to_string()))?;
        let tool = tools.get_mut(tool_id)
            .ok_or_else(|| MCPError::ToolError(format!("Tool not found: {}", tool_id)))?;

        // Check if tool can process request
        if tool.state != ToolState::Ready {
            return Err(MCPError::ToolError(format!("Tool {} is not ready", tool_id)));
        }

        // Check concurrent operations limit
        if tool.active_requests.len() >= tool.config.max_concurrent_operations {
            return Err(MCPError::ResourceLimitExceeded(format!(
                "Tool {} has reached maximum concurrent operations",
                tool_id
            )));
        }

        // Add request to active requests
        tool.active_requests.insert(request_id.to_string(), SystemTime::now());

        // Update tool state
        tool.state = ToolState::Processing;

        // Emit processing started event
        self.emit_event(ToolEvent::ProcessingStarted {
            tool_id: tool_id.to_string(),
            request_id: request_id.to_string(),
            timestamp: SystemTime::now(),
        })?;

        Ok(())
    }

    /// Complete request processing
    pub async fn complete_processing(&mut self, tool_id: &str, request_id: &str) -> Result<(), MCPError> {
        info!(tool_id = %tool_id, request_id = %request_id, "Completing request processing");

        // Get tool
        let mut tools = self.tools.write()
            .map_err(|_| MCPError::ToolError("Failed to acquire lock".to_string()))?;
        let tool = tools.get_mut(tool_id)
            .ok_or_else(|| MCPError::ToolError(format!("Tool not found: {}", tool_id)))?;

        // Remove request from active requests
        tool.active_requests.remove(request_id);

        // Update tool state if no active requests
        if tool.active_requests.is_empty() {
            tool.state = ToolState::Ready;
        }

        // Emit processing completed event
        self.emit_event(ToolEvent::ProcessingCompleted {
            tool_id: tool_id.to_string(),
            request_id: request_id.to_string(),
            timestamp: SystemTime::now(),
        })?;

        Ok(())
    }

    /// Pause a tool
    pub async fn pause_tool(&mut self, tool_id: &str) -> Result<(), MCPError> {
        info!(tool_id = %tool_id, "Pausing tool");

        // Get tool
        let mut tools = self.tools.write()
            .map_err(|_| MCPError::ToolError("Failed to acquire lock".to_string()))?;
        let tool = tools.get_mut(tool_id)
            .ok_or_else(|| MCPError::ToolError(format!("Tool not found: {}", tool_id)))?;

        // Check if tool can be paused
        if tool.state != ToolState::Ready && tool.state != ToolState::Processing {
            return Err(MCPError::ToolError(format!("Tool {} cannot be paused in current state", tool_id)));
        }

        // Update tool state
        tool.state = ToolState::Paused;

        // Emit paused event
        self.emit_event(ToolEvent::Paused {
            tool_id: tool_id.to_string(),
            timestamp: SystemTime::now(),
        })?;

        Ok(())
    }

    /// Resume a tool
    pub async fn resume_tool(&mut self, tool_id: &str) -> Result<(), MCPError> {
        info!(tool_id = %tool_id, "Resuming tool");

        // Get tool
        let mut tools = self.tools.write()
            .map_err(|_| MCPError::ToolError("Failed to acquire lock".to_string()))?;
        let tool = tools.get_mut(tool_id)
            .ok_or_else(|| MCPError::ToolError(format!("Tool not found: {}", tool_id)))?;

        // Check if tool can be resumed
        if tool.state != ToolState::Paused {
            return Err(MCPError::ToolError(format!("Tool {} is not paused", tool_id)));
        }

        // Update tool state
        tool.state = if tool.active_requests.is_empty() {
            ToolState::Ready
        } else {
            ToolState::Processing
        };

        // Emit resumed event
        self.emit_event(ToolEvent::Resumed {
            tool_id: tool_id.to_string(),
            timestamp: SystemTime::now(),
        })?;

        Ok(())
    }

    /// Shutdown a tool
    pub async fn shutdown_tool(&mut self, tool_id: &str) -> Result<(), MCPError> {
        info!(tool_id = %tool_id, "Shutting down tool");

        // Get tool
        let mut tools = self.tools.write()
            .map_err(|_| MCPError::ToolError("Failed to acquire lock".to_string()))?;
        let tool = tools.get_mut(tool_id)
            .ok_or_else(|| MCPError::ToolError(format!("Tool not found: {}", tool_id)))?;

        // Update tool state
        tool.state = ToolState::ShuttingDown;

        // Emit shutdown started event
        self.emit_event(ToolEvent::ShutdownStarted {
            tool_id: tool_id.to_string(),
            timestamp: SystemTime::now(),
        })?;

        // TODO: Perform actual shutdown (e.g., cleanup resources, disconnect from services)

        // Remove tool from registry
        tools.remove(tool_id);

        // Emit shutdown completed event
        self.emit_event(ToolEvent::ShutdownCompleted {
            tool_id: tool_id.to_string(),
            timestamp: SystemTime::now(),
        })?;

        Ok(())
    }

    /// Get tool state
    pub fn get_tool_state(&self, tool_id: &str) -> Result<ToolState, MCPError> {
        let tools = self.tools.read()
            .map_err(|_| MCPError::ToolError("Failed to acquire lock".to_string()))?;
        let tool = tools.get(tool_id)
            .ok_or_else(|| MCPError::ToolError(format!("Tool not found: {}", tool_id)))?;
        Ok(tool.state)
    }

    /// Subscribe to tool events
    pub fn subscribe(&self) -> broadcast::Receiver<ToolEvent> {
        self.event_tx.subscribe()
    }

    /// Emit a tool event
    fn emit_event(&self, event: ToolEvent) -> Result<(), MCPError> {
        self.event_tx.send(event)
            .map_err(|e| MCPError::ToolError(format!("Failed to emit event: {}", e)))?;
        Ok(())
    }

    /// Validate tool configuration
    fn validate_tool_config(&self, config: &ToolConfig) -> Result<(), MCPError> {
        // Check required fields
        if config.id.is_empty() {
            return Err(MCPError::ToolError("Tool ID is required".to_string()));
        }
        if config.name.is_empty() {
            return Err(MCPError::ToolError("Tool name is required".to_string()));
        }
        if config.version.is_empty() {
            return Err(MCPError::ToolError("Tool version is required".to_string()));
        }

        // Validate resource limits
        if config.resource_limits.max_memory == 0 {
            return Err(MCPError::ToolError("Invalid memory limit".to_string()));
        }
        if config.resource_limits.max_cpu_percent == 0 || config.resource_limits.max_cpu_percent > 100 {
            return Err(MCPError::ToolError("Invalid CPU limit".to_string()));
        }

        Ok(())
    }

    /// Check tool dependencies
    fn check_dependencies(&self, config: &ToolConfig) -> Result<(), MCPError> {
        let tools = self.tools.read()
            .map_err(|_| MCPError::ToolError("Failed to acquire lock".to_string()))?;

        for dep in &config.dependencies {
            if !tools.contains_key(dep) {
                return Err(MCPError::DependencyError(format!("Missing dependency: {}", dep)));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(id: &str) -> ToolConfig {
        ToolConfig {
            id: id.to_string(),
            name: format!("Test Tool {}", id),
            version: "1.0.0".to_string(),
            description: "Test tool".to_string(),
            capabilities: vec!["test".to_string()],
            dependencies: vec![],
            max_concurrent_operations: 5,
            resource_limits: ResourceLimits {
                max_memory: 1024 * 1024 * 100, // 100MB
                max_cpu_percent: 50,
                max_storage: 1024 * 1024 * 1000, // 1GB
                max_bandwidth: 1024 * 1024, // 1MB/s
            },
            security_requirements: SecurityRequirements {
                security_level: crate::ai::mcp_tools::types::SecurityLevel::Medium,
                required_permissions: vec!["test".to_string()],
                encryption_required: false,
                authentication_required: true,
            },
        }
    }

    #[tokio::test]
    async fn test_tool_lifecycle() {
        let mut registry = RegistryService::new();
        let config = create_test_config("test1");

        // Register tool
        registry.register_tool(config).await.unwrap();
        assert_eq!(registry.get_tool_state("test1").unwrap(), ToolState::Registered);

        // Initialize tool
        registry.initialize_tool("test1").await.unwrap();
        assert_eq!(registry.get_tool_state("test1").unwrap(), ToolState::Ready);

        // Start processing
        registry.start_processing("test1", "req1").await.unwrap();
        assert_eq!(registry.get_tool_state("test1").unwrap(), ToolState::Processing);

        // Complete processing
        registry.complete_processing("test1", "req1").await.unwrap();
        assert_eq!(registry.get_tool_state("test1").unwrap(), ToolState::Ready);

        // Pause tool
        registry.pause_tool("test1").await.unwrap();
        assert_eq!(registry.get_tool_state("test1").unwrap(), ToolState::Paused);

        // Resume tool
        registry.resume_tool("test1").await.unwrap();
        assert_eq!(registry.get_tool_state("test1").unwrap(), ToolState::Ready);

        // Shutdown tool
        registry.shutdown_tool("test1").await.unwrap();
        assert!(registry.get_tool_state("test1").is_err());
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let mut registry = RegistryService::new();
        let mut config = create_test_config("test2");
        config.max_concurrent_operations = 2;

        // Register and initialize tool
        registry.register_tool(config).await.unwrap();
        registry.initialize_tool("test2").await.unwrap();

        // Start two requests
        registry.start_processing("test2", "req1").await.unwrap();
        registry.start_processing("test2", "req2").await.unwrap();

        // Third request should fail
        assert!(registry.start_processing("test2", "req3").await.is_err());

        // Complete one request
        registry.complete_processing("test2", "req1").await.unwrap();

        // Now third request should succeed
        registry.start_processing("test2", "req3").await.unwrap();
    }

    #[tokio::test]
    async fn test_dependency_validation() {
        let mut registry = RegistryService::new();
        let config1 = create_test_config("test3");
        let mut config2 = create_test_config("test4");
        config2.dependencies = vec!["test3".to_string()];

        // Register first tool
        registry.register_tool(config1).await.unwrap();

        // Register second tool with dependency
        registry.register_tool(config2).await.unwrap();

        // Try to register tool with missing dependency
        let mut config3 = create_test_config("test5");
        config3.dependencies = vec!["missing".to_string()];
        assert!(registry.register_tool(config3).await.is_err());
    }
} 