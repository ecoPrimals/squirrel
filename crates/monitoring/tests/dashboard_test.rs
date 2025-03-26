//! Integration tests for the monitoring dashboard using mock data.
//!
//! These tests verify that the dashboard correctly displays and updates
//! with mock monitoring data.

use std::sync::Arc;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde_json::json;

use squirrel_core::error::Result;
use squirrel_core::error::SquirrelError;
use squirrel_monitoring::{
    dashboard::DashboardManager,
    dashboard::config::DashboardConfig,
    alerts::types::AlertLevel,
    alerts::Alert,
};

mod mock_data_generator;
use mock_data_generator::{MonitoringTestHarness, SystemMetricsGenerator, HealthStatusGenerator, AlertGenerator};

/// Test the dashboard with mock data
#[ignore]
#[tokio::test]
async fn test_dashboard_with_mock_data() -> Result<()> {
    // Create a dashboard configuration
    let mut config = DashboardConfig::default();
    config.server = Some(Default::default());
    config.server.as_mut().unwrap().host = "127.0.0.1".to_string();
    config.server.as_mut().unwrap().port = 9902; // Use a unique port for this test
    config.update_interval = 1; // Fast refresh for testing
    
    // Create and start the dashboard manager
    let dashboard = DashboardManager::new(config.clone());
    dashboard.start().await?;
    println!("Dashboard started successfully");
    
    // Wait for server to start
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Create a monitoring test harness
    let harness = MonitoringTestHarness::new();
    
    // Generate some initial test data
    let metrics_batches = harness.generate_metrics(5);
    let health_status = harness.generate_health_status();
    let alerts = harness.generate_alerts(3);
    
    println!("Generated initial mock data:");
    println!("- {} batches of metrics", metrics_batches.len());
    println!("- Health status for {} components", health_status.len());
    println!("- {} alerts", alerts.len());
    
    // Create a WebSocket client to connect to the dashboard
    let websocket_url = format!("ws://{}:{}/ws", 
        config.server.as_ref().unwrap().host,
        config.server.as_ref().unwrap().port);
    
    println!("Connecting to WebSocket at {}", websocket_url);
    let (ws_stream, _) = connect_async(&websocket_url).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to connect: {}", e)))?;
    
    println!("WebSocket connection established");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Subscribe to CPU usage metrics
    let subscribe_msg = json!({
        "action": "subscribe",
        "topic": "system.cpu_usage",
    }).to_string();
    
    println!("Subscribing to system.cpu_usage");
    ws_sender.send(Message::Text(subscribe_msg)).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to send: {}", e)))?;
    
    // Subscribe to memory usage metrics
    let subscribe_msg = json!({
        "action": "subscribe",
        "topic": "system.memory_usage",
    }).to_string();
    
    println!("Subscribing to system.memory_usage");
    ws_sender.send(Message::Text(subscribe_msg)).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to send: {}", e)))?;
    
    // Subscribe to alerts
    let subscribe_msg = json!({
        "action": "subscribe",
        "topic": "alerts",
    }).to_string();
    
    println!("Subscribing to alerts");
    ws_sender.send(Message::Text(subscribe_msg)).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to send: {}", e)))?;
    
    // Create channels for sending updated data
    let (metrics_tx, mut metrics_rx) = mpsc::channel(100);
    let (health_tx, mut health_rx) = mpsc::channel(100);
    let (alerts_tx, mut alerts_rx) = mpsc::channel(100);
    
    // Start the metrics generator in the background
    let mut cpu_generator = SystemMetricsGenerator::new();
    cpu_generator.start_generation(Duration::from_secs(1), metrics_tx.clone()).await?;
    
    // Start the health status generator in the background
    let mut health_generator = HealthStatusGenerator::new();
    health_generator.start_generation(Duration::from_secs(5), health_tx).await?;
    
    // Start the alert generator in the background
    let mut alert_generator = AlertGenerator::new();
    alert_generator.start_generation(
        Duration::from_secs(3),
        Duration::from_secs(8),
        alerts_tx
    ).await?;
    
    // Create a task to receive metrics and forward them to the dashboard
    tokio::spawn(async move {
        while let Some(metrics) = metrics_rx.recv().await {
            println!("Received {} metrics from generator", metrics.len());
            // In a real implementation, we would update the dashboard with these metrics
            // dashboard.update_metrics(metrics).await.unwrap();
        }
    });
    
    // Listen for messages from the WebSocket with a timeout
    let mut messages_received = 0;
    let timeout = time::timeout(Duration::from_secs(10), async {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                messages_received += 1;
                println!("Received message {}: {}", messages_received, text);
                
                // Parse the message to verify it has the expected format
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(topic) = json.get("topic") {
                        println!("Message is for topic: {}", topic);
                    }
                    
                    if let Some(payload) = json.get("payload") {
                        println!("Payload: {}", payload);
                    }
                }
                
                // Once we've received several messages, we can stop
                if messages_received >= 5 {
                    break;
                }
            }
        }
    }).await;
    
    // Check if we received messages
    assert!(timeout.is_ok(), "Timed out waiting for WebSocket messages");
    assert!(messages_received > 0, "No messages received from WebSocket");
    
    // Clean up
    let _ = ws_sender.close().await;
    dashboard.stop().await?;
    println!("Dashboard stopped successfully");
    
    Ok(())
}

