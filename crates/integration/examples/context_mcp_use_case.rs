use squirrel_integration::{
    ContextMcpAdapter,
    ContextMcpAdapterConfig,
    create_context_mcp_adapter_with_config,
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;
use tracing_subscriber::{fmt, EnvFilter};

/// Example demonstrating a real-world use case for Context-MCP integration
///
/// This example showcases:
/// 1. Creating contexts in the Squirrel Context system
/// 2. Automatic synchronization to MCP
/// 3. Handling context updates
/// 4. Status monitoring
/// 5. Manual synchronization
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .init();
    
    println!("=== Context-MCP Integration Use Case ===\n");
    
    // Create adapter with custom configuration
    let config = ContextMcpAdapterConfig {
        sync_interval_secs: 5,  // Set short interval for fast demo
        ..Default::default()
    };
    
    println!("Creating and initializing Context-MCP adapter...");
    let adapter = create_context_mcp_adapter_with_config(config).await?;
    adapter.initialize().await?;
    
    // Make adapter available to multiple tasks
    let adapter = Arc::new(adapter);
    
    // Clone for status monitoring task
    let status_adapter = adapter.clone();
    
    // Start status monitoring in background
    tokio::spawn(async move {
        loop {
            let status = status_adapter.get_status().await;
            println!("\nAdapter Status:");
            println!("  Connected to MCP: {}", status.connected_to_mcp);
            println!("  Connected to Context: {}", status.connected_to_context);
            println!("  Circuit Breaker: {}", status.circuit_breaker_state);
            println!("  Error Count: {}", status.error_count);
            println!("  Successful Syncs: {}", status.successful_syncs);
            println!("  Last Sync: {:?}", status.last_sync);
            
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
    
    // Get access to the Squirrel context manager for direct operations
    let context_manager = adapter.context_manager();
    
    // === Use Case 1: Create User Preferences Context ===
    println!("\n[Use Case 1] Creating user preferences context...");
    
    // Create a context with user preferences
    let user_prefs_id = "user_prefs_01";
    let user_prefs_data = json!({
        "theme": "dark",
        "fontSize": 14,
        "showNotifications": true,
        "language": "en-US"
    });
    
    println!("Creating context in Squirrel context system...");
    context_manager.create_context(
        user_prefs_id,
        "User Preferences",
        user_prefs_data.clone(),
        Some(json!({"lastUpdated": "2024-04-03T12:00:00Z"})),
    ).await?;
    
    // Wait for automatic sync to occur
    println!("Waiting for automatic sync to MCP...");
    tokio::time::sleep(Duration::from_secs(6)).await;
    
    // === Use Case 2: Update Context and Verify Sync ===
    println!("\n[Use Case 2] Updating context and verifying sync...");
    
    // Update the user preferences
    let updated_prefs = json!({
        "theme": "light",
        "fontSize": 16,
        "showNotifications": true,
        "language": "en-US"
    });
    
    println!("Updating context in Squirrel context system...");
    context_manager.update_context(
        user_prefs_id,
        updated_prefs.clone(),
        Some(json!({"lastUpdated": "2024-04-03T12:30:00Z"})),
    ).await?;
    
    // Wait for automatic sync
    println!("Waiting for automatic sync to MCP...");
    tokio::time::sleep(Duration::from_secs(6)).await;
    
    // === Use Case 3: Manual Sync ===
    println!("\n[Use Case 3] Performing manual sync...");
    
    // Create another context
    let app_state_id = "app_state_01";
    let app_state_data = json!({
        "currentView": "dashboard",
        "isLoggedIn": true,
        "lastActivity": "2024-04-03T12:35:00Z",
        "activeSessions": 1
    });
    
    println!("Creating app state context...");
    context_manager.create_context(
        app_state_id,
        "Application State",
        app_state_data.clone(),
        None,
    ).await?;
    
    // Perform manual sync instead of waiting
    println!("Performing immediate manual sync...");
    match adapter.sync_all().await {
        Ok(()) => println!("Manual sync completed successfully"),
        Err(e) => println!("Manual sync failed: {}", e),
    }
    
    // === Use Case 4: Retrieve Context ===
    println!("\n[Use Case 4] Retrieving context data...");
    
    // Retrieve the user preferences context
    match context_manager.with_context(user_prefs_id).await {
        Ok(context) => {
            println!("Retrieved user preferences context:");
            println!("  Name: {}", context.name);
            println!("  Data: {}", context.data);
            println!("  Metadata: {:?}", context.metadata);
        },
        Err(e) => println!("Failed to retrieve context: {}", e),
    }
    
    // === Use Case 5: Context Deletion ===
    println!("\n[Use Case 5] Deleting context...");
    
    // Delete the app state context
    match context_manager.delete_context(app_state_id).await {
        Ok(()) => println!("Deleted app state context successfully"),
        Err(e) => println!("Failed to delete context: {}", e),
    }
    
    // Wait for automatic sync
    println!("Waiting for automatic sync to MCP...");
    tokio::time::sleep(Duration::from_secs(6)).await;
    
    println!("\nContext-MCP integration use case completed!");
    println!("Press Ctrl+C to exit");
    
    // Keep the main task running
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
} 