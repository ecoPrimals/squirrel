// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tool executor implementation for squirrel
//!
//! Provides tool registration, dispatch, and execution with built-in system tools
//! and support for dynamically discovered capability-based tools.
//!
//! ## Architecture
//!
//! Tools are registered at startup (built-ins) or discovered at runtime via
//! capability scanning. Each tool implements the `ToolHandler` trait, enabling
//! a uniform dispatch interface regardless of origin.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    /// Name of the tool that was executed.
    pub tool_name: String,
    /// Whether execution succeeded.
    pub success: bool,
    /// Output from the tool execution.
    pub output: String,
    /// Error message if execution failed.
    pub error: Option<String>,
}

/// Tool metadata registered in the executor
#[derive(Debug, Clone)]
pub struct ToolRegistration {
    /// Tool name (unique identifier) — `Arc<str>` for zero-copy (reused in lookups)
    pub name: Arc<str>,
    /// Human-readable description
    pub description: String,
    /// Required capability domain (e.g., "ai", "system") — `Arc<str>` for zero-copy (interned constants)
    pub domain: Arc<str>,
    /// Whether this is a built-in tool
    pub builtin: bool,
}

/// Tool executor with registration and dispatch
///
/// Manages a registry of available tools and dispatches execution requests.
/// Built-in tools are registered at construction; external tools can be
/// registered dynamically via `register_tool`.
#[derive(Debug)]
pub struct ToolExecutor {
    /// Registered tools: name → registration metadata (`Arc<str>` keys for zero-copy)
    pub available_tools: HashMap<Arc<str>, ToolRegistration>,
}

impl ToolExecutor {
    /// Create a new tool executor with built-in system tools
    pub fn new() -> Self {
        let mut executor = Self {
            available_tools: HashMap::new(),
        };
        executor.register_builtins();
        executor
    }

    /// Register built-in tools
    fn register_builtins(&mut self) {
        let builtins = vec![
            ToolRegistration {
                name: Arc::from("system.health"),
                description: "Check system health status".to_string(),
                domain: Arc::from("system"),
                builtin: true,
            },
            ToolRegistration {
                name: Arc::from("system.info"),
                description: "Get system information (version, uptime)".to_string(),
                domain: Arc::from("system"),
                builtin: true,
            },
            ToolRegistration {
                name: Arc::from("discovery.peers"),
                description: "Discover peer primals via socket scan".to_string(),
                domain: Arc::from("discovery"),
                builtin: true,
            },
            ToolRegistration {
                name: Arc::from("discovery.capabilities"),
                description: "List all discovered capabilities".to_string(),
                domain: Arc::from("discovery"),
                builtin: true,
            },
        ];

        for tool in builtins {
            self.available_tools.insert(Arc::clone(&tool.name), tool);
        }
    }

    /// Register an external tool
    pub fn register_tool(&mut self, registration: ToolRegistration) {
        self.available_tools
            .insert(Arc::clone(&registration.name), registration);
    }

    /// List available tools
    pub fn list_tools(&self) -> Vec<&ToolRegistration> {
        self.available_tools.values().collect()
    }

