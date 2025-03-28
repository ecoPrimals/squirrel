//! Gradual Migration Example
//!
//! This example demonstrates how to gradually migrate from the old transport
//! implementation to the new one using the migration utilities.

mod transport_utilities;

use transport_utilities::{MigratingTransport, create_memory_transport_pair};
use mcp::error::Result;
use mcp::message::MessageBuilder;
use std::env;
use std::sync::Arc;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Gradual Migration Example ===");
    
    // Read environment variable to control migration (or use command line argument)
    let use_new_transport = env::var("USE_NEW_TRANSPORT")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);
    
    println!("Using new transport: {}", use_new_transport);
    
    // Create a transport instance using the migration utility
    let transport = MigratingTransport::new("127.0.0.1:8080", use_new_transport);
    
    // Use the transport with a unified API, regardless of which implementation is used
    println!("Connecting transport...");
    
    // We won't actually connect in this example, but the API would be the same
    // transport.connect().await?;
    
    // Check if connected
    let connected = transport.is_connected().await;
    println!("Connected: {}", connected);
    
    if connected {
        // Create a message
        let message = MessageBuilder::new()
            .with_message_type("command")
            .with_payload(serde_json::json!({
                "action": "get_status"
            }))
            .build();
        
        // Send message using unified API
        println!("Sending message...");
        // transport.send_message(message).await?;
    }
    
    // Demonstrate in-memory transport with new implementation
    println!("\n=== Memory Transport Example ===");
    
    // Create memory transport pair (always uses new implementation)
    let (client, server) = create_memory_transport_pair();
    
    // Spawn server handler
    let server_clone = server.clone();
    tokio::spawn(async move {
        // Connect the server side
        server_clone.connect().await.unwrap();
        println!("Server connected");
        
        // Wait for a message
        println!("Server waiting for message...");
        let message = server_clone.receive_message().await.unwrap();
        println!("Server received message: {:?}", message.message_type);
        
        // Send a response
        let response = MessageBuilder::new()
            .with_message_type("response")
            .with_payload(serde_json::json!({
                "status": "ok",
                "echo": message.payload
            }))
            .build();
            
        println!("Server sending response...");
        server_clone.send_message(response).await.unwrap();
    });
    
    // Connect client
    println!("Connecting client...");
    client.connect().await?;
    
    // Send a test message
    let test_message = MessageBuilder::new()
        .with_message_type("test")
        .with_payload(serde_json::json!({
            "hello": "world"
        }))
        .build();
        
    println!("Client sending message...");
    client.send_message(test_message).await?;
    
    // Receive the response
    println!("Client waiting for response...");
    let response = client.receive_message().await?;
    println!("Client received response: {:?}", response.message_type);
    
    // Give time for the server task to complete
    sleep(Duration::from_millis(100)).await;
    
    // Disconnect
    println!("Disconnecting client...");
    client.disconnect().await?;
    
    println!("\n=== Migration Example Complete ===");
    println!("To change transport implementation, set USE_NEW_TRANSPORT=1");
    
    Ok(())
} 