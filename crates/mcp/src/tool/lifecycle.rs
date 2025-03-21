//! Tool lifecycle hooks for MCP
//!
//! This module provides implementations of tool lifecycle hooks for the MCP protocol.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use crate::tool::{
    Tool,
    ToolError,
    ToolLifecycleHook,
    ToolState,
};

/// Type alias for tool state history entries
pub type StateHistoryEntry = (ToolState, chrono::DateTime<Utc>);

/// Type alias for tool state history map
pub type StateHistoryMap = HashMap<String, Vec<StateHistoryEntry>>;

/// A basic tool lifecycle hook that logs events and maintains state history
#[derive(Debug)]
pub struct BasicLifecycleHook {
    /// History of state changes for each tool
    state_history: Arc<RwLock<StateHistoryMap>>,
    /// Maximum history entries to keep per tool
    max_history_entries: usize,
}

impl Default for BasicLifecycleHook {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicLifecycleHook {
    /// Creates a new basic lifecycle hook
    pub fn new() -> Self {
        Self {
            state_history: Arc::new(RwLock::new(HashMap::new())),
            max_history_entries: 100,
        }
    }
    
    /// Sets the maximum number of history entries to keep per tool
    pub fn with_max_history_entries(mut self, max_entries: usize) -> Self {
        self.max_history_entries = max_entries;
        self
    }
    
    /// Gets the state history for a tool
    pub async fn get_state_history(&self, tool_id: &str) -> Vec<(ToolState, chrono::DateTime<Utc>)> {
        let history = self.state_history.read().await;
        history.get(tool_id).cloned().unwrap_or_default()
    }
    
    /// Adds a state change to the history
    async fn record_state_change(&self, tool_id: &str, state: ToolState) {
        let mut history = self.state_history.write().await;
        let tool_history = history.entry(tool_id.to_string()).or_insert_with(Vec::new);
        
        // Add the new state change
        tool_history.push((state, Utc::now()));
        
        // Trim the history if it exceeds the maximum size
        if tool_history.len() > self.max_history_entries {
            let excess = tool_history.len() - self.max_history_entries;
            tool_history.drain(0..excess);
        }
    }
}

#[async_trait]
impl ToolLifecycleHook for BasicLifecycleHook {
    #[instrument(skip(self, tool))]
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        info!("Tool registered: {} ({})", tool.name, tool.id);
        
        // Record the initial state
        self.record_state_change(&tool.id, ToolState::Registered).await;
        
        // Log the tool capabilities
        for capability in &tool.capabilities {
            debug!("Capability registered: {} for tool {}", capability, tool.id);
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool unregistered: {}", tool_id);
        
        // Record the final state
        self.record_state_change(tool_id, ToolState::Unregistered).await;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_activate(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool activated: {}", tool_id);
        
        // Record the state change
        self.record_state_change(tool_id, ToolState::Active).await;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool deactivated: {}", tool_id);
        
        // Record the state change
        self.record_state_change(tool_id, ToolState::Inactive).await;
        
        Ok(())
    }
    
    #[instrument(skip(self, error))]
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        warn!("Tool error: {} - {}", tool_id, error);
        
        // Record the error state
        self.record_state_change(tool_id, ToolState::Error).await;
        
        Ok(())
    }
}

/// A lifecycle hook that performs additional validation and security checks
#[derive(Debug)]
pub struct SecurityLifecycleHook {
    /// The security level required for capabilities by default
    default_security_level: u8,
    /// Tool IDs that are allowed to register
    allowed_tool_ids: Vec<String>,
    /// Whether to enforce allowed tool IDs
    enforce_allowed_tools: bool,
}

impl Default for SecurityLifecycleHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityLifecycleHook {
    /// Creates a new security lifecycle hook
    pub fn new() -> Self {
        Self {
            default_security_level: 1,
            allowed_tool_ids: Vec::new(),
            enforce_allowed_tools: false,
        }
    }
    
    /// Sets the default security level for capabilities
    pub fn with_default_security_level(mut self, level: u8) -> Self {
        self.default_security_level = level;
        self
    }
    
    /// Adds an allowed tool ID
    pub fn allow_tool(mut self, tool_id: impl Into<String>) -> Self {
        self.allowed_tool_ids.push(tool_id.into());
        self
    }
    
    /// Sets whether to enforce allowed tool IDs
    pub fn enforce_allowed_tools(mut self, enforce: bool) -> Self {
        self.enforce_allowed_tools = enforce;
        self
    }
    
