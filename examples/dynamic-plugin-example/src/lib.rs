// Dynamic Plugin Example Implementation
//
// This file demonstrates how to implement a dynamically loadable plugin
// for the Squirrel Plugin System.

use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use squirrel_plugins::interfaces::{
    Plugin, CommandsPlugin, ToolPlugin, 
    CommandInfo, CommandHelp, CommandArgument, CommandOption,
    ToolInfo, ToolMetadata, ToolAvailability
};
use squirrel_plugins::errors::{Result, PluginError};
use squirrel_plugins::dynamic::{PluginMetadata, PluginDependency};

// Generate a stable UUID for this plugin
// In a real plugin, you would generate this once and keep it constant
fn plugin_id() -> Uuid {
    Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap_or_else(|_| Uuid::new_v4())
}

// ===== Plugin Implementation =====

/// Example dynamic plugin that implements both CommandsPlugin and ToolPlugin
#[derive(Debug)]
struct ExampleDynamicPlugin {
    /// Plugin ID
    id: Uuid,
    
    /// Plugin name
    name: String,
    
    /// Plugin version
    version: String,
    
    /// Plugin description
    description: String,
    
    /// Plugin author
    author: String,
    
    /// API Version
    api_version: String,
    
    /// Available commands
    commands: Vec<CommandInfo>,
    
    /// Available tools
    tools: Vec<ToolInfo>,
    
    /// Plugin state
    state: Arc<RwLock<HashMap<String, Value>>>,
}

