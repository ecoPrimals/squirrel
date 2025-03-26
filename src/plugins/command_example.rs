// Command Plugin Example
//
// This module provides an example command plugin implementation.

use std::sync::Arc;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::plugins::builders::CommandPluginBuilder;
use crate::plugins::interfaces::{CommandArgument, CommandOption};
use squirrel_mcp::plugins::interfaces::PluginMetadata;

/// Create an example command plugin
pub fn create_command_example_plugin() -> Box<dyn crate::plugins::CommandsPlugin> {
    // Create example metadata
    let metadata = PluginMetadata {
        id: Uuid::new_v4(),
        name: "example-command-plugin".to_string(),
        description: "An example command plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "DataScienceBioLab".to_string(),
        capabilities: vec!["command".to_string()],
        permissions: vec!["basic".to_string()],
        tags: vec!["example".to_string()],
        dependencies: Vec::new(),
        signature: None,
    };
    
    // Create a command plugin
    CommandPluginBuilder::new(metadata)
        .with_command_full(
            "hello",
            "Say hello to someone",
            Some("example"),
            vec!["greeting", "example"],
            false,
        )
        .with_command_help(
            "hello",
            "Say hello to someone",
            "hello [name]",
            vec!["hello World", "hello DataScienceBioLab"],
            vec![
                CommandArgument {
                    name: "name".to_string(),
                    description: "Name to greet".to_string(),
                    required: true,
                    data_type: "string".to_string(),
                },
            ],
            vec![
                CommandOption {
                    name: "language".to_string(),
                    description: "Language to use".to_string(),
                    required: false,
                    data_type: "string".to_string(),
                    short_flag: Some('l'),
                    long_flag: Some("language".to_string()),
                },
            ],
        )
        .with_command_schema(
            "hello",
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name to greet"
                    },
                    "language": {
                        "type": "string",
                        "description": "Language to use",
                        "enum": ["en", "es", "fr", "de", "ja"]
                    }
                },
                "required": ["name"]
            }),
        )
        .with_command_handler("hello", |args| {
            let name = match args.get("name") {
                Some(val) => val.as_str().unwrap_or("World"),
                None => "World",
            };
            
            let language = match args.get("language") {
                Some(val) => val.as_str().unwrap_or("en"),
                None => "en",
            };
            
            let greeting = match language {
                "es" => format!("¡Hola, {}!", name),
                "fr" => format!("Bonjour, {} !", name),
                "de" => format!("Hallo, {}!", name),
                "ja" => format!("こんにちは、{}!", name),
                _ => format!("Hello, {}!", name),
            };
            
            Ok(json!({ "message": greeting }))
        })
        .with_command_full(
            "echo",
            "Echo back the input",
            Some("example"),
            vec!["utility", "example"],
            false,
        )
        .with_command_help(
            "echo",
            "Echo back the input",
            "echo <message>",
            vec!["echo Hello, World!"],
            vec![
                CommandArgument {
                    name: "message".to_string(),
                    description: "Message to echo".to_string(),
                    required: true,
                    data_type: "string".to_string(),
                },
            ],
            vec![],
        )
        .with_command_schema(
            "echo",
            json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo"
                    }
                },
                "required": ["message"]
            }),
        )
        .with_command_handler("echo", |args| {
            let message = match args.get("message") {
                Some(val) => val.as_str().unwrap_or("").to_string(),
                None => "".to_string(),
            };
            
            Ok(json!({ "message": message }))
        })
        .build()
} 