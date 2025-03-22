use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use url::Url;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use serde_json::{json, Value};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::env;

/// Enhanced WebSocket tester for dashboard WebSocket functionality
/// 
/// Tests the following scenarios:
/// - Multiple concurrent client connections
/// - Subscription to different components
/// - Message reception verification
/// - Reconnection handling
/// - Connection interruption recovery
/// - Performance under load
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Default configuration
    let mut base_url = "ws://localhost:8765/ws";
    let mut num_clients = 10;
    let mut test_duration = Duration::from_secs(120);
    let components = vec![
        "system_cpu".to_string(), 
        "system_memory".to_string(), 
        "network_traffic".to_string(), 
        "disk_usage".to_string(),
        "health_status".to_string()
    ];
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--url" => {
                if i + 1 < args.len() {
                    base_url = &args[i + 1];
                    i += 2;
                } else {
                    i += 1;
                }
            },
            "--num-clients" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        num_clients = n;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            },
            "--duration" => {
                if i + 1 < args.len() {
                    if let Ok(d) = args[i + 1].parse::<u64>() {
                        test_duration = Duration::from_secs(d);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            },
            "--help" => {
                println!("Enhanced WebSocket Test");
                println!("Usage: enhanced_websocket_test [OPTIONS]");
                println!("Options:");
                println!("  --url URL               WebSocket server URL (default: ws://localhost:8765/ws)");
                println!("  --num-clients N         Number of clients to create (default: 10)");
                println!("  --duration SECONDS      Test duration in seconds (default: 120)");
                println!("  --help                  Show this help message");
                return Ok(());
            },
            _ => {
                i += 1;
            }
        }
    }
    
    println!("Starting enhanced WebSocket test with {} clients", num_clients);
    println!("Test will run for {} seconds", test_duration.as_secs());
    println!("Connecting to {}", base_url);
    
    // Setup statistics collection
    let stats = Arc::new(Mutex::new(TestStatistics::default()));
    
    // Run multiple clients in parallel
    let client_handles = run_multiple_clients(
        base_url, 
        num_clients, 
        components, 
        test_duration,
        stats.clone()
    ).await?;
    
    // Wait for test to complete
    let start_time = Instant::now();
    println!("Test running... Press Ctrl+C to stop early");
    
    // Wait for test duration or Ctrl+C
    tokio::select! {
        _ = tokio::time::sleep(test_duration) => {
            println!("Test duration completed");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Test interrupted by user");
        }
    }
    
    // Calculate actual test duration
    let elapsed = start_time.elapsed();
    println!("Test ran for {:.2} seconds", elapsed.as_secs_f64());
    
    // Print test statistics
    print_statistics(stats).await;
    
    // Close client connections
    for handle in client_handles {
        handle.abort();
    }
    
    println!("Test completed");
    Ok(())
}

/// Run multiple WebSocket clients in parallel
async fn run_multiple_clients(
    base_url: &str,
    num_clients: usize,
    components: Vec<String>,
    test_duration: Duration,
    stats: Arc<Mutex<TestStatistics>>,
) -> Result<Vec<JoinHandle<()>>, Box<dyn std::error::Error>> {
    let mut handles = Vec::with_capacity(num_clients);
    
    // Launch each client with a small delay to avoid thundering herd
    for client_id in 0..num_clients {
        let url = Url::parse(base_url)?;
        let client_components = components.clone();
        let client_stats = stats.clone();
        
        // Each client gets a unique ID
        let client_name = format!("client-{}", client_id);
        println!("Starting {}", client_name);
        
        // Create client task
        let handle = tokio::spawn(async move {
            let result = run_client(
                url, 
                &client_name, 
                client_components, 
                test_duration, 
                client_stats
            ).await;
            
            if let Err(e) = result {
                eprintln!("Error in {}: {}", client_name, e);
            }
        });
        
        handles.push(handle);
        
        // Small delay between client starts
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    Ok(handles)
}

/// Run an individual WebSocket client
async fn run_client(
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
    
    // Each client subscribes to a random subset of components
    let selected_components: Vec<String> = {
        let mut rng = thread_rng();
        let num_components = rng.gen_range(1..=components.len());
        components
            .choose_multiple(&mut rng, num_components)
            .into_iter()
            .cloned()
            .collect()
    };
    
    println!("{} subscribing to {} components", client_name, selected_components.len());
    
    let start_time = Instant::now();
    
    // Run client with reconnection logic
    while start_time.elapsed() < test_duration {
        connection_attempts += 1;
        
        println!("{} connecting (attempt {})", client_name, connection_attempts);
        
        // Connect to the server
        let connection_result = connect_async(url.clone()).await;
        match connection_result {
            Ok((ws_stream, _)) => {
                // Update statistics
                {
                    let mut stats = stats.lock().await;
                    stats.connections_successful += 1;
                    stats.current_connections += 1;
                }
                
                println!("{} connected successfully", client_name);
                
                // Process the connection
                if let Err(e) = handle_client_connection(
                    ws_stream,
                    client_name,
                    &selected_components,
                    received_messages.clone(),
                    &mut subscriptions,
                    stats.clone()
                ).await {
                    println!("{} connection error: {}", client_name, e);
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
                
                println!("{} connection failed: {}", client_name, e);
            }
        }
        
        // Exponential backoff for reconnection attempts
        let backoff = Duration::from_millis(
            std::cmp::min(100 * (2_u64.pow(connection_attempts as u32)), 5000)
        );
        
        println!("{} reconnecting after {}ms", client_name, backoff.as_millis());
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
            }
        );
    }
    
    println!("{} test complete", client_name);
    Ok(())
}