    /// Validates a tool's security metadata
    fn validate_tool_security(&self, tool: &Tool) -> Result<(), ToolError> {
        // Check if the tool is allowed to register
        if self.enforce_allowed_tools && !self.allowed_tool_ids.contains(&tool.id) {
            return Err(ToolError::ValidationFailed(format!(
                "Tool ID '{}' is not in the allowed list",
                tool.id
            )));
        }
        
        // Ensure the tool has a security level
        if tool.security_level < self.default_security_level {
            return Err(ToolError::ValidationFailed(format!(
                "Tool '{}' has insufficient security level: {} (minimum: {})",
                tool.id, tool.security_level, self.default_security_level
            )));
        }
        
        Ok(())
    }
}

#[async_trait]
impl ToolLifecycleHook for SecurityLifecycleHook {
    #[instrument(skip(self, tool))]
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        info!("Validating security for tool: {} ({})", tool.name, tool.id);
        
        // Validate the tool's security metadata
        self.validate_tool_security(tool)?;
        
        // Log the security level
        info!(
            "Tool '{}' registered with security level: {}",
            tool.id, tool.security_level
        );
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Security hook: Tool unregistered: {}", tool_id);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_activate(&self, tool_id: &str) -> Result<(), ToolError> {
        // For a real security hook, we might revalidate the tool here
        // or check if activation is allowed based on current security context
        info!("Security hook: Tool activated: {}", tool_id);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Security hook: Tool deactivated: {}", tool_id);
        Ok(())
    }
    
    #[instrument(skip(self, error))]
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        error!("Security critical tool error: {} - {}", tool_id, error);
        
        // In a real security hook, we might take different actions based on the error
        // such as quarantining the tool or raising alerts
        
        Ok(())
    }
}

/// A composite lifecycle hook that combines multiple hooks
#[derive(Debug, Default)]
pub struct CompositeLifecycleHook {
    /// The hooks to execute
    hooks: Vec<Arc<dyn ToolLifecycleHook + Send + Sync>>,
}

impl CompositeLifecycleHook {
    /// Creates a new composite lifecycle hook
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
        }
    }
    
    /// Adds a hook to the composite
    pub fn add_hook<H>(&mut self, hook: H)
    where
        H: ToolLifecycleHook + Send + Sync + 'static,
    {
        self.hooks.push(Arc::new(hook));
    }
    
    /// Creates a new composite lifecycle hook with the given hooks
    pub fn with_hooks<I, H>(hooks: I) -> Self
    where
        I: IntoIterator<Item = H>,
        H: ToolLifecycleHook + Send + Sync + 'static,
    {
        Self {
            hooks: hooks.into_iter().map(|h| Arc::new(h) as Arc<dyn ToolLifecycleHook + Send + Sync>).collect(),
        }
    }
}

#[async_trait]
impl ToolLifecycleHook for CompositeLifecycleHook {
    #[instrument(skip(self, tool))]
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        for hook in &self.hooks {
            if let Err(err) = hook.on_register(tool).await {
                warn!("Hook failed during registration of tool {}: {}", tool.id, err);
                return Err(err);
            }
        }
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut last_error = None;
        
