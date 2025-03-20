use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use futures::StreamExt;
use bytes::BytesMut;
use serde::{Serialize, Deserialize};

mod frame;
use frame::{Frame, MessageCodec, FrameReader, FrameWriter};

use crate::mcp::error::{MCPError, Result};
use crate::mcp::types::{
    MCPMessage,
    ProtocolVersion,
    ProtocolState,
    SecurityLevel,
    CompressionFormat,
    EncryptionFormat,
    MessageMetadata,
    MessageType,
};
use crate::mcp::security::{SecurityManager, SecurityConfig, Credentials};

const MAX_MESSAGE_SIZE: usize = 1024 * 1024 * 10; // 10MB
const DEFAULT_PORT: u16 = 9000;

#[derive(Debug)]
pub struct Transport {
    state: Arc<RwLock<TransportState>>,
    config: TransportConfig,
    message_tx: mpsc::Sender<MCPMessage>,
    message_rx: mpsc::Receiver<MCPMessage>,
    security_manager: Arc<SecurityManager>,
}

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub max_message_size: usize,
    pub protocol_version: ProtocolVersion,
    pub security_level: SecurityLevel,
    pub compression: CompressionFormat,
    pub encryption: EncryptionFormat,
}

#[derive(Debug)]
struct TransportState {
    connections: Vec<Connection>,
    active_sessions: usize,
    protocol_state: ProtocolState,
}

#[derive(Debug)]
struct Connection {
    id: String,
    stream: TcpStream,
    remote_addr: std::net::SocketAddr,
    created_at: chrono::DateTime<chrono::Utc>,
    last_activity: chrono::DateTime<chrono::Utc>,
    state: ConnectionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    New,
    Handshaking,
    Active,
    Closing,
    Closed,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: DEFAULT_PORT,
            max_connections: 100,
            max_message_size: MAX_MESSAGE_SIZE,
            protocol_version: ProtocolVersion::new(1, 0),
            security_level: SecurityLevel::None,
            compression: CompressionFormat::None,
            encryption: EncryptionFormat::None,
        }
    }
}

impl Transport {
    pub async fn new(config: TransportConfig) -> Result<Self> {
        let (tx, rx) = mpsc::channel(1000);
        
        let security_config = SecurityConfig {
            min_security_level: config.security_level,
            encryption_format: config.encryption,
            token_validity: 3600,
            max_auth_attempts: 3,
        };

        let transport = Self {
            state: Arc::new(RwLock::new(TransportState {
                connections: Vec::new(),
                active_sessions: 0,
                protocol_state: ProtocolState::Initialized,
            })),
            config,
            message_tx: tx,
            message_rx: rx,
            security_manager: Arc::new(SecurityManager::new(security_config)?),
        };

        Ok(transport)
    }

