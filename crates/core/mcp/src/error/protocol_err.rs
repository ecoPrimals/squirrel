// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use thiserror::Error;

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
        Self::InvalidFormat(msg)
    }
}

impl From<&str> for ProtocolError {
    fn from(msg: &str) -> Self {
        Self::InvalidFormat(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    // SPDX-License-Identifier: AGPL-3.0-only
    // Inline tests follow the pattern used in `context.rs` and `severity.rs`.

    use super::ProtocolError;
    use std::fmt::Write as _;

    #[test]
    fn protocol_error_display_invalid_version() {
        let err = ProtocolError::InvalidVersion("v".into());
        assert!(err.to_string().contains("version"));
    }

    #[test]
    fn protocol_error_display_invalid_state() {
        let err = ProtocolError::InvalidState("s".into());
        assert!(err.to_string().contains("state"));
    }

    #[test]
    fn protocol_error_display_invalid_format() {
        let err = ProtocolError::InvalidFormat("fmt".into());
        assert!(err.to_string().contains("format"));
    }

    #[test]
    fn protocol_error_display_negotiation_failed() {
        let err = ProtocolError::NegotiationFailed("n".into());
        assert!(err.to_string().contains("negotiation"));
    }

    #[test]
    fn protocol_error_display_handshake_failed() {
        let err = ProtocolError::HandshakeFailed("h".into());
        assert!(err.to_string().contains("handshake"));
    }

    #[test]
    fn protocol_error_display_sync_failed() {
        let err = ProtocolError::SyncFailed("sync".into());
        assert!(err.to_string().contains("synchronization"));
    }

    #[test]
    fn protocol_error_display_unsupported_capability() {
        let err = ProtocolError::UnsupportedCapability("cap".into());
        assert!(err.to_string().contains("capability"));
    }

    #[test]
    fn protocol_error_display_configuration_error() {
        let err = ProtocolError::ConfigurationError("cfg".into());
        assert!(err.to_string().contains("configuration"));
    }

    #[test]
    fn protocol_error_display_protocol_already_initialized() {
        let err = ProtocolError::ProtocolAlreadyInitialized;
        assert!(err.to_string().contains("already initialized"));
    }

    #[test]
    fn protocol_error_display_protocol_not_initialized() {
        let err = ProtocolError::ProtocolNotInitialized;
        assert!(err.to_string().contains("not initialized"));
    }

    #[test]
    fn protocol_error_display_protocol_not_ready() {
        let err = ProtocolError::ProtocolNotReady;
        assert!(err.to_string().contains("not ready"));
    }

    #[test]
    fn protocol_error_display_state_serialization() {
        let err = ProtocolError::StateSerialization("ser".into());
        assert!(err.to_string().contains("serialize"));
    }

    #[test]
    fn protocol_error_display_state_deserialization() {
        let err = ProtocolError::StateDeserialization("de".into());
        assert!(err.to_string().contains("deserialize"));
    }

    #[test]
    fn protocol_error_display_handler_already_exists() {
        let err = ProtocolError::HandlerAlreadyExists("m".into());
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn protocol_error_display_handler_not_found() {
        let err = ProtocolError::HandlerNotFound("m".into());
        assert!(err.to_string().contains("No handler"));
    }

    #[test]
    fn protocol_error_display_invalid_payload() {
        let err = ProtocolError::InvalidPayload("p".into());
        assert!(err.to_string().contains("payload"));
    }

    #[test]
    fn protocol_error_display_message_too_large() {
        let err = ProtocolError::MessageTooLarge("big".into());
        assert!(err.to_string().contains("large"));
    }

    #[test]
    fn protocol_error_display_invalid_timestamp() {
        let err = ProtocolError::InvalidTimestamp("ts".into());
        assert!(err.to_string().contains("timestamp"));
    }

    #[test]
    fn protocol_error_display_message_timeout() {
        let err = ProtocolError::MessageTimeout("to".into());
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn protocol_error_display_invalid_security_metadata() {
        let err = ProtocolError::InvalidSecurityMetadata("sec".into());
        assert!(err.to_string().contains("security"));
    }

    #[test]
    fn protocol_error_display_validation_failed() {
        let err = ProtocolError::ValidationFailed("val".into());
        assert!(err.to_string().contains("validation"));
    }

    #[test]
    fn protocol_error_display_recovery_failed() {
        let err = ProtocolError::RecoveryFailed("rec".into());
        assert!(err.to_string().contains("Recovery"));
    }

    #[test]
    fn protocol_error_display_wire() {
        let err = ProtocolError::Wire("w".into());
        assert!(err.to_string().contains("Wire"));
    }

    #[test]
    fn protocol_error_display_remote_error() {
        let err = ProtocolError::RemoteError("peer".into());
        assert!(err.to_string().contains("Remote"));
    }

    #[test]
    fn protocol_error_display_serialization_error() {
        let err = ProtocolError::SerializationError("ser".into());
        assert!(err.to_string().contains("Serialization"));
    }

    #[test]
    fn protocol_error_display_deserialization_error() {
        let err = ProtocolError::DeserializationError("de".into());
        assert!(err.to_string().contains("Deserialization"));
    }

    #[test]
    fn protocol_error_debug_all_variants() {
        let cases: Vec<ProtocolError> = vec![
            ProtocolError::InvalidVersion("a".into()),
            ProtocolError::InvalidState("b".into()),
            ProtocolError::InvalidFormat("c".into()),
            ProtocolError::NegotiationFailed("d".into()),
            ProtocolError::HandshakeFailed("e".into()),
            ProtocolError::SyncFailed("f".into()),
            ProtocolError::UnsupportedCapability("g".into()),
            ProtocolError::ConfigurationError("h".into()),
            ProtocolError::ProtocolAlreadyInitialized,
            ProtocolError::ProtocolNotInitialized,
            ProtocolError::ProtocolNotReady,
            ProtocolError::StateSerialization("i".into()),
            ProtocolError::StateDeserialization("j".into()),
            ProtocolError::HandlerAlreadyExists("k".into()),
            ProtocolError::HandlerNotFound("l".into()),
            ProtocolError::InvalidPayload("m".into()),
            ProtocolError::MessageTooLarge("n".into()),
            ProtocolError::InvalidTimestamp("o".into()),
            ProtocolError::MessageTimeout("p".into()),
            ProtocolError::InvalidSecurityMetadata("q".into()),
            ProtocolError::ValidationFailed("r".into()),
            ProtocolError::RecoveryFailed("s".into()),
            ProtocolError::Wire("t".into()),
            ProtocolError::RemoteError("u".into()),
            ProtocolError::SerializationError("v".into()),
            ProtocolError::DeserializationError("w".into()),
        ];
        assert_eq!(cases.len(), 26);
        for e in cases {
            let mut buf = String::new();
            write!(&mut buf, "{e:?}").expect("format");
            assert!(!buf.is_empty());
        }
    }

    #[test]
    fn protocol_error_from_string() {
        let err: ProtocolError = "oops".to_string().into();
        assert!(matches!(err, ProtocolError::InvalidFormat(s) if s == "oops"));
    }

    #[test]
    fn protocol_error_from_str_slice() {
        let err: ProtocolError = "slice".into();
        assert!(matches!(err, ProtocolError::InvalidFormat(s) if s == "slice"));
    }
}
