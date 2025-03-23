use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use url::Url;
use std::time::{Duration, Instant};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use serde_json::json;
use clap::Parser;

/// WebSocket Load Testing Utility
///
/// This tool simulates multiple clients connecting to the WebSocket server
/// and measures performance metrics to evaluate server scalability.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// WebSocket server URL
    #[arg(short, long, default_value = "ws://localhost:8765/ws")]
    url: String,

    /// Number of simultaneous clients
    #[arg(short, long, default_value_t = 10)]
    clients: usize,

    /// Test duration in seconds
    #[arg(short, long, default_value_t = 30)]
    duration: u64,

    /// Components to subscribe to (comma separated)
    #[arg(short, long, default_value = "system_cpu,system_memory,network_traffic,disk_usage,health_status")]
    components: String,

    /// Message sending interval in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    interval: u64,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Default)]
struct TestStatistics {
    connections_successful: AtomicUsize,
    connection_failures: AtomicUsize,
    messages_sent: AtomicUsize,
    messages_received: AtomicUsize,
    batch_messages_received: AtomicUsize,
    compressed_messages_received: AtomicUsize,
    regular_messages_received: AtomicUsize,
    errors: AtomicUsize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = Args::parse();
    
    // Print test configuration
    println!("🚀 WebSocket Load Test");
    println!("URL: {}", args.url);
    println!("Clients: {}", args.clients);
    println!("Test duration: {}s", args.duration);
    println!("Components: {}", args.components);
    println!("Message interval: {}ms", args.interval);
    
    // Parse components
    let components: Vec<String> = args.components
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    // Create statistics tracker
    let stats = Arc::new(TestStatistics::default());
    
    // Start time
    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(args.duration);
    
    // Create client handles
    let mut client_handles = Vec::with_capacity(args.clients);
    
    // Start clients
    println!("Starting {} clients...", args.clients);
    for client_id in 0..args.clients {
        let url = Url::parse(&args.url)?;
        let components = components.clone();
        let stats = stats.clone();
        let interval = Duration::from_millis(args.interval);
        let verbose = args.verbose;
        let end = end_time;
        
        // Start client in background
        let handle = tokio::spawn(async move {
            let _ = run_client(
                client_id, 
                url, 
                components, 
                interval, 
                stats,
                end,
                verbose
            ).await;
        });
        
        client_handles.push(handle);
        
        // Small delay between client starts to avoid overwhelming the server
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // Print live statistics during the test
    let stats_clone = stats.clone();
    let interval = Duration::from_secs(1);
    let mut interval_timer = tokio::time::interval(interval);
    
    while Instant::now() < end_time {
        interval_timer.tick().await;
        
        let elapsed = start_time.elapsed().as_secs();
        let conn_success = stats_clone.connections_successful.load(Ordering::Relaxed);
        let conn_failures = stats_clone.connection_failures.load(Ordering::Relaxed);
        let msgs_sent = stats_clone.messages_sent.load(Ordering::Relaxed);
        let msgs_received = stats_clone.messages_received.load(Ordering::Relaxed);
        let batch_msgs = stats_clone.batch_messages_received.load(Ordering::Relaxed);
        let compressed_msgs = stats_clone.compressed_messages_received.load(Ordering::Relaxed);
        let errors = stats_clone.errors.load(Ordering::Relaxed);
        
        println!(
            "[{:3}s] Connected: {:4} | Failed: {:4} | Msgs Sent: {:6} | Received: {:6} | Batched: {:6} | Compressed: {:6} | Errors: {:4}",
            elapsed, conn_success, conn_failures, msgs_sent, msgs_received, batch_msgs, compressed_msgs, errors
        );
    }
    
    // Wait for all clients to finish
    println!("Test completed. Waiting for clients to shut down...");
    for handle in client_handles {
        let _ = handle.await;
    }
    
    // Print final statistics
    let total_duration = start_time.elapsed();
    let conn_success = stats.connections_successful.load(Ordering::Relaxed);
    let conn_failures = stats.connection_failures.load(Ordering::Relaxed);
    let msgs_sent = stats.messages_sent.load(Ordering::Relaxed);
    let msgs_received = stats.messages_received.load(Ordering::Relaxed);
    let batch_msgs = stats.batch_messages_received.load(Ordering::Relaxed);
    let compressed_msgs = stats.compressed_messages_received.load(Ordering::Relaxed);
    let regular_msgs = stats.regular_messages_received.load(Ordering::Relaxed);
    let errors = stats.errors.load(Ordering::Relaxed);
    
    println!("\n📊 Test Results:");
    println!("Test duration: {:.2}s", total_duration.as_secs_f64());
    println!("Successful connections: {}", conn_success);
    println!("Failed connections: {}", conn_failures);
    println!("Messages sent: {}", msgs_sent);
    println!("Messages received: {}", msgs_received);
    println!("  - Regular messages: {}", regular_msgs);
    println!("  - Batched messages: {}", batch_msgs);
    println!("  - Compressed messages: {}", compressed_msgs);
    println!("Errors: {}", errors);
    
    if msgs_received > 0 {
        let msgs_per_second = msgs_received as f64 / total_duration.as_secs_f64();
        println!("Message throughput: {:.2} msgs/sec", msgs_per_second);
        
        let batch_percentage = if msgs_received > 0 {
            (batch_msgs as f64 / msgs_received as f64) * 100.0
        } else {
            0.0
        };
        
        let compression_percentage = if msgs_received > 0 {
            (compressed_msgs as f64 / msgs_received as f64) * 100.0
        } else {
            0.0
        };
        
        println!("Batch utilization: {:.2}%", batch_percentage);
        println!("Compression utilization: {:.2}%", compression_percentage);
    }
    
    Ok(())
}

/// Run a single client
async fn run_client(
    client_id: usize,
    url: Url,
    components: Vec<String>,
    interval: Duration,
    stats: Arc<TestStatistics>,
    end_time: Instant,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Client {} starting", client_id);
    }
    
