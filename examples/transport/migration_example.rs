//! Transport Usage Example
//!
//! This example demonstrates how to use the transport module with different
//! transport types and configurations.

use mcp::error::Result;
use mcp::types::MCPMessage;
use mcp::transport::{Transport, TransportMetadata};
use mcp::transport::tcp::{TcpTransport, TcpTransportConfig};
use mcp::transport::memory::MemoryChannel;
use mcp::transport::websocket::{WebSocketTransport, WebSocketConfig};

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Transport Usage Example ===");
    
    // ===== TCP TRANSPORT =====
    println!("\n=== TCP TRANSPORT ===");
    
    // Create TCP transport configuration
    let tcp_config = TcpTransportConfig::default()
        .with_remote_address("127.0.0.1:8080")
        .with_connection_timeout(5000)
        .with_keep_alive_interval(Some(30000))
        .with_max_reconnect_attempts(3);
    
    // Create TCP transport instance
    let mut tcp_transport = TcpTransport::new(tcp_config);
    
    // In a real application, you would connect here
    println!("In a real app, you would connect using tcp_transport.connect().await");
    
    // Create and wrap in Arc for thread-safe sharing
    println!("Creating Arc-wrapped transport for thread safety");
    let tcp_transport = Arc::new(tcp_transport);
    
    // Create a sample message
    let message = MCPMessage::new(
        mcp::types::MessageType::Request,
        serde_json::json!({
            "action": "get_status",
            "source": "example",
            "destination": "server"
        })
    );
    
    // In a real app, you would send messages like this
    println!("In a real app, you would send messages with: tcp_transport.send_message(message).await");
    
    // ===== WEBSOCKET TRANSPORT =====
    println!("\n=== WEBSOCKET TRANSPORT ===");
    
    // Create WebSocket transport configuration
    let ws_config = WebSocketConfig::default()
        .with_url("ws://localhost:8080")
        .with_connection_timeout(5000);
    
    // Create WebSocket transport instance
    let mut ws_transport = WebSocketTransport::new(ws_config);
    
    println!("In a real app, you would connect using ws_transport.connect().await");
    
    // ===== IN-MEMORY TRANSPORT FOR TESTING =====
    println!("\n=== IN-MEMORY TRANSPORT FOR TESTING ===");
    
    // Create in-memory transport pair
    println!("Creating in-memory transport pair");
    let (mut client_transport, mut server_transport) = MemoryChannel::create_pair();
    
    // Connect both sides
    println!("Connecting both transport sides");
    
    // In a real test, you would connect like this:
    client_transport.connect().await?;
    server_transport.connect().await?;
    
    // Wrap in Arc for thread-safe sharing
    let client_transport = Arc::new(client_transport);
    let server_transport = Arc::new(server_transport);
    
    // Run a simple demonstration with the memory transport
    let server_transport_clone = Arc::clone(&server_transport);
    tokio::spawn(async move {
        println!("Server transport starting in separate task");
        
        // In a real test, you would receive and respond to messages:
        /*
        if let Ok(message) = server_transport_clone.receive_message().await {
            println!("Server received: {:?}", message);
            
            // Create a response
            let response = MCPMessage::new(
                mcp::types::MessageType::Response,
                serde_json::json!({
                    "status": "success",
                    "source": "server",
                    "destination": "example"
                })
            );
            
            // Send the response
            server_transport_clone.send_message(response).await.unwrap();
        }
        */
    });
    
    // Wait a moment for the server to be ready
    sleep(Duration::from_millis(100)).await;
    
    // Check if connected
    let connected = client_transport.is_connected().await;
    println!("Client transport connected: {}", connected);
    
    if connected {
        // Create a test message
        let test_message = MCPMessage::new(
            mcp::types::MessageType::Request,
            serde_json::json!({
                "action": "test",
                "source": "example",
                "destination": "server"
            })
        );
        
        println!("In a real app, you would send messages with: client_transport.send_message(test_message).await");
        
        // In a real test, you would also receive the response:
        /*
        if let Ok(response) = client_transport.receive_message().await {
            println!("Client received response: {:?}", response);
        }
        */
    }
    
    // Give time for the async task to complete
    sleep(Duration::from_millis(200)).await;
    
    // ===== METADATA AND DIAGNOSTICS =====
    println!("\n=== TRANSPORT METADATA ===");
    
    // Get and display transport metadata
    let metadata = client_transport.get_metadata();
    println!("Transport Type: {}", metadata.transport_type);
    println!("Remote Address: {}", metadata.remote_address);
    println!("Encryption: {:?}", metadata.encryption);
    println!("Compression: {:?}", metadata.compression);
    
    println!("\n=== Transport Example Complete ===");
    Ok(())
} 