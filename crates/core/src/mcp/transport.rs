use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use crate::error::{MCPError, Result};
use crate::mcp::types::{MCPMessage, CompressionFormat};
use crate::mcp::compression;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::SocketAddr;

pub struct Transport {
    listener: Option<TcpListener>,
    stream: Option<TcpStream>,
    is_server: bool,
    compression_format: CompressionFormat,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            listener: None,
            stream: None,
            is_server: false,
            compression_format: CompressionFormat::None,
        }
    }

    pub fn with_compression(compression_format: CompressionFormat) -> Self {
        Self {
            listener: None,
            stream: None,
            is_server: false,
            compression_format,
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
        
        // Update message metadata with compression format
        let mut message = message.clone();
        message.metadata.compression = self.compression_format;
        
        // Serialize and compress the message
        let data = serde_json::to_vec(&message)
            .map_err(|e| MCPError::Serialization(e.to_string()))?;
        let compressed_data = compression::compress(&data, self.compression_format)?;
        
        // Write the compressed data length first (as u32)
        let len = compressed_data.len() as u32;
        stream.write_all(&len.to_be_bytes()).await
            .map_err(|e| MCPError::Connection(e.to_string()))?;
        
        // Write the compressed data
        stream.write_all(&compressed_data).await
            .map_err(|e| MCPError::Connection(e.to_string()))?;
        
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<MCPMessage> {
        let stream = self.get_stream()?;
        
        // Read the message length first (u32)
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await
            .map_err(|e| MCPError::Connection(e.to_string()))?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        // Read the compressed data
        let mut compressed_data = vec![0u8; len];
        stream.read_exact(&mut compressed_data).await
            .map_err(|e| MCPError::Connection(e.to_string()))?;
        
        // Decompress and deserialize
        let data = compression::decompress(&compressed_data, self.compression_format)?;
        let message: MCPMessage = serde_json::from_slice(&data)
            .map_err(|e| MCPError::Deserialization(e.to_string()))?;
        
        Ok(message)
    }

    pub fn set_compression_format(&mut self, format: CompressionFormat) {
        self.compression_format = format;
    }
}

pub struct Server {
    listener: TcpListener,
    compression_format: CompressionFormat,
}

impl Server {
    pub async fn bind(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| MCPError::Io(e))?;
        
        Ok(Self { 
            listener,
            compression_format: CompressionFormat::None,
        })
    }

    pub fn with_compression(mut self, format: CompressionFormat) -> Self {
        self.compression_format = format;
        self
    }

    pub async fn accept(&self) -> Result<Transport> {
        let (stream, _) = self.listener.accept().await
            .map_err(|e| MCPError::Io(e))?;
        
        Ok(Transport {
            listener: None,
            stream: Some(stream),
            is_server: false,
            compression_format: self.compression_format,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use std::time::Duration;

    #[test]
    fn test_message_compression() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Start server
            let server = Server::bind("127.0.0.1:0").await.unwrap()
                .with_compression(CompressionFormat::Zstd);
            let server_addr = server.listener.local_addr().unwrap();

            // Client connection
            let mut client = Transport::with_compression(CompressionFormat::Zstd);
            client.connect(server_addr).await.unwrap();

            // Accept client connection
            let mut server_transport = server.accept().await.unwrap();

            // Test message
            let test_message = MCPMessage::new(
                crate::mcp::types::MessageType::Command,
                crate::mcp::types::ProtocolVersion::new(1, 0),
                crate::mcp::types::SecurityLevel::None,
                vec![0u8; 1000], // Large payload to test compression
            );

            // Send and receive
            client.send_message(&test_message).await.unwrap();
            let received = server_transport.receive_message().await.unwrap();

            assert_eq!(received.id, test_message.id);
            assert_eq!(received.payload, test_message.payload);
        });
    }
} 