    pub async fn start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| MCPError::Connection(e.into()))?;

        println!("Transport layer listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    if !self.can_accept_connection().await {
                        println!("Rejecting connection from {}: max connections reached", addr);
                        continue;
                    }

                    let connection = Connection {
                        id: uuid::Uuid::new_v4().to_string(),
                        stream,
                        remote_addr: addr,
                        created_at: chrono::Utc::now(),
                        last_activity: chrono::Utc::now(),
                        state: ConnectionState::New,
                    };

                    self.handle_connection(connection).await?;
                }
                Err(e) => {
                    println!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    async fn can_accept_connection(&self) -> bool {
        let state = self.state.read().await;
        state.active_sessions < self.config.max_connections
    }

    async fn handle_connection(&self, mut connection: Connection) -> Result<()> {
        let mut state = self.state.write().await;
        state.active_sessions += 1;
        state.connections.push(connection.clone());
        drop(state);
        
        // Create message codec
        let codec = MessageCodec::new();
        
        // Create frame reader and writer
        let (read_half, write_half) = connection.stream.split();
        let mut frame_reader = FrameReader::new(read_half);
        let mut frame_writer = FrameWriter::new(write_half);
        
        // Start handshake process
        self.initiate_handshake(&mut frame_writer, &codec).await?;
        
        // Handle incoming messages
        while let Some(frame) = frame_reader.read_frame().await? {
            let message = codec.decode_message(frame).await?;
            
            // Handle secure messages
            if message.security_level >= SecurityLevel::Standard {
                self.handle_secure_message(message, &connection, &mut frame_writer, &codec).await?;
            } else {
                self.handle_message(message, &mut frame_writer, &codec).await?;
            }
        }
        
        Ok(())
    }

    async fn initiate_handshake(
        &self,
        frame_writer: &mut FrameWriter<impl tokio::io::AsyncWrite + Unpin>,
        codec: &MessageCodec,
    ) -> Result<()> {
        let handshake = MCPMessage::new(
            crate::mcp::types::MessageType::Handshake,
            self.config.protocol_version.clone(),
            self.config.security_level,
            Vec::new(),
        );

        let frame = codec.encode_message(&handshake).await?;
        frame_writer.write_frame(frame).await?;
        
        Ok(())
    }

    async fn handle_message(
        &self,
        message: MCPMessage,
        frame_writer: &mut FrameWriter<impl tokio::io::AsyncWrite + Unpin>,
        codec: &MessageCodec,
    ) -> Result<()> {
        match message.message_type {
            MessageType::Handshake => {
                // Handle handshake response
                let response = MCPMessage::new(
                    MessageType::HandshakeResponse,
                    self.config.protocol_version.clone(),
                    self.config.security_level,
                    Vec::new(),
                );
                
                let frame = codec.encode_message(&response).await?;
                frame_writer.write_frame(frame).await?;
            }
            MessageType::HandshakeResponse => {
                // Update connection state to active
                let mut state = self.state.write().await;
                if let Some(conn) = state.connections.iter_mut()
                    .find(|c| c.id == connection.id) {
                    conn.state = ConnectionState::Active;
                }
            }
            _ => {
                // Forward message to message channel
                self.message_tx.send(message).await.map_err(|_| {
                    MCPError::State("Failed to forward message".into())
                })?;
            }
        }
        
        Ok(())
    }

    async fn handle_secure_message(
        &self,
        message: MCPMessage,
        connection: &Connection,
        frame_writer: &mut FrameWriter<impl tokio::io::AsyncWrite + Unpin>,
        codec: &MessageCodec,
    ) -> Result<()> {
        match message.message_type {
            MessageType::AuthRequest => {
                let credentials: Credentials = serde_json::from_slice(&message.payload)
                    .map_err(|e| MCPError::SerdeJson(e))?;

                // Authenticate client
                let token = self.security_manager.authenticate(&credentials).await?;

                // Send auth response
                let response = MCPMessage::new(
                    MessageType::AuthResponse,
                    self.config.protocol_version.clone(),
                    credentials.security_level,
                    serde_json::to_vec(&token)?,
                );

                let frame = codec.encode_message(&response).await?;
                frame_writer.write_frame(frame).await?;
            }
            _ => {
                // Verify token and security level
                let token = String::from_utf8(message.payload.clone())
                    .map_err(|_| MCPError::InvalidMessage("Invalid token format".into()))?;

                let session = self.security_manager.authorize(&token, message.security_level).await?;

                // Decrypt message payload if needed
                let decrypted_payload = if message.metadata.encryption != EncryptionFormat::None {
                    self.security_manager.decrypt(&session.id, &message.payload).await?
                } else {
                    message.payload
                };

                // Create new message with decrypted payload
                let processed_message = MCPMessage {
                    payload: decrypted_payload,
                    ..message
                };

                // Forward message to message channel
                self.message_tx.send(processed_message).await.map_err(|_| {
                    MCPError::State("Failed to forward message".into())
                })?;
            }
        }
        
        Ok(())
    }

    async fn send_message(
        &self,
        connection: &Connection,
        message: MCPMessage,
    ) -> Result<()> {
        let codec = MessageCodec::new();
        let frame = codec.encode_message(&message).await?;
        
        let mut writer = FrameWriter::new(&connection.stream);
        writer.write_frame(frame).await?;
        
        Ok(())
    }

    async fn send_secure_message(
        &self,
        connection: &Connection,
        message: MCPMessage,
        session_id: &str,
    ) -> Result<()> {
        let codec = MessageCodec::new();
        
        // Encrypt payload if needed
        let encrypted_payload = if message.metadata.encryption != EncryptionFormat::None {
            self.security_manager.encrypt(session_id, &message.payload).await?
        } else {
            message.payload
        };

        // Create new message with encrypted payload
        let secure_message = MCPMessage {
            payload: encrypted_payload,
            ..message
        };

        let frame = codec.encode_message(&secure_message).await?;
        
        let mut writer = FrameWriter::new(&connection.stream);
        writer.write_frame(frame).await?;
        
        Ok(())
    }

    pub async fn broadcast(&self, message: MCPMessage) -> Result<()> {
        let state = self.state.read().await;
        for connection in &state.connections {
            if connection.state == ConnectionState::Active {
                self.send_message(connection, message.clone()).await?;
            }
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        let mut state = self.state.write().await;
        state.protocol_state = ProtocolState::ShuttingDown;
        
        // Close all connections
        for connection in &mut state.connections {
            connection.state = ConnectionState::Closing;
            // Implement connection cleanup
        }
        
        state.protocol_state = ProtocolState::Closed;
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<Option<MCPMessage>> {
        match self.message_rx.recv().await {
            Some(message) => Ok(Some(message)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_transport_creation() {
        let config = TransportConfig::default();
        let transport = Transport::new(config).await.unwrap();
        
        let state = transport.state.read().await;
        assert_eq!(state.active_sessions, 0);
        assert_eq!(state.protocol_state, ProtocolState::Initialized);
    }
} 