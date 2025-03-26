// Tool Plugin Example
//
// This module provides an example tool plugin implementation.

use std::sync::Arc;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::plugins::builders::ToolPluginBuilder;
use crate::plugins::interfaces::ToolAvailability;
use squirrel_mcp::plugins::interfaces::PluginMetadata;

/// Create an example tool plugin
pub fn create_tool_example_plugin() -> Box<dyn crate::plugins::ToolPlugin> {
    // Create example metadata
    let metadata = PluginMetadata {
        id: Uuid::new_v4(),
        name: "example-tool-plugin".to_string(),
        description: "An example tool plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "DataScienceBioLab".to_string(),
        capabilities: vec!["tool".to_string()],
        permissions: vec!["basic".to_string()],
        tags: vec!["example".to_string()],
        dependencies: Vec::new(),
        signature: None,
    };
    
    // Create a tool plugin
    ToolPluginBuilder::new(metadata)
        .with_tool_full(
            "analyze",
            "Text analysis tool",
            Some("utility"),
            vec!["text", "example"],
            false,
        )
        .with_tool_metadata(
            "analyze",
            "Text analysis tool",
            "1.0.0",
            Some("DataScienceBioLab"),
            None,
            Some("MIT"),
            Vec::new(),
            Some(json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to analyze"
                    },
                    "options": {
                        "type": "object",
                        "properties": {
                            "include_stats": {
                                "type": "boolean",
                                "description": "Include statistical analysis"
                            }
                        }
                    }
                },
                "required": ["text"]
            })),
            Some(json!({
                "type": "object",
                "properties": {
                    "word_count": {
                        "type": "integer",
                        "description": "Number of words"
                    },
                    "char_count": {
                        "type": "integer",
                        "description": "Number of characters"
                    },
                    "stats": {
                        "type": "object",
                        "description": "Statistical analysis results (if requested)"
                    }
                }
            })),
        )
        .with_tool_handler("analyze", |args| {
            let text = match args.get("text") {
                Some(val) => val.as_str().unwrap_or(""),
                None => "",
            };
            
            let include_stats = match args.get("options") {
                Some(opts) => match opts.get("include_stats") {
                    Some(val) => val.as_bool().unwrap_or(false),
                    None => false,
                },
                None => false,
            };
            
            // Simple analysis
            let word_count = text.split_whitespace().count();
            let char_count = text.chars().count();
            
            let mut result = json!({
                "word_count": word_count,
                "char_count": char_count
            });
            
            // Add stats if requested
            if include_stats {
                let avg_word_length = if word_count > 0 {
                    text.split_whitespace()
                        .map(|word| word.len())
                        .sum::<usize>() as f64 / word_count as f64
                } else {
                    0.0
                };
                
                let stats = json!({
                    "avg_word_length": avg_word_length,
                    "longest_word": text.split_whitespace().max_by_key(|word| word.len()).unwrap_or(""),
                    "shortest_word": text.split_whitespace().min_by_key(|word| word.len()).unwrap_or(""),
                });
                
                if let Value::Object(ref mut map) = result {
                    map.insert("stats".to_string(), stats);
                }
            }
            
            Ok(result)
        })
        .with_tool_availability_checker("analyze", || {
            Ok(ToolAvailability {
                available: true,
                reason: None,
                missing_dependencies: Vec::new(),
                installation_instructions: None,
            })
        })
        .with_tool_full(
            "calculator",
            "Simple calculator tool",
            Some("utility"),
            vec!["math", "example"],
            false,
        )
        .with_tool_metadata(
            "calculator",
            "Simple calculator tool",
            "1.0.0",
            Some("DataScienceBioLab"),
            None,
            Some("MIT"),
            Vec::new(),
            Some(json!({
                "type": "object",
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
                        "description": "Operation to perform",
                        "enum": ["add", "subtract", "multiply", "divide"]
                    }
                },
                "required": ["a", "b", "operation"]
            })),
            Some(json!({
                "type": "object",
                "properties": {
                    "result": {
                        "type": "number",
                        "description": "Result of the operation"
                    },
                    "operation": {
                        "type": "string",
                        "description": "Operation performed"
                    }
                }
            })),
        )
        .with_tool_handler("calculator", |args| {
            let a = match args.get("a") {
                Some(val) => val.as_f64().unwrap_or(0.0),
                None => 0.0,
            };
            
            let b = match args.get("b") {
                Some(val) => val.as_f64().unwrap_or(0.0),
                None => 0.0,
            };
            
            let operation = match args.get("operation") {
                Some(val) => val.as_str().unwrap_or("add"),
                None => "add",
            };
            
            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b == 0.0 {
                        return Err(crate::plugins::PluginError::ExecutionError(
                            "Division by zero".to_string(),
                        ));
                    }
                    a / b
                }
                _ => a + b,
            };
            
            Ok(json!({
                "result": result,
                "operation": operation
            }))
        })
        .with_tool_availability_checker("calculator", || {
            Ok(ToolAvailability {
                available: true,
                reason: None,
                missing_dependencies: Vec::new(),
                installation_instructions: None,
            })
        })
        .build()
} 