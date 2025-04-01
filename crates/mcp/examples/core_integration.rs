use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// A simplified core integration example
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting simplified core integration example");
    
    // Create protocol adapter - not used in this simplified example
    let _protocol_adapter = squirrel_mcp::protocol::create_protocol_adapter();
    info!("Created protocol adapter successfully");
    
    // Create a logger - not used directly in this simplified example
    let _logger = squirrel_mcp::logging::Logger::new("core-integration-example");
    info!("Created logger successfully");
    
    // Note: We're avoiding creating the auth manager and metrics collector
    // as they might require complex setup and configuration
    
    info!("Core integration example completed");
    Ok(())
}

/// Utility function to demonstrate unit testing
#[cfg(test)]
mod tests {
    use super::*;
    use squirrel_mcp::protocol::MockMCPProtocol;
    use squirrel_mcp::protocol::MCPResponse;
    use squirrel_mcp::protocol::Status;
    
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
            Arc::new(squirrel_mcp::security::SimpleAuthManager::new()),
            Arc::new(squirrel_mcp::metrics::MetricsCollector::new()),
            squirrel_mcp::logging::Logger::new(),
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