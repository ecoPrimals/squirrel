// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

// Dynamic Plugin Example
//
// This module provides an example of how to implement a dynamically loadable plugin.

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;
use std::sync::LazyLock;

use crate::plugins::interfaces::{Plugin, CommandsPlugin, CommandInfo, CommandHelp, CommandArgument, CommandOption};
use crate::plugins::errors::Result;

#[derive(Debug)]
struct DynamicExamplePlugin {
    id: Uuid,
    name: String,
    version: String,
    api_version: String,
    description: String,
    author: String,
    commands: Vec<CommandInfo>,
}

/// Mock metadata structure for example
struct MockMetadata {
    id: Uuid,
    name: String,
    version: String,
    description: String,
    author: String,
}

impl squirrel_mcp::plugins::interfaces::PluginMetadata for MockMetadata {
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn author(&self) -> &str {
        &self.author
    }
}

#[async_trait]
impl Plugin for DynamicExamplePlugin {
    fn metadata(&self) -> &dyn squirrel_mcp::plugins::interfaces::PluginMetadata {
        // Create a static metadata instance
        static METADATA: LazyLock<MockMetadata> = LazyLock::new(|| {
            MockMetadata {
                id: Uuid::new_v4(),
                name: "dynamic-example-plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A dynamic example plugin for structure demonstration".to_string(),
                author: "ecoPrimals Contributors".to_string(),
            }
        });
        
        &*METADATA
    }

    async fn initialize(&self) -> Result<()> {
        // Initialize plugin resources
        println!("Initializing dynamic example plugin");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        // Start plugin operations
        println!("Starting dynamic example plugin");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        // Stop plugin operations
        println!("Stopping dynamic example plugin");
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        // Clean up plugin resources
        println!("Shutting down dynamic example plugin");
        Ok(())
    }
}

#[async_trait]
impl CommandsPlugin for DynamicExamplePlugin {
    fn get_commands(&self) -> Vec<CommandInfo> {
        self.commands.clone()
    }

    async fn execute_command(&self, name: &str, args: Value) -> Result<Value> {
        match name {
            "hello" => {
                let name = args["name"].as_str().unwrap_or("World");
                Ok(serde_json::json!({
                    "message": format!("Hello, {}!", name)
                }))
            },
            "echo" => {
                Ok(serde_json::json!({
                    "echo": args
                }))
            },
            _ => Err(crate::plugins::errors::PluginError::CommandNotFound(name.to_string()))
        }
    }

    fn get_command_help(&self, name: &str) -> Option<CommandHelp> {
        match name {
            "hello" => Some(CommandHelp {
                name: "hello".to_string(),
                description: "Say hello to someone".to_string(),
                usage: "hello [--name <name>]".to_string(),
                examples: vec!["hello".to_string(), "hello --name John".to_string()],
                arguments: vec![],
                options: vec![
                    CommandOption {
                        name: "name".to_string(),
                        description: "The name to greet".to_string(),
                        required: false,
                        data_type: "string".to_string(),
                        short_flag: Some('n'),
                        long_flag: Some("name".to_string()),
                    }
                ],
            }),
            "echo" => Some(CommandHelp {
                name: "echo".to_string(),
                description: "Echo back the input".to_string(),
                usage: "echo <text>".to_string(),
                examples: vec!["echo hello world".to_string()],
                arguments: vec![
                    CommandArgument {
                        name: "text".to_string(),
                        description: "Text to echo".to_string(),
                        required: true,
                        data_type: "string".to_string(),
                    }
                ],
                options: vec![],
            }),
            _ => None
        }
    }

    fn get_command_schema(&self, name: &str) -> Option<Value> {
        match name {
            "hello" => Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name to greet"
                    }
                }
            })),
            "echo" => Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to echo"
                    }
                },
                "required": ["text"]
            })),
            _ => None
        }
    }
}

/// This function demonstrates how to create a dynamic plugin interface.
/// In a real shared library, you would use #[no_mangle] and extern "C" functions.
pub fn create_dynamic_example_plugin() -> Box<dyn Plugin> {
    let plugin = DynamicExamplePlugin {
        id: Uuid::new_v4(),
        name: "dynamic-example".to_string(),
        version: "1.0.0".to_string(),
        api_version: "1.0.0".to_string(),
        description: "An example dynamically loaded plugin".to_string(),
        author: "ecoPrimals Contributors".to_string(),
        commands: vec![
            CommandInfo {
                name: "hello".to_string(),
                description: "Say hello to someone".to_string(),
                category: Some("examples".to_string()),
                tags: vec!["greeting".to_string()],
                requires_auth: false,
            },
            CommandInfo {
                name: "echo".to_string(),
                description: "Echo back the input".to_string(),
                category: Some("examples".to_string()),
                tags: vec!["utility".to_string()],
                requires_auth: false,
            },
        ],
    };
    
    Box::new(plugin)
}

/// This shows how the exported functions would look in a real dynamic library:
/// 
/// ```
/// #[no_mangle]
/// pub extern "C" fn create_plugin() -> *mut dyn Plugin {
///     let plugin = create_dynamic_example_plugin();
///     Box::into_raw(plugin)
/// }
/// 
/// #[no_mangle]
/// pub extern "C" fn get_plugin_metadata() -> *mut PluginMetadata {
///     let metadata = PluginMetadata {
///         id: Uuid::new_v4(),
///         name: "dynamic-example".to_string(),
///         version: "1.0.0".to_string(),
///         api_version: "1.0.0".to_string(),
///         description: "An example dynamically loaded plugin".to_string(),
///         author: "ecoPrimals Contributors".to_string(),
///         dependencies: Vec::new(),
///     };
///     Box::into_raw(Box::new(metadata))
/// }
/// 
/// ❌ ELIMINATED: Unsafe plugin destruction replaced with safe Arc patterns
/// 
/// The old approach required unsafe code:
/// pub extern "C" fn destroy_plugin(plugin: *mut dyn Plugin) {
///     // Unsafe code was needed for raw pointer handling
/// }
///
/// ✅ NEW SAFE APPROACH: Use Arc reference counting
/// pub fn destroy_plugin_safe(plugin_id: String) -> bool {
///     // Uses Arc<dyn Plugin> for safe shared ownership
///     // No unsafe code needed - automatic memory management
/// }
/// }
/// ``` 