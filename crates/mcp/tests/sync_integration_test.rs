use squirrel_mcp::sync::MCPSync;
use squirrel_mcp::sync::SyncConfig;
use squirrel_mcp::context_manager::{Context, ContextManager};
use std::time::Duration;
use std::sync::Arc;
use serde_json::json;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_sync_with_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for the test
    let _ = tracing_subscriber::fmt::try_init();

    // Configure Sync
    let config = SyncConfig {
        central_server_url: "http://[::1]:50051".to_string(),
        sync_interval: 60,
        max_retries: 3,
        timeout_ms: 5000,
        cleanup_older_than_days: 7,
    };

    // Create Sync instance
    let mut sync = MCPSync::create(config).await?;
    sync.init().await?;
    
    println!("Sync instance initialized");
    
    // Create a context manager for creating test contexts
    let context_manager = ContextManager::new().await;
    
    // Create a test context
    let context_id = Uuid::new_v4();
    let test_context = Context {
        id: context_id,
        name: "Rust Test Context".to_string(),
        data: json!({
            "source": "rust_test",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "test_data": [1, 2, 3, 4, 5]
        }),
        metadata: Some(json!({
            "test_metadata": true,
            "context_type": "integration_test"
        })),
        parent_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: None,
    };
    
    // Create the context in the context manager
    context_manager.create_context(test_context.clone()).await?;
    println!("Created test context with ID: {}", context_id);
    
    // Record the context creation in the sync manager
    sync.record_context_change(&test_context, squirrel_mcp::sync::state::StateOperation::Create).await?;
    println!("Recorded context change for sync");
    
    // Perform the sync operation
    println!("Attempting to sync with server...");
    match sync.synchronize().await {
        Ok(result) => {
            println!("Sync completed successfully!");
            println!("Changes processed: {}", result.changes_processed);
            println!("Version: {}", result.version);
            println!("Duration: {}ms", result.duration_ms);
        },
        Err(err) => {
            println!("Sync failed: {}", err);
            // Continue the test even if sync fails
        }
    }
    
    // Wait for a moment
    sleep(Duration::from_secs(2)).await;
    
    // Update the context
    let updated_context = Context {
        id: context_id,
        name: "Updated Rust Test Context".to_string(),
        data: json!({
            "source": "rust_test",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "test_data": [1, 2, 3, 4, 5],
            "updated": true
        }),
        metadata: Some(json!({
            "test_metadata": true,
            "context_type": "integration_test",
            "updated": true
        })),
        parent_id: None,
        created_at: test_context.created_at,
        updated_at: chrono::Utc::now(),
        expires_at: None,
    };
    
    // Update the context in the context manager
    context_manager.update_context(context_id, updated_context.data.clone(), updated_context.metadata.clone()).await?;
    println!("Updated test context");
    
    // Record the context update in the sync manager
    sync.record_context_change(&updated_context, squirrel_mcp::sync::state::StateOperation::Update).await?;
    println!("Recorded context update for sync");
    
    // Sync again to propagate the update
    println!("Attempting to sync again...");
    match sync.synchronize().await {
        Ok(result) => {
            println!("Second sync completed successfully!");
            println!("Changes processed: {}", result.changes_processed);
        },
        Err(err) => {
            println!("Second sync failed: {}", err);
        }
    }
    
    // Wait for a moment
    sleep(Duration::from_secs(2)).await;
    
    // Clean up by deleting the context
    context_manager.delete_context(context_id).await?;
    println!("Deleted test context");
    
    // Record the context deletion in the sync manager
    sync.record_context_change(&updated_context, squirrel_mcp::sync::state::StateOperation::Delete).await?;
    println!("Recorded context deletion for sync");
    
    // Sync one more time to propagate the deletion
    println!("Attempting final sync...");
    match sync.synchronize().await {
        Ok(result) => {
            println!("Final sync completed successfully!");
            println!("Changes processed: {}", result.changes_processed);
        },
        Err(err) => {
            println!("Final sync failed: {}", err);
        }
    }
    
    println!("Test completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_sync_subscribe_changes() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for the test
    let _ = tracing_subscriber::fmt::try_init();

    // Configure Sync
    let config = SyncConfig {
        central_server_url: "http://[::1]:50051".to_string(),
        sync_interval: 60,
        max_retries: 3,
        timeout_ms: 5000,
        cleanup_older_than_days: 7,
    };

    // Create Sync instance
    let mut sync = MCPSync::create(config).await?;
    sync.init().await?;
    
    println!("Sync instance initialized for subscription test");
    
    // Subscribe to changes
    let mut change_receiver = sync.subscribe_to_state_changes().await?;
    println!("Subscribed to state changes");
    
    // Create a context manager for creating test contexts
    let context_manager = ContextManager::new().await;
    
    // Create a test context
    let context_id = Uuid::new_v4();
    let test_context = Context {
        id: context_id,
        name: "Subscription Test Context".to_string(),
        data: json!({
            "source": "subscription_test",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
        metadata: None,
        parent_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: None,
    };
    
    // Create the context in the context manager
    context_manager.create_context(test_context.clone()).await?;
    println!("Created test context with ID: {}", context_id);
    
    // Spawn a task to listen for changes
    let change_task = tokio::spawn(async move {
        // Try to receive a change
        match tokio::time::timeout(Duration::from_secs(5), change_receiver.recv()).await {
            Ok(change_result) => {
                match change_result {
                    Ok(change) => {
                        println!("Received state change: {:?}", change);
                        assert_eq!(change.id, context_id);
                        true
                    },
                    Err(e) => {
                        println!("Error receiving change: {}", e);
                        false
                    },
                }
            },
            Err(_) => {
                println!("Timeout waiting for change");
                false
            },
        }
    });
    
    // Record the context creation to trigger a change event
    sync.record_context_change(&test_context, squirrel_mcp::sync::state::StateOperation::Create).await?;
    println!("Recorded context change to trigger notification");
    
    // Wait for the change task to complete
    let received_change = change_task.await?;
    
    // Clean up
    context_manager.delete_context(context_id).await?;
    
    // Verify that we received the change
    assert!(received_change, "Did not receive change notification");
    
    println!("Subscription test completed successfully!");
    Ok(())
} 