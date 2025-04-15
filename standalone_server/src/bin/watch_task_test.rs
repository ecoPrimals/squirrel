use anyhow::Result;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use tokio_stream::StreamExt;
use taskserver_standalone::mcp_task;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "http://[::1]:50052";
    println!("Running WatchTask test against server at {}", addr);
    
    let mut client = taskserver_standalone::client::connect(addr).await?;
    
    // Create a test task
    println!("Creating a test task...");
    let test_id = Uuid::new_v4().to_string();
    let request = Request::new(mcp_task::CreateTaskRequest {
        name: format!("Watch Test Task {}", test_id),
        description: "Task for testing watch functionality".to_string(),
        priority: mcp_task::TaskPriority::Medium as i32,
        input_data: serde_json::to_vec(&serde_json::json!({
            "test": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))?,
        metadata: serde_json::to_vec(&serde_json::json!({
            "test": "watch_task",
        }))?,
        prerequisite_task_ids: vec![],
        context_id: "watch-test".to_string(),
        agent_id: "test-agent".to_string(),
        agent_type: mcp_task::AgentType::LocalPython as i32,
    });
    
    let response = client.create_task(request).await?;
    let task_id = response.into_inner().task_id;
    println!("Created task with ID: {}", task_id);
    
    // Start watching the task in a separate task
    let watch_task_id = task_id.clone();
    let watch_handle = tokio::spawn(async move {
        let mut watch_client = taskserver_standalone::client::connect(addr).await.unwrap();
        
        let request = Request::new(mcp_task::WatchTaskRequest {
            task_id: watch_task_id,
            include_initial_state: true,
            timeout_seconds: 30,
        });
        
        let mut stream = watch_client.watch_task(request).await.unwrap().into_inner();
        
        let mut update_count = 0;
        while let Some(update) = stream.message().await.unwrap() {
            update_count += 1;
            
            if let Some(task) = update.task {
                println!(
                    "Task update {}: Status={}, Progress={}%, Message='{}'{}",
                    update_count,
                    task.status,
                    task.progress_percent,
                    task.progress_message,
                    if update.is_initial_state { " (Initial state)" } else { "" },
                );
                
                // If task is completed, we're done
                if task.status == 4 {
                    println!("Task completed, ending watch");
                    break;
                }
            } else {
                println!("Received update without task: {}", update.error_message);
                break;
            }
        }
        
        println!("Watch stream ended. Received {} updates", update_count);
    });
    
    // Wait a moment for the watcher to start
    sleep(Duration::from_secs(1)).await;
    
    // Assign the task
    println!("Assigning task to test agent...");
    let request = Request::new(mcp_task::AssignTaskRequest {
        task_id: task_id.clone(),
        agent_id: "test-agent".to_string(),
        agent_type: mcp_task::AgentType::LocalPython as i32,
    });
    client.assign_task(request).await?;
    
    // Wait a bit
    sleep(Duration::from_secs(1)).await;
    
    // Update task progress
    println!("Reporting progress...");
    let request = Request::new(mcp_task::ReportProgressRequest {
        task_id: task_id.clone(),
        progress_percent: 25,
        progress_message: "Starting work".to_string(),
        interim_results: vec![],
    });
    client.report_progress(request).await?;
    
    // Wait a bit
    sleep(Duration::from_secs(2)).await;
    
    // Update more progress
    println!("Reporting more progress...");
    let request = Request::new(mcp_task::ReportProgressRequest {
        task_id: task_id.clone(),
        progress_percent: 75,
        progress_message: "Almost done".to_string(),
        interim_results: vec![],
    });
    client.report_progress(request).await?;
    
    // Wait a bit
    sleep(Duration::from_secs(2)).await;
    
    // Complete the task
    println!("Completing the task...");
    let request = Request::new(mcp_task::CompleteTaskRequest {
        task_id: task_id.clone(),
        output_data: serde_json::to_vec(&serde_json::json!({
            "result": "success",
            "completion_time": chrono::Utc::now().to_rfc3339(),
        }))?,
        metadata: vec![],
    });
    client.complete_task(request).await?;
    
    // Wait for the watcher to finish
    watch_handle.await?;
    
    println!("WatchTask test completed successfully!");
    Ok(())
} 