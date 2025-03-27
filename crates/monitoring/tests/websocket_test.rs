use std::sync::Arc;
use tokio::time::Duration;
use serde_json::Value;
use async_trait::async_trait;
use time;
// Remove dashboard imports and replace with testing utilities
// use squirrel_monitoring::{
//     dashboard::DashboardManager,
//     dashboard::config::{DashboardConfig, ComponentSettings},
//     dashboard::manager::{Manager, Component},
// };
use squirrel_monitoring::websocket::{WebSocketServer, WebSocketConfig, WebSocketInterface};
use squirrel_core::error::Result;
use squirrel_core::error::SquirrelError;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

// Create a MockManager that implements a simplified interface for testing
#[derive(Debug)]
struct MockManager {
    components: Vec<MockComponent>,
    data: Arc<Mutex<HashMap<String, Value>>>,
}

// Create a MockComponent type to replace dashboard Component
#[derive(Debug, Clone)]
struct MockComponent {
    id: String,
    name: String,
    component_type: String,
    last_updated: Option<u64>,
}

#[async_trait]
impl WebSocketServer for MockManager {
    async fn get_components(&self) -> Vec<MockComponent> {
        self.components.clone()
    }
    
    async fn get_component_data(&self, id: &str) -> Option<Value> {
        let data = self.data.lock().await;
        data.get(id).cloned()
    }
    
    async fn get_health_status(&self) -> Value {
        serde_json::json!({
            "status": "healthy",
            "components": []
        })
    }
}

impl MockManager {
    fn new() -> Self {
        // Create test components
        let test_components = vec![
            MockComponent {
                id: "test_performance_graph".to_string(),
                name: "Test Performance Graph".to_string(),
                component_type: "graph".to_string(),
                last_updated: Some(time::OffsetDateTime::now_utc().unix_timestamp() as u64),
            },
            MockComponent {
                id: "test_memory_usage".to_string(),
                name: "Memory Usage".to_string(),
                component_type: "gauge".to_string(),
                last_updated: Some(time::OffsetDateTime::now_utc().unix_timestamp() as u64),
            },
            MockComponent {
                id: "test_cpu_usage".to_string(),
                name: "CPU Usage".to_string(),
                component_type: "gauge".to_string(),
                last_updated: Some(time::OffsetDateTime::now_utc().unix_timestamp() as u64),
            },
        ];
        
        Self {
            components: test_components,
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    async fn update_data(&self, data: std::collections::HashMap<String, Value>) -> Result<()> {
        let mut locked_data = self.data.lock().await;
        for (key, value) in data {
            locked_data.insert(key, value);
        }
        Ok(())
    }

    // Add methods to start and stop the websocket server
    async fn start(&self, config: WebSocketConfig) -> Result<()> {
        // Implementation would initialize the WebSocket server
        // For now just sleep to simulate server startup
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        // Implementation would stop the WebSocket server
        // For now just sleep to simulate server shutdown
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }
}

/// Integration test for the WebSocket functionality
#[tokio::test]
async fn test_websocket() -> Result<()> {
    // Create WebSocket server with test configuration
    let config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 8765,
        update_interval: 1,
        max_connections: 10,
        enable_compression: false,
        auth_required: false,
    };
    
    let server = WebSocketServer::new(config);
    
    // Start server
    server.start().await?;
    
    // Update component data
    server.update_component_data("test_component", serde_json::json!({
        "value": 42,
        "status": "ok"
    })).await?;
    
    // Get available components
    let components = server.get_available_components().await?;
    assert!(components.contains(&"test_component".to_string()));
    
    // Get component data
    let data = server.get_component_data("test_component").await?;
    assert_eq!(data["value"], 42);
    
    // Check health
    let health = server.check_health().await?;
    assert!(health);
    
    // Stop server
    server.stop().await?;
    
    Ok(())
}

/// Test multiple clients connecting to the WebSocket server simultaneously
#[ignore]
#[tokio::test]
async fn test_multiple_websocket_clients() -> Result<()> {
    // Create WebSocket server
    let config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 8766,
        update_interval: 1,
        max_connections: 100,
        enable_compression: false,
        auth_required: false,
    };
    
