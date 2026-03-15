// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tool management and execution system
//!
//! Provides a unified interface for registering, discovering, and executing tools.
//! The `ToolManager` wraps a `ToolExecutor` with concurrent-safe access and
//! lifecycle management.

use std::sync::Arc;
use tokio::sync::RwLock;

pub mod executor;

pub use executor::*;

/// Thread-safe tool management system
///
/// Wraps the `ToolExecutor` with `Arc<RwLock<..>>` for safe concurrent access
/// from both the JSON-RPC and tarpc servers.
#[derive(Debug)]
pub struct ToolManager {
    /// The underlying executor with tool registry and dispatch
    executor: Arc<RwLock<ToolExecutor>>,
}

impl ToolManager {
    /// Create a new tool manager with built-in tools pre-registered
    pub fn new() -> Self {
        Self {
            executor: Arc::new(RwLock::new(ToolExecutor::new())),
        }
    }

    /// Register an external tool
    pub async fn register_tool(&self, registration: ToolRegistration) {
        let mut executor = self.executor.write().await;
        executor.register_tool(registration);
    }

    /// List all available tools
    pub async fn list_tools(&self) -> Vec<ToolRegistration> {
        let executor = self.executor.read().await;
        executor.list_tools().into_iter().cloned().collect()
    }

    /// Execute a tool by name
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        args: &str,
    ) -> Result<ToolExecutionResult, crate::error::PrimalError> {
        let executor = self.executor.read().await;
        executor.execute_tool(tool_name, args).await
    }

    /// Check if a tool is registered
    pub async fn has_tool(&self, tool_name: &str) -> bool {
        let executor = self.executor.read().await;
        executor.available_tools.contains_key(tool_name)
    }

    /// Get tool count
    pub async fn tool_count(&self) -> usize {
        let executor = self.executor.read().await;
        executor.available_tools.len()
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_manager_creation() {
        let manager = ToolManager::new();
        assert!(manager.tool_count().await >= 4); // Built-in tools
    }

    #[tokio::test]
    async fn test_tool_manager_register() {
        let manager = ToolManager::new();
        let initial = manager.tool_count().await;

        manager
            .register_tool(ToolRegistration {
                name: Arc::from("test.tool"),
                description: "A test tool".to_string(),
                domain: Arc::from("test"),
                builtin: false,
            })
            .await;

        assert_eq!(manager.tool_count().await, initial + 1);
        assert!(manager.has_tool("test.tool").await);
    }

    #[tokio::test]
    async fn test_tool_manager_execute() {
        let manager = ToolManager::new();
        let result = manager.execute_tool("system.health", "").await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_tool_manager_list() {
        let manager = ToolManager::new();
        let tools = manager.list_tools().await;
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name.as_ref() == "system.health"));
    }

    #[tokio::test]
    async fn test_tool_manager_default() {
        let manager = ToolManager::default();
        assert!(manager.tool_count().await >= 4);
    }

    #[tokio::test]
    async fn test_tool_manager_execute_unknown() {
        let manager = ToolManager::new();
        let result = manager.execute_tool("unknown.tool", "").await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_tool_manager_has_tool() {
        let manager = ToolManager::new();
        assert!(manager.has_tool("system.health").await);
        assert!(!manager.has_tool("nonexistent").await);
    }
}
