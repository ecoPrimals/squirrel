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
            Self::NotConnected(msg) => write!(f, "Client not connected: {msg}"),
            Self::Timeout(msg) => write!(f, "Timeout: {msg}"),
            Self::ResponseChannelClosed(msg) => write!(f, "Response channel closed: {msg}"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
            Self::InvalidMessage(msg) => write!(f, "Invalid message: {msg}"),
            Self::AlreadyConnected(msg) => write!(f, "Already connected: {msg}"),
            Self::RemoteError(msg) => write!(f, "Remote error: {msg}"),
        }
    }
}

impl Error for ClientError {} 