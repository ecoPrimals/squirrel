//! Integration tests for Context-MCP synchronization functionality
//!
//! These tests verify the bidirectional synchronization capabilities of the Context-MCP adapter.

use std::sync::Arc;
use tokio::time::Duration;

use squirrel_integration::{
    ContextMcpAdapter,
    ContextMcpAdapterConfig,
    create_context_mcp_adapter_with_config,
    SyncDirection,
};

use squirrel_mcp::{Context, McpConfig, MCPAdapter};
use serde_json::json;
use uuid::Uuid;

/// Creates a test context in MCP
async fn create_test_mcp_context(mcp_adapter: &Arc<MCPAdapter>, suffix: &str) -> (String, Context) {
    let context_id = format!("test-sync-{}", suffix);
    
    let test_context = Context {
        id: context_id.clone(),
        account_id: "test-account".to_string(),
        metadata: Some(json!({
            "test_data": {
                "name": format!("Test Sync {}", suffix),
                "created_at": chrono::Utc::now().to_rfc3339(),
                "tags": ["test", "sync"]
            }
        })),
        ..Default::default()
    };
    
    if let Err(e) = mcp_adapter.create_context(&test_context).await {
        panic!("Failed to create test context {}: {}", context_id, e);
    }
    
    (context_id, test_context)
}

/// Helper function to check if a context exists in MCP
async fn context_exists_in_mcp(mcp_adapter: &Arc<MCPAdapter>, context_id: &str) -> bool {
    mcp_adapter.get_context(context_id).await.is_ok()
}

/// Helper function to check if a context exists in Squirrel
async fn context_exists_in_squirrel(context_adapter: &ContextMcpAdapter, context_id: &str) -> bool {
    let squirrel_context_manager = context_adapter.context_manager();
    squirrel_context_manager.with_context(context_id).await.is_ok()
}

#[tokio::test]
async fn test_sync_mcp_to_squirrel() {
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create test context in MCP
    let (context_id, _) = create_test_mcp_context(&mcp_adapter, "m2s").await;
    
    // Create Context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig {
        sync_interval_secs: 0, // Disable automatic sync
        ..Default::default()
    };
    
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Initialize adapter
    if let Err(e) = context_adapter.initialize().await {
        panic!("Failed to initialize Context-MCP adapter: {}", e);
    }
    
    // The context should not exist in Squirrel yet
    assert!(!context_exists_in_squirrel(&context_adapter, &context_id).await, 
            "Context should not exist in Squirrel before sync");
    
    // Sync from MCP to Squirrel
    if let Err(e) = context_adapter.sync_direction(SyncDirection::McpToSquirrel).await {
        panic!("Failed to sync from MCP to Squirrel: {}", e);
    }
    
    // Wait for sync to complete
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // The context should now exist in Squirrel
    assert!(context_exists_in_squirrel(&context_adapter, &context_id).await, 
            "Context should exist in Squirrel after sync");
    
    // Clean up
    if let Err(e) = mcp_adapter.delete_context(&context_id).await {
        eprintln!("Warning: Failed to delete test context {}: {}", context_id, e);
    }
}

#[tokio::test]
async fn test_sync_squirrel_to_mcp() {
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create Context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig {
        sync_interval_secs: 0, // Disable automatic sync
        ..Default::default()
    };
    
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Initialize adapter
    if let Err(e) = context_adapter.initialize().await {
        panic!("Failed to initialize Context-MCP adapter: {}", e);
    }
    
    // Create a context in Squirrel
    let squirrel_id = format!("squirrel-{}", Uuid::new_v4());
    let squirrel_context_manager = context_adapter.context_manager();
    
    if let Err(e) = squirrel_context_manager.create_context(
        &squirrel_id,
        "Test Squirrel Context",
        json!({
            "name": "Test Context",
            "description": "Created in Squirrel"
        }),
        Some(json!({
            "tags": ["test", "squirrel"],
            "created_at": chrono::Utc::now().to_rfc3339()
        })),
    ).await {
        panic!("Failed to create context in Squirrel: {}", e);
    }
    
    // The context should not exist in MCP yet
    assert!(!context_exists_in_mcp(&mcp_adapter, &squirrel_id).await, 
            "Context should not exist in MCP before sync");
    
    // Sync from Squirrel to MCP
    if let Err(e) = context_adapter.sync_direction(SyncDirection::SquirrelToMcp).await {
        panic!("Failed to sync from Squirrel to MCP: {}", e);
    }
    
    // Wait for sync to complete
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // The context should now exist in MCP
    assert!(context_exists_in_mcp(&mcp_adapter, &squirrel_id).await, 
            "Context should exist in MCP after sync");
    
    // Clean up
    if let Err(e) = mcp_adapter.delete_context(&squirrel_id).await {
        eprintln!("Warning: Failed to delete test context {}: {}", squirrel_id, e);
    }
    
    if let Err(e) = squirrel_context_manager.delete_context(&squirrel_id).await {
        eprintln!("Warning: Failed to delete Squirrel context {}: {}", squirrel_id, e);
    }
}

