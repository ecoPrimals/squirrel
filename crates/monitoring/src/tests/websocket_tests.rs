use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use url::Url;
use serde_json::json;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use tracing::{error, debug};

use crate::dashboard::{server, Manager};

// Struct to hold test statistics
#[derive(Debug, Default)]
struct TestStatistics {
    connections_successful: usize,
    connection_failures: usize,
    connections_closed: usize,
    current_connections: usize,
    total_messages_received: usize,
    reconnection_tests: usize,
    messages_per_component: HashMap<String, usize>,
    client_stats: HashMap<String, ClientStats>,
}

#[derive(Debug, Default)]
struct ClientStats {
    connection_attempts: usize,
    messages_received: usize,
    components_subscribed: usize,
    reconnections_successful: usize,
}

// Trait to select multiple items from a vector
trait ChooseMultiple<T> {
    fn choose_multiple<R: Rng>(&self, rng: &mut R, amount: usize) -> Vec<&T>;
}

impl<T> ChooseMultiple<T> for Vec<T> {
    fn choose_multiple<R: Rng>(&self, rng: &mut R, amount: usize) -> Vec<&T> {
        let mut indices: Vec<usize> = (0..self.len()).collect();
        indices.shuffle(rng);
        indices.truncate(amount.min(self.len()));
        indices.into_iter().map(|i| &self[i]).collect()
    }
}

/// Sets up and runs a WebSocket server for testing
async fn setup_test_server() -> (u16, Arc<Manager>, JoinHandle<()>) {
    // Create a random port for testing
    let port = 3000 + thread_rng().gen_range(1000..5000);
    let addr = format!("127.0.0.1:{}", port).parse().unwrap();
    
    // Create a test dashboard manager
    let manager = Arc::new(Manager::new_with_default_config());
    
    // Start the server
    let server_manager = manager.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::start_server(server_manager, addr).await {
            error!("Server error: {}", e);
        }
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    (port, manager, server_handle)
}

/// Test a single WebSocket client connection
#[tokio::test]
#[ignore = "Requires full WebSocket server to be working"]
async fn test_single_client_connection() {
    // Setup test server
    let (port, _manager, server_handle) = setup_test_server().await;
    let url = Url::parse(&format!("ws://127.0.0.1:{}/ws", port)).unwrap();
    
    // Connect to the WebSocket server
    let connection_result = connect_async(url).await;
    assert!(connection_result.is_ok(), "Failed to connect to WebSocket server");
    
    let (ws_stream, _) = connection_result.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to a component
    let component_id = "system_cpu";
    let subscribe_msg = json!({
        "type": "subscribe",
        "componentId": component_id
    }).to_string();
    
    let send_result = write.send(Message::Text(subscribe_msg)).await;
    assert!(send_result.is_ok(), "Failed to send subscription message");
    
    // Wait for a message
    let received = tokio::time::timeout(Duration::from_secs(5), read.next()).await;
    assert!(received.is_ok(), "Timed out waiting for message");
    
    // Verify message was received
    let message = received.unwrap();
    assert!(message.is_some(), "No message received");
    assert!(message.unwrap().is_ok(), "Invalid message received");
    
    // Cleanup
    server_handle.abort();
}

/// Test multiple client connections with various subscription patterns
#[tokio::test]
#[ignore = "Requires full WebSocket server to be working"]
async fn test_multiple_clients() {
    // Setup test server
    let (port, _manager, server_handle) = setup_test_server().await;
    let base_url = format!("ws://127.0.0.1:{}/ws", port);
    
    // Test parameters
    let num_clients = 5;
    let components = vec![
        "system_cpu".to_string(), 
        "system_memory".to_string(), 
        "network_traffic".to_string(), 
        "disk_usage".to_string(),
        "health_status".to_string()
    ];
    
    // Statistics tracking
    let stats = Arc::new(Mutex::new(TestStatistics::default()));
    
    // Run multiple clients
    let client_handles = run_test_clients(
        &base_url,
        num_clients,
        components,
        Duration::from_secs(10),
        stats.clone()
    ).await;
    
    // Wait for test to complete
    tokio::time::sleep(Duration::from_secs(12)).await;
    
    // Check statistics
    let final_stats = stats.lock().await;
    assert_eq!(final_stats.connections_successful, num_clients, "Not all clients connected successfully");
    assert!(final_stats.total_messages_received > 0, "No messages were received");
    
    // Cleanup
    for handle in client_handles {
        handle.abort();
    }
    server_handle.abort();
}

/// Test client reconnection behavior
#[tokio::test]
#[ignore = "Requires full WebSocket server to be working"]
async fn test_client_reconnection() {
    // Setup test server
    let (port, _manager, server_handle) = setup_test_server().await;
    let url = Url::parse(&format!("ws://127.0.0.1:{}/ws", port)).unwrap();
    
    // Connect to the WebSocket server
    let connection_result = connect_async(url.clone()).await;
    assert!(connection_result.is_ok(), "Failed to connect to WebSocket server");
    
    // Close the connection
    let (ws_stream, _) = connection_result.unwrap();
    let (mut write, _) = ws_stream.split();
    write.close().await.expect("Failed to close connection");
    
    // Wait a moment
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Reconnect
    let reconnection_result = connect_async(url).await;
    assert!(reconnection_result.is_ok(), "Failed to reconnect to WebSocket server");
    
    // Cleanup
    server_handle.abort();
}

