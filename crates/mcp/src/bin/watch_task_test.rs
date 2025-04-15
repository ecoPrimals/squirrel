use anyhow::Result;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use tokio_stream::StreamExt;
use tonic::Request;
use tonic::transport::Channel;
use serde_json::json;

// Import from the crate's generated module
use squirrel_mcp::generated::mcp_task::task_service_client::TaskServiceClient;
use squirrel_mcp::generated::mcp_task::{
    TaskStatus, TaskPriority, AgentType,
    CreateTaskRequest, GetTaskRequest, CompleteTaskRequest,
    ReportProgressRequest, WatchTaskRequest
};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "http://[::1]:50052";
    println!("Running WatchTask test against server at {}", addr);
    
    // Connect to the task service
    let channel = Channel::from_shared(addr.to_string())?
        .connect()
        .await?;
    
    let mut client = TaskServiceClient::new(channel);
    
    // Create a test task
    println!("Creating a test task...");
    let test_id = Uuid::new_v4().to_string();
    let request = Request::new(CreateTaskRequest {
        name: format!("Watch Test Task {}", test_id),
        description: "Task for testing watch functionality".to_string(),
        priority: TaskPriority::Medium as i32,
        input_data: serde_json::to_vec(&json!({
            "test": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))?,
        metadata: serde_json::to_vec(&json!({
            "test": "watch_task",
        }))?,
        prerequisite_task_ids: vec![],
        context_id: "watch-test".to_string(),
        agent_id: "test-agent".to_string(),
        agent_type: AgentType::LocalPython as i32,
    });
    
    let response = client.create_task(request).await?;
    let task_id = response.into_inner().task_id;
    println!("Created task with ID: {}", task_id);
    
    // Start watching the task in a separate task
    let watch_task_id = task_id.clone();
    let watch_handle = tokio::spawn(async move {
        // Create new client for the watcher
        let watch_channel = Channel::from_shared(addr.to_string())
            .expect("Invalid server address")
            .connect()
            .await
            .expect("Failed to connect to task service");
        
        let mut watch_client = TaskServiceClient::new(watch_channel);
        
        let request = Request::new(WatchTaskRequest {
            task_id: watch_task_id,
            include_initial_state: true,
            timeout_seconds: 30,
            only_watchable: false,
            filter_updates: false,
        });
        
        println!("Starting to watch task for updates...");
        
        match watch_client.watch_task(request).await {
            Ok(response) => {
                let mut stream = response.into_inner();
                let mut update_count = 0;
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(update) => {
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
                                if task.status == TaskStatus::Completed as i32 {
                                    println!("Task completed, ending watch");
                                    break;
                                }
                            } else {
                                println!("Received update without task: {}", update.error_message);
                                break;
                            }
                        },
                        Err(e) => {
                            println!("Error in watch stream: {}", e);
                            break;
                        }
                    }
                }
                
                println!("Watch stream ended. Received {} updates", update_count);
            },
            Err(e) => {
                eprintln!("Failed to watch task: {}", e);
            }
        }
    });
    
    // Wait a moment for the watcher to start
    sleep(Duration::from_millis(1500)).await;
    
    // Update task progress
    println!("Reporting progress...");
    let request = Request::new(ReportProgressRequest {
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
    let request = Request::new(ReportProgressRequest {
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
    let request = Request::new(CompleteTaskRequest {
        task_id: task_id.clone(),
        output_data: serde_json::to_vec(&json!({
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