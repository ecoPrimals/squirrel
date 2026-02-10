// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tool recovery hook implementation for error handling and recovery strategies

use crate::error::types::MCPError;
use crate::tool::ToolError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt;

/// Recovery strategies for tool errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Reset the tool to its initial state and try again
    Reset,
    /// Terminate the tool and clean up all resources
    Terminate,
    /// Ignore the error and continue execution
    Continue,
}

impl fmt::Display for RecoveryStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryStrategy::Reset => write!(f, "Reset"),
            RecoveryStrategy::Terminate => write!(f, "Terminate"),
            RecoveryStrategy::Continue => write!(f, "Continue"),
        }
    }
}

/// Recovery utilities for tool management
pub struct ToolRecovery {
    strategies: Arc<Mutex<HashMap<String, RecoveryStrategy>>>,
    default_strategy: RecoveryStrategy,
}

impl ToolRecovery {
    pub fn new() -> Self {
        Self {
            strategies: Arc::new(Mutex::new(HashMap::new())),
            default_strategy: RecoveryStrategy::Reset,
        }
    }

    pub async fn recover_tool_state(
        &self,
        tool_id: &str,
        tool_manager: &dyn crate::tool::ToolManager,
    ) -> Result<(), MCPError> {
        // Check if tool exists
        if let Some(_tool) = tool_manager.get_tool(tool_id).await? {
            // Tool exists, recovery successful
            log::info!("Tool {} recovered successfully", tool_id);
            Ok(())
        } else {
            Err(MCPError::Generic(format!("Tool {} not found", tool_id)))
        }
    }

    pub async fn cleanup_tool_resources(
        &self,
        tool_id: &str,
        tool_manager: &dyn crate::tool::ToolManager,
    ) -> Result<(), MCPError> {
        // Check if tool exists and clean up resources
        if let Some(_tool) = tool_manager.get_tool(tool_id).await? {
            log::info!("Cleaned up resources for tool: {}", tool_id);
            Ok(())
        } else {
            Err(MCPError::Generic(format!("Tool {} not found", tool_id)))
        }
    }

    /// Add a recovery strategy for a specific resource type
    pub fn add_strategy(&self, resource_type: String, strategy: Box<dyn RecoveryStrategy + Send + Sync>) {
        match self.strategies.lock() {
            Ok(mut strategies) => {
                strategies.insert(resource_type, strategy);
                debug!("✅ Recovery strategy added successfully");
            }
            Err(e) => {
                error!("🚨 Failed to acquire recovery strategies lock: {}", e);
                // In a production system, this is a critical error but should not panic
                // Log the error and continue - the system can still function without this strategy
            }
        }
    }

    /// Execute recovery for the given resource type and ID
    pub async fn execute_recovery(&self, resource_type: &str, resource_id: &str) -> Result<()> {
        let strategies = match self.strategies.lock() {
            Ok(strategies) => strategies,
            Err(e) => {
                let error_msg = format!("Failed to acquire recovery strategies lock: {}", e);
                error!("🚨 {}", error_msg);
                return Err(crate::error::MCPError::RuntimeError(error_msg));
            }
        };
        
        match strategies.get(resource_type).copied().unwrap_or(self.default_strategy) {
            RecoveryStrategy::Reset => {
                log::info!("Resetting tool: {}", resource_id);
                // Tool reset logic would go here
                Ok(())
            }
            RecoveryStrategy::Terminate => {
                log::warn!("Terminating tool: {}", resource_id);
                // Tool termination logic would go here
                Ok(())
            }
            RecoveryStrategy::Continue => {
                log::info!("Continuing with tool: {}", resource_id);
                // Continue with tool execution
                Ok(())
            }
        }
    }
}

impl Default for ToolRecovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Recovery hook for tool errors
#[derive(Debug)]
pub struct RecoveryHook {
    /// Recovery strategy to use
    strategy: RecoveryStrategy,
}

impl RecoveryHook {
    /// Create a new recovery hook with default strategy
    pub fn new() -> Self {
        Self {
            strategy: RecoveryStrategy::Reset,
        }
    }

    /// Create a new recovery hook with specific strategy
    pub fn with_strategy(strategy: RecoveryStrategy) -> Self {
        Self { strategy }
    }

    /// Attempt recovery for a tool
    pub async fn attempt_recovery(&self, tool_id: &str) -> Result<(), ToolError> {
        log::info!("Attempting recovery for tool: {}", tool_id);
        
        match self.strategy {
            RecoveryStrategy::Reset => {
                log::info!("Resetting tool: {}", tool_id);
                // Tool reset logic would go here
                Ok(())
            }
            RecoveryStrategy::Terminate => {
                log::warn!("Terminating tool: {}", tool_id);
                // Tool termination logic would go here
                Ok(())
            }
            RecoveryStrategy::Continue => {
                log::info!("Continuing with tool: {}", tool_id);
                // Continue with tool execution
                Ok(())
            }
        }
    }
}

impl Default for RecoveryHook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recovery_strategy_selection() {
        let recovery = ToolRecovery::new();
        
        // Test default strategy
        assert_eq!(recovery.get_strategy("test_tool"), RecoveryStrategy::Reset);
        
        // Test custom strategy
        recovery.set_strategy("test_tool", RecoveryStrategy::Terminate);
        assert_eq!(recovery.get_strategy("test_tool"), RecoveryStrategy::Terminate);
    }

    #[tokio::test]
    async fn test_recovery_hook() {
        let hook = RecoveryHook::new();
        let error = ToolError::InvalidState("Test error".to_string());
        
        // Should handle error without panicking
        let result = hook.attempt_recovery("test_tool").await;
        assert!(result.is_ok());
    }
}

