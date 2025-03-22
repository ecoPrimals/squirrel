use anyhow::Result;

/// MCP client trait for the web interface
pub trait MCPClient: Send + Sync {
    /// Send a message to the MCP
    fn send_message(&self, message: &str) -> Result<String>;
    
    /// Receive a message from the MCP
    fn receive_message(&self) -> Result<String>;
}

/// Default implementation of MCPClient
#[derive(Debug)]
pub struct DefaultMockMCPClient;

impl MCPClient for DefaultMockMCPClient {
    fn send_message(&self, message: &str) -> Result<String> {
        Ok(format!("Sent: {}", message))
    }
    
    fn receive_message(&self) -> Result<String> {
        Ok("Mock response".to_string())
    }
} 