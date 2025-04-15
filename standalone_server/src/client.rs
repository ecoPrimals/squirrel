use anyhow::Result;
use std::time::Duration;
use tonic::{transport::Channel, Request, Streaming};
use uuid::Uuid;

use crate::mcp_task;
use crate::mcp_task::{
    task_service_client::TaskServiceClient,
    CreateTaskRequest, TaskPriority, AgentType, WatchTaskRequest, WatchTaskResponse,
};

pub async fn connect(addr: &str) -> Result<TaskServiceClient<Channel>> {
    let channel = Channel::from_shared(addr.to_string())?
        .timeout(Duration::from_secs(5))
        .connect()
        .await?;
    
    Ok(TaskServiceClient::new(channel))
}

/// Watch a task for updates - returns a stream of task updates
pub async fn watch_task(
    client: &mut TaskServiceClient<Channel>, 
    task_id: &str, 
    include_initial_state: bool, 
    timeout_seconds: i32,
    only_watchable: bool,
    filter_updates: bool
) -> Result<Streaming<WatchTaskResponse>> {
    let request = Request::new(WatchTaskRequest {
        task_id: task_id.to_string(),
        include_initial_state,
        timeout_seconds,
        only_watchable,
        filter_updates,
    });
    
    let response = client.watch_task(request).await?;
    Ok(response.into_inner())
}

pub async fn run_test_client(addr: &str) -> Result<()> {
    println!("Connecting to task server at {}...", addr);
    
    let mut client = connect(addr).await?;
    
    // Create a task
    let test_id = Uuid::new_v4().to_string();
    let request = Request::new(CreateTaskRequest {
        name: format!("Test Task {}", test_id),
        description: "This is a test task created by the test client".to_string(),
        priority: TaskPriority::Medium as i32,
        input_data: serde_json::to_vec(&serde_json::json!({
            "test": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))?,
        metadata: serde_json::to_vec(&serde_json::json!({
            "client": "test_client",
            "version": "0.1.0",
        }))?,
        prerequisite_task_ids: vec![],
        context_id: "test-context".to_string(),
        agent_id: "".to_string(), // Let's not pre-assign
        agent_type: 0,
    });
    
    let response = client.create_task(request).await?;
    let task_id = response.into_inner().task_id;
    
    println!("Created task with ID: {}", task_id);
    
    // List tasks
    let request = Request::new(mcp_task::ListTasksRequest {
        status: 0, // Any status
        agent_id: "".to_string(),
        agent_type: 0,
        context_id: "".to_string(),
        limit: 10,
        offset: 0,
    });
    
    let response = client.list_tasks(request).await?;
    let tasks = response.into_inner().tasks;
    
    println!("Found {} tasks:", tasks.len());
    for task in tasks {
        println!("Task: {} - {}", task.id, task.name);
    }
    
    // Get specific task
    let request = Request::new(mcp_task::GetTaskRequest {
        task_id: task_id.clone(),
    });
    
    let response = client.get_task(request).await?;
    let task = response.into_inner().task.unwrap();
    
    println!("Retrieved task: {} - {}", task.id, task.name);
    println!("Current status: {}", task.status);
    
    // Assign the task first
    let request = Request::new(mcp_task::AssignTaskRequest {
        task_id: task_id.clone(),
        agent_id: "test-agent".to_string(),
        agent_type: AgentType::LocalPython as i32,
    });
    
    client.assign_task(request).await?;
    println!("Assigned task to test-agent");
    
    // Report progress
    let request = Request::new(mcp_task::ReportProgressRequest {
        task_id: task_id.clone(),
        progress_percent: 50,
        progress_message: "Task is halfway done".to_string(),
        interim_results: vec![],
    });
    
    client.report_progress(request).await?;
    println!("Updated task progress to 50%");
    
    // Complete task
    let request = Request::new(mcp_task::CompleteTaskRequest {
        task_id: task_id.clone(),
        output_data: serde_json::to_vec(&serde_json::json!({
            "result": "success",
            "completion_time": chrono::Utc::now().to_rfc3339(),
        }))?,
        metadata: vec![],
    });
    
    client.complete_task(request).await?;
    println!("Completed task");
    
    // Get task again to verify status
    let request = Request::new(mcp_task::GetTaskRequest {
        task_id,
    });
    
    let response = client.get_task(request).await?;
    let task = response.into_inner().task.unwrap();
    
    println!(
        "Final task state: {} - Status: {} - Progress: {}%",
        task.id, task.status, task.progress_percent
    );
    
    println!("Test client completed successfully!");
    Ok(())
} 