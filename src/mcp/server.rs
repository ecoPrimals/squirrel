use std::fmt;
use tokio::net::{TcpListener, TcpStream};
use crate::mcp::protocol::{McpProtocol, McpMessage};
use crate::core::error::Result;

pub struct McpServer {
    listener: TcpListener,
    protocol: McpProtocol,
}

impl McpServer {
    pub async fn new(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let protocol = McpProtocol::new();
        Ok(Self { listener, protocol })
    }

    pub async fn accept(&self) -> Result<TcpStream> {
        let (stream, _) = self.listener.accept().await?;
        Ok(stream)
    }

    pub async fn send(&self, stream: &mut TcpStream, message: McpMessage) -> Result<()> {
        self.protocol.send(stream, message).await
    }

    pub async fn receive(&self, stream: &mut TcpStream) -> Result<McpMessage> {
        self.protocol.receive(stream).await
    }
} 