use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use url::Url;
use std::time::Duration;

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
    
    // Subscribe to component updates
    let subscribe_msg = r#"{"type":"subscribe","componentId":"system_cpu"}"#;
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    println!("Subscribed to component updates");
    
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
                            println!("Received message: {}", text);
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