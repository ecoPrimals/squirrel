#[cfg(test)]
mod tests {
    use crate::task::{MCPTaskClient, TaskClientConfig};
    
    /// Basic test to ensure client can be initialized
    #[tokio::test]
    async fn test_client_init() {
        let client = MCPTaskClient::new();
        assert_eq!(client.server_address(), "http://localhost:50051");
        assert_eq!(client.max_retries(), 3);
    }
    
    /// Test with custom configuration
    #[tokio::test]
    async fn test_client_config() {
        let config = TaskClientConfig {
            server_address: "localhost:8080".to_string(),
            connect_timeout_ms: 15000,
            request_timeout_ms: 10000,
            max_retries: 5,
            initial_backoff_ms: 200,
            max_backoff_ms: 3000,
        };
        
        let client = MCPTaskClient::with_config(config.clone());
        
        assert_eq!(client.server_address(), "localhost:8080");
        assert_eq!(client.connect_timeout(), 15000);
        assert_eq!(client.request_timeout(), 10000);
        assert_eq!(client.max_retries(), 5);
        assert_eq!(client.initial_backoff(), 200);
        assert_eq!(client.max_backoff(), 3000);
    }
    
    // Note: The following tests require a running task service
    // and are disabled by default. To run them, start the task server
    // and uncomment the tests.
    
    /*
    /// Integration test for task creation and retrieval
    #[tokio::test]
    async fn test_create_and_get_task() -> Result<()> {
        let mut client = MCPTaskClient::new();
        client.connect().await?;
        
        let task_name = "Test Task";
        let task_id = client.create_task(
            task_name,
            "Test task description",
            TaskPriority::Medium,
            Some(json!({"test": true})),
            None,
            None,
            vec![],
        ).await?;
        
        assert!(!task_id.is_empty());
        
        let task = client.get_task(&task_id).await?;
        assert_eq!(task.name, task_name);
        
        Ok(())
    }
    
    /// Integration test for the task lifecycle
    #[tokio::test]
    async fn test_task_lifecycle() -> Result<()> {
        let mut client = MCPTaskClient::new();
        client.connect().await?;
        
        // Create a task
        let task_id = client.create_task(
            "Lifecycle Test",
            "Testing the complete task lifecycle",
            TaskPriority::High,
            Some(json!({"lifecycle": true})),
            None,
            None,
            vec![],
        ).await?;
        
        // Assign it
        client.assign_task(&task_id, "test-agent", AgentType::System).await?;
        
        // Update progress
        client.report_progress(&task_id, 50, Some("Halfway there")).await?;
        
        // Complete it
        client.complete_task(
            &task_id,
            Some(json!({"result": "success"})),
            None,
        ).await?;
        
        // Verify it's completed
        let task = client.get_task(&task_id).await?;
        assert_eq!(task.status, 4); // COMPLETED = 4
        
        Ok(())
    }
    */
} 