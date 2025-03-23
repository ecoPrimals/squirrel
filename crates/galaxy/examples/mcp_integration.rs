#[cfg(feature = "mcp-integration")]
use galaxy::{create_adapter_with_config, GalaxyConfig, Error};

// Import the squirrel_mcp types for MessageId
use serde_json::json;
use squirrel_mcp::types::MessageId;

#[cfg(feature = "mcp-integration")]
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create a config for the Galaxy instance
    // NOTE: Replace "YOUR_API_KEY" with a valid Galaxy API key to run this example
    let config = GalaxyConfig::new("https://usegalaxy.org/api")
        .with_api_key("YOUR_API_KEY");
    
    // Create an adapter with the specified configuration
    let mut adapter = create_adapter_with_config(config)?;
    
    // Initialize MCP integration
    adapter.initialize_mcp()?;
    println!("MCP integration initialized\n");
    
    // Let Galaxy adapter handle message creation
    println!("Sending discover_tools command...");
    let request = galaxy::adapter::mcp_types::Message {
        id: MessageId("1".to_string()),
        message_type: galaxy::adapter::mcp_types::MessageType::Command,
        payload: json!({
            "command": "discover_tools",
            "tool_prefix": "Cut",
            "limit": 5,
            "offset": 0
        }),
    };
    
    let response = adapter.handle_message(request).await?;
    println!("Received response: {:?}\n", response);
    
    // Parse the response - we expect a Response type with command field
    if let Some(command) = response.payload.get("command").and_then(|c| c.as_str()) {
        if command == "discover_tools_response" {
            if let Some(tools) = response.payload.get("tools").and_then(|t| t.as_array()) {
                println!("Found {} tools:", tools.len());
                
                for (i, tool) in tools.iter().enumerate() {
                    println!("{}. {} ({})", 
                        i + 1, 
                        tool.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown"),
                        tool.get("id").and_then(|id| id.as_str()).unwrap_or("Unknown ID")
                    );
                    println!("   Description: {}", 
                        tool.get("description").and_then(|d| d.as_str()).unwrap_or("No description")
                    );
                    
                    if let Some(params) = tool.get("parameters").and_then(|p| p.as_array()) {
                        println!("   Parameters:");
                        for param in params {
                            println!("     - {} ({}): {}", 
                                param.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown"),
                                param.get("parameter_type").and_then(|t| t.as_str()).unwrap_or("Unknown"),
                                if param.get("required").and_then(|r| r.as_bool()).unwrap_or(false) {
                                    "Required"
                                } else {
                                    "Optional"
                                }
                            );
                        }
                    }
                    println!();
                }
            }
        } else {
            println!("Unexpected response command: {}", command);
        }
    } else {
        println!("Response missing command field");
    }
    
    // Now let's try an execution example
    println!("Demonstrating tool execution via MCP...");
    
    // Create a history
    let history = adapter.create_history("MCP Tool Execution Example").await?;
    println!("Created history: {} (ID: {})\n", history.metadata.name, history.metadata.id);
    
    // Upload a dataset
    let content = "Column1\tColumn2\tColumn3\nValue1\tValue2\tValue3\nValue4\tValue5\tValue6";
    let dataset = adapter.upload_dataset(
        "test_data.txt", 
        content.as_bytes().to_vec(),
        "tabular",
        Some(&history.metadata.id)
    ).await?;
    println!("Uploaded dataset: {} (ID: {})\n", dataset.metadata.name, dataset.metadata.id);
    
    // Create a tool execution request via MCP
    let execution_request = galaxy::adapter::mcp_types::Message {
        id: MessageId("2".to_string()),
        message_type: galaxy::adapter::mcp_types::MessageType::Command,
        payload: json!({
            "command": "execute_tool",
            "tool_id": "Cut1",
            "parameters": {
                "input": dataset.metadata.id,
                "columnList": "2",
                "delimiter": "T"
            },
            "context": {
                "history_id": history.metadata.id
            }
        }),
    };
    
    println!("Sending tool execution request...");
    let exec_response = adapter.handle_message(execution_request).await?;
    println!("Received execution response: {:?}\n", exec_response);
    
    // Parse the job ID from the response
    if let Some(command) = exec_response.payload.get("command").and_then(|c| c.as_str()) {
        if command == "execute_tool_response" {
            let job_id = exec_response.payload.get("job_id")
                .and_then(|j| j.as_str())
                .unwrap_or("unknown");
            println!("Job started with ID: {}\n", job_id);
            
            // Poll for job status via MCP
            println!("Polling for job status...");
            loop {
                let status_request = galaxy::adapter::mcp_types::Message {
                    id: MessageId("3".to_string()),
                    message_type: galaxy::adapter::mcp_types::MessageType::Command,
                    payload: json!({
                        "command": "get_job_status",
                        "job_id": job_id
                    }),
                };
                
                let status_response = adapter.handle_message(status_request).await?;
                println!("Job status response: {:?}", status_response);
                
                if let Some(command) = status_response.payload.get("command").and_then(|c| c.as_str()) {
                    if command == "get_job_status_response" {
                        let state = status_response.payload.get("state")
                            .and_then(|s| s.as_str())
                            .unwrap_or("unknown");
                        let is_terminal = status_response.payload.get("is_terminal")
                            .and_then(|t| t.as_bool())
                            .unwrap_or(false);
                            
                        println!("Job state: {}", state);
                        
                        if is_terminal {
                            let is_successful = status_response.payload.get("is_successful")
                                .and_then(|s| s.as_bool())
                                .unwrap_or(false);
                                
                            if is_successful {
                                println!("Job completed successfully!\n");
                                
                                // Get results via MCP
                                let results_request = galaxy::adapter::mcp_types::Message {
                                    id: MessageId("4".to_string()),
                                    message_type: galaxy::adapter::mcp_types::MessageType::Command,
                                    payload: json!({
                                        "command": "get_job_results",
                                        "job_id": job_id
                                    }),
                                };
                                
                                let results_response = adapter.handle_message(results_request).await?;
                                if let Some(command) = results_response.payload.get("command").and_then(|c| c.as_str()) {
                                    if command == "get_job_results_response" {
                                        if let Some(outputs) = results_response.payload.get("outputs").and_then(|o| o.as_array()) {
                                            println!("Job produced {} outputs:", outputs.len());
                                            
                                            for (i, output) in outputs.iter().enumerate() {
                                                let dataset_id = output.get("id").and_then(|id| id.as_str()).unwrap_or("unknown");
                                                println!("{}. Output name: {}, Dataset ID: {}", 
                                                    i + 1,
                                                    output.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown"),
                                                    dataset_id
                                                );
                                                
                                                // Download the output content
                                                let content = adapter.download_dataset(dataset_id).await?;
                                                println!("   Content ({} bytes):", content.len());
                                                println!("   {}", String::from_utf8_lossy(&content[..content.len().min(200)]));
                                                if content.len() > 200 {
                                                    println!("   ... (truncated)");
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!("Job failed with state: {}", state);
                            }
                            break;
                        }
                    }
                }
                
                // Wait before polling again
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
    
    println!("MCP integration example completed successfully!");
    Ok(())
}

#[cfg(not(feature = "mcp-integration"))]
fn main() {
    println!("This example requires the 'mcp-integration' feature");
    println!("Run with: cargo run --example mcp_integration --features mcp-integration");
} 