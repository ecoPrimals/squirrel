use serde_json::json;
use squirrel_mcp::plugins::interfaces::PluginMetadata;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

mod plugins;

use plugins::{
    builders::{CommandPluginBuilder, ToolPluginBuilder},
    create_plugin_discovery, create_plugin_manager,
    interfaces::{CommandArgument, CommandOption, ToolAvailability},
    PluginStatus,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Squirrel Plugin System Demo");
    println!("==========================\n");

    // Create the plugin manager
    let manager = create_plugin_manager();
    println!("Plugin manager created");

    // Create the plugin discovery
    let discovery = create_plugin_discovery(manager.clone());
    println!("Plugin discovery created");

    // Create example plugins
    println!("\nCreating example plugins...");
    let command_plugin = create_hello_world_plugin();
    let tool_plugin = create_calculator_plugin();

    // Register and initialize plugins
    println!("\nRegistering and initializing plugins...");
    let command_id = command_plugin.metadata().id;
    let tool_id = tool_plugin.metadata().id;

    manager.register_plugin(command_plugin).await?;
    manager.register_plugin(tool_plugin).await?;

    println!("Registered plugins:");
    for plugin in manager.get_all_plugins().await? {
        println!(" - {} ({})", plugin.metadata().name, plugin.metadata().id);
    }

    // Initialize and start the plugins
    manager.initialize_plugin(&command_id).await?;
    manager.initialize_plugin(&tool_id).await?;
    manager.start_plugin(&command_id).await?;
    manager.start_plugin(&tool_id).await?;

    println!("\nPlugin statuses:");
    println!(" - Hello World Plugin: {:?}", manager.get_plugin_status(&command_id).await?);
    println!(" - Calculator Plugin: {:?}", manager.get_plugin_status(&tool_id).await?);

    // Execute commands
    println!("\nExecuting commands...");
    
    // Execute hello command
    let args = json!({
        "name": "DataScienceBioLab",
        "language": "en"
    });
    let result = manager.execute_command(&command_id, "hello", &args).await?;
    println!("Result of 'hello': {}", result.get("greeting").unwrap());

    // Execute hello command in Spanish
    let args = json!({
        "name": "DataScienceBioLab",
        "language": "es"
    });
    let result = manager.execute_command(&command_id, "hello", &args).await?;
    println!("Result of 'hello' in Spanish: {}", result.get("greeting").unwrap());

    // Execute calculator tool with addition
    println!("\nExecuting tools...");
    let args = json!({
        "a": 10,
        "b": 5,
        "operation": "add"
    });
    let result = manager.execute_tool(&tool_id, "calculator", &args).await?;
    println!("Result of 'calculator' add: {} {} {} = {}", 
        args["a"], args["operation"], args["b"], result["result"]);

    // Execute calculator tool with multiplication
    let args = json!({
        "a": 10,
        "b": 5,
        "operation": "multiply"
    });
    let result = manager.execute_tool(&tool_id, "calculator", &args).await?;
    println!("Result of 'calculator' multiply: {} {} {} = {}", 
        args["a"], args["operation"], args["b"], result["result"]);

    // Discover plugins from filesystem
    println!("\nDiscovering plugins from filesystem...");
    if Path::new("./plugins").exists() {
        let discovered = discovery.discover_plugins("./plugins").await?;
        println!("Discovered {} plugins", discovered.len());
        
        for plugin in discovered {
            println!(" - {} ({})", plugin.name, plugin.id);
        }
    } else {
        println!("No plugins directory found. Create a './plugins' directory and add example plugin files.");
    }

    // Stop the plugins
    println!("\nStopping plugins...");
    manager.stop_plugin(&command_id).await?;
    manager.stop_plugin(&tool_id).await?;
    
    println!("\nPlugin statuses after stopping:");
    println!(" - Hello World Plugin: {:?}", manager.get_plugin_status(&command_id).await?);
    println!(" - Calculator Plugin: {:?}", manager.get_plugin_status(&tool_id).await?);

    println!("\nDemo completed successfully!");
    Ok(())
}

/// Create a hello world command plugin
fn create_hello_world_plugin() -> Arc<dyn plugins::CommandsPlugin> {
    // Create metadata
    let metadata = PluginMetadata {
        id: Uuid::new_v4(),
        name: "hello-world-plugin".to_string(),
        description: "A simple hello world command plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "DataScienceBioLab".to_string(),
        capabilities: vec!["command".to_string()],
        permissions: vec!["basic".to_string()],
        tags: vec!["example".to_string(), "greeting".to_string()],
        dependencies: Vec::new(),
        signature: None,
    };

    // Create the plugin
    CommandPluginBuilder::new(metadata)
        .with_command_full(
            "hello",
            "Say hello to someone",
            Some("greetings"),
            vec!["greeting", "example"],
            false,
        )
        .with_command_help(
            "hello",
            "Say hello to someone",
            "hello [name]",
            vec!["hello World", "hello DataScienceBioLab"],
            vec![CommandArgument {
                name: "name".to_string(),
                description: "Name to greet".to_string(),
                required: true,
                data_type: "string".to_string(),
            }],
            vec![CommandOption {
                name: "language".to_string(),
                description: "Language to use".to_string(),
                required: false,
                data_type: "string".to_string(),
                short_flag: Some('l'),
                long_flag: Some("language".to_string()),
            }],
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

            Ok(json!({ "greeting": greeting }))
        })
        .build()
}

/// Create a calculator tool plugin
fn create_calculator_plugin() -> Arc<dyn plugins::ToolPlugin> {
    // Create metadata
    let metadata = PluginMetadata {
        id: Uuid::new_v4(),
        name: "calculator-plugin".to_string(),
        description: "A simple calculator tool plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "DataScienceBioLab".to_string(),
        capabilities: vec!["tool".to_string()],
        permissions: vec!["basic".to_string()],
        tags: vec!["example".to_string(), "math".to_string()],
        dependencies: Vec::new(),
        signature: None,
    };

    // Create the plugin
    ToolPluginBuilder::new(metadata)
        .with_tool_full(
            "calculator",
            "Simple calculator tool",
            Some("math"),
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
                        return Err(plugins::PluginError::ExecutionError(
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