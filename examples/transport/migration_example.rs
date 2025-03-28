//! Migration Example for Transport Module
//!
//! This example demonstrates how to migrate from the old transport implementation
//! to the new one, showing both approaches side by side for comparison.

use mcp::error::Result;
use mcp::message::{MCPMessage, MessageBuilder};
use mcp::transport_old::{Transport, TransportConfig, TransportState};
use mcp::transport::{Transport as NewTransport, tcp::TcpTransport};
use mcp::transport_old::compat;

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Transport Migration Example ===");
    
    // ===== OLD APPROACH =====
    println!("\n=== OLD APPROACH ===");
    
    // Create old transport configuration
    let mut old_config = TransportConfig::default();
    old_config.remote_address = Some("127.0.0.1:8080".to_string());
    old_config.connection_timeout_ms = 5000;
    old_config.encryption_enabled = true;
    
    // Create old transport instance
    let old_transport = Transport::new(old_config.clone());
    
    // Connect using old transport
    println!("Connecting using old transport...");
    // Note: We're not actually connecting since this is just a demonstration
    // old_transport.connect().await?;
    
    // Check connection status using old approach
    if old_transport.state == TransportState::Connected {
        println!("Connected using old transport!");
        
        // Create a message
        let message = MessageBuilder::new()
            .with_message_type("command")
            .with_payload(serde_json::json!({
                "action": "get_status"
            }))
            .build();
            
        // Send message using old transport
        println!("Sending message using old transport...");
        // old_transport.send_message(message).await?;
    } else {
        println!("Not connected with old transport.");
    }
    
    // ===== NEW APPROACH =====
    println!("\n=== NEW APPROACH ===");
    
    // Option 1: Create new transport configuration directly
    let mut new_config = TcpTransport::default_config();
    new_config.remote_address = Some("127.0.0.1:8080".to_string());
    new_config.connection_timeout = 5000;
    new_config.encryption = Some("aes256gcm".to_string());
    
    // Create new transport instance
    let new_transport = Arc::new(TcpTransport::new(new_config));
    
    // Connect using new transport
    println!("Connecting using new transport...");
    // Note: We're not actually connecting since this is just a demonstration
    // new_transport.connect().await?;
    
    // Check connection status using new approach
    if false { // Replace with: new_transport.is_connected().await
        println!("Connected using new transport!");
        
        // Create a message (same as before)
        let message = MessageBuilder::new()
            .with_message_type("command")
            .with_payload(serde_json::json!({
                "action": "get_status"
            }))
            .build();
            
        // Send message using new transport
        println!("Sending message using new transport...");
        // new_transport.send_message(message).await?;
    } else {
        println!("Not connected with new transport.");
    }
    
    // ===== USING COMPATIBILITY LAYER =====
    println!("\n=== USING COMPATIBILITY LAYER ===");
    
    // Option 2: Use compatibility layer to convert config
    println!("Converting old config to new config...");
    let converted_config = compat::convert_to_new_tcp_config(&old_config);
    println!("  Remote address: {:?}", converted_config.remote_address);
    println!("  Timeout: {} ms", converted_config.connection_timeout);
    println!("  Encryption: {:?}", converted_config.encryption);
    
    // Create new transport from old transport
    println!("Creating new transport from old transport...");
    let new_transport_from_old = compat::create_new_tcp_transport(&old_transport)?;
    
    // ===== IN-MEMORY TRANSPORT FOR TESTING =====
    println!("\n=== IN-MEMORY TRANSPORT FOR TESTING ===");
    
    // Create in-memory transport pair
    println!("Creating in-memory transport pair...");
    let (client_transport, server_transport) = compat::create_memory_transport();
    
    // Run a simple test with the memory transport
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        
        // For demonstration, we'll just print this
        println!("Server transport ready for testing!");
        
        // In a real scenario, we'd do:
        // let message = server_transport.receive_message().await.unwrap();
        // server_transport.send_message(response_message).await.unwrap();
    });
    
    // Connect the client
    println!("Connecting client transport...");
    client_transport.connect().await?;
    
    // Check if connected
    if client_transport.is_connected().await {
        println!("Client transport connected to in-memory server!");
        
        // Create a test message
        let test_message = MessageBuilder::new()
            .with_message_type("test")
            .with_payload(serde_json::json!({
                "test": "message"
            }))
            .build();
            
        // Send message using memory transport
        println!("Sending test message using memory transport...");
        // We're not actually sending to avoid having to handle real async communication
        // client_transport.send_message(test_message).await?;
        
        // In a real scenario, we'd also do:
        // let response = client_transport.receive_message().await?;
        // println!("Received response: {:?}", response);
    }
    
    // Give time for the async task to complete
    sleep(Duration::from_millis(200)).await;
    
    println!("\n=== Migration Example Complete ===");
    Ok(())
} 