impl ExampleDynamicPlugin {
    /// Create a new instance of the plugin
    fn new() -> Self {
        Self {
            id: plugin_id(),
            name: "dynamic-example".to_string(),
            version: "0.1.0".to_string(),
            description: "Example dynamic plugin with commands and tools".to_string(),
            author: "DataScienceBioLab".to_string(),
            api_version: "1.0.0".to_string(),
            commands: vec![
                CommandInfo {
                    name: "greet".to_string(),
                    description: "Greet a user with different languages".to_string(),
                    category: Some("examples".to_string()),
                    tags: vec!["greeting".to_string(), "dynamic".to_string()],
                    requires_auth: false,
                },
                CommandInfo {
                    name: "calculate".to_string(),
                    description: "Perform a calculation".to_string(),
                    category: Some("examples".to_string()),
                    tags: vec!["math".to_string(), "dynamic".to_string()],
                    requires_auth: false,
                },
            ],
            tools: vec![
                ToolInfo {
                    name: "format".to_string(),
                    description: "Format text in different styles".to_string(),
                    category: Some("formatting".to_string()),
                    tags: vec!["text".to_string(), "dynamic".to_string()],
                    requires_auth: false,
                },
                ToolInfo {
                    name: "convert".to_string(),
                    description: "Convert between different units".to_string(),
                    category: Some("utilities".to_string()),
                    tags: vec!["conversion".to_string(), "dynamic".to_string()],
                    requires_auth: false,
                },
            ],
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// Plugin trait implementation
#[async_trait]
impl Plugin for ExampleDynamicPlugin {
    fn metadata(&self) -> &dyn squirrel_mcp::plugins::interfaces::PluginMetadata {
        // This is normally used for static plugins, dynamically loaded plugins
        // use the no_mangle get_plugin_metadata function instead
        unimplemented!("This method is not used for dynamic plugins")
    }
    
    async fn initialize(&self) -> Result<()> {
        // Initialize plugin state
        info!("Initializing dynamic plugin: {}", self.name);
        let mut state = self.state.write().await;
        state.insert("initialized".to_string(), Value::Bool(true));
        state.insert("start_time".to_string(), Value::String(chrono::Utc::now().to_string()));
        
        Ok(())
    }
    
    async fn start(&self) -> Result<()> {
        // Start plugin operations
        info!("Starting dynamic plugin: {}", self.name);
        let mut state = self.state.write().await;
        state.insert("running".to_string(), Value::Bool(true));
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // Stop plugin operations
        info!("Stopping dynamic plugin: {}", self.name);
        let mut state = self.state.write().await;
        state.insert("running".to_string(), Value::Bool(false));
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        // Clean up resources
        info!("Shutting down dynamic plugin: {}", self.name);
        let mut state = self.state.write().await;
        state.insert("initialized".to_string(), Value::Bool(false));
        state.insert("end_time".to_string(), Value::String(chrono::Utc::now().to_string()));
        
        Ok(())
    }
}

// CommandsPlugin trait implementation
#[async_trait]
impl CommandsPlugin for ExampleDynamicPlugin {
    fn get_commands(&self) -> Vec<CommandInfo> {
        self.commands.clone()
    }
    
    async fn execute_command(&self, name: &str, args: Value) -> Result<Value> {
        match name {
            "greet" => {
                let name = args["name"].as_str().unwrap_or("User");
                let language = args["language"].as_str().unwrap_or("en");
                
                let greeting = match language {
                    "en" => format!("Hello, {}!", name),
                    "es" => format!("¡Hola, {}!", name),
                    "fr" => format!("Bonjour, {} !", name),
                    "de" => format!("Hallo, {}!", name),
                    "ja" => format!("こんにちは、{}さん!", name),
                    _ => format!("Hello, {}!", name),
                };
                
                debug!("Generated greeting: {}", greeting);
                
                Ok(serde_json::json!({
                    "greeting": greeting,
                    "language": language
                }))
            },
            "calculate" => {
                let a = args["a"].as_f64().ok_or_else(|| {
                    PluginError::InvalidArgument("'a' must be a number".into())
                })?;
                
                let b = args["b"].as_f64().ok_or_else(|| {
                    PluginError::InvalidArgument("'b' must be a number".into())
                })?;
                
                let operation = args["operation"].as_str().unwrap_or("add");
                
                let result = match operation {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => {
                        if b == 0.0 {
                            return Err(PluginError::InvalidArgument("Cannot divide by zero".into()));
                        }
                        a / b
                    },
                    _ => {
                        return Err(PluginError::InvalidArgument(
                            format!("Unknown operation: {}", operation)
                        ));
                    }
                };
                
                debug!("Calculated result: {} {} {} = {}", a, operation, b, result);
                
                Ok(serde_json::json!({
                    "result": result,
                    "operation": operation,
                    "a": a,
                    "b": b
                }))
            },
            _ => Err(PluginError::CommandNotFound(name.to_string()))
        }
    }
    
    fn get_command_help(&self, name: &str) -> Option<CommandHelp> {
        match name {
            "greet" => Some(CommandHelp {
                name: "greet".to_string(),
                description: "Greet a user in different languages".to_string(),
                usage: "greet [--name <name>] [--language <lang>]".to_string(),
                examples: vec![
                    "greet".to_string(),
                    "greet --name John".to_string(),
                    "greet --name Maria --language es".to_string(),
                ],
                arguments: vec![],
                options: vec![
                    CommandOption {
                        name: "name".to_string(),
                        description: "The name to greet".to_string(),
                        required: false,
                        data_type: "string".to_string(),
                        short_flag: Some('n'),
                        long_flag: Some("name".to_string()),
                    },
                    CommandOption {
                        name: "language".to_string(),
                        description: "The language for the greeting (en, es, fr, de, ja)".to_string(),
                        required: false,
                        data_type: "string".to_string(),
                        short_flag: Some('l'),
                        long_flag: Some("language".to_string()),
                    },
                ],
            }),
            "calculate" => Some(CommandHelp {
                name: "calculate".to_string(),
                description: "Perform a calculation with two numbers".to_string(),
                usage: "calculate --a <num> --b <num> [--operation <op>]".to_string(),
                examples: vec![
                    "calculate --a 5 --b 3".to_string(),
                    "calculate --a 10 --b 2 --operation divide".to_string(),
                ],
                arguments: vec![],
                options: vec![
                    CommandOption {
                        name: "a".to_string(),
                        description: "First number".to_string(),
                        required: true,
                        data_type: "number".to_string(),
                        short_flag: None,
                        long_flag: Some("a".to_string()),
                    },
                    CommandOption {
                        name: "b".to_string(),
                        description: "Second number".to_string(),
                        required: true,
                        data_type: "number".to_string(),
                        short_flag: None,
                        long_flag: Some("b".to_string()),
                    },
                    CommandOption {
                        name: "operation".to_string(),
                        description: "Operation to perform (add, subtract, multiply, divide)".to_string(),
                        required: false,
                        data_type: "string".to_string(),
                        short_flag: Some('o'),
                        long_flag: Some("operation".to_string()),
                    },
                ],
            }),
            _ => None
        }
    }
    
    fn get_command_schema(&self, name: &str) -> Option<Value> {
        match name {
            "greet" => Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name to greet"
                    },
                    "language": {
                        "type": "string",
                        "description": "The language for the greeting",
                        "enum": ["en", "es", "fr", "de", "ja"],
                        "default": "en"
                    }
                }
            })),
            "calculate" => Some(serde_json::json!({
                "type": "object",
                "required": ["a", "b"],
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
                        "enum": ["add", "subtract", "multiply", "divide"],
                        "default": "add"
                    }
                }
            })),
            _ => None
        }
    }
}

