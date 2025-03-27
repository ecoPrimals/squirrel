use serde_json::json;
use squirrel_monitoring::websocket::WebSocketConfig;
use crate::mock_websocket::MockWebSocketServer;

mod mock_websocket;

#[tokio::test]
async fn test_mock_websocket() {
    // Create a mock WebSocket server
    let config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 8888,
        ..Default::default()
    };
    
    let server = MockWebSocketServer::new(config);
    
    // Test start
    assert!(server.start().await.is_ok());
    
    // Add some test data
    let test_data = json!({
        "value": 42,
        "status": "ok",
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    server.update_component_data("test_component", test_data.clone()).await
        .expect("Failed to update component data");
    
    // Simulate connections
    server.add_connection().await;
    server.add_connection().await;
    
    // Test component listing
    let components = server.get_available_components().await.expect("Failed to get components");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0], "test_component");
    
    // Test getting component data
    let data = server.get_component_data("test_component").await.expect("Failed to get component data");
    assert_eq!(data["value"].as_i64().unwrap(), 42);
    assert_eq!(data["status"].as_str().unwrap(), "ok");
    
    // Test health status
    let health = server.get_health_status().await.expect("Failed to get health status");
    assert_eq!(health["running"].as_bool().unwrap(), true);
    assert_eq!(health["connection_count"].as_i64().unwrap(), 2);
    
    // Test removing connections
    server.remove_connection().await;
    let updated_health = server.get_health_status().await.expect("Failed to get updated health status");
    assert_eq!(updated_health["connection_count"].as_i64().unwrap(), 1);
    
    // Test stopping the server
    assert!(server.stop().await.is_ok());
    
    // Verify server status after stopping
    let final_health = server.get_health_status().await.expect("Failed to get final health status");
    assert_eq!(final_health["running"].as_bool().unwrap(), false);
} 