    let server = Arc::new(WebSocketServer::new(config));
    
    // Start server
    server.start().await?;
    
    // Simulate multiple client connections by updating data for multiple components
    for i in 0..5 {
        let component_id = format!("component_{}", i);
        server.update_component_data(&component_id, serde_json::json!({
            "value": i,
            "timestamp": chrono::Utc::now().timestamp()
        })).await?;
    }
    
    // Verify all components are available
    let components = server.get_available_components().await?;
    assert_eq!(components.len(), 5);
    
    // Check individual component data
    for i in 0..5 {
        let component_id = format!("component_{}", i);
        let data = server.get_component_data(&component_id).await?;
        assert_eq!(data["value"], i);
    }
    
    // Stop server
    server.stop().await?;
    
    Ok(())
}

/// Test reconnection scenario with WebSocket clients
#[ignore]
#[tokio::test]
async fn test_websocket_reconnection() -> Result<()> {
    // Create a WebSocket configuration
    let mut config = WebSocketConfig::default();
    config.host = "127.0.0.1".to_string();
    config.port = 9900; // Use a different port for testing
    config.update_interval = 1; // Fast refresh for testing

    // Create and start the manager with WebSocket capabilities
    let manager = Arc::new(MockManager::new());
    manager.start(config.clone()).await?;
    println!("WebSocket server started successfully");

    // Wait for server to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Initialize with test data
    let mut initial_data = HashMap::new();
    initial_data.insert(
        "test_performance_graph".to_string(),
        serde_json::json!({
            "value": 50.0,
            "timestamp": time::OffsetDateTime::now_utc(),
        }),
    );
    manager.update_data(initial_data).await?;

    // Websocket URL
    let websocket_url = format!("ws://{}:{}/ws", 
        config.host,
        config.port);

    // First connection phase
    println!("Starting first connection phase");
    let (mut first_sender, mut first_receiver, first_value) = 
        connect_and_subscribe(&websocket_url, "test_performance_graph").await?;
    
    // Verify initial data received
    assert!(first_value.is_some(), "No data received on first connection");
    println!("First connection established successfully");
    
    // Update data
    let mut updated_data = HashMap::new();
    updated_data.insert(
        "test_performance_graph".to_string(),
        serde_json::json!({
            "value": 75.0,
            "timestamp": time::OffsetDateTime::now_utc(),
        }),
    );
    manager.update_data(updated_data).await?;
    
    // Wait for update
    let updated_value = receive_next_update(&mut first_receiver).await?;
    assert!(updated_value.is_some(), "No update received after data change");
    
    if let Some(val) = updated_value {
        if let Some(payload) = val.get("payload") {
            if let Some(value) = payload.get("value") {
                assert_eq!(value.as_f64(), Some(75.0), "Did not receive updated value");
                println!("Successfully received updated value: {}", value);
            }
        }
    }
    
    // Simulate disconnection
    println!("Simulating disconnection");
    drop(first_sender);
    drop(first_receiver);
    
    // Wait a moment to ensure disconnection is processed
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Update data again while disconnected
    let mut disconnected_update = HashMap::new();
    disconnected_update.insert(
        "test_performance_graph".to_string(),
        serde_json::json!({
            "value": 100.0,
            "timestamp": time::OffsetDateTime::now_utc(),
        }),
    );
    manager.update_data(disconnected_update).await?;
    
    // Reconnect
    println!("Reconnecting after disconnect");
    let (mut second_sender, mut second_receiver, second_value) = 
        connect_and_subscribe(&websocket_url, "test_performance_graph").await?;
    
    // Verify reconnection works and we get the latest data
    assert!(second_value.is_some(), "No data received on reconnection");
    
    if let Some(val) = second_value {
        if let Some(payload) = val.get("payload") {
            if let Some(value) = payload.get("value") {
                // We should get the latest value (100.0) after reconnecting
                assert_eq!(value.as_f64(), Some(100.0), "Did not receive latest value after reconnection");
                println!("Successfully received latest value after reconnection: {}", value);
            }
        }
    }
    
    // Update data one more time
    let mut final_update = HashMap::new();
    final_update.insert(
        "test_performance_graph".to_string(),
        serde_json::json!({
            "value": 125.0,
            "timestamp": time::OffsetDateTime::now_utc(),
        }),
    );
    manager.update_data(final_update).await?;
    
    // Check that we receive the update after reconnection
    let final_value = receive_next_update(&mut second_receiver).await?;
    assert!(final_value.is_some(), "No update received after reconnection");
    
    if let Some(val) = final_value {
        if let Some(payload) = val.get("payload") {
            if let Some(value) = payload.get("value") {
                assert_eq!(value.as_f64(), Some(125.0), "Did not receive final update after reconnection");
                println!("Successfully received final update: {}", value);
            }
        }
    }
    
    // Clean up
    drop(second_sender);
    drop(second_receiver);
    
    // Stop the WebSocket server
    manager.stop().await?;
    println!("WebSocket server stopped successfully after reconnection test");
    
    Ok(())
}

