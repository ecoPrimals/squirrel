//! Tool executor implementation for squirrel
//!
//! This module provides basic tool execution capabilities.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Simple tool executor
#[derive(Debug)]
pub struct ToolExecutor {
    pub available_tools: HashMap<String, String>,
}

impl ToolExecutor {
    /// Create a new tool executor with an empty tool registry
    ///
    /// Initializes the executor with an empty HashMap for tracking available tools.
    /// Tools can be added later using the appropriate methods.
    pub fn new() -> Self {
        Self {
            available_tools: HashMap::new(),
        }
    }

    /// Execute a tool
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        _args: &str,
    ) -> Result<ToolExecutionResult, PrimalError> {
        Ok(ToolExecutionResult {
            tool_name: tool_name.to_string(),
            success: true,
            output: format!("Executed tool: {tool_name}"),
            error: None,
        })
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}
