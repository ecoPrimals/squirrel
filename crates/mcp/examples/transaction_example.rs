use squirrel_mcp::error::Result;
use squirrel_mcp::protocol::{MessageType, MCPMessage};
use squirrel_mcp::transport::memory::MemoryChannel;
use tokio::main;

/// Example demonstrating a simple transaction flow between two parties
#[main]
async fn main() -> Result<()> {
    // Create a pair of memory transports that are already connected
    let (client, server) = MemoryChannel::create_connected_pair_arc();
    
    println!("Transport endpoints are connected and ready");
    
    // Send a command from client to server
    let command = MCPMessage::new(
        MessageType::Command,
        serde_json::json!({
            "action": "get_data",
            "resource": "customer_profile",
            "id": "12345"
        })
    );
    
    println!("Client sending command: {:?}", command);
    client.send_message(command.clone()).await?;
    
    // Server receives the command
    let received = server.receive_message().await?;
    println!("Server received command: {:?}", received);
    
    // Server processes and sends a response
    let response = MCPMessage::new(
        MessageType::Response,
        serde_json::json!({
            "status": "success",
            "data": {
                "name": "John Doe",
                "email": "john@example.com",
                "subscription": "premium"
            },
            "request_id": received.id
        })
    );
    
    println!("Server sending response: {:?}", response);
    server.send_message(response).await?;
    
    // Client receives the response
    let received_response = client.receive_message().await?;
    println!("Client received response: {:?}", received_response);
    
    // Disconnect
    client.disconnect().await?;
    server.disconnect().await?;
    
    println!("Transaction completed successfully");
    Ok(())
} 