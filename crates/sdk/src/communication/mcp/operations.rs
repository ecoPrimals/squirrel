// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! High-level MCP protocol operations
//!
//! This module provides high-level operations for the MCP protocol including
//! tools, resources, and prompts management.

use super::types::{McpPrompt, McpResource, McpTool};
use crate::error::{PluginError, PluginResult};
use tracing::debug;
// Removed: use squirrel_mcp_config::get_service_endpoints;

/// Operation handler for MCP protocol operations
///
/// Provides high-level operations for interacting with MCP servers including
/// tool execution, resource access, and prompt management.
#[derive(Debug)]
pub struct OperationHandler {
    /// Operation counter for tracking requests
    operation_counter: u64,
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
        }
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

        // This is a placeholder implementation
        // In a real implementation, this would send a request to the MCP server
        // and parse the response
        Ok(vec![
            McpTool {
                name: "calculator".to_string(),
                description: "Performs basic arithmetic operations".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string"},
                        "operands": {"type": "array", "items": {"type": "number"}}
                    },
                    "required": ["operation", "operands"]
                }),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "number"}
                    }
                })),
            },
            McpTool {
                name: "text_processor".to_string(),
                description: "Processes text content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string"},
                        "operation": {"type": "string", "enum": ["uppercase", "lowercase", "reverse"]}
                    },
                    "required": ["text", "operation"]
                }),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "processed_text": {"type": "string"}
                    }
                })),
            },
        ])
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

        // This is a placeholder implementation
        // In a real implementation, this would send a request to the MCP server
        // and return the actual tool execution result
        match name {
            "calculator" => {
                if let (Some(operation), Some(operands)) = (
                    input.get("operation").and_then(|v| v.as_str()),
                    input.get("operands").and_then(|v| v.as_array()),
                ) {
                    let numbers: Vec<f64> = operands.iter().filter_map(|v| v.as_f64()).collect();

                    let result = match operation {
                        "add" => numbers.iter().sum(),
                        "multiply" => numbers.iter().product(),
                        "subtract" => {
                            numbers.first().unwrap_or(&0.0) - numbers.get(1).unwrap_or(&0.0)
                        }
                        "divide" => {
                            let divisor = numbers.get(1).unwrap_or(&1.0);
                            if *divisor != 0.0 {
                                numbers.first().unwrap_or(&0.0) / divisor
                            } else {
                                return Err(PluginError::McpError {
                                    message: "Division by zero".to_string(),
                                });
                            }
                        }
                        _ => {
                            return Err(PluginError::McpError {
                                message: format!("Unknown operation: {}", operation),
                            });
                        }
                    };

                    Ok(serde_json::json!({"result": result}))
                } else {
                    Err(PluginError::McpError {
                        message: "Invalid input for calculator".to_string(),
                    })
                }
            }
            "text_processor" => {
                if let (Some(text), Some(operation)) = (
                    input.get("text").and_then(|v| v.as_str()),
                    input.get("operation").and_then(|v| v.as_str()),
                ) {
                    let processed_text = match operation {
                        "uppercase" => text.to_uppercase(),
                        "lowercase" => text.to_lowercase(),
                        "reverse" => text.chars().rev().collect(),
                        _ => {
                            return Err(PluginError::McpError {
                                message: format!("Unknown text operation: {}", operation),
                            });
                        }
                    };

                    Ok(serde_json::json!({"processed_text": processed_text}))
                } else {
                    Err(PluginError::McpError {
                        message: "Invalid input for text_processor".to_string(),
                    })
                }
            }
            _ => Err(PluginError::McpError {
                message: format!("Unknown tool: {}", name),
            }),
        }
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

        // This is a placeholder implementation
        Ok(vec![
            McpResource {
                uri: "file:///config/app.json".to_string(),
                name: "Application Configuration".to_string(),
                description: "Main application configuration file".to_string(),
                metadata: serde_json::json!({
                    "size": 2048,
                    "format": "json",
                    "last_modified": "2024-01-01T00:00:00Z",
                    "permissions": "read-only"
                }),
            },
            McpResource {
                uri: "file:///logs/app.log".to_string(),
                name: "Application Logs".to_string(),
                description: "Current application log file".to_string(),
                metadata: serde_json::json!({
                    "size": 10240,
                    "format": "text",
                    "last_modified": "2024-01-01T12:00:00Z",
                    "permissions": "read-only"
                }),
            },
        ])
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

        // This is a placeholder implementation
        match uri {
            "file:///config/app.json" => Ok(serde_json::json!({
                "content": {
                    "app_name": "Squirrel SDK",
                    "version": "1.0.0",
                    "debug": true,
                    "database": {
                        "host": std::env::var("STORAGE_HOST")
                            .unwrap_or_else(|_| "localhost".to_string()),
                        "port": 5432,
                        "name": "squirrel_db"
                    }
                },
                "content_type": "application/json"
            })),
            "file:///logs/app.log" => Ok(serde_json::json!({
                "content": "2024-01-01T12:00:00Z INFO Application started\n2024-01-01T12:00:01Z INFO Database connected\n2024-01-01T12:00:02Z INFO Ready to accept connections",
                "content_type": "text/plain"
            })),
            _ => Err(PluginError::McpError {
                message: format!("Resource not found: {}", uri),
            }),
        }
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

        // This is a placeholder implementation
        Ok(vec![
            McpPrompt {
                name: "summarize".to_string(),
                description: "Summarizes text content".to_string(),
                template: "Please provide a concise summary of the following text:\n\n{text}\n\nSummary:".to_string(),
                parameters: serde_json::json!({
                    "text": {
                        "type": "string",
                        "description": "The text to summarize",
                        "required": true
                    }
                }),
            },
            McpPrompt {
                name: "code_review".to_string(),
                description: "Reviews code for quality and best practices".to_string(),
                template: "Please review the following {language} code for quality, best practices, and potential issues:\n\n```{language}\n{code}\n```\n\nReview:".to_string(),
                parameters: serde_json::json!({
                    "code": {
                        "type": "string",
                        "description": "The code to review",
                        "required": true
                    },
                    "language": {
                        "type": "string",
                        "description": "The programming language",
                        "required": true
                    }
                }),
            },
        ])
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

        // This is a placeholder implementation
        match name {
            "summarize" => {
                let text = parameters
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No text provided");

                let processed_template = format!(
                    "Please provide a concise summary of the following text:\n\n{}\n\nSummary:",
                    text
                );

                Ok(McpPrompt {
                    name: "summarize".to_string(),
                    description: "Summarizes text content".to_string(),
                    template: processed_template,
                    parameters: serde_json::json!({
                        "text": {
                            "type": "string",
                            "description": "The text to summarize",
                            "required": true
                        }
                    }),
                })
            }
            "code_review" => {
                let code = parameters
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No code provided");
                let language = parameters
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let processed_template = format!(
                    "Please review the following {} code for quality, best practices, and potential issues:\n\n```{}\n{}\n```\n\nReview:",
                    language, language, code
                );

                Ok(McpPrompt {
                    name: "code_review".to_string(),
                    description: "Reviews code for quality and best practices".to_string(),
                    template: processed_template,
                    parameters: serde_json::json!({
                        "code": {
                            "type": "string",
                            "description": "The code to review",
                            "required": true
                        },
                        "language": {
                            "type": "string",
                            "description": "The programming language",
                            "required": true
                        }
                    }),
                })
            }
            _ => Err(PluginError::McpError {
                message: format!("Unknown prompt: {}", name),
            }),
        }
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
    }

    #[tokio::test]
    async fn test_list_tools() {
        let mut handler = OperationHandler::new();
        let tools = handler.list_tools().await.expect("should succeed");

        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "calculator");
        assert_eq!(tools[1].name, "text_processor");
    }

    #[tokio::test]
    async fn test_execute_tool_calculator() {
        let mut handler = OperationHandler::new();

        // Test addition
        let input = json!({"operation": "add", "operands": [1, 2, 3]});
        let result = handler
            .execute_tool("calculator", input)
            .await
            .expect("should succeed");
        assert_eq!(result["result"], 6.0);

        // Test multiplication
        let input = json!({"operation": "multiply", "operands": [2, 3, 4]});
        let result = handler
            .execute_tool("calculator", input)
            .await
            .expect("should succeed");
        assert_eq!(result["result"], 24.0);

        // Test division by zero
        let input = json!({"operation": "divide", "operands": [10, 0]});
        let result = handler.execute_tool("calculator", input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_tool_text_processor() {
        let mut handler = OperationHandler::new();

        // Test uppercase
        let input = json!({"text": "hello world", "operation": "uppercase"});
        let result = handler
            .execute_tool("text_processor", input)
            .await
            .expect("should succeed");
        assert_eq!(result["processed_text"], "HELLO WORLD");

        // Test lowercase
        let input = json!({"text": "HELLO WORLD", "operation": "lowercase"});
        let result = handler
            .execute_tool("text_processor", input)
            .await
            .expect("should succeed");
        assert_eq!(result["processed_text"], "hello world");

        // Test reverse
        let input = json!({"text": "hello", "operation": "reverse"});
        let result = handler
            .execute_tool("text_processor", input)
            .await
            .expect("should succeed");
        assert_eq!(result["processed_text"], "olleh");
    }

    #[tokio::test]
    async fn test_execute_unknown_tool() {
        let mut handler = OperationHandler::new();
        let input = json!({});
        let result = handler.execute_tool("unknown_tool", input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_resources() {
        let mut handler = OperationHandler::new();
        let resources = handler.list_resources().await.expect("should succeed");

        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].name, "Application Configuration");
        assert_eq!(resources[1].name, "Application Logs");
    }

    #[tokio::test]
    async fn test_get_resource() {
        let mut handler = OperationHandler::new();

        // Test getting config resource
        let content = handler
            .get_resource("file:///config/app.json")
            .await
            .expect("should succeed");
        assert!(content.get("content").is_some());
        assert_eq!(content["content_type"], "application/json");

        // Test getting log resource
        let content = handler
            .get_resource("file:///logs/app.log")
            .await
            .expect("should succeed");
        assert!(content.get("content").is_some());
        assert_eq!(content["content_type"], "text/plain");

        // Test unknown resource
        let result = handler.get_resource("file:///unknown").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_prompts() {
        let mut handler = OperationHandler::new();
        let prompts = handler.list_prompts().await.expect("should succeed");

        assert_eq!(prompts.len(), 2);
        assert_eq!(prompts[0].name, "summarize");
        assert_eq!(prompts[1].name, "code_review");
    }

    #[tokio::test]
    async fn test_get_prompt() {
        let mut handler = OperationHandler::new();

        // Test summarize prompt
        let params = json!({"text": "This is a test text"});
        let prompt = handler
            .get_prompt("summarize", params)
            .await
            .expect("should succeed");
        assert_eq!(prompt.name, "summarize");
        assert!(prompt.template.contains("This is a test text"));

        // Test code review prompt
        let params = json!({"code": "fn main() {}", "language": "rust"});
        let prompt = handler
            .get_prompt("code_review", params)
            .await
            .expect("should succeed");
        assert_eq!(prompt.name, "code_review");
        assert!(prompt.template.contains("fn main() {}"));
        assert!(prompt.template.contains("rust"));

        // Test unknown prompt
        let result = handler.get_prompt("unknown_prompt", json!({})).await;
        assert!(result.is_err());
    }
}
