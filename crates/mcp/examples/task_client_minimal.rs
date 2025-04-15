use anyhow::Result;
use tokio::time::{sleep, Duration};
use tonic::Request;
use tonic::transport::Channel;
use tokio_stream::StreamExt;
use serde_json::json;

// Import generated types - we'll need to use string paths since the mcp_task module may not be properly exported yet
use mcp_task::task_service_client::TaskServiceClient;
use mcp_task::{
    TaskStatus, TaskPriority, AgentType,
    CreateTaskRequest, GetTaskRequest, CompleteTaskRequest,
    ReportProgressRequest, WatchTaskRequest, WatchTaskResponse,
    Task as GenTask
};

// Use the raw generated module - this is a workaround until the proper module structure is in place
mod mcp_task {
    // Re-export everything from the generated code
    // This assumes the OUT_DIR is in the standard location
    include!(concat!(env!("OUT_DIR"), "/mcp.task.rs"));
}

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to the task service
    let channel = Channel::from_static("http://[::1]:50052")
        .connect()
        .await?;
    
    let mut client = TaskServiceClient::new(channel);
    println!("Connected to task service");
    
    // Create a task
    let input_data = json!({
        "key": "value",
        "number": 42
    }).to_string();
    
    let task = GenTask {
        id: "".to_string(),
        name: "Test Task".to_string(),
        description: "A test task created from the minimal client".to_string(),
        status_code: TaskStatus::Pending as i32,
        priority_code: TaskPriority::Medium as i32,
        agent_type: AgentType::Unspecified as i32,
        progress: 0.0,
        agent_id: None,
        context_id: Some("test-context".to_string()),
        parent_id: None,
        prerequisites: Vec::new(),
        created_at: None,
        updated_at: None,
        completed_at: None,
        input_data: Some(input_data),
        output_data: None,
        error_message: None,
        status_message: None,
        deadline: None,
        watchable: true,
        retry_count: 0,
        max_retries: 3,
    };
    
    let create_request = Request::new(CreateTaskRequest {
        task: Some(task),
    });
    
    let create_response = client.create_task(create_request).await?;
    let created_task = create_response.into_inner().task.unwrap();
    let task_id = created_task.id.clone();
    println!("Created task with ID: {}", task_id);
    
    // Start watching the task in a separate task 
    let watch_task_id = task_id.clone();
    let watch_handle = tokio::spawn(async move {
        // We need to create a new client for the watcher to avoid sharing issues
        let watch_channel = Channel::from_static("http://[::1]:50052")
            .connect()
            .await
            .expect("Failed to connect to task service");
        
        let mut watch_client = TaskServiceClient::new(watch_channel);
        
        // Create a watch request
        let watch_request = Request::new(WatchTaskRequest {
            task_id: watch_task_id,
            include_initial_state: true,
            timeout_seconds: 30,
            only_watchable: true,
            filter_updates: true,
        });
        
        // Start watching for task updates
        println!("Starting to watch task {}", watch_task_id);
        
        match watch_client.watch_task(watch_request).await {
            Ok(response) => {
                let mut stream = response.into_inner();
                let mut update_count = 0;
                
                // Process each update as it comes in
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(update) => {
                            update_count += 1;
                            if let Some(task) = update.task {
                                println!(
                                    "Task update {}: Status={}, Progress={}%, Message='{}'",
                                    update_count,
                                    task.status_code,
                                    task.progress,
                                    task.status_message.unwrap_or_default()
                                );
                                
                                // If task is completed, we're done
                                if task.status_code == TaskStatus::Completed as i32 {
                                    println!("Task completed, ending watch");
                                    break;
                                }
                            } else {
                                println!("Received update without task");
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
                println!("Failed to watch task: {}", e);
            }
        }
    });
    
    // Get the task
    let get_request = Request::new(GetTaskRequest {
        id: task_id.clone(),
    });
    
    let get_response = client.get_task(get_request).await?;
    let task = get_response.into_inner().task.unwrap();
    println!("Retrieved task: {} (status: {})", task.name, task.status_code);
    
    // Wait a moment to ensure watch is running
    sleep(Duration::from_secs(1)).await;
    
    // Update progress a few times
    println!("Reporting initial progress...");
    let progress_request = Request::new(ReportProgressRequest {
        task_id: task_id.clone(),
        progress: 25.0,
        status_message: Some("Starting work".to_string()),
    });
    client.report_progress(progress_request).await?;
    
    // Wait a bit
    sleep(Duration::from_secs(2)).await;
    
    // Update more progress
    println!("Reporting more progress...");
    let progress_request = Request::new(ReportProgressRequest {
        task_id: task_id.clone(),
        progress: 75.0,
        status_message: Some("Almost done".to_string()),
    });
    client.report_progress(progress_request).await?;
    
    // Wait a bit
    sleep(Duration::from_secs(2)).await;
    
    // Complete the task
    let output_data = json!({
        "result": "success",
        "completed": true
    }).to_string();
    
    let complete_request = Request::new(CompleteTaskRequest {
        task_id: task_id.clone(),
        output_data: Some(output_data),
    });
    
    let complete_response = client.complete_task(complete_request).await?;
    println!("Completed task");
    
    // Wait for the watch task to finish
    watch_handle.await?;
    
    // Get the task again to verify it's completed
    let get_request = Request::new(GetTaskRequest {
        id: task_id.clone(),
    });
    
    let get_response = client.get_task(get_request).await?;
    let task = get_response.into_inner().task.unwrap();
    println!("Final task status: {}", task.status_code);
    
    println!("Example completed successfully");
    Ok(())
} 