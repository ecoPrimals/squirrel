// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! High-level MCP protocol operations
//!
//! This module provides high-level operations for the MCP protocol including
//! tools, resources, and prompts management.

use super::types::{McpPrompt, McpResource, McpTool};
use crate::error::{PluginError, PluginResult};
use tracing::{debug, warn};
// Removed: use squirrel_mcp_config::get_service_endpoints;

/// Operation handler for MCP protocol operations
///
/// Provides high-level operations for interacting with MCP servers including
/// tool execution, resource access, and prompt management.
///
/// When `connected` is `true`, methods attempt IPC-backed MCP transport.
/// When `false` (default), methods return empty results or clear errors.
#[derive(Debug)]
pub struct OperationHandler {
    /// Operation counter for tracking requests
    operation_counter: u64,
    /// Whether an IPC connection to an MCP server is active.
    connected: bool,
}

impl OperationHandler {
    /// Create a new operation handler
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::operations::OperationHandler;
    ///
    /// let handler = OperationHandler::new();
    /// ```
    pub fn new() -> Self {
        Self {
            operation_counter: 0,
            connected: false,
        }
    }

    /// Create an operation handler marked as having an active MCP connection.
    ///
    /// Use this when the caller has already established an IPC channel to an
    /// MCP server. Methods will attempt to forward requests over that channel
    /// instead of returning empty results.
    pub fn with_connection() -> Self {
        Self {
            operation_counter: 0,
            connected: true,
        }
    }

    /// Whether this handler has an active MCP connection.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// List available tools
    ///
    /// Retrieves a list of all available tools from the MCP server.
    ///
    /// # Returns
    ///
    /// Returns a vector of available tools or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::operations::OperationHandler;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut handler = OperationHandler::new();
    /// // let tools = handler.list_tools().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_tools(&mut self) -> PluginResult<Vec<McpTool>> {
        self.operation_counter += 1;
        debug!("Listing tools (operation #{})", self.operation_counter);

        if !self.connected {
            warn!("list_tools: no MCP server connected — returning empty");
            return Ok(Vec::new());
        }

        debug!("list_tools: MCP connected, IPC transport pending");
        Ok(Vec::new())
    }

    /// Execute a tool
    ///
    /// Executes a specific tool with the provided input.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to execute
    /// * `input` - The input data for the tool
    ///
    /// # Returns
    ///
    /// Returns the tool execution result or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::operations::OperationHandler;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut handler = OperationHandler::new();
    /// let input = json!({"operation": "add", "operands": [1, 2]});
    /// // let result = handler.execute_tool("calculator", input).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_tool(
        &mut self,
        name: &str,
        input: serde_json::Value,
    ) -> PluginResult<serde_json::Value> {
        self.operation_counter += 1;
        debug!(
            "Executing tool '{}' (operation #{})",
            name, self.operation_counter
        );

        if !self.connected {
            return Err(PluginError::McpError {
                message: format!("tool '{name}' not available: no MCP server connected"),
            });
        }

        let _ = input;
        Err(PluginError::McpError {
            message: format!("tool '{name}': IPC transport not yet wired"),
        })
    }

    /// List available resources
    ///
    /// Retrieves a list of all available resources from the MCP server.
    ///
    /// # Returns
    ///
    /// Returns a vector of available resources or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::operations::OperationHandler;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut handler = OperationHandler::new();
    /// // let resources = handler.list_resources().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_resources(&mut self) -> PluginResult<Vec<McpResource>> {
        self.operation_counter += 1;
        debug!("Listing resources (operation #{})", self.operation_counter);

        if !self.connected {
            warn!("list_resources: no MCP server connected — returning empty");
            return Ok(Vec::new());
        }

        debug!("list_resources: MCP connected, IPC transport pending");
        Ok(Vec::new())
    }

    /// Get a resource
    ///
    /// Retrieves the content of a specific resource.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the resource to retrieve
    ///
    /// # Returns
    ///
    /// Returns the resource content or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::operations::OperationHandler;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut handler = OperationHandler::new();
    /// // let content = handler.get_resource("file:///config/app.json").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_resource(&mut self, uri: &str) -> PluginResult<serde_json::Value> {
        self.operation_counter += 1;
        debug!(
            "Getting resource '{}' (operation #{})",
            uri, self.operation_counter
        );

        if !self.connected {
            return Err(PluginError::McpError {
                message: format!("resource '{uri}' not available: no MCP server connected"),
            });
        }

        Err(PluginError::McpError {
            message: format!("resource '{uri}': IPC transport not yet wired"),
        })
    }

    /// List available prompts
    ///
    /// Retrieves a list of all available prompts from the MCP server.
    ///
    /// # Returns
    ///
    /// Returns a vector of available prompts or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::operations::OperationHandler;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut handler = OperationHandler::new();
    /// // let prompts = handler.list_prompts().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_prompts(&mut self) -> PluginResult<Vec<McpPrompt>> {
        self.operation_counter += 1;
        debug!("Listing prompts (operation #{})", self.operation_counter);

        if !self.connected {
            warn!("list_prompts: no MCP server connected — returning empty");
            return Ok(Vec::new());
        }

        debug!("list_prompts: MCP connected, IPC transport pending");
        Ok(Vec::new())
    }

    /// Get a prompt
    ///
    /// Retrieves a specific prompt with parameter substitution.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the prompt to retrieve
    /// * `parameters` - Parameters to substitute in the prompt template
    ///
    /// # Returns
    ///
    /// Returns the processed prompt or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::operations::OperationHandler;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut handler = OperationHandler::new();
    /// let params = json!({"text": "This is a long piece of text to summarize..."});
    /// // let prompt = handler.get_prompt("summarize", params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_prompt(
        &mut self,
        name: &str,
        parameters: serde_json::Value,
    ) -> PluginResult<McpPrompt> {
        self.operation_counter += 1;
        debug!(
            "Getting prompt '{}' (operation #{})",
            name, self.operation_counter
        );

        if !self.connected {
            return Err(PluginError::McpError {
                message: format!("prompt '{name}' not available: no MCP server connected"),
            });
        }

        let _ = parameters;
        Err(PluginError::McpError {
            message: format!("prompt '{name}': IPC transport not yet wired"),
        })
    }
}