#[tokio::test]
async fn test_bidirectional_sync() {
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create context in MCP
    let (mcp_context_id, _) = create_test_mcp_context(&mcp_adapter, "bidir-mcp").await;
    
    // Create Context-MCP adapter
    let context_mcp_config = ContextMcpAdapterConfig {
        sync_interval_secs: 0, // Disable automatic sync
        ..Default::default()
    };
    
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Initialize adapter
    if let Err(e) = context_adapter.initialize().await {
        panic!("Failed to initialize Context-MCP adapter: {}", e);
    }
    
    // Create a context in Squirrel
    let squirrel_id = format!("squirrel-bidir-{}", Uuid::new_v4());
    let squirrel_context_manager = context_adapter.context_manager();
    
    if let Err(e) = squirrel_context_manager.create_context(
        &squirrel_id,
        "Test Bidirectional Sync",
        json!({
            "name": "Squirrel Context",
            "description": "Created in Squirrel for bidirectional sync"
        }),
        Some(json!({
            "tags": ["test", "bidirectional"],
            "created_at": chrono::Utc::now().to_rfc3339()
        })),
    ).await {
        panic!("Failed to create context in Squirrel: {}", e);
    }
    
    // Sync bidirectionally
    if let Err(e) = context_adapter.sync_direction(SyncDirection::Bidirectional).await {
        panic!("Failed to sync bidirectionally: {}", e);
    }
    
    // Wait for sync to complete
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // The MCP context should now exist in Squirrel
    assert!(context_exists_in_squirrel(&context_adapter, &mcp_context_id).await, 
            "MCP context should exist in Squirrel after bidirectional sync");
    
    // The Squirrel context should now exist in MCP
    assert!(context_exists_in_mcp(&mcp_adapter, &squirrel_id).await, 
            "Squirrel context should exist in MCP after bidirectional sync");
    
    // Clean up
    if let Err(e) = mcp_adapter.delete_context(&mcp_context_id).await {
        eprintln!("Warning: Failed to delete MCP context {}: {}", mcp_context_id, e);
    }
    
    if let Err(e) = mcp_adapter.delete_context(&squirrel_id).await {
        eprintln!("Warning: Failed to delete Squirrel->MCP context {}: {}", squirrel_id, e);
    }
    
    if let Err(e) = squirrel_context_manager.delete_context(&squirrel_id).await {
        eprintln!("Warning: Failed to delete Squirrel context {}: {}", squirrel_id, e);
    }
    
    if let Err(e) = squirrel_context_manager.delete_context(&mcp_context_id).await {
        eprintln!("Warning: Failed to delete MCP->Squirrel context {}: {}", mcp_context_id, e);
    }
}

#[tokio::test]
async fn test_mcp_change_propagation() {
    // Create test MCP adapter
    let mcp_config = McpConfig::default();
    let mcp_adapter = Arc::new(MCPAdapter::new(mcp_config));
    if let Err(e) = mcp_adapter.initialize() {
        panic!("Failed to initialize MCP adapter: {}", e);
    }
    
    // Create Context-MCP adapter with automatic sync enabled
    let context_mcp_config = ContextMcpAdapterConfig {
        sync_interval_secs: 1, // Quick sync for testing
        ..Default::default()
    };
    
    let context_adapter = match create_context_mcp_adapter_with_config(context_mcp_config).await {
        Ok(adapter) => adapter,
        Err(e) => panic!("Failed to create Context-MCP adapter: {}", e),
    };
    
    // Initialize adapter (this will start subscribing to MCP changes)
    if let Err(e) = context_adapter.initialize().await {
        panic!("Failed to initialize Context-MCP adapter: {}", e);
    }
    
    // Create a context in MCP after subscription is active
    let (mcp_context_id, _) = create_test_mcp_context(&mcp_adapter, "change-prop").await;
    
    // Wait for change to propagate
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // The context should be synchronized to Squirrel automatically
    assert!(context_exists_in_squirrel(&context_adapter, &mcp_context_id).await, 
            "Context should be automatically synchronized to Squirrel");
    
    // Update the context in MCP
    let updated_data = json!({
        "name": "Updated Test Context",
        "description": "This context was updated in MCP"
    });
    
    if let Err(e) = mcp_adapter.update_context(
        mcp_context_id.clone(), 
        updated_data.clone(),
        None
    ).await {
        panic!("Failed to update context in MCP: {}", e);
    }
    
    // Wait for change to propagate
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Verify the update was synchronized to Squirrel
    let squirrel_context_manager = context_adapter.context_manager();
    match squirrel_context_manager.with_context(&mcp_context_id).await {
        Ok(context) => {
            assert_eq!(context.data.get("name").and_then(|v| v.as_str()), 
                      updated_data.get("name").and_then(|v| v.as_str()),
                      "Context name should be updated in Squirrel");
        },
        Err(e) => panic!("Failed to get context from Squirrel: {}", e),
    }
    
    // Clean up
    if let Err(e) = mcp_adapter.delete_context(&mcp_context_id).await {
        eprintln!("Warning: Failed to delete test context {}: {}", mcp_context_id, e);
    }
    
    // Wait for deletion to propagate
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // The context should be deleted from Squirrel automatically
    assert!(!context_exists_in_squirrel(&context_adapter, &mcp_context_id).await, 
            "Context should be automatically deleted from Squirrel");
} 