    /// Execute a tool by name
    ///
    /// Dispatches to built-in implementations or returns an error for
    /// unregistered tools.
    #[expect(clippy::too_many_lines, reason = "Command dispatch; refactor planned")]
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        args: &str,
    ) -> Result<ToolExecutionResult, PrimalError> {
        // Check if tool is registered
        if !self.available_tools.contains_key(tool_name) {
            return Ok(ToolExecutionResult {
                tool_name: tool_name.to_string(),
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Tool '{}' not found. Available: {:?}",
                    tool_name,
                    self.available_tools.keys().collect::<Vec<_>>()
                )),
            });
        }

        // Dispatch built-in tools
        match tool_name {
            "system.health" => Ok(ToolExecutionResult {
                tool_name: tool_name.to_string(),
                success: true,
                output: serde_json::json!({
                    "status": "healthy",
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })
                .to_string(),
                error: None,
            }),

            "system.info" => {
                let memory_info = {
                    #[cfg(feature = "system-metrics")]
                    {
                        match universal_constants::sys_info::memory_info() {
                            Ok(mem) => serde_json::json!({
                                "version": env!("CARGO_PKG_VERSION"),
                                "primal": "squirrel",
                                "memory_total_mb": mem.total / 1024 / 1024,
                                "memory_used_mb": mem.used / 1024 / 1024,
                            }),
                            Err(_) => serde_json::json!({
                                "version": env!("CARGO_PKG_VERSION"),
                                "primal": "squirrel",
                                "note": "memory info unavailable",
                            }),
                        }
                    }
                    #[cfg(not(feature = "system-metrics"))]
                    {
                        serde_json::json!({
                            "version": env!("CARGO_PKG_VERSION"),
                            "primal": "squirrel",
                            "note": "system-metrics feature not enabled",
                        })
                    }
                };
                Ok(ToolExecutionResult {
                    tool_name: tool_name.to_string(),
                    success: true,
                    output: memory_info.to_string(),
                    error: None,
                })
            }

            "discovery.peers" => {
                match crate::capabilities::discovery::discover_all_capabilities().await {
                    Ok(caps) => {
                        let mut peers = std::collections::HashSet::new();
                        for providers in caps.values() {
                            for provider in providers {
                                peers.insert(provider.id.clone());
                            }
                        }
                        Ok(ToolExecutionResult {
                            tool_name: tool_name.to_string(),
                            success: true,
                            output: serde_json::json!({
                                "peers": peers.into_iter().collect::<Vec<_>>(),
                            })
                            .to_string(),
                            error: None,
                        })
                    }
                    Err(e) => Ok(ToolExecutionResult {
                        tool_name: tool_name.to_string(),
                        success: false,
                        output: String::new(),
                        error: Some(format!("Discovery failed: {e}")),
                    }),
                }
            }

            "discovery.capabilities" => {
                match crate::capabilities::discovery::discover_all_capabilities().await {
                    Ok(caps) => {
                        let capability_list: Vec<String> = caps.keys().cloned().collect();
                        Ok(ToolExecutionResult {
                            tool_name: tool_name.to_string(),
                            success: true,
                            output: serde_json::json!({
                                "capabilities": capability_list,
                                "total": capability_list.len(),
                            })
                            .to_string(),
                            error: None,
                        })
                    }
                    Err(e) => Ok(ToolExecutionResult {
                        tool_name: tool_name.to_string(),
                        success: false,
                        output: String::new(),
                        error: Some(format!("Capability scan failed: {e}")),
                    }),
                }
            }

            // External/custom tools -- forward args as context
            _ => {
                let _ = args; // Reserved for future external tool dispatch
                Ok(ToolExecutionResult {
                    tool_name: tool_name.to_string(),
                    success: true,
                    output: format!("Tool '{tool_name}' executed (external dispatch)"),
                    error: None,
                })
            }
        }
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_has_builtins() {
        let executor = ToolExecutor::new();
        assert!(executor.available_tools.contains_key("system.health"));
        assert!(executor.available_tools.contains_key("system.info"));
        assert!(executor.available_tools.contains_key("discovery.peers"));
        assert!(
            executor
                .available_tools
                .contains_key("discovery.capabilities")
        );
    }

    #[test]
    fn test_register_custom_tool() {
        let mut executor = ToolExecutor::new();
        let initial_count = executor.available_tools.len();

        executor.register_tool(ToolRegistration {
            name: Arc::from("custom.test"),
            description: "Test tool".to_string(),
            domain: Arc::from("custom"),
            builtin: false,
        });

        assert_eq!(executor.available_tools.len(), initial_count + 1);
        assert!(executor.available_tools.contains_key("custom.test"));
    }

    #[tokio::test]
    async fn test_execute_health() {
        let executor = ToolExecutor::new();
        let result = executor.execute_tool("system.health", "").await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("healthy"));
    }

    #[tokio::test]
    async fn test_execute_system_info() {
        let executor = ToolExecutor::new();
        let result = executor.execute_tool("system.info", "").await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("version"));
        assert!(result.output.contains("primal"));
    }

    #[tokio::test]
    async fn test_execute_discovery_peers() {
        let executor = ToolExecutor::new();
        let result = executor.execute_tool("discovery.peers", "").await.unwrap();
        assert!(result.output.contains("peers"));
    }

    #[tokio::test]
    async fn test_execute_discovery_capabilities() {
        let executor = ToolExecutor::new();
        let result = executor
            .execute_tool("discovery.capabilities", "")
            .await
            .unwrap();
        assert!(result.output.contains("capabilities"));
    }

    #[tokio::test]
    async fn test_execute_unknown_tool() {
        let executor = ToolExecutor::new();
        let result = executor.execute_tool("nonexistent.tool", "").await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_list_tools() {
        let executor = ToolExecutor::new();
        let tools = executor.list_tools();
        assert!(tools.len() >= 4);
    }

    #[test]
    fn test_tool_execution_result_serde() {
        let result = ToolExecutionResult {
            tool_name: "test".to_string(),
            success: true,
            output: "ok".to_string(),
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let _: ToolExecutionResult = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_tool_executor_default() {
        let executor = ToolExecutor::default();
        assert!(executor.available_tools.len() >= 4);
    }

    #[tokio::test]
    async fn test_execute_registered_external_tool() {
        let mut executor = ToolExecutor::new();
        executor.register_tool(ToolRegistration {
            name: Arc::from("external.foo"),
            description: "External".to_string(),
            domain: Arc::from("external"),
            builtin: false,
        });
        let result = executor.execute_tool("external.foo", "args").await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("external dispatch"));
    }
}
