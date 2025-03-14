use std::fmt;
use tokio::net::TcpStream;
use crate::mcp::protocol::{McpProtocol, McpMessage};
use crate::core::error::Result;

pub struct McpClient {
    stream: TcpStream,
    protocol: McpProtocol,
}

impl McpClient {
    pub async fn new(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let protocol = McpProtocol::new();
        Ok(Self { stream, protocol })
    }

    pub async fn send(&mut self, message: McpMessage) -> Result<()> {
        self.protocol.send(&mut self.stream, message).await
    }

    pub async fn receive(&mut self) -> Result<McpMessage> {
        self.protocol.receive(&mut self.stream).await
    }
} 