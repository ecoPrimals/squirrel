// Plugin Tools Module
//
// This module defines the interface for plugin tools.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fmt;

use crate::plugins::context::PluginContext;
use crate::plugins::errors::{PluginError, Result};

/// Input schema for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSchema {
    /// JSON Schema defining the input format
    pub schema: Value,
}

/// Output schema for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSchema {
    /// JSON Schema defining the output format
    pub schema: Value,
}

/// Description of a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescription {
    /// Name of the tool
    pub name: String,
    
    /// Description of what the tool does
    pub description: String,
    
    /// Help text for using the tool
    pub help: String,
    
    /// Input schema
    pub input_schema: InputSchema,
    
    /// Output schema
    pub output_schema: OutputSchema,
}

impl fmt::Display for ToolDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.name,
            self.description
        )
    }
}

/// A tool provided by a plugin
pub trait Tool: Send + Sync {
    /// Get the tool description
    fn description(&self) -> ToolDescription;
    
    /// Execute the tool
    fn execute(&self, input: &Value, context: &PluginContext) -> Result<Value>;
}

/// A plugin that provides tools
#[async_trait]
pub trait ToolPlugin: Send + Sync {
    /// List the tools provided by this plugin
    async fn list_tools(&self) -> Result<Vec<ToolDescription>>;
    
    /// Execute a tool by name
    async fn execute_tool(&self, name: &str, input: &Value, context: &PluginContext) -> Result<Value>;
}