        // Try to execute all hooks even if some fail
        for hook in &self.hooks {
            if let Err(err) = hook.on_unregister(tool_id).await {
                warn!("Hook failed during unregistration of tool {}: {}", tool_id, err);
                last_error = Some(err);
            }
        }
        
        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }
    
    #[instrument(skip(self))]
    async fn on_activate(&self, tool_id: &str) -> Result<(), ToolError> {
        for hook in &self.hooks {
            if let Err(err) = hook.on_activate(tool_id).await {
                warn!("Hook failed during activation of tool {}: {}", tool_id, err);
                return Err(err);
            }
        }
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut last_error = None;
        
        // Try to execute all hooks even if some fail
        for hook in &self.hooks {
            if let Err(err) = hook.on_deactivate(tool_id).await {
                warn!("Hook failed during deactivation of tool {}: {}", tool_id, err);
                last_error = Some(err);
            }
        }
        
        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }
    
    #[instrument(skip(self, error))]
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        let mut last_error = None;
        
        // Try to execute all hooks even if some fail
        for hook in &self.hooks {
            if let Err(err) = hook.on_error(tool_id, error).await {
                warn!("Hook failed during error handling for tool {}: {}", tool_id, err);
                last_error = Some(err);
            }
        }
        
        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::Capability;
    
    #[tokio::test]
    async fn test_basic_lifecycle_hook() {
        let hook = BasicLifecycleHook::new().with_max_history_entries(10);
        
        // Create a test tool
        let tool = Tool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool".to_string(),
            capabilities: vec![
                Capability {
                    name: "test".to_string(),
                    description: "A test capability".to_string(),
                    parameters: Vec::new(),
                    return_type: None,
                },
            ],
            security_level: 1,
        };
        
        // Test the registration hook
        let result = hook.on_register(&tool).await;
        assert!(result.is_ok(), "Registration hook failed: {:?}", result);
        
        // Test the activation hook
        let result = hook.on_activate(&tool.id).await;
        assert!(result.is_ok(), "Activation hook failed: {:?}", result);
        
        // Test the deactivation hook
        let result = hook.on_deactivate(&tool.id).await;
        assert!(result.is_ok(), "Deactivation hook failed: {:?}", result);
        
        // Test the error hook
        let error = ToolError::ExecutionFailed("Test error".to_string());
        let result = hook.on_error(&tool.id, &error).await;
        assert!(result.is_ok(), "Error hook failed: {:?}", result);
        
        // Test the unregistration hook
        let result = hook.on_unregister(&tool.id).await;
        assert!(result.is_ok(), "Unregistration hook failed: {:?}", result);
        
        // Check the state history
        let history = hook.get_state_history(&tool.id).await;
        assert_eq!(history.len(), 5, "Expected 5 state changes, got {}", history.len());
        
        assert_eq!(history[0].0, ToolState::Registered);
        assert_eq!(history[1].0, ToolState::Active);
        assert_eq!(history[2].0, ToolState::Inactive);
        assert_eq!(history[3].0, ToolState::Error);
        assert_eq!(history[4].0, ToolState::Unregistered);
    }
    
    #[tokio::test]
    async fn test_security_lifecycle_hook() {
        let hook = SecurityLifecycleHook::new()
            .with_default_security_level(2)
            .allow_tool("allowed-tool")
            .enforce_allowed_tools(true);
        
        // Create an allowed tool with sufficient security level
        let allowed_tool = Tool {
            id: "allowed-tool".to_string(),
            name: "Allowed Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "An allowed tool".to_string(),
            capabilities: Vec::new(),
            security_level: 2,
        };
        
        // Test the registration hook with an allowed tool
        let result = hook.on_register(&allowed_tool).await;
        assert!(result.is_ok(), "Registration hook failed for allowed tool: {:?}", result);
        
        // Create a disallowed tool
        let disallowed_tool = Tool {
            id: "disallowed-tool".to_string(),
            name: "Disallowed Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A disallowed tool".to_string(),
            capabilities: Vec::new(),
            security_level: 2,
        };
        
        // Test the registration hook with a disallowed tool
        let result = hook.on_register(&disallowed_tool).await;
        assert!(result.is_err(), "Registration hook should fail for disallowed tool");
        
        // Create a tool with insufficient security level
        let insecure_tool = Tool {
            id: "allowed-tool".to_string(), // Same ID as allowed tool
            name: "Insecure Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "An insecure tool".to_string(),
            capabilities: Vec::new(),
            security_level: 1, // Below required level
        };
        
        // Test the registration hook with an insecure tool
        let result = hook.on_register(&insecure_tool).await;
        assert!(result.is_err(), "Registration hook should fail for insecure tool");
    }
    
    #[tokio::test]
    async fn test_composite_lifecycle_hook() {
        let basic_hook = BasicLifecycleHook::new();
        let security_hook = SecurityLifecycleHook::new()
            .with_default_security_level(1)
            .enforce_allowed_tools(false);
        
        let mut composite_hook = CompositeLifecycleHook::new();
        composite_hook.add_hook(basic_hook);
        composite_hook.add_hook(security_hook);
        
        // Create a test tool
        let tool = Tool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool".to_string(),
            capabilities: Vec::new(),
            security_level: 1,
        };
        
        // Test the registration hook
        let result = composite_hook.on_register(&tool).await;
        assert!(result.is_ok(), "Composite registration hook failed: {:?}", result);
        
        // Test the activation hook
        let result = composite_hook.on_activate(&tool.id).await;
        assert!(result.is_ok(), "Composite activation hook failed: {:?}", result);
        
        // Test the error hook
        let error = ToolError::ExecutionFailed("Test error".to_string());
        let result = composite_hook.on_error(&tool.id, &error).await;
        assert!(result.is_ok(), "Composite error hook failed: {:?}", result);
    }
} 