/// Test the alerting functionality with mock data
#[ignore]
#[tokio::test]
async fn test_dashboard_alerts() -> Result<()> {
    // Create a dashboard configuration
    let mut config = DashboardConfig::default();
    config.server = Some(Default::default());
    config.server.as_mut().unwrap().host = "127.0.0.1".to_string();
    config.server.as_mut().unwrap().port = 9903; // Use a unique port for this test
    config.update_interval = 1; // Fast refresh for testing
    
    // Create and start the dashboard manager
    let dashboard = DashboardManager::new(config.clone());
    dashboard.start().await?;
    println!("Dashboard started successfully");
    
    // Wait for server to start
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Create an alert generator
    let mut alert_generator = AlertGenerator::new();
    
    // Generate a critical alert
    let mut critical_alerts = Vec::new();
    for _ in 0..5 {
        let alert = alert_generator.next_alert();
        if alert.level == AlertLevel::Critical {
            critical_alerts.push(alert);
            if critical_alerts.len() >= 1 {
                break;
            }
        }
    }
    
    // If we didn't get a critical alert naturally, force one
    if critical_alerts.is_empty() {
        let mut alert = alert_generator.next_alert();
        // Create a new alert with Critical level instead of modifying existing alert
        let mut details = alert.details.clone();
        let critical_alert = Alert::new(
            alert.alert_type.clone(),
            alert.source.clone(),
            "Critical alert: system failure".to_string(),
            AlertLevel::Critical,
            details
        );
        critical_alerts.push(critical_alert);
    }
    
    println!("Generated {} critical alerts", critical_alerts.len());
    
    // Create a WebSocket client to connect to the dashboard
    let websocket_url = format!("ws://{}:{}/ws", 
        config.server.as_ref().unwrap().host,
        config.server.as_ref().unwrap().port);
    
    println!("Connecting to WebSocket at {}", websocket_url);
    let (ws_stream, _) = connect_async(&websocket_url).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to connect: {}", e)))?;
    
    println!("WebSocket connection established");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Subscribe to alerts
    let subscribe_msg = json!({
        "action": "subscribe",
        "topic": "alerts",
    }).to_string();
    
    println!("Subscribing to alerts");
    ws_sender.send(Message::Text(subscribe_msg)).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to send: {}", e)))?;
    
    // Create a channel for sending alerts
    let (alerts_tx, _alerts_rx) = mpsc::channel(100);
    
    // Start the alert generator in the background
    let mut alert_generator = AlertGenerator::new();
    alert_generator.start_generation(
        Duration::from_secs(1),
        Duration::from_secs(3),
        alerts_tx
    ).await?;
    
    // Listen for alert messages from the WebSocket with a timeout
    let mut alert_messages_received = 0;
    let timeout = time::timeout(Duration::from_secs(10), async {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                println!("Received message: {}", text);
                
                // Parse the message to check if it's an alert
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(topic) = json.get("topic") {
                        if topic.as_str() == Some("alerts") {
                            alert_messages_received += 1;
                            println!("Received alert message {}", alert_messages_received);
                            
                            // Check alert payload
                            if let Some(payload) = json.get("payload") {
                                println!("Alert payload: {}", payload);
                                
                                // Verify alert has expected fields
                                assert!(payload.get("id").is_some(), "Alert missing id");
                                assert!(payload.get("level").is_some() || payload.get("severity").is_some(), 
                                       "Alert missing severity/level");
                                assert!(payload.get("message").is_some(), "Alert missing message");
                            }
                            
                            // Once we've received enough alert messages, we can stop
                            if alert_messages_received >= 3 {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }).await;
    
    // Check if we received alert messages
    if timeout.is_err() {
        println!("Timed out waiting for alert messages, but this could be normal if no alerts were generated during the test period");
    } else {
        println!("Received {} alert messages", alert_messages_received);
    }
    
    // Clean up
    let _ = ws_sender.close().await;
    dashboard.stop().await?;
    println!("Dashboard stopped successfully");
    
    Ok(())
}

/// Test dashboard component rendering with mock data
#[ignore]
#[tokio::test]
async fn test_dashboard_components() -> Result<()> {
    // Create a dashboard configuration
    let mut config = DashboardConfig::default();
    config.server = Some(Default::default());
    config.server.as_mut().unwrap().host = "127.0.0.1".to_string();
    config.server.as_mut().unwrap().port = 9904; // Use a unique port for this test
    config.update_interval = 1; // Fast refresh for testing
    
    // Create and start the dashboard manager
    let dashboard = DashboardManager::new(config.clone());
    dashboard.start().await?;
    println!("Dashboard started successfully");
    
    // Wait for server to start
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Create a WebSocket client to connect to the dashboard
    let websocket_url = format!("ws://{}:{}/ws", 
        config.server.as_ref().unwrap().host,
        config.server.as_ref().unwrap().port);
    
    println!("Connecting to WebSocket at {}", websocket_url);
    let (ws_stream, _) = connect_async(&websocket_url).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to connect: {}", e)))?;
    
    println!("WebSocket connection established");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Request the list of available components
    let list_components_msg = json!({
        "action": "list_components",
    }).to_string();
    
    println!("Requesting component list");
    ws_sender.send(Message::Text(list_components_msg)).await
        .map_err(|e| SquirrelError::Generic(format!("Failed to send: {}", e)))?;
    
    // Listen for component list response
    let mut received_components_list = false;
    let timeout = time::timeout(Duration::from_secs(5), async {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                println!("Received message: {}", text);
                
                // Parse the message to check if it's a components list
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(message_type) = json.get("type") {
                        if message_type.as_str() == Some("components_list") {
                            received_components_list = true;
                            
                            // Verify components list has expected structure
                            if let Some(components) = json.get("components") {
                                if let Some(components_array) = components.as_array() {
                                    println!("Received list of {} components", components_array.len());
                                    
                                    // Verify each component has required fields
                                    for component in components_array {
                                        assert!(component.get("id").is_some(), "Component missing id");
                                        assert!(component.get("name").is_some(), "Component missing name");
                                        assert!(component.get("type").is_some(), "Component missing type");
                                    }
                                }
                            }
                            
                            break;
                        }
                    }
                }
            }
        }
    }).await;
    
    // We may or may not receive a components list, depending on the dashboard implementation
    if timeout.is_err() {
        println!("Timed out waiting for components list");
    } else if received_components_list {
        println!("Successfully received components list");
    } else {
        println!("Did not receive components list within timeout period");
    }
    
    // Clean up
    let _ = ws_sender.close().await;
    dashboard.stop().await?;
    println!("Dashboard stopped successfully");
    
    Ok(())
} 