// ToolPlugin trait implementation
#[async_trait]
impl ToolPlugin for ExampleDynamicPlugin {
    fn get_tools(&self) -> Vec<ToolInfo> {
        self.tools.clone()
    }
    
    async fn execute_tool(&self, name: &str, args: Value) -> Result<Value> {
        match name {
            "format" => {
                let text = args["text"].as_str().ok_or_else(|| {
                    PluginError::InvalidArgument("'text' is required".into())
                })?;
                
                let style = args["style"].as_str().unwrap_or("normal");
                
                let formatted = match style {
                    "uppercase" => text.to_uppercase(),
                    "lowercase" => text.to_lowercase(),
                    "title" => {
                        let mut result = String::new();
                        let mut capitalize = true;
                        
                        for c in text.chars() {
                            if capitalize && c.is_alphabetic() {
                                result.push(c.to_uppercase().next().unwrap());
                                capitalize = false;
                            } else {
                                result.push(c);
                                if c.is_whitespace() {
                                    capitalize = true;
                                }
                            }
                        }
                        
                        result
                    },
                    "reverse" => text.chars().rev().collect(),
                    _ => text.to_string(),
                };
                
                debug!("Formatted text: {} -> {}", text, formatted);
                
                Ok(serde_json::json!({
                    "original": text,
                    "formatted": formatted,
                    "style": style
                }))
            },
            "convert" => {
                let value = args["value"].as_f64().ok_or_else(|| {
                    PluginError::InvalidArgument("'value' must be a number".into())
                })?;
                
                let from_unit = args["from_unit"].as_str().ok_or_else(|| {
                    PluginError::InvalidArgument("'from_unit' is required".into())
                })?;
                
                let to_unit = args["to_unit"].as_str().ok_or_else(|| {
                    PluginError::InvalidArgument("'to_unit' is required".into())
                })?;
                
                // Simplified conversion logic for example
                let (result, factor) = match (from_unit, to_unit) {
                    // Length
                    ("m", "km") => (value / 1000.0, 0.001),
                    ("km", "m") => (value * 1000.0, 1000.0),
                    ("m", "cm") => (value * 100.0, 100.0),
                    ("cm", "m") => (value / 100.0, 0.01),
                    ("m", "ft") => (value * 3.28084, 3.28084),
                    ("ft", "m") => (value / 3.28084, 0.3048),
                    
                    // Temperature
                    ("c", "f") => (value * 9.0/5.0 + 32.0, -1.0), // Special case
                    ("f", "c") => ((value - 32.0) * 5.0/9.0, -1.0), // Special case
                    
                    // Weight
                    ("kg", "lb") => (value * 2.20462, 2.20462),
                    ("lb", "kg") => (value / 2.20462, 0.453592),
                    
                    // Same unit, no conversion needed
                    (from, to) if from == to => (value, 1.0),
                    
                    // Unknown conversion
                    _ => {
                        return Err(PluginError::InvalidArgument(
                            format!("Conversion from '{}' to '{}' is not supported", from_unit, to_unit)
                        ));
                    }
                };
                
                debug!("Converted {} {} to {} {}", value, from_unit, result, to_unit);
                
                Ok(serde_json::json!({
                    "value": value,
                    "from_unit": from_unit,
                    "to_unit": to_unit,
                    "result": result,
                    "conversion_factor": factor
                }))
            },
            _ => Err(PluginError::ToolNotFound(name.to_string()))
        }
    }
    