/// Helper function to run multiple test clients
async fn run_test_clients(
    base_url: &str,
    num_clients: usize,
    components: Vec<String>,
    test_duration: Duration,
    stats: Arc<Mutex<TestStatistics>>,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::with_capacity(num_clients);
    
    for client_id in 0..num_clients {
        let url = Url::parse(base_url).unwrap();
        let client_components = components.clone();
        let client_stats = stats.clone();
        let client_name = format!("test-client-{}", client_id);
        
        let handle = tokio::spawn(async move {
            let _ = run_test_client(
                url,
                &client_name,
                client_components,
                test_duration,
                client_stats
            ).await;
        });
        
        handles.push(handle);
        
        // Small delay between client starts
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    handles
}

/// Run an individual test client
async fn run_test_client(
    url: Url,
    client_name: &str,
    components: Vec<String>,
    test_duration: Duration,
    stats: Arc<Mutex<TestStatistics>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Client state
    let mut connection_attempts = 0;
    let received_messages = Arc::new(Mutex::new(0_usize));
    let mut subscriptions = HashMap::new();
    
    // Select random components to subscribe to (do random selection before the async part)
    let selected_components: Vec<String>;
    {
        let mut rng = thread_rng();
        let num_components = rng.gen_range(1..=components.len());
        selected_components = components
            .choose_multiple(&mut rng, num_components)
            .into_iter()
            .cloned()
            .collect();
    }
    
    debug!("{} subscribing to {} components", client_name, selected_components.len());
    
    let start_time = Instant::now();
    
    // Run client with reconnection logic
    while start_time.elapsed() < test_duration {
        connection_attempts += 1;
        
        debug!("{} connecting (attempt {})", client_name, connection_attempts);
        
        // Connect to the server
        match connect_async(url.clone()).await {
            Ok((ws_stream, _)) => {
                // Update statistics
                {
                    let mut stats = stats.lock().await;
                    stats.connections_successful += 1;
                    stats.current_connections += 1;
                }
                
                debug!("{} connected successfully", client_name);
                
                // Process the connection
                if let Err(e) = handle_test_connection(
                    ws_stream,
                    client_name,
                    &selected_components,
                    received_messages.clone(),
                    &mut subscriptions,
                    stats.clone()
                ).await {
                    debug!("{} connection error: {}", client_name, e);
                }
                
                // Update statistics after disconnection
                {
                    let mut stats = stats.lock().await;
                    stats.current_connections -= 1;
                    stats.connections_closed += 1;
                }
            },
            Err(e) => {
                // Update statistics
                {
                    let mut stats = stats.lock().await;
                    stats.connection_failures += 1;
                }
                
                debug!("{} connection failed: {}", client_name, e);
            }
        }
        
        // Random reconnection timeout (100-1000ms)
        let backoff = Duration::from_millis(thread_rng().gen_range(100..1000));
        tokio::time::sleep(backoff).await;
    }
    
    // Record final statistics
    {
        let mut stats = stats.lock().await;
        let msg_count = *received_messages.lock().await;
        stats.total_messages_received += msg_count;
        stats.client_stats.insert(
            client_name.to_string(), 
            ClientStats {
                connection_attempts,
                messages_received: msg_count,
                components_subscribed: subscriptions.len(),
                reconnections_successful: connection_attempts - 1,
            }
        );
    }
    
    debug!("{} test complete", client_name);
    Ok(())
}

/// Handle an individual test client
async fn handle_test_connection(
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    client_name: &str,
    components: &[String],
    received_messages: Arc<Mutex<usize>>,
    subscriptions: &mut HashMap<String, Instant>,
    stats: Arc<Mutex<TestStatistics>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut write, mut read) = ws_stream.split();
    let client_id = client_name.to_string(); // Clone the client name
    
    // Create a channel for signaling tasks to exit
    let (exit_tx, mut exit_rx) = mpsc::channel::<()>(1);
    let exit_tx_clone = exit_tx.clone();
    
    // Set up ping interval
    let mut ping_interval = tokio::time::interval(Duration::from_secs(5));
    
    // Subscribe to components
    for component in components {
        let subscribe_msg = json!({
            "type": "subscribe",
            "componentId": component
        }).to_string();
        
        if let Err(e) = write.send(Message::Text(subscribe_msg)).await {
            return Err(format!("Failed to subscribe to {}: {}", component, e).into());
        }
        
        subscriptions.insert(component.clone(), Instant::now());
        
        // Update statistics
        {
            let mut stats = stats.lock().await;
            stats.messages_per_component.entry(component.clone())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
    }
    
    // Spawn task to receive messages
    let receive_task = tokio::spawn({
        let exit_tx = exit_tx_clone;
        let received_msgs = received_messages.clone();
        let client_name = client_id.clone(); // Use the cloned client name
        async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(_text)) => {
                        // Update message count
                        let mut count = received_msgs.lock().await;
                        *count += 1;
                    },
                    Ok(Message::Close(_)) => {
                        let _ = exit_tx.send(()).await;
                        break;
                    },
                    Err(e) => {
                        debug!("{} error: {}", client_name, e);
                        let _ = exit_tx.send(()).await;
                        break;
                    },
                    _ => {}
                }
            }
        }
    });
    
    // Set a random timeout to simulate different client behaviors
    let timeout_duration = Duration::from_millis(thread_rng().gen_range(1000..8000));
    let timeout = tokio::time::sleep(timeout_duration);
    
    // Main loop
    tokio::select! {
        _ = ping_interval.tick() => {
            // Send ping
            let ping_msg = json!({"type":"ping"}).to_string();
            if let Err(e) = write.send(Message::Text(ping_msg)).await {
                debug!("{} error sending ping: {}", client_name, e);
                return Err(e.into());
            }
        }
        _ = exit_rx.recv() => {
            debug!("{} received exit signal", client_name);
            return Ok(());
        }
        _ = timeout => {
            debug!("{} timeout triggered", client_name);
            return Ok(());
        }
    }
    
    // Clean up
    receive_task.abort();
    Ok(())
} 