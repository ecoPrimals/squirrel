//! Error types specific to WebSocket operations.

use thiserror::Error;

/// Errors that can occur in WebSocket operations
#[derive(Debug, Error)]
pub enum WebSocketError {
    /// Invalid WebSocket command format
    #[error("Invalid command format: {0}")]
    InvalidCommand(String),
    
    /// Unknown command received
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    
    /// Missing required parameter
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    
    /// Invalid parameter type
    #[error("Invalid parameter type: {0}")]
    InvalidParameterType(String),
    
    /// Unauthorized access to a channel or resource
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    /// Subscription error
    #[error("Subscription error: {0}")]
    SubscriptionError(String),
    
    /// Error sending a WebSocket message
    #[error("Send error: {0}")]
    SendError(String),
    
    /// Unsupported message type
    #[error("Unsupported message type: {0}")]
    UnsupportedMessageType(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

// Manual implementation of Clone since serde_json::Error doesn't implement Clone
impl Clone for WebSocketError {
    fn clone(&self) -> Self {
        match self {
            Self::InvalidCommand(s) => Self::InvalidCommand(s.clone()),
            Self::UnknownCommand(s) => Self::UnknownCommand(s.clone()),
            Self::MissingParameter(s) => Self::MissingParameter(s.clone()),
            Self::InvalidParameterType(s) => Self::InvalidParameterType(s.clone()),
            Self::Unauthorized(s) => Self::Unauthorized(s.clone()),
            Self::SubscriptionError(s) => Self::SubscriptionError(s.clone()),
            Self::SendError(s) => Self::SendError(s.clone()),
            Self::UnsupportedMessageType(s) => Self::UnsupportedMessageType(s.clone()),
            Self::Internal(s) => Self::Internal(s.clone()),
            Self::JsonError(e) => Self::JsonError(serde_json::Error::io(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            )),
        }
    }
}

impl WebSocketError {
    /// Get the error code for this error
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidCommand(_) => "INVALID_COMMAND",
            Self::UnknownCommand(_) => "UNKNOWN_COMMAND",
            Self::MissingParameter(_) => "MISSING_PARAMETER",
            Self::InvalidParameterType(_) => "INVALID_PARAMETER_TYPE",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::SubscriptionError(_) => "SUBSCRIPTION_ERROR",
            Self::SendError(_) => "SEND_ERROR",
            Self::UnsupportedMessageType(_) => "UNSUPPORTED_MESSAGE_TYPE",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::JsonError(_) => "JSON_ERROR",
        }
    }
} 