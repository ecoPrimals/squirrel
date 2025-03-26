#![no_main]

use serde_json::{Value, json};
use std::any::Any;
use uuid::Uuid;
use tokio::sync::RwLock;
use std::fmt::Debug;
use async_trait::async_trait;
use std::sync::Arc;

// Import the correct interfaces
use squirrel_interfaces::plugins::{Plugin, CommandsPlugin, PluginMetadata, CommandMetadata};
use anyhow;

/// A simple test plugin for fuzzing command execution
#[derive(Debug)]
struct TestCommandPlugin {
    metadata: PluginMetadata,
    state: RwLock<Option<Value>>,
}

impl TestCommandPlugin {
    /// Create a new test plugin
    fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: format!("test-plugin-{}", Uuid::new_v4()),
                name: "test-command-plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Test plugin for fuzzing".to_string(),
                author: "Fuzzer".to_string(),
                capabilities: Vec::new(),
            },
            state: RwLock::new(None),
        }
    }
}

#[async_trait]
impl Plugin for TestCommandPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> anyhow::Result<()> {
        Ok(())
    }
    
    async fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl CommandsPlugin for TestCommandPlugin {
    fn get_available_commands(&self) -> Vec<CommandMetadata> {
        vec![
            CommandMetadata {
                id: "echo".to_string(),
                name: "Echo".to_string(),
                description: "Echo back the input".to_string(),
                input_schema: json!({}),
                output_schema: json!({}),
                permissions: Vec::new(),
            },
            CommandMetadata {
                id: "add".to_string(),
                name: "Add".to_string(),
                description: "Add two numbers".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["a", "b"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "number"}
                    }
                }),
                permissions: Vec::new(),
            },
            CommandMetadata {
                id: "subtract".to_string(),
                name: "Subtract".to_string(),
                description: "Subtract two numbers".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["a", "b"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "number"}
                    }
                }),
                permissions: Vec::new(),
            },
            CommandMetadata {
                id: "multiply".to_string(),
                name: "Multiply".to_string(),
                description: "Multiply two numbers".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["a", "b"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "number"}
                    }
                }),
                permissions: Vec::new(),
            },
            CommandMetadata {
                id: "divide".to_string(),
                name: "Divide".to_string(),
                description: "Divide two numbers".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["a", "b"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "number"}
                    }
                }),
                permissions: Vec::new(),
            },
            CommandMetadata {
                id: "store".to_string(),
                name: "Store".to_string(),
                description: "Store a value in the plugin state".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string"},
                        "value": {}
                    },
                    "required": ["key", "value"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "status": {"type": "string"}
                    }
                }),
                permissions: Vec::new(),
            },
            CommandMetadata {
                id: "retrieve".to_string(),
                name: "Retrieve".to_string(),
                description: "Retrieve a value from the plugin state".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string"}
                    },
                    "required": ["key"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "found": {"type": "boolean"},
                        "value": {}
                    }
                }),
                permissions: Vec::new(),
            },
        ]
    }
    
    fn get_command_metadata(&self, command_id: &str) -> Option<CommandMetadata> {
        self.get_available_commands().into_iter().find(|cmd| cmd.id == command_id)
    }
    
    async fn execute_command(&self, command_id: &str, args: Value) -> anyhow::Result<Value> {
        match command_id {
            "echo" => {
                // Simply echo back the arguments
                Ok(args)
            },
            "add" => {
                // Add two numbers
                let a = args.get("a").and_then(Value::as_f64).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'a' parameter"))?;
                let b = args.get("b").and_then(Value::as_f64).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'b' parameter"))?;
                
                Ok(json!({ "result": a + b }))
            },
            "subtract" => {
                // Subtract two numbers
                let a = args.get("a").and_then(Value::as_f64).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'a' parameter"))?;
                let b = args.get("b").and_then(Value::as_f64).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'b' parameter"))?;
                
                Ok(json!({ "result": a - b }))
            },
            "multiply" => {
                // Multiply two numbers
                let a = args.get("a").and_then(Value::as_f64).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'a' parameter"))?;
                let b = args.get("b").and_then(Value::as_f64).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'b' parameter"))?;
                
                Ok(json!({ "result": a * b }))
            },
            "divide" => {
                // Divide two numbers
                let a = args.get("a").and_then(Value::as_f64).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'a' parameter"))?;
                let b = args.get("b").and_then(Value::as_f64).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'b' parameter"))?;
                
                if b == 0.0 {
                    return Err(anyhow::anyhow!("Division by zero"));
                }
                
                Ok(json!({ "result": a / b }))
            },
            "store" => {
                // Store a value in the plugin state
                let key = args.get("key").and_then(Value::as_str).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'key' parameter"))?;
                let value = args.get("value").ok_or_else(|| 
                    anyhow::anyhow!("Missing 'value' parameter"))?;
                
                let mut state = self.state.write().await;
                let current_state = state.get_or_insert_with(|| json!({}));
                
                if let Value::Object(ref mut obj) = current_state {
                    obj.insert(key.to_string(), value.clone());
                }
                
                Ok(json!({ "status": "ok" }))
            },
            "retrieve" => {
                // Retrieve a value from the plugin state
                let key = args.get("key").and_then(Value::as_str).ok_or_else(|| 
                    anyhow::anyhow!("Missing or invalid 'key' parameter"))?;
                
                let state = self.state.read().await;
                if state.is_none() {
                    return Ok(json!({ "found": false }));
                }
                
                if let Some(Value::Object(ref obj)) = *state {
                    if let Some(value) = obj.get(key) {
                        return Ok(json!({ "found": true, "value": value }));
                    }
                }
                
                Ok(json!({ "found": false }))
            },
            _ => Err(anyhow::anyhow!("Command not found: {}", command_id)),
        }
    }
    
    fn get_command_help(&self, command_id: &str) -> Option<String> {
        match command_id {
            "echo" => Some("Echo the input back as the output".to_string()),
            "add" => Some("Add two numbers: {\"a\": number, \"b\": number}".to_string()),
            "subtract" => Some("Subtract two numbers: {\"a\": number, \"b\": number}".to_string()),
            "multiply" => Some("Multiply two numbers: {\"a\": number, \"b\": number}".to_string()),
            "divide" => Some("Divide two numbers: {\"a\": number, \"b\": number}".to_string()),
            "store" => Some("Store a value: {\"key\": string, \"value\": any}".to_string()),
            "retrieve" => Some("Retrieve a value: {\"key\": string}".to_string()),
            _ => None,
        }
    }
}