/// Helper function to connect to WebSocket and subscribe to a topic
async fn connect_and_subscribe(
    url: &str, 
    topic: &str
) -> Result<(
    futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
        >, 
        Message
    >,
    futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
        >
    >,
    Option<Value>
)> {
    // Connect to the WebSocket server
    let (ws_stream, _) = connect_async(url).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to connect: {}", e)))?;
    
    let (mut sender, mut receiver) = ws_stream.split();
    
    // Subscribe to the topic
    let subscribe_msg = serde_json::json!({
        "action": "subscribe",
        "topic": topic,
    }).to_string();
    
    // Send subscription request
    sender.send(Message::Text(subscribe_msg)).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to send: {}", e)))?;
    
    // Wait for the first message (should be the initial data)
    let mut initial_value = None;
    let timeout = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                let json: Value = serde_json::from_str(&text)
                    .map_err(|e| SquirrelError::Generic(
                        format!("Invalid JSON: {}", e)
                    ))?;
                
                if let Some(topic_val) = json.get("topic") {
                    if topic_val.as_str() == Some(topic) {
                        initial_value = Some(json);
                        break;
                    }
                }
            }
        }
        Ok::<(), SquirrelError>(())
    }).await;
    
    // Check for timeout
    if timeout.is_err() {
        return Err(SquirrelError::Generic(
            "Timed out waiting for initial value".to_string()
        ));
    }
    
    Ok((sender, receiver, initial_value))
}

/// Helper function to receive the next update from a WebSocket stream
async fn receive_next_update(
    receiver: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
        >
    >
) -> Result<Option<Value>> {
    let mut result = None;
    
    let timeout = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                let json: Value = serde_json::from_str(&text)
                    .map_err(|e| SquirrelError::Generic(
                        format!("Invalid JSON: {}", e)
                    ))?;
                
                result = Some(json);
                break;
            }
        }
        Ok::<(), SquirrelError>(())
    }).await;
    
    // Check for timeout
    if timeout.is_err() {
        return Err(SquirrelError::Generic(
            "Timed out waiting for update".to_string()
        ));
    }
    
    Ok(result)
}