/// Handle a specific WebSocket connection for a client
async fn handle_client_connection(
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    client_name: &str,
    components: &[String],
    received_messages: Arc<Mutex<usize>>,
    subscriptions: &mut HashMap<String, Instant>,
    stats: Arc<Mutex<TestStatistics>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut write, mut read) = ws_stream.split();
    
    // Create communication channels
    let (exit_tx, mut exit_rx) = mpsc::channel::<()>(1);
    
    // Set up ping interval
    let mut ping_interval = tokio::time::interval(Duration::from_secs(15));
    
    // Subscribe to components
    for component in components {
        let subscribe_msg = json!({
            "type": "subscribe",
            "componentId": component
        }).to_string();
        
        write.send(Message::Text(subscribe_msg)).await?;
        subscriptions.insert(component.to_string(), Instant::now());
        
        println!("{} subscribed to {}", client_name, component);
        
        // Small delay between subscriptions
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Spawn task to receive messages
    let receive_handle = tokio::spawn({
        let client_name = client_name.to_string();
        let stats = stats.clone();
        let exit_tx = exit_tx.clone();
        let received_messages = received_messages.clone();
        
        async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        match msg {
                            Message::Text(text) => {
                                // Parse the message
                                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                    // Update received count
                                    {
                                        let mut count = received_messages.lock().await;
                                        *count += 1;
                                        
                                        // Periodically log received messages
                                        if *count % 10 == 0 {
                                            println!("{} received {} messages", client_name, count);
                                        }
                                    }
                                    
                                    // Update statistics
                                    {
                                        let mut stats = stats.lock().await;
                                        stats.total_messages_received += 1;
                                        
                                        if let Some(component_id) = value.get("component_id").and_then(|v| v.as_str()) {
                                            stats.messages_per_component
                                                .entry(component_id.to_string())
                                                .and_modify(|c| *c += 1)
                                                .or_insert(1);
                                        }
                                    }
                                }
                            },
                            Message::Close(_) => {
                                println!("{} received close frame", client_name);
                                let _ = exit_tx.send(()).await;
                                break;
                            },
                            _ => {
                                // Ignore other message types
                            }
                        }
                    },
                    Err(e) => {
                        println!("{} receive error: {}", client_name, e);
                        let _ = exit_tx.send(()).await;
                        break;
                    }
                }
            }
        }
    });
    
    // Random disconnect probability for testing
    let should_simulate_disconnect = {
        let mut rng = thread_rng();
        let random_time = Duration::from_secs(rng.gen_range(30..120));
        let disconnect_probability = rng.gen_bool(0.2);
        (random_time, disconnect_probability)
    };
    
    // Main loop
    let (random_time, should_disconnect) = should_simulate_disconnect;
    
    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                // Send a ping periodically
                let ping_msg = json!({"type": "ping"}).to_string();
                if let Err(e) = write.send(Message::Text(ping_msg)).await {
                    println!("{} error sending ping: {}", client_name, e);
                    break;
                }
            }
            _ = exit_rx.recv() => {
                println!("{} exiting due to exit signal", client_name);
                break;
            }
            // Simulate random disconnections for testing reconnection
            _ = tokio::time::sleep(random_time), if should_disconnect => {
                println!("{} simulating random disconnection", client_name);
                // Record reconnection test
                {
                    let mut stats = stats.lock().await;
                    stats.reconnection_tests += 1;
                }
                break;
            }
        }
    }
    
    // Clean up
    receive_handle.abort();
    
    Ok(())
}

/// Print the test statistics
async fn print_statistics(stats: Arc<Mutex<TestStatistics>>) {
    let stats = stats.lock().await;
    
    println!("\n====== TEST STATISTICS ======");
    println!("Connections successful: {}", stats.connections_successful);
    println!("Connection failures: {}", stats.connection_failures);
    println!("Connections closed: {}", stats.connections_closed);
    println!("Current connections: {}", stats.current_connections);
    println!("Total messages received: {}", stats.total_messages_received);
    println!("Reconnection tests: {}", stats.reconnection_tests);
    
    println!("\n--- Messages Per Component ---");
    for (component, count) in &stats.messages_per_component {
        println!("{}: {}", component, count);
    }
    
    println!("\n--- Client Statistics ---");
    for (client, stats) in &stats.client_stats {
        println!(
            "{}: {} attempts, {} messages, {} subscriptions", 
            client, 
            stats.connection_attempts, 
            stats.messages_received,
            stats.components_subscribed
        );
    }
    
    println!("============================\n");
}

/// Extension trait for random selection from a vector
trait ChooseMultiple<T> {
    fn choose_multiple<R: Rng>(&self, rng: &mut R, amount: usize) -> Vec<&T>;
}

impl<T> ChooseMultiple<T> for Vec<T> {
    fn choose_multiple<R: Rng>(&self, rng: &mut R, amount: usize) -> Vec<&T> {
        use rand::seq::SliceRandom;
        let mut indices: Vec<usize> = (0..self.len()).collect();
        indices.shuffle(rng);
        indices.truncate(amount);
        indices.iter().map(|&i| &self[i]).collect()
    }
}

/// Statistics for an individual client
#[derive(Debug, Default)]
struct ClientStats {
    connection_attempts: usize,
    messages_received: usize,
    components_subscribed: usize,
}

/// Test statistics
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