/// Parse the fuzzer input into a command name and arguments
fn parse_command_fuzzer_input(data: &[u8]) -> (String, Value) {
    // Ensure we have enough data
    if data.len() < 2 {
        return ("echo".to_string(), json!({}));
    }
    
    // Use the first byte to determine the command
    let cmd_idx = data[0] as usize % 8;
    let commands = [
        "echo", "add", "subtract", "multiply", "divide", "store", "retrieve", "invalid"
    ];
    
    let command = commands[cmd_idx].to_string();
    
    // Try to parse the rest as JSON
    let args_data = &data[1..];
    match serde_json::from_slice::<Value>(args_data) {
        Ok(json) => (command, json),
        Err(_) => {
            // If JSON parsing fails, create some simple arguments based on the data
            match command.as_str() {
                "add" | "subtract" | "multiply" | "divide" => {
                    // Create numeric arguments
                    let a = if args_data.len() > 0 { args_data[0] as f64 } else { 1.0 };
                    let b = if args_data.len() > 1 { args_data[1] as f64 } else { 1.0 };
                    (command, json!({ "a": a, "b": b }))
                },
                "store" => {
                    // Create storage arguments
                    let key = if args_data.len() > 0 { 
                        format!("key_{}", args_data[0] % 10) 
                    } else { 
                        "default_key".to_string() 
                    };
                    let value = if args_data.len() > 1 { 
                        args_data[1] as i32
                    } else { 
                        0 
                    };
                    (command, json!({ "key": key, "value": value }))
                },
                "retrieve" => {
                    // Create retrieval arguments
                    let key = if args_data.len() > 0 { 
                        format!("key_{}", args_data[0] % 10) 
                    } else { 
                        "default_key".to_string() 
                    };
                    (command, json!({ "key": key }))
                },
                _ => (command, json!({})),
            }
        }
    }
}

// Entry point for the fuzzer
#[no_mangle]
pub extern "C" fn LLVMFuzzerTestOneInput(data: &[u8]) -> i32 {
    // Ensure we have at least some data
    if data.is_empty() {
        return 0;
    }
    
    // Parse the fuzzer input into a command and arguments
    let (command, args) = parse_command_fuzzer_input(data);
    
    // Create a test plugin
    let plugin = TestCommandPlugin::new();
    
    // Set up a tokio runtime to run our async code
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build runtime");
    
    // Execute the command with the fuzzer-generated arguments
    let _result = rt.block_on(async {
        // First try to set some initial state to make retrieve commands more interesting
        if command == "retrieve" {
            let _ = plugin.execute_command("store", json!({
                "key": "key_1",
                "value": "value1"
            })).await;
            
            let _ = plugin.execute_command("store", json!({
                "key": "key_2",
                "value": 42
            })).await;
            
            let _ = plugin.execute_command("store", json!({
                "key": "key_3",
                "value": [1, 2, 3]
            })).await;
        }
        
        // Execute the command
        let result = plugin.execute_command(&command, args).await;
        
        // We don't actually care about the result, we just want to make sure it doesn't crash
        result
    });

    // Return 0 to indicate successful fuzzing
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_command_fuzzer_input() {
        // Test empty input
        let (cmd, args) = parse_command_fuzzer_input(&[]);
        assert_eq!(cmd, "echo");
        assert_eq!(args, json!({}));
        
        // Test single byte input
        let (cmd, args) = parse_command_fuzzer_input(&[0]);
        assert_eq!(cmd, "echo");
        assert_eq!(args, json!({}));
        
        // Test add command
        let (cmd, args) = parse_command_fuzzer_input(&[1, 10, 20]);
        assert_eq!(cmd, "add");
        assert_eq!(args, json!({"a": 10.0, "b": 20.0}));
    }
} 