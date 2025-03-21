//! Example demonstrating the MCP tool management system
//!
//! This example shows how to:
//! 1. Create and register tools
//! 2. Use lifecycle hooks
//! 3. Execute tool capabilities
//! 4. Handle tool errors

use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use serde_json::json;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use squirrel_mcp::tool::{
    BasicLifecycleHook,
    BasicToolExecutor,
    Capability,
    CompositeLifecycleHook,
    ExecutionStatus,
    Parameter,
    ParameterType,
    RemoteToolExecutor,
    SecurityLifecycleHook,
    Tool,
    ToolError,
    ToolManager,
    ToolState,
};

/// Creates a calculator tool
fn create_calculator_tool() -> Tool {
    Tool {
        id: "calculator".to_string(),
        name: "Calculator".to_string(),
        version: "1.0.0".to_string(),
        description: "A simple calculator tool".to_string(),
        capabilities: vec![
            Capability {
                name: "add".to_string(),
                description: "Adds two numbers".to_string(),
                parameters: vec![
                    Parameter {
                        name: "a".to_string(),
                        description: "First number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                    Parameter {
                        name: "b".to_string(),
                        description: "Second number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                ],
                return_type: None,
            },
            Capability {
                name: "subtract".to_string(),
                description: "Subtracts two numbers".to_string(),
                parameters: vec![
                    Parameter {
                        name: "a".to_string(),
                        description: "First number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                    Parameter {
                        name: "b".to_string(),
                        description: "Second number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                ],
                return_type: None,
            },
            Capability {
                name: "multiply".to_string(),
                description: "Multiplies two numbers".to_string(),
                parameters: vec![
                    Parameter {
                        name: "a".to_string(),
                        description: "First number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                    Parameter {
                        name: "b".to_string(),
                        description: "Second number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                ],
                return_type: None,
            },
            Capability {
                name: "divide".to_string(),
                description: "Divides two numbers".to_string(),
                parameters: vec![
                    Parameter {
                        name: "a".to_string(),
                        description: "First number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                    Parameter {
                        name: "b".to_string(),
                        description: "Second number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                ],
                return_type: None,
            },
        ],
        security_level: 1,
    }
}

/// Creates a text processing tool
fn create_text_tool() -> Tool {
    Tool {
        id: "text-processor".to_string(),
        name: "Text Processor".to_string(),
        version: "1.0.0".to_string(),
        description: "A text processing tool".to_string(),
        capabilities: vec![
            Capability {
                name: "echo".to_string(),
                description: "Echoes back a message".to_string(),
                parameters: vec![
                    Parameter {
                        name: "message".to_string(),
                        description: "The message to echo".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                ],
                return_type: None,
            },
            Capability {
                name: "reverse".to_string(),
                description: "Reverses a string".to_string(),
                parameters: vec![
                    Parameter {
                        name: "text".to_string(),
                        description: "The text to reverse".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                ],
                return_type: None,
            },
            Capability {
                name: "count".to_string(),
                description: "Counts characters in a string".to_string(),
                parameters: vec![
                    Parameter {
                        name: "text".to_string(),
                        description: "The text to count".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                ],
                return_type: None,
            },
        ],
        security_level: 1,
    }
}

/// Creates a calculator executor
fn create_calculator_executor() -> BasicToolExecutor {
    let mut executor = BasicToolExecutor::new("calculator");
    
    // Register add handler
    executor.register_handler("add", |context| {
        if let Some(a) = context.parameters.get("a").and_then(|v| v.as_f64()) {
            if let Some(b) = context.parameters.get("b").and_then(|v| v.as_f64()) {
                Ok(json!({
                    "result": a + b,
                    "operation": "add",
                }))
            } else {
                Err(ToolError::ValidationFailed("Parameter 'b' must be a number".to_string()))
            }
        } else {
            Err(ToolError::ValidationFailed("Parameter 'a' must be a number".to_string()))
        }
    });
    
    // Register subtract handler
    executor.register_handler("subtract", |context| {
        if let Some(a) = context.parameters.get("a").and_then(|v| v.as_f64()) {
            if let Some(b) = context.parameters.get("b").and_then(|v| v.as_f64()) {
                Ok(json!({
                    "result": a - b,
                    "operation": "subtract",
                }))
            } else {
                Err(ToolError::ValidationFailed("Parameter 'b' must be a number".to_string()))
            }
        } else {
            Err(ToolError::ValidationFailed("Parameter 'a' must be a number".to_string()))
        }
    });
    
    // Register multiply handler
    executor.register_handler("multiply", |context| {
        if let Some(a) = context.parameters.get("a").and_then(|v| v.as_f64()) {
            if let Some(b) = context.parameters.get("b").and_then(|v| v.as_f64()) {
                Ok(json!({
                    "result": a * b,
                    "operation": "multiply",
                }))
            } else {
                Err(ToolError::ValidationFailed("Parameter 'b' must be a number".to_string()))
            }
        } else {
            Err(ToolError::ValidationFailed("Parameter 'a' must be a number".to_string()))
        }
    });
    
    // Register divide handler
    executor.register_handler("divide", |context| {
        if let Some(a) = context.parameters.get("a").and_then(|v| v.as_f64()) {
            if let Some(b) = context.parameters.get("b").and_then(|v| v.as_f64()) {
                if b == 0.0 {
                    return Err(ToolError::ExecutionFailed("Division by zero".to_string()));
                }
                
                Ok(json!({
                    "result": a / b,
                    "operation": "divide",
                }))
            } else {
                Err(ToolError::ValidationFailed("Parameter 'b' must be a number".to_string()))
            }
        } else {
            Err(ToolError::ValidationFailed("Parameter 'a' must be a number".to_string()))
        }
    });
    
    executor
}

/// Creates a text processor executor
fn create_text_executor() -> BasicToolExecutor {
    let mut executor = BasicToolExecutor::new("text-processor");
    
    // Register echo handler
    executor.register_handler("echo", |context| {
        if let Some(message) = context.parameters.get("message").and_then(|v| v.as_str()) {
            Ok(json!({
                "message": message,
                "timestamp": Utc::now().to_rfc3339(),
            }))
        } else {
            Err(ToolError::ValidationFailed("Parameter 'message' must be a string".to_string()))
        }
    });
    
    // Register reverse handler
    executor.register_handler("reverse", |context| {
        if let Some(text) = context.parameters.get("text").and_then(|v| v.as_str()) {
            let reversed: String = text.chars().rev().collect();
            
            Ok(json!({
                "original": text,
                "reversed": reversed,
                "length": text.len(),
            }))
        } else {
            Err(ToolError::ValidationFailed("Parameter 'text' must be a string".to_string()))
        }
    });
    
    // Register count handler
    executor.register_handler("count", |context| {
        if let Some(text) = context.parameters.get("text").and_then(|v| v.as_str()) {
            let char_count = text.chars().count();
            let word_count = text.split_whitespace().count();
            
            Ok(json!({
                "text": text,
                "char_count": char_count,
                "word_count": word_count,
                "has_spaces": text.contains(' '),
            }))
        } else {
            Err(ToolError::ValidationFailed("Parameter 'text' must be a string".to_string()))
        }
    });
    
    executor
}

/// Creates a remote tool
fn create_remote_tool() -> Tool {
    Tool {
        id: "remote-service".to_string(),
        name: "Remote Service".to_string(),
        version: "1.0.0".to_string(),
        description: "A remote service tool".to_string(),
        capabilities: vec![
            Capability {
                name: "remote_echo".to_string(),
                description: "Echoes back a message from a remote service".to_string(),
                parameters: vec![
                    Parameter {
                        name: "message".to_string(),
                        description: "The message to echo".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                ],
                return_type: None,
            },
            Capability {
                name: "remote_compute".to_string(),
                description: "Performs a computation on a remote service".to_string(),
                parameters: vec![
                    Parameter {
                        name: "a".to_string(),
                        description: "First number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                    Parameter {
                        name: "b".to_string(),
                        description: "Second number".to_string(),
                        parameter_type: ParameterType::Number,
                        required: true,
                    },
                    Parameter {
                        name: "operation".to_string(),
                        description: "Operation to perform (add, subtract, multiply, divide)".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                ],
                return_type: None,
            },
        ],
        security_level: 2, // Higher security level for remote services
    }
}

/// Creates a remote service executor
fn create_remote_executor() -> RemoteToolExecutor {
    RemoteToolExecutor::new("remote-service", "https://example.com/api")
        .with_capability("remote_echo")
        .with_capability("remote_compute")
        .with_timeout(5000)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
    
    info!("Starting Tool Management Example");
    
    // Create lifecycle hooks
    let basic_hook = BasicLifecycleHook::new();
    
    let security_hook = SecurityLifecycleHook::new()
        .with_default_security_level(1)
        .allow_tool("calculator")
        .allow_tool("text-processor")
        .allow_tool("remote-service")
        .enforce_allowed_tools(true);
    
    // Create a composite hook with both hooks
    let mut composite_hook = CompositeLifecycleHook::new();
    composite_hook.add_hook(basic_hook);
    composite_hook.add_hook(security_hook);
    
    // Create the tool manager
    let manager = Arc::new(ToolManager::new(composite_hook));
    
    // Create and register the calculator tool
    let calculator_tool = create_calculator_tool();
    let calculator_executor = create_calculator_executor();
    
    info!("Registering calculator tool");
    manager.register_tool(calculator_tool, calculator_executor).await?;
    
    // Create and register the text processor tool
    let text_tool = create_text_tool();
    let text_executor = create_text_executor();
    
    info!("Registering text processor tool");
    manager.register_tool(text_tool, text_executor).await?;
    
    // Create and register the remote service tool
    let remote_tool = create_remote_tool();
    let remote_executor = create_remote_executor();
    
    info!("Registering remote service tool");
    manager.register_tool(remote_tool, remote_executor).await?;
    
    // Activate all tools
    info!("Activating tools");
    manager.activate_tool("calculator").await?;
    manager.activate_tool("text-processor").await?;
    manager.activate_tool("remote-service").await?;
    
    // Display current states
    let tools = manager.get_all_tools().await;
    let all_states = manager.get_all_tool_states().await;
    
    for tool in &tools {
        let state = all_states.get(&tool.id).copied().unwrap_or(ToolState::Unregistered);
        info!("Tool: {} - State: {}", tool.id, state);
    }
    
    // Execute calculator capabilities
    info!("\nExecuting calculator capabilities:");
    
    // Addition
    {
        let mut parameters = HashMap::new();
        parameters.insert("a".to_string(), json!(5));
        parameters.insert("b".to_string(), json!(3));
        
        info!("  Executing add(5, 3)");
        let result = manager.execute_capability(
            "calculator",
            "add",
            parameters,
            None,
            None,
        ).await?;
        
        info!("  Result: {:?}", result.output);
        assert_eq!(result.status, ExecutionStatus::Success);
        let result_value = result.output.as_ref().unwrap()["result"].as_f64().unwrap();
        assert_eq!(result_value, 8.0);
    }
    
    // Subtraction
    {
        let mut parameters = HashMap::new();
        parameters.insert("a".to_string(), json!(10));
        parameters.insert("b".to_string(), json!(4));
        
        info!("  Executing subtract(10, 4)");
        let result = manager.execute_capability(
            "calculator",
            "subtract",
            parameters,
            None,
            None,
        ).await?;
        
        info!("  Result: {:?}", result.output);
        assert_eq!(result.status, ExecutionStatus::Success);
        let result_value = result.output.as_ref().unwrap()["result"].as_f64().unwrap();
        assert_eq!(result_value, 6.0);
    }
    
    // Division
    {
        let mut parameters = HashMap::new();
        parameters.insert("a".to_string(), json!(20));
        parameters.insert("b".to_string(), json!(5));
        
        info!("  Executing divide(20, 5)");
        let result = manager.execute_capability(
            "calculator",
            "divide",
            parameters,
            None,
            None,
        ).await?;
        
        info!("  Result: {:?}", result.output);
        assert_eq!(result.status, ExecutionStatus::Success);
        let result_value = result.output.as_ref().unwrap()["result"].as_f64().unwrap();
        assert_eq!(result_value, 4.0);
    }
    
    // Division by zero (error case)
    {
        let mut parameters = HashMap::new();
        parameters.insert("a".to_string(), json!(10));
        parameters.insert("b".to_string(), json!(0));
        
        info!("  Executing divide(10, 0) - expecting error");
        let result = manager.execute_capability(
            "calculator",
            "divide",
            parameters,
            None,
            None,
        ).await?;
        
        info!("  Error: {:?}", result.error_message);
        assert_eq!(result.status, ExecutionStatus::Failure);
        assert!(result.error_message.as_ref().unwrap().contains("Division by zero"));
    }
    
    // Execute text processor capabilities
    info!("\nExecuting text processor capabilities:");
    
    // Echo
    {
        let mut parameters = HashMap::new();
        parameters.insert("message".to_string(), json!("Hello, MCP!"));
        
        info!("  Executing echo('Hello, MCP!')");
        let result = manager.execute_capability(
            "text-processor",
            "echo",
            parameters,
            None,
            None,
        ).await?;
        
        info!("  Result: {:?}", result.output);
        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(result.output.as_ref().unwrap()["message"], json!("Hello, MCP!"));
    }
    
    // Reverse
    {
        let mut parameters = HashMap::new();
        parameters.insert("text".to_string(), json!("Machine Context Protocol"));
        
        info!("  Executing reverse('Machine Context Protocol')");
        let result = manager.execute_capability(
            "text-processor",
            "reverse",
            parameters,
            None,
            None,
        ).await?;
        
        info!("  Result: {:?}", result.output);
        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(
            result.output.as_ref().unwrap()["reversed"],
            json!("locotorP txetnoC enihcaM")
        );
    }
    
    // Count
    {
        let mut parameters = HashMap::new();
        parameters.insert("text".to_string(), json!("MCP: Machine Context Protocol"));
        
        info!("  Executing count('MCP: Machine Context Protocol')");
        let result = manager.execute_capability(
            "text-processor",
            "count",
            parameters,
            None,
            None,
        ).await?;
        
        info!("  Result: {:?}", result.output);
        assert_eq!(result.status, ExecutionStatus::Success);
        let char_count = result.output.as_ref().unwrap()["char_count"].as_i64().unwrap();
        let word_count = result.output.as_ref().unwrap()["word_count"].as_i64().unwrap();
        assert_eq!(char_count, 29);
        assert_eq!(word_count, 4);
    }
    
    // Execute remote service capabilities
    info!("\nExecuting remote service capabilities:");
    
    // Remote echo
    {
        let mut parameters = HashMap::new();
        parameters.insert("message".to_string(), json!("Hello from remote!"));
        
        info!("  Executing remote_echo('Hello from remote!')");
        let result = manager.execute_capability(
            "remote-service",
            "remote_echo",
            parameters,
            None,
            None,
        ).await?;
        
        info!("  Result: {:?}", result.output);
        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(result.output.as_ref().unwrap()["message"], json!("Hello from remote!"));
        assert_eq!(result.output.as_ref().unwrap()["remote"], json!(true));
    }
    
    // Remote compute
    {
        let mut parameters = HashMap::new();
        parameters.insert("a".to_string(), json!(15));
        parameters.insert("b".to_string(), json!(5));
        parameters.insert("operation".to_string(), json!("multiply"));
        
        info!("  Executing remote_compute(15, 5, 'multiply')");
        let result = manager.execute_capability(
            "remote-service",
            "remote_compute",
            parameters,
            None,
            None,
        ).await?;
        
        info!("  Result: {:?}", result.output);
        assert_eq!(result.status, ExecutionStatus::Success);
        let result_value = result.output.as_ref().unwrap()["result"].as_f64().unwrap();
        assert_eq!(result_value, 75.0);
    }
    
    // Test error cases
    info!("\nTesting error cases:");
    
    // Missing required parameter
    {
        let mut parameters = HashMap::new();
        parameters.insert("a".to_string(), json!(5));
        // Missing 'b' parameter
        
        info!("  Executing add(5, <missing>) - should fail");
        match manager.execute_capability(
            "calculator",
            "add",
            parameters,
            None,
            None,
        ).await {
            Ok(result) => {
                info!("  Error properly handled: {:?}", result.error_message);
                assert_eq!(result.status, ExecutionStatus::Failure);
            },
            Err(err) => {
                info!("  Error: {}", err);
            },
        }
    }
    
    // Invalid tool ID
    {
        let mut parameters = HashMap::new();
        parameters.insert("a".to_string(), json!(5));
        parameters.insert("b".to_string(), json!(3));
        
        info!("  Executing with invalid tool ID - should fail");
        match manager.execute_capability(
            "nonexistent-tool",
            "add",
            parameters,
            None,
            None,
        ).await {
            Ok(_) => panic!("Execution should have failed"),
            Err(err) => {
                info!("  Error as expected: {}", err);
            },
        }
    }
    
    // Invalid capability
    {
        let mut parameters = HashMap::new();
        parameters.insert("a".to_string(), json!(5));
        parameters.insert("b".to_string(), json!(3));
        
        info!("  Executing with invalid capability - should fail");
        match manager.execute_capability(
            "calculator",
            "invalid-capability",
            parameters,
            None,
            None,
        ).await {
            Ok(_) => panic!("Execution should have failed"),
            Err(err) => {
                info!("  Error as expected: {}", err);
            },
        }
    }
    
    // Cleanup
    info!("\nDeactivating and unregistering tools");
    manager.deactivate_tool("calculator").await?;
    manager.deactivate_tool("text-processor").await?;
    manager.deactivate_tool("remote-service").await?;
    
    manager.unregister_tool("calculator").await?;
    manager.unregister_tool("text-processor").await?;
    manager.unregister_tool("remote-service").await?;
    
    // Final state
    let remaining_tools = manager.get_all_tools().await;
    info!("Remaining tools: {}", remaining_tools.len());
    assert!(remaining_tools.is_empty(), "All tools should be unregistered");
    
    info!("Tool management example completed successfully!");
    Ok(())
} 