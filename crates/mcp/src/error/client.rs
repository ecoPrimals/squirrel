use std::fmt;
use std::error::Error;

/// Errors that can occur in the MCP client
#[derive(Debug, Clone)]
pub enum ClientError {
    /// Client is not connected to the server
    NotConnected(String),
    
    /// Request timed out
    Timeout(String),
    
    /// Response channel was closed
    ResponseChannelClosed(String),
    
    /// Failed to serialize or deserialize a message
    SerializationError(String),
    
    /// Failed to connect to server
    ConnectionFailed(String),
    
    /// Invalid message received
    InvalidMessage(String),
    
    /// Client is already connected
    AlreadyConnected(String),
    
    /// Error received from remote endpoint
    RemoteError(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::NotConnected(msg) => write!(f, "Client not connected: {}", msg),
            ClientError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            ClientError::ResponseChannelClosed(msg) => write!(f, "Response channel closed: {}", msg),
            ClientError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ClientError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            ClientError::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            ClientError::AlreadyConnected(msg) => write!(f, "Already connected: {}", msg),
            ClientError::RemoteError(msg) => write!(f, "Remote error: {}", msg),
        }
    }
}

impl Error for ClientError {} 