use std::time::Duration;
use mcp::task::{MCPTaskClient, TaskClientConfig, TaskPriority, AgentType};
use tokio::time;
use anyhow::Result;
use serde_json::json;

/// Example demonstrating usage of the MCPTaskClient
#[tokio::main]
async fn main() -> Result<()> {
    // Configure logging
    tracing_subscriber::fmt::init();
    
    // Create a client with custom configuration
    let config = TaskClientConfig {
        server_address: "[::1]:50052".to_string(),
        connect_timeout_sec: 10,
        ..Default::default()
    };
    
    let mut client = MCPTaskClient::with_config(config);
    
    // Connect to the task service
    client.connect().await?;
    println!("Connected to task service");
    
    // Create a task
    let input_data = json!({
        "key1": "value1",
        "key2": 42,
        "nested": {
            "inner": "data"
        }
    });
    
    let context_id = "example-context-1";
    let task_id = client.create_task(
        "Example Task",
        "This is an example task created from the client example",
        TaskPriority::Medium,
        Some(input_data),
        Some(json!({"tags": ["example", "test"]})),
        Some(context_id),
        vec![],
    ).await?;
    
    println!("Created task with ID: {}", task_id);
    
    // Retrieve the task
    let task = client.get_task(&task_id).await?;
    println!("Retrieved task: {} (status: {:?})", task.name, task.status);
    
    // Assign the task
    client.assign_task(&task_id, "example-agent", AgentType::System).await?;
    println!("Assigned task to example-agent");
    
    // Update progress
    for i in 1..=4 {
        let progress = i * 20;
        let message = format!("Processing step {} of 5", i);
        
        client.report_progress(
            &task_id,
            progress,
            Some(&message),
            Some(json!({"processed_items": i * 10})),
        ).await?;
        
        println!("Updated progress to {}%: {}", progress, message);
        time::sleep(Duration::from_millis(500)).await;
    }
    
    // Complete the task
    let output_data = json!({
        "result": "success",
        "items_processed": 50,
        "execution_time_ms": 2533
    });
    
    client.complete_task(
        &task_id,
        Some(output_data),
        Some(json!({"completion_flag": true})),
    ).await?;
    
    println!("Completed task successfully");
    
    // Retrieve the completed task
    let completed_task = client.get_task(&task_id).await?;
    println!("Final task status: {:?}", completed_task.status);
    
    // List tasks in the context
    let tasks = client.list_tasks(
        None,
        None,
        None,
        Some(context_id),
        None,
        None,
    ).await?;
    
    println!("Found {} tasks in context", tasks.len());
    for (i, task) in tasks.iter().enumerate() {
        println!("{}. {} (status: {:?})", i+1, task.name, task.status);
    }
    
    println!("Example completed successfully");
    Ok(())
} 