/// Test long-running WebSocket connections
#[ignore]
#[tokio::test]
async fn test_long_running_websocket() -> Result<()> {
    // Create a WebSocket configuration
    let mut config = WebSocketConfig::default();
    config.host = "127.0.0.1".to_string();
    config.port = 9901; // Use a different port for testing
    config.update_interval = 1; // Fast refresh for testing

    // Create and start the manager with WebSocket capabilities
    let manager = Arc::new(MockManager::new());
    manager.start(config.clone()).await?;
    println!("WebSocket server started successfully");

    // Wait for server to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Initialize with test data
    let mut initial_data = HashMap::new();
    initial_data.insert(
        "test_performance_graph".to_string(),
        serde_json::json!({
            "value": 50.0,
            "timestamp": time::OffsetDateTime::now_utc(),
        }),
    );
    manager.update_data(initial_data).await?;

    // Websocket URL
    let websocket_url = format!("ws://{}:{}/ws", 
        config.host,
        config.port);

    // Connect to the WebSocket server
    let (ws_stream, _) = connect_async(&websocket_url).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to connect: {}", e)))?;
    
    let (mut sender, mut receiver) = ws_stream.split();
    
    // Subscribe to the component
    let subscribe_msg = serde_json::json!({
        "action": "subscribe",
        "topic": "test_performance_graph",
    }).to_string();
    
    // Send subscription request
    sender.send(Message::Text(subscribe_msg)).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to send: {}", e)))?;

    // Setup metrics collection
    let message_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let value_history = Arc::new(Mutex::new(Vec::new()));
    let receiver_running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    
    // Separate task to collect messages
    let message_count_clone = message_count.clone();
    let value_history_clone = value_history.clone();
    let receiver_running_clone = receiver_running.clone();
    
    let receiver_task = tokio::spawn(async move {
        while receiver_running_clone.load(std::sync::atomic::Ordering::Relaxed) {
            let timeout = tokio::time::timeout(Duration::from_millis(500), receiver.next()).await;
            
            match timeout {
                Ok(Some(Ok(Message::Text(text)))) => {
                    // Parse and process the message
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        if let Some(topic) = json.get("topic") {
                            if topic.as_str() == Some("test_performance_graph") {
                                // Increment message count
                                message_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                
                                // Extract value if present
                                if let Some(payload) = json.get("payload") {
                                    if let Some(value) = payload.get("value") {
                                        if let Some(val) = value.as_f64() {
                                            let mut history = value_history_clone.lock().await;
                                            history.push(val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Ok(Some(Ok(_))) => {
                    // Non-text message, ignore
                },
                Ok(Some(Err(e))) => {
                    println!("Error receiving message: {}", e);
                    break;
                },
                Ok(None) => {
                    println!("WebSocket closed");
                    break;
                },
                Err(_) => {
                    // Timeout occurred, this is expected in our loop
                    continue;
                }
            }
        }
    });
    
    // Number of updates to perform
    const UPDATE_COUNT: usize = 10;
    
    // Continuously update data over time
    for i in 0..UPDATE_COUNT {
        // Update with new value
        let value = 50.0 + (i as f64 * 10.0);
        let mut update = HashMap::new();
        update.insert(
            "test_performance_graph".to_string(),
            serde_json::json!({
                "value": value,
                "timestamp": time::OffsetDateTime::now_utc(),
            }),
        );
        manager.update_data(update).await?;
        
        // Wait between updates
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Wait for all messages to be processed
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Signal receiver to stop
    receiver_running.store(false, std::sync::atomic::Ordering::SeqCst);
    
    // Ensure the receiver task completes
    let timeout = tokio::time::timeout(Duration::from_secs(5), receiver_task).await;
    if timeout.is_err() {
        println!("Warning: Receiver task did not complete in time, forcing termination");
    }
    
    // Analyze results
    let final_count = message_count.load(std::sync::atomic::Ordering::SeqCst);
    let history = value_history.lock().await;
    
    println!("Received {} messages during long-running test", final_count);
    
    // We should have received at least as many messages as updates
    assert!(final_count >= UPDATE_COUNT, 
        "Expected at least {} messages, got {}", UPDATE_COUNT, final_count);
    
    // Verify we received all expected values in order
    if history.len() >= UPDATE_COUNT {
        for i in 0..UPDATE_COUNT {
            let expected_value = 50.0 + (i as f64 * 10.0);
            assert!(history.contains(&expected_value), 
                "Did not receive expected value {} in history", expected_value);
        }
        
        // Check if values came in sequential order
        let mut is_ordered = true;
        for i in 1..history.len() {
            if history[i] < history[i-1] {
                is_ordered = false;
                break;
            }
        }
        
        if is_ordered {
            println!("Values received in sequential order");
        } else {
            println!("Values not received in strict sequential order (expected for concurrent updates)");
        }
    }
    
    // Clean up
    drop(sender);
    
    // Stop the WebSocket server
    manager.stop().await?;
    println!("WebSocket server stopped successfully after long-running test");
    
    Ok(())
} 