use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use mcp::integration::CoreMCPAdapter;
use mcp::protocol::{MCPProtocol, MCPMessage, MessageType};
use mcp::types::{CoreState, StateUpdate};
use mcp::error::MCPResult;
use tracing::info;

/// Example of using the CoreMCPAdapter to integrate a core system with MCP
#[tokio::main]
async fn main() -> MCPResult<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Starting Core-MCP integration example");
    
    // Set up MCP protocol instance
    let mcp_protocol = mcp::protocol::InMemoryMCPProtocol::new();
    
    // Create initial core state
    let core_state = CoreState {
        version: "1.0.0".to_string(),
        status: "initializing".to_string(),
        components: vec![
            "database".to_string(),
            "storage".to_string(),
            "network".to_string(),
        ],
        ..Default::default()
    };
    
    // Set up components needed for the adapter
    let core_state = Arc::new(RwLock::new(core_state));
    let mcp = Arc::new(mcp_protocol);
    let auth_manager = Arc::new(mcp::security::SimpleAuthManager::new());
    let metrics = Arc::new(mcp::metrics::MetricsCollector::new());
    let logger = mcp::logging::Logger::new();
    
    // Create the Core-MCP adapter
    let adapter = CoreMCPAdapter::new(
        core_state.clone(),
        mcp.clone(),
        auth_manager,
        metrics,
        logger,
    );
    
    // Initialize the adapter by registering message handlers
    adapter.initialize().await?;
    info!("Core-MCP adapter initialized");
    
    // Create a subscription to state notifications
    let mut state_subscription = mcp.subscribe(MessageType::StateNotification).await?;
    
    // Spawn a task to handle state notifications
    let notification_task = tokio::spawn(async move {
        info!("State notification listener started");
        while let Some(message) = state_subscription.next().await {
            info!("Received state notification: {:?}", message);
            if let Ok(update) = serde_json::from_value::<StateUpdate>(message.payload.clone()) {
                info!("State update type: {}", update.update_type);
            }
        }
    });
    
    // Simulate a state update
    {
        let mut state = core_state.write().await;
        state.status = "active".to_string();
        
        // Create a state update notification
        let update = StateUpdate {
            update_type: "status_change".to_string(),
            component: "core".to_string(),
            changes: serde_json::json!({
                "status": "active"
            }),
        };
        
        // Send notification through the adapter
        adapter.notify_state_update(update).await?;
    }
    
    // Create a query message to test state query handling
    let query_message = MCPMessage {
        id: "query-1".to_string(),
        message_type: MessageType::StateQuery,
        payload: serde_json::json!({}),
        metadata: Default::default(),
        timestamp: chrono::Utc::now(),
        version: "1.0".to_string(),
    };
    
    // Send the query message
    let response = mcp.send_message(query_message).await?;
    info!("State query response: {:?}", response);
    
    // Create a command message to reset the state
    let command_message = MCPMessage {
        id: "command-1".to_string(),
        message_type: MessageType::CoreCommand,
        payload: serde_json::json!({
            "command": "reset_state",
            "parameters": {}
        }),
        metadata: Default::default(),
        timestamp: chrono::Utc::now(),
        version: "1.0".to_string(),
    };
    
    // Send the command message
    let response = mcp.send_message(command_message).await?;
    info!("Command response: {:?}", response);
    
    // Sleep to allow notification to propagate
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Gracefully shut down notification task
    notification_task.abort();
    
    info!("Core-MCP integration example completed");
    Ok(())
}

/// Utility function to demonstrate unit testing
#[cfg(test)]
mod tests {
    use super::*;
    use mcp::protocol::MockMCPProtocol;
    use mcp::protocol::MCPResponse;
    use mcp::protocol::Status;
    
    #[tokio::test]
    async fn test_state_update() {
        // Set up mock and other components
        let mut mock_mcp = MockMCPProtocol::new();
        let core_state = Arc::new(RwLock::new(CoreState::default()));
        
        // Configure the mock to expect send_message
        mock_mcp.expect_send_message()
            .returning(|message| {
                Ok(MCPResponse {
                    id: message.id,
                    status: Status::Success,
                    payload: serde_json::json!({"result": "notification_sent"}),
                    error: None,
                    timestamp: chrono::Utc::now(),
                })
            });
        
        // Create adapter with mock MCP
        let adapter = CoreMCPAdapter::new(
            core_state,
            Arc::new(mock_mcp),
            Arc::new(mcp::security::SimpleAuthManager::new()),
            Arc::new(mcp::metrics::MetricsCollector::new()),
            mcp::logging::Logger::new(),
        );
        
        // Create a state update
        let update = StateUpdate {
            update_type: "test_update".to_string(),
            component: "test".to_string(),
            changes: serde_json::json!({"test": true}),
        };
        
        // Test the notification
        let result = adapter.notify_state_update(update).await;
        assert!(result.is_ok());
    }
} 