    async fn check_tool_availability(&self, name: &str) -> Result<ToolAvailability> {
        // All tools are available in this example
        let tool_exists = self.tools.iter().any(|t| t.name == name);
        
        if tool_exists {
            Ok(ToolAvailability {
                available: true,
                reason: None,
            })
        } else {
            Ok(ToolAvailability {
                available: false,
                reason: Some(format!("Tool '{}' not found", name)),
            })
        }
    }
    
    fn get_tool_metadata(&self, name: &str) -> Option<ToolMetadata> {
        match name {
            "format" => Some(ToolMetadata {
                name: "format".to_string(),
                description: "Format text in different styles".to_string(),
                version: "1.0.0".to_string(),
                author: "DataScienceBioLab".to_string(),
                schema: serde_json::json!({
                    "input": {
                        "type": "object",
                        "required": ["text"],
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "The text to format"
                            },
                            "style": {
                                "type": "string",
                                "description": "The formatting style to apply",
                                "enum": ["normal", "uppercase", "lowercase", "title", "reverse"],
                                "default": "normal"
                            }
                        }
                    },
                    "output": {
                        "type": "object",
                        "properties": {
                            "original": {
                                "type": "string",
                                "description": "The original text"
                            },
                            "formatted": {
                                "type": "string",
                                "description": "The formatted text"
                            },
                            "style": {
                                "type": "string",
                                "description": "The style that was applied"
                            }
                        }
                    }
                }),
            }),
            "convert" => Some(ToolMetadata {
                name: "convert".to_string(),
                description: "Convert between different units".to_string(),
                version: "1.0.0".to_string(),
                author: "DataScienceBioLab".to_string(),
                schema: serde_json::json!({
                    "input": {
                        "type": "object",
                        "required": ["value", "from_unit", "to_unit"],
                        "properties": {
                            "value": {
                                "type": "number",
                                "description": "The value to convert"
                            },
                            "from_unit": {
                                "type": "string",
                                "description": "The source unit",
                                "examples": ["m", "km", "cm", "ft", "kg", "lb", "c", "f"]
                            },
                            "to_unit": {
                                "type": "string",
                                "description": "The target unit",
                                "examples": ["m", "km", "cm", "ft", "kg", "lb", "c", "f"]
                            }
                        }
                    },
                    "output": {
                        "type": "object",
                        "properties": {
                            "value": {
                                "type": "number",
                                "description": "The original value"
                            },
                            "from_unit": {
                                "type": "string",
                                "description": "The source unit"
                            },
                            "to_unit": {
                                "type": "string",
                                "description": "The target unit"
                            },
                            "result": {
                                "type": "number",
                                "description": "The converted value"
                            },
                            "conversion_factor": {
                                "type": "number",
                                "description": "The conversion factor applied"
                            }
                        }
                    }
                }),
            }),
            _ => None
        }
    }
}

// ===== FFI Export Functions =====

/// Create a new plugin instance.
/// This function is called by the plugin system to create the plugin.
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    let plugin = ExampleDynamicPlugin::new();
    Box::into_raw(Box::new(plugin))
}

/// Get plugin metadata.
/// This function is called by the plugin system to retrieve metadata about the plugin.
#[no_mangle]
pub extern "C" fn get_plugin_metadata() -> *mut PluginMetadata {
    let metadata = PluginMetadata {
        id: plugin_id(),
        name: "dynamic-example".to_string(),
        version: "0.1.0".to_string(),
        api_version: "1.0.0".to_string(),
        description: "Example dynamic plugin with commands and tools".to_string(),
        author: "DataScienceBioLab".to_string(),
        dependencies: Vec::new(),
    };
    
    Box::into_raw(Box::new(metadata))
}

/// Destroy the plugin.
/// This function is called by the plugin system when the plugin is unloaded.
#[no_mangle]
pub extern "C" fn destroy_plugin(plugin_ptr: *mut dyn Plugin) {
    if !plugin_ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(plugin_ptr);
        }
    }
} 