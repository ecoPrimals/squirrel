use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use crate::error::{MCPError, Result};
use crate::mcp::types::MCPMessage;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::SocketAddr;

pub struct Transport {
    listener: Option<TcpListener>,
    stream: Option<TcpStream>,
    is_server: bool,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            listener: None,
            stream: None,
            is_server: false,
        }
    }

    pub async fn bind(&mut self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await.map_err(|e| MCPError::Connection(e.to_string()))?;
        self.listener = Some(listener);
        self.is_server = true;
        Ok(())
    }

    pub async fn connect(&mut self, addr: SocketAddr) -> Result<()> {
        let stream = TcpStream::connect(addr).await.map_err(|e| MCPError::Connection(e.to_string()))?;
        self.stream = Some(stream);
        self.is_server = false;
        Ok(())
    }

    pub async fn accept(&mut self) -> Result<()> {
        if !self.is_server {
            return Err(MCPError::Connection("Transport is not a server".to_string()));
        }

        let listener = self.listener.as_ref().ok_or_else(|| MCPError::Connection("No listener available".to_string()))?;
        let (stream, _) = listener.accept().await.map_err(|e| MCPError::Connection(e.to_string()))?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn get_stream(&mut self) -> Result<&mut TcpStream> {
        self.stream.as_mut().ok_or_else(|| MCPError::Connection("No active stream".to_string()))
    }

    pub async fn send_message(&mut self, message: &MCPMessage) -> Result<()> {
        let stream = self.get_stream()?;
        let data = serde_json::to_vec(message).map_err(|e| MCPError::Serialization(e.to_string()))?;
        stream.write_all(&data).await.map_err(|e| MCPError::Connection(e.to_string()))?;
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<MCPMessage> {
        let stream = self.get_stream()?;
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await.map_err(|e| MCPError::Connection(e.to_string()))?;
        serde_json::from_slice(&buffer).map_err(|e| MCPError::Deserialization(e.to_string()))
    }
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn bind(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| MCPError::Io(e))?;
        
        Ok(Self { listener })
    }

    pub async fn accept(&self) -> Result<Transport> {
        let (stream, _) = self.listener.accept().await
            .map_err(|e| MCPError::Io(e))?;
        
        Ok(Transport {
            listener: None,
            stream: Some(stream),
            is_server: false,
        })
    }
} 