    // Try to connect
    match connect_async(url).await {
        Ok((ws_stream, _)) => {
            // Connection successful
            stats.connections_successful.fetch_add(1, Ordering::Relaxed);
            
            if verbose {
                println!("Client {} connected", client_id);
            }
            
            // Process the connection
            if let Err(e) = handle_connection(
                client_id,
                ws_stream,
                components,
                interval,
                stats.clone(),
                end_time,
                verbose
            ).await {
                if verbose {
                    println!("Client {} error: {}", client_id, e);
                }
                stats.errors.fetch_add(1, Ordering::Relaxed);
            }
        },
        Err(e) => {
            // Connection failed
            if verbose {
                println!("Client {} connection failed: {}", client_id, e);
            }
            stats.connection_failures.fetch_add(1, Ordering::Relaxed);
            stats.errors.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    Ok(())
}

/// Handle an individual client connection
async fn handle_connection(
    client_id: usize,
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    components: Vec<String>,
    interval: Duration,
    stats: Arc<TestStatistics>,
    end_time: Instant,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to components
    for component in &components {
        let subscribe_msg = json!({
            "type": "subscribe",
            "componentId": component
        }).to_string();
        
        write.send(Message::Text(subscribe_msg)).await?;
        stats.messages_sent.fetch_add(1, Ordering::Relaxed);
        
        if verbose {
            println!("Client {} subscribed to {}", client_id, component);
        }
    }
    
    // Create a background task to receive messages
    let receive_stats = stats.clone();
    let verbose_flag = verbose;
    let receive_task = tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            // Increment message count
                            receive_stats.messages_received.fetch_add(1, Ordering::Relaxed);
                            
                            // Parse the message to check if it's batched or compressed
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                                match parsed["type"].as_str() {
                                    Some("batch") => {
                                        receive_stats.batch_messages_received.fetch_add(1, Ordering::Relaxed);
                                        if verbose_flag {
                                            let updates = parsed["updates"].as_array().map_or(0, |arr| arr.len());
                                            println!("Client {} received batch with {} updates", client_id, updates);
                                        }
                                    },
                                    Some("compressed") => {
                                        receive_stats.compressed_messages_received.fetch_add(1, Ordering::Relaxed);
                                        if verbose_flag {
                                            println!("Client {} received compressed message", client_id);
                                        }
                                    },
                                    _ => {
                                        receive_stats.regular_messages_received.fetch_add(1, Ordering::Relaxed);
                                    }
                                }
                            }
                        },
                        _ => {
                            // Ignore non-text messages
                        }
                    }
                },
                Err(_) => {
                    receive_stats.errors.fetch_add(1, Ordering::Relaxed);
                    break;
                }
            }
        }
    });
    
    // Set up a periodic ping
    let mut interval_timer = tokio::time::interval(interval);
    
    while Instant::now() < end_time {
        interval_timer.tick().await;
        
        // Send ping message
        let ping_msg = json!({"type":"ping"}).to_string();
        if let Err(_) = write.send(Message::Text(ping_msg)).await {
            stats.errors.fetch_add(1, Ordering::Relaxed);
            break;
        }
        
        stats.messages_sent.fetch_add(1, Ordering::Relaxed);
    }
    
    // Clean up
    let _ = write.close().await;
    let _ = receive_task.await;
    
    Ok(())
} 