//! Example Tool plugin implementation
//!
//! This file demonstrates how to create a tool plugin using the Squirrel plugin system.
//! It implements a plugin that provides code analysis and formatting tools.

use crate::error::Result;
use crate::plugin::{
    Plugin, PluginMetadata, PluginState,
    types::{ToolPlugin, ToolPluginImpl, ToolPluginBuilder},
};
use async_trait::async_trait;
use futures::future::BoxFuture;
use serde_json::{json, Value};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Create a simple tool plugin using the builder pattern
pub fn create_example_tool_plugin() -> Box<dyn ToolPlugin> {
    ToolPluginBuilder::new(PluginMetadata {
        id: Uuid::new_v4(),
        name: "example-tools".to_string(),
        version: "0.1.0".to_string(),
        description: "Example tool plugin".to_string(),
        author: "Squirrel Team".to_string(),
        dependencies: vec![],
        capabilities: vec!["tool".to_string()],
    })
    .with_tool("code-analyzer", json!({
        "languages": ["rust", "javascript", "typescript"],
        "metrics": ["complexity", "lines", "dependencies"]
    }))
    .with_tool("code-formatter", json!({
        "languages": ["rust", "javascript", "typescript"],
        "style": "default"
    }))
    .build()
}

/// Custom tool plugin implementation with real functionality
#[derive(Debug)]
pub struct CodeToolsPlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Plugin state
    state: RwLock<Option<PluginState>>,
    /// Available tools
    tools: Vec<String>,
    /// Tool configurations
    tool_configs: HashMap<String, Value>,
}

impl CodeToolsPlugin {
    /// Create a new code tools plugin
    pub fn new() -> Self {
        let mut plugin = Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: "code-tools".to_string(),
                version: "0.1.0".to_string(),
                description: "Advanced code analysis and formatting tools".to_string(),
                author: "Squirrel Team".to_string(),
                dependencies: vec![],
                capabilities: vec!["tool".to_string()],
            },
            state: RwLock::new(None),
            tools: vec![
                "analyze".to_string(),
                "format".to_string(),
            ],
            tool_configs: HashMap::new(),
        };
        
        // Set up tool configurations
        plugin.tool_configs.insert("analyze".to_string(), json!({
            "languages": ["rust", "javascript", "typescript"],
            "depth": "full",
            "include_metrics": true
        }));
        
        plugin.tool_configs.insert("format".to_string(), json!({
            "languages": ["rust", "javascript", "typescript"],
            "style": "default",
            "line_width": 100
        }));
        
        plugin
    }
    
    /// Create a new code tools plugin with custom metadata
    pub fn with_metadata(metadata: PluginMetadata) -> Self {
        let mut plugin = Self {
            metadata,
            state: RwLock::new(None),
            tools: vec![
                "analyze".to_string(),
                "format".to_string(),
            ],
            tool_configs: HashMap::new(),
        };
        
        // Set up tool configurations
        plugin.tool_configs.insert("analyze".to_string(), json!({
            "languages": ["rust", "javascript", "typescript"],
            "depth": "full",
            "include_metrics": true
        }));
        
        plugin.tool_configs.insert("format".to_string(), json!({
            "languages": ["rust", "javascript", "typescript"],
            "style": "default",
            "line_width": 100
        }));
        
        plugin
    }
    
    /// Analyze code and return metrics
    async fn analyze_code(&self, code: &str, language: &str) -> Result<Value> {
        // This is a simplified mock implementation
        // A real implementation would use language-specific analysis tools
        
        let loc = code.lines().count();
        let functions = code.matches("fn ").count();
        let comments = code.lines().filter(|line| line.trim().starts_with("//")).count();
        
        let complexity = if code.contains("if") || code.contains("match") || code.contains("for") {
            "medium"
        } else {
            "low"
        };
        
        Ok(json!({
            "language": language,
            "lines_of_code": loc,
            "functions": functions,
            "comments": comments,
            "complexity": complexity,
            "analyzed_at": chrono::Utc::now().to_rfc3339(),
        }))
    }
    
    /// Format code according to language standards
    async fn format_code(&self, code: &str, language: &str) -> Result<String> {
        // This is a simplified mock implementation
        // A real implementation would use language-specific formatters
        
        let formatted = match language {
            "rust" => {
                // Simple indentation for Rust-like code
                let mut result = String::new();
                let mut indent = 0;
                
                for line in code.lines() {
                    let trimmed = line.trim();
                    
                    if trimmed.contains('}') && indent > 0 {
                        indent -= 1;
                    }
                    
                    if !trimmed.is_empty() {
                        let spaces = " ".repeat(indent * 4);
                        result.push_str(&format!("{}{}\n", spaces, trimmed));
                    } else {
                        result.push('\n');
                    }
                    
                    if trimmed.contains('{') {
                        indent += 1;
                    }
                }
                
                result
            },
            _ => {
                // For other languages, just normalize whitespace
                code.lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };
        
        Ok(formatted)
    }
}

