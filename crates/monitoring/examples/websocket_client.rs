use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use url::Url;
use std::time::Duration;
use std::io::Read;
use serde_json::{Value, json};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use flate2::read::GzDecoder;

/// Simplified WebSocket client for testing the dashboard WebSocket server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the dashboard WebSocket server
    let url = Url::parse("ws://localhost:8765/ws")?;
    println!("Connecting to {}", url);
    
    // Establish WebSocket connection
    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connection established");
    
    // Process the WebSocket connection
    handle_connection(ws_stream).await?;
    
    Ok(())
}

/// Handle the WebSocket connection by sending and receiving messages
async fn handle_connection(ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), Box<dyn std::error::Error>> {
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to multiple components to increase likelihood of compressed messages
    let components = ["system_cpu", "system_memory", "network_traffic", "disk_usage", "health_status"];
    
    for component in components {
        let subscribe_msg = json!({
            "type": "subscribe",
            "componentId": component
        }).to_string();
        write.send(Message::Text(subscribe_msg)).await?;
        println!("Subscribed to component: {}", component);
    }
    
    // Set up a task to periodically send a ping
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    
    // Create a channel to signal when we want to exit
    let (exit_tx, mut exit_rx) = tokio::sync::mpsc::channel::<()>(1);
    let exit_tx_clone = exit_tx.clone();
    
    // Spawn a task to receive messages
    let receive_task = tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            // Parse JSON
                            match serde_json::from_str::<Value>(&text) {
                                Ok(parsed) => {
                                    // Check if it's a compressed message
                                    if parsed["type"] == "compressed" && parsed["compressed"] == true {
                                        println!("Received compressed message, decompressing...");
                                        
                                        // Extract compressed data
                                        if let Some(compressed_data) = parsed["compressed_data"].as_str() {
                                            // Decode base64
                                            match BASE64.decode(compressed_data) {
                                                Ok(data) => {
                                                    // Decompress the data
                                                    let mut decoder = GzDecoder::new(&data[..]);
                                                    let mut decompressed = String::new();
                                                    
                                                    match decoder.read_to_string(&mut decompressed) {
                                                        Ok(_) => {
                                                            println!("Successfully decompressed message: {} bytes", decompressed.len());
                                                            // Optionally parse the decompressed JSON
                                                            if let Ok(parsed_decompressed) = serde_json::from_str::<Value>(&decompressed) {
                                                                println!("Message type: {}", parsed_decompressed["type"]);
                                                                println!("Updates: {} items", parsed_decompressed["updates"].as_array().map_or(0, |a| a.len()));
                                                            }
                                                        },
                                                        Err(e) => println!("Failed to decompress: {}", e)
                                                    }
                                                },
                                                Err(e) => println!("Base64 decode error: {}", e)
                                            }
                                        }
                                    } else if parsed["type"] == "batch" {
                                        // Handle batch message
                                        println!("Received batch message with {} updates", 
                                                parsed["updates"].as_array().map_or(0, |a| a.len()));
                                    } else {
                                        // Regular message
                                        println!("Received message: {}", text);
                                    }
                                },
                                Err(e) => println!("Failed to parse JSON: {}", e)
                            }
                        },
                        Message::Close(_) => {
                            println!("Server closed the connection");
                            let _ = exit_tx.send(()).await;
                            break;
                        },
                        _ => {
                            println!("Received non-text message");
                        }
                    }
                },
                Err(e) => {
                    println!("Error receiving message: {}", e);
                    let _ = exit_tx.send(()).await;
                    break;
                }
            }
        }
    });
    
    // Main loop
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Send a ping every 15 seconds
                let ping_msg = r#"{"type":"ping"}"#;
                if let Err(e) = write.send(Message::Text(ping_msg.to_string())).await {
                    println!("Error sending ping: {}", e);
                    break;
                }
                println!("Sent ping");
            }
            _ = exit_rx.recv() => {
                println!("Exiting client");
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Received Ctrl-C, shutting down");
                let _ = exit_tx_clone.send(()).await;
                break;
            }
        }
    }
    
    // Wait for the receive task to complete
    let _ = receive_task.await;
    
    Ok(())
} 