// Add tests for the tool plugin
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    // Mock implementation of the ToolPlugin trait for testing
    struct MockToolPlugin {
        tools: Vec<ToolDescription>,
    }
    
    #[async_trait]
    impl ToolPlugin for MockToolPlugin {
        async fn list_tools(&self) -> Result<Vec<ToolDescription>> {
            Ok(self.tools.clone())
        }
        
        async fn execute_tool(&self, name: &str, input: &Value, context: &PluginContext) -> Result<Value> {
            // Simple text analysis tool
            if name == "text_analysis" {
                // Parse input
                let text = input.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::ExecutionError("Missing text parameter".to_string()))?;
                
                let include_stats = input.get("options")
                    .and_then(|v| v.get("include_stats"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // Perform text analysis
                let words = text.split_whitespace().count();
                let chars = text.chars().count();
                
                // Build result
                let mut result = json!({
                    "word_count": words,
                    "char_count": chars
                });
                
                // Add stats if requested
                if include_stats {
                    let lines = text.lines().count();
                    let paragraphs = text.split("\n\n").count();
                    
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("stats".to_string(), json!({
                            "lines": lines,
                            "paragraphs": paragraphs,
                            "avg_word_length": if words > 0 { chars as f64 / words as f64 } else { 0.0 }
                        }));
                    }
                }
                
                Ok(result)
            } else if name == "calculator" {
                // Simple calculator tool
                let a = input.get("a")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| PluginError::ExecutionError("Missing or invalid 'a' parameter".to_string()))?;
                
                let b = input.get("b")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| PluginError::ExecutionError("Missing or invalid 'b' parameter".to_string()))?;
                
                let operation = input.get("operation")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| PluginError::ExecutionError("Missing operation parameter".to_string()))?;
                
                // Perform calculation
                let result = match operation {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => {
                        if b == 0.0 {
                            return Err(PluginError::ExecutionError("Division by zero".to_string()));
                        }
                        a / b
                    }
                    _ => return Err(PluginError::ExecutionError(
                        format!("Invalid operation: {}", operation)
                    ))
                };
                
                // Build result
                Ok(json!({
                    "result": result,
                    "operation": operation
                }))
            } else {
                Err(PluginError::NotFoundError(format!("Tool not found: {}", name)))
            }
        }
    }
    
    // Helper function to create a mock tool plugin
    fn create_mock_tool_plugin() -> MockToolPlugin {
        let tools = vec![
            ToolDescription {
                name: "text_analysis".to_string(),
                description: "Analyzes text and returns word and character counts.".to_string(),
                help: "Provide text to analyze in the 'text' field.".to_string(),
                input_schema: InputSchema {
                    schema: json!({
                        "type": "object",
                        "required": ["text"],
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "The text to analyze"
                            },
                            "options": {
                                "type": "object",
                                "properties": {
                                    "include_stats": {
                                        "type": "boolean",
                                        "description": "Whether to include additional statistics"
                                    }
                                }
                            }
                        }
                    })
                },
                output_schema: OutputSchema {
                    schema: json!({
                        "type": "object",
                        "properties": {
                            "word_count": {
                                "type": "integer",
                                "description": "The number of words in the text"
                            },
                            "char_count": {
                                "type": "integer",
                                "description": "The number of characters in the text"
                            },
                            "stats": {
                                "type": "object",
                                "description": "Additional statistics about the text"
                            }
                        }
                    })
                },
            },
            ToolDescription {
                name: "calculator".to_string(),
                description: "Performs basic arithmetic operations.".to_string(),
                help: "Provide two numbers and an operation (add, subtract, multiply, divide).".to_string(),
                input_schema: InputSchema {
                    schema: json!({
                        "type": "object",
                        "required": ["a", "b", "operation"],
                        "properties": {
                            "a": {
                                "type": "number",
                                "description": "First number"
                            },
                            "b": {
                                "type": "number",
                                "description": "Second number"
                            },
                            "operation": {
                                "type": "string",
                                "enum": ["add", "subtract", "multiply", "divide"],
                                "description": "Operation to perform"
                            }
                        }
                    })
                },
                output_schema: OutputSchema {
                    schema: json!({
                        "type": "object",
                        "properties": {
                            "result": {
                                "type": "number",
                                "description": "The result of the operation"
                            },
                            "operation": {
                                "type": "string",
                                "description": "The operation that was performed"
                            }
                        }
                    })
                },
            }
        ];
        
        MockToolPlugin { tools }
    }
    
    #[tokio::test]
    async fn test_list_tools() -> Result<()> {
        let plugin = create_mock_tool_plugin();
        
        let tools = plugin.list_tools().await?;
        
        // Verify that there are two tools
        assert_eq!(tools.len(), 2);
        
        // Verify the first tool
        assert_eq!(tools[0].name, "text_analysis");
        assert!(!tools[0].description.is_empty());
        
        // Verify the second tool
        assert_eq!(tools[1].name, "calculator");
        assert!(!tools[1].description.is_empty());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_execute_text_analysis_tool() -> Result<()> {
        let plugin = create_mock_tool_plugin();
        let context = PluginContext::new();
        
        // Test basic text analysis
        let input = json!({
            "text": "Hello, world! This is a test."
        });
        
        let result = plugin.execute_tool("text_analysis", &input, &context).await?;
        
        // Verify the result
        assert!(result.is_object());
        assert_eq!(result.get("word_count").unwrap().as_u64().unwrap(), 5);
        assert_eq!(result.get("char_count").unwrap().as_u64().unwrap(), 27);
        assert!(result.get("stats").is_none());
        
        // Test text analysis with stats
        let input_with_stats = json!({
            "text": "Hello, world!\n\nThis is a test.",
            "options": {
                "include_stats": true
            }
        });
        
        let result = plugin.execute_tool("text_analysis", &input_with_stats, &context).await?;
        
        // Verify the result with stats
        assert!(result.is_object());
        assert_eq!(result.get("word_count").unwrap().as_u64().unwrap(), 5);
        assert_eq!(result.get("char_count").unwrap().as_u64().unwrap(), 29);
        
        let stats = result.get("stats").unwrap();
        assert!(stats.is_object());
        assert_eq!(stats.get("lines").unwrap().as_u64().unwrap(), 3);
        assert_eq!(stats.get("paragraphs").unwrap().as_u64().unwrap(), 2);
        assert!(stats.get("avg_word_length").unwrap().as_f64().unwrap() > 0.0);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_execute_calculator_tool() -> Result<()> {
        let plugin = create_mock_tool_plugin();
        let context = PluginContext::new();
        
        // Test addition
        let add_input = json!({
            "a": 5,
            "b": 3,
            "operation": "add"
        });
        
        let add_result = plugin.execute_tool("calculator", &add_input, &context).await?;
        
        // Verify the addition result
        assert!(add_result.is_object());
        assert_eq!(add_result.get("result").unwrap().as_f64().unwrap(), 8.0);
        assert_eq!(add_result.get("operation").unwrap().as_str().unwrap(), "add");
        
        // Test multiplication
        let mult_input = json!({
            "a": 5,
            "b": 3,
            "operation": "multiply"
        });
        
        let mult_result = plugin.execute_tool("calculator", &mult_input, &context).await?;
        
        // Verify the multiplication result
        assert!(mult_result.is_object());
        assert_eq!(mult_result.get("result").unwrap().as_f64().unwrap(), 15.0);
        assert_eq!(mult_result.get("operation").unwrap().as_str().unwrap(), "multiply");
        
        // Test division by zero
        let div_zero_input = json!({
            "a": 5,
            "b": 0,
            "operation": "divide"
        });
        
        let div_zero_result = plugin.execute_tool("calculator", &div_zero_input, &context).await;
        
        // Verify that division by zero returns an error
        assert!(div_zero_result.is_err());
        if let Err(PluginError::ExecutionError(msg)) = div_zero_result {
            assert_eq!(msg, "Division by zero");
        } else {
            panic!("Expected ExecutionError");
        }
        
        Ok(())
    }
} 