impl Plugin for CodeToolsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    
    fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    
    fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>> {
        Box::pin(async move {
            let state = self.state.read().await;
            Ok(state.clone())
        })
    }
    
    fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let mut guard = self.state.write().await;
            *guard = Some(state);
            Ok(())
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(Self {
            metadata: self.metadata.clone(),
            state: RwLock::new(None),
            tools: self.tools.clone(),
            tool_configs: self.tool_configs.clone(),
        })
    }
}

#[async_trait]
impl ToolPlugin for CodeToolsPlugin {
    async fn execute_tool(&self, tool: &str, args: Value) -> Result<Value> {
        match tool {
            "analyze" => {
                let code = args.get("code").and_then(Value::as_str).unwrap_or("");
                let language = args.get("language").and_then(Value::as_str).unwrap_or("rust");
                
                self.analyze_code(code, language).await
            },
            "format" => {
                let code = args.get("code").and_then(Value::as_str).unwrap_or("");
                let language = args.get("language").and_then(Value::as_str).unwrap_or("rust");
                
                let formatted = self.format_code(code, language).await?;
                Ok(json!({ "formatted_code": formatted }))
            },
            _ => {
                Ok(json!({
                    "error": format!("Unknown tool: {}", tool),
                    "available_tools": self.tools
                }))
            }
        }
    }
    
    fn get_tool_config(&self, tool: &str) -> Option<Value> {
        self.tool_configs.get(tool).cloned()
    }
    
    fn list_tools(&self) -> Vec<String> {
        self.tools.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_builder_plugin() {
        let plugin = create_example_tool_plugin();
        
        // Test metadata
        assert_eq!(plugin.metadata().name, "example-tools");
        
        // Test tools
        let tools = plugin.list_tools();
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"code-analyzer".to_string()));
        assert!(tools.contains(&"code-formatter".to_string()));
        
        // Test tool config
        let config = plugin.get_tool_config("code-analyzer").unwrap();
        assert!(config.get("languages").is_some());
    }
    
    #[tokio::test]
    async fn test_code_tools_plugin() {
        let plugin = CodeToolsPlugin::new();
        
        // Test code analysis
        let code = r#"
fn main() {
    println!("Hello, world!");
    let x = Some(5);
    let y = x.unwrap();
}
"#;
        
        let analyze_args = json!({
            "code": code,
            "language": "rust"
        });
        
        let result = plugin.execute_tool("analyze", analyze_args).await.unwrap();
        assert!(result.get("lines_of_code").is_some());
        assert!(result.get("functions").is_some());
        
        // Test code formatting
        let format_args = json!({
            "code": code,
            "language": "rust"
        });
        
        let result = plugin.execute_tool("format", format_args).await.unwrap();
        assert!(result.get("formatted_code").is_some());
    }
} 