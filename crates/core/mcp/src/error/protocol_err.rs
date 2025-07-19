use thiserror::Error;
// Commented out missing module:
// use crate::protocol::adapter_wire::WireFormatError;

/// Errors related to the MCP protocol
///
/// This enum represents various error conditions that can occur during protocol
/// operations, including version mismatches, invalid states, and message format errors.
#[derive(Debug, Clone, Error)]
pub enum ProtocolError {
    /// Error when the protocol version is invalid or incompatible
    #[error("Invalid protocol version: {0}")]
    InvalidVersion(String),

    /// Error when the protocol is in an invalid state for the requested operation
    #[error("Invalid protocol state: {0}")]
    InvalidState(String),

    /// Error when a message doesn't conform to the expected format
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    /// Error when protocol negotiation fails between endpoints
    #[error("Protocol negotiation failed: {0}")]
    NegotiationFailed(String),

    /// Error when the protocol handshake process fails
    #[error("Protocol handshake failed: {0}")]
    HandshakeFailed(String),

    /// Error when protocol synchronization cannot be established
    #[error("Protocol synchronization failed: {0}")]
    SyncFailed(String),

    /// Error when a requested protocol capability is not supported
    #[error("Protocol capability not supported: {0}")]
    UnsupportedCapability(String),

    /// Error related to protocol configuration settings
    #[error("Protocol configuration error: {0}")]
    ConfigurationError(String),

    /// Error when trying to initialize a protocol that's already initialized
    #[error("Protocol already initialized")]
    ProtocolAlreadyInitialized,

    /// Error when using a protocol that hasn't been initialized
    #[error("Protocol not initialized")]
    ProtocolNotInitialized,

    /// Error when the protocol is not in a ready state for the operation
    #[error("Protocol not ready")]
    ProtocolNotReady,

    /// Error when serializing protocol state
    #[error("Failed to serialize state: {0}")]
    StateSerialization(String),

    /// Error when deserializing protocol state
    #[error("Failed to deserialize state: {0}")]
    StateDeserialization(String),

    /// Error when a handler already exists for a message type
    #[error("Handler already exists for message type: {0}")]
    HandlerAlreadyExists(String),

    /// Error when no handler is found for a message type
    #[error("No handler found for message type: {0}")]
    HandlerNotFound(String),

    /// Error when a message payload is invalid
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),

    /// Error when a message exceeds the allowed size limit
    #[error("Message too large: {0}")]
    MessageTooLarge(String),

    /// Error when a message timestamp is invalid
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    /// Error when a message operation times out
    #[error("Message timeout: {0}")]
    MessageTimeout(String),

    /// Error when security metadata is invalid
    #[error("Invalid security metadata: {0}")]
    InvalidSecurityMetadata(String),

    /// Error when message validation fails
    #[error("Message validation failed: {0}")]
    ValidationFailed(String),

    /// Error when protocol recovery attempts fail
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),

    /// Error in the wire format encoding/decoding
    #[error("Wire format error: {0}")]
    Wire(String),

    /// Error reported by the remote peer
    #[error("Remote protocol error: {0}")]
    RemoteError(String),

    /// Error when serializing data
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error when deserializing data
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl From<String> for ProtocolError {
    fn from(msg: String) -> Self {
        ProtocolError::InvalidFormat(msg)
    }
}

impl From<&str> for ProtocolError {
    fn from(msg: &str) -> Self {
        ProtocolError::InvalidFormat(msg.to_string())
    }
}

// Comment out implementation for missing type
// impl From<WireFormatError> for ProtocolError {
//     fn from(error: WireFormatError) -> Self {
//         ProtocolError::WireFormatError(error.to_string())
//     }
// }
