//! Example plugins for the Squirrel system
//!
//! This crate provides example implementations of plugins for the Squirrel system.
//! These examples can be used as templates for creating custom plugins.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use squirrel_interfaces::plugins::{Plugin, PluginMetadata, CommandsPlugin, CommandMetadata};
use std::sync::Arc;

/// A basic utility plugin that provides text formatting functions
#[derive(Debug)]
pub struct UtilityPlugin {
    metadata: PluginMetadata,
}

impl UtilityPlugin {
    /// Create a new utility plugin
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "utility-plugin",
            env!("CARGO_PKG_VERSION"),
            "A utility plugin providing text formatting functions",
            "DataScienceBioLab",
        )
        .with_capability("text_formatting")
        .with_capability("command_execution");

        Self { metadata }
    }

    /// Format text by capitalizing it
    pub fn capitalize(&self, text: &str) -> String {
        let mut chars = text.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Format text by reversing it
    pub fn reverse(&self, text: &str) -> String {
        text.chars().rev().collect()
    }

    /// Format text by converting to uppercase
    pub fn to_uppercase(&self, text: &str) -> String {
        text.to_uppercase()
    }

    /// Format text by converting to lowercase
    pub fn to_lowercase(&self, text: &str) -> String {
        text.to_lowercase()
    }
}

#[async_trait]
impl Plugin for UtilityPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing utility plugin");
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down utility plugin");
        Ok(())
    }
}

/// Create a new utility plugin instance
///
/// # Returns
///
/// A new utility plugin instance wrapped in an Arc
pub fn create_utility_plugin() -> Arc<UtilityPlugin> {
    Arc::new(UtilityPlugin::new())
}

// Also implement CommandsPlugin to expose text formatting as commands
#[async_trait]
impl CommandsPlugin for UtilityPlugin {
    fn get_available_commands(&self) -> Vec<CommandMetadata> {
        vec![
            CommandMetadata {
                id: "text.capitalize".to_string(),
                name: "capitalize".to_string(),
                description: "Capitalize text".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["text"],
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "Text to capitalize"
                        }
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {
                            "type": "string",
                            "description": "Capitalized text"
                        }
                    }
                }),
                permissions: vec!["text.format".to_string()],
            },
            CommandMetadata {
                id: "text.reverse".to_string(),
                name: "reverse".to_string(),
                description: "Reverse text".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["text"],
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "Text to reverse"
                        }
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {
                            "type": "string",
                            "description": "Reversed text"
                        }
                    }
                }),
                permissions: vec!["text.format".to_string()],
            },
            CommandMetadata {
                id: "text.uppercase".to_string(),
                name: "uppercase".to_string(),
                description: "Convert text to uppercase".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["text"],
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "Text to convert"
                        }
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {
                            "type": "string",
                            "description": "Uppercase text"
                        }
                    }
                }),
                permissions: vec!["text.format".to_string()],
            },
            CommandMetadata {
                id: "text.lowercase".to_string(),
                name: "lowercase".to_string(),
                description: "Convert text to lowercase".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["text"],
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "Text to convert"
                        }
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {
                            "type": "string",
                            "description": "Lowercase text"
                        }
                    }
                }),
                permissions: vec!["text.format".to_string()],
            },
        ]
    }

    fn get_command_metadata(&self, command_id: &str) -> Option<CommandMetadata> {
        self.get_available_commands()
            .into_iter()
            .find(|cmd| cmd.id == command_id)
    }

    async fn execute_command(&self, command_id: &str, input: Value) -> Result<Value> {
        let text = input.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'text' parameter"))?;

        let result = match command_id {
            "text.capitalize" => self.capitalize(text),
            "text.reverse" => self.reverse(text),
            "text.uppercase" => self.to_uppercase(text),
            "text.lowercase" => self.to_lowercase(text),
            _ => return Err(anyhow::anyhow!("Unknown command: {}", command_id)),
        };

        Ok(serde_json::json!({
            "result": result
        }))
    }

    fn get_command_help(&self, command_id: &str) -> Option<String> {
        match command_id {
            "text.capitalize" => Some("Capitalizes the first letter of the text".to_string()),
            "text.reverse" => Some("Reverses the characters in the text".to_string()),
            "text.uppercase" => Some("Converts all characters to uppercase".to_string()),
            "text.lowercase" => Some("Converts all characters to lowercase".to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        let plugin = UtilityPlugin::new();
        assert_eq!(plugin.capitalize("hello"), "Hello");
        assert_eq!(plugin.capitalize("world"), "World");
        assert_eq!(plugin.capitalize(""), "");
        assert_eq!(plugin.capitalize("a"), "A");
    }

    #[test]
    fn test_reverse() {
        let plugin = UtilityPlugin::new();
        assert_eq!(plugin.reverse("hello"), "olleh");
        assert_eq!(plugin.reverse("world"), "dlrow");
        assert_eq!(plugin.reverse(""), "");
        assert_eq!(plugin.reverse("a"), "a");
    }

    #[test]
    fn test_to_uppercase() {
        let plugin = UtilityPlugin::new();
        assert_eq!(plugin.to_uppercase("hello"), "HELLO");
        assert_eq!(plugin.to_uppercase("World"), "WORLD");
        assert_eq!(plugin.to_uppercase(""), "");
        assert_eq!(plugin.to_uppercase("A"), "A");
    }

    #[test]
    fn test_to_lowercase() {
        let plugin = UtilityPlugin::new();
        assert_eq!(plugin.to_lowercase("HELLO"), "hello");
        assert_eq!(plugin.to_lowercase("World"), "world");
        assert_eq!(plugin.to_lowercase(""), "");
        assert_eq!(plugin.to_lowercase("a"), "a");
    }

    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let plugin = UtilityPlugin::new();
        assert!(plugin.initialize().await.is_ok());
        assert!(plugin.shutdown().await.is_ok());
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = UtilityPlugin::new();
        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "utility-plugin");
        assert!(metadata.capabilities.contains(&"text_formatting".to_string()));
    }

    #[tokio::test]
    async fn test_command_execution() -> Result<()> {
        let plugin = UtilityPlugin::new();
        
        // Test capitalize command
        let result = plugin.execute_command(
            "text.capitalize", 
            serde_json::json!({"text": "hello"})
        ).await?;
        
        assert_eq!(result.get("result").unwrap().as_str().unwrap(), "Hello");
        
        // Test reverse command
        let result = plugin.execute_command(
            "text.reverse", 
            serde_json::json!({"text": "hello"})
        ).await?;
        
        assert_eq!(result.get("result").unwrap().as_str().unwrap(), "olleh");
        
        Ok(())
    }
} 