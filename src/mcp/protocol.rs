use std::fmt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use serde::{Deserialize, Serialize};
use crate::core::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpMessage {
    Connect {
        client_id: String,
        version: String,
    },
    Connected {
        server_id: String,
        version: String,
    },
    Disconnect {
        reason: String,
    },
    Data {
        payload: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub struct McpProtocol {
    version: String,
}

impl McpProtocol {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub async fn send(&self, stream: &mut TcpStream, message: McpMessage) -> Result<()> {
        let data = serde_json::to_vec(&message)?;
        let len = data.len() as u32;
        stream.write_u32_le(len).await?;
        stream.write_all(&data).await?;
        Ok(())
    }

    pub async fn receive(&self, stream: &mut TcpStream) -> Result<McpMessage> {
        let len = stream.read_u32_le().await? as usize;
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await?;
        let message = serde_json::from_slice(&data)?;
        Ok(message)
    }
} 