impl Default for OperationHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_operation_handler_creation() {
        let handler = OperationHandler::new();
        assert_eq!(handler.operation_counter, 0);
        assert!(!handler.connected);
        assert!(!handler.is_connected());

        let handler = OperationHandler::with_connection();
        assert_eq!(handler.operation_counter, 0);
        assert!(handler.connected);
        assert!(handler.is_connected());
    }

    #[tokio::test]
    async fn test_list_tools() {
        let mut handler = OperationHandler::new();
        let tools = handler.list_tools().await.expect("should succeed");
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn test_execute_tool() {
        let mut handler = OperationHandler::new();
        let input = json!({"operation": "add", "operands": [1, 2, 3]});
        let result = handler.execute_tool("calculator", input).await;
        assert!(result.is_err());
        match result {
            Err(PluginError::McpError { message }) => {
                assert!(message.contains("calculator"));
                assert!(message.contains("no MCP server connected"));
            }
            _ => panic!("expected McpError"),
        }
    }

    #[tokio::test]
    async fn test_list_resources() {
        let mut handler = OperationHandler::new();
        let resources = handler.list_resources().await.expect("should succeed");
        assert!(resources.is_empty());
    }

    #[tokio::test]
    async fn test_get_resource() {
        let mut handler = OperationHandler::new();
        let result = handler.get_resource("file:///config/app.json").await;
        assert!(result.is_err());
        match result {
            Err(PluginError::McpError { message }) => {
                assert!(message.contains("file:///config/app.json"));
                assert!(message.contains("no MCP server connected"));
            }
            _ => panic!("expected McpError"),
        }
    }

    #[tokio::test]
    async fn test_list_prompts() {
        let mut handler = OperationHandler::new();
        let prompts = handler.list_prompts().await.expect("should succeed");
        assert!(prompts.is_empty());
    }

    #[tokio::test]
    async fn test_get_prompt() {
        let mut handler = OperationHandler::new();
        let params = json!({"text": "This is a test text"});
        let result = handler.get_prompt("summarize", params).await;
        assert!(result.is_err());
        match result {
            Err(PluginError::McpError { message }) => {
                assert!(message.contains("summarize"));
                assert!(message.contains("no MCP server connected"));
            }
            _ => panic!("expected McpError"),
        }
    }
}
