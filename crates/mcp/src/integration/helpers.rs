//! Helper functions for the integration module.

use crate::error::MCPError;
use crate::protocol::{MCPMessage, MessageType};
use crate::types::{MCPResponse, ResponseStatus, MessageMetadata};
use crate::security::Credentials;
use serde_json::{Value, json};
use tracing::error;

/// Helper function to determine if an error should trigger circuit breaking
pub const fn is_circuit_breaking_error(error: &MCPError) -> bool {
    matches!(error, 
        MCPError::Connection(_) | 
        MCPError::Network(_) |
        MCPError::Transport(_)
    )
}

/// Helper function to extract credentials (assuming payload structure or metadata)
pub fn extract_credentials(message: &MCPMessage) -> Option<Credentials> {
    // Example: Try extracting from a specific metadata field first
    if let Some(sec_meta) = message.metadata.as_ref().and_then(|m| m.get("security")) {
        if let Ok(creds) = serde_json::from_value(sec_meta.clone()) {
            return Some(creds);
        }
    }
    // Example: Fallback to checking the payload itself if it's a specific command
    if message.type_ == MessageType::Command {
         // --- Use from_value for payload --- 
        if let Ok(payload_val) = serde_json::from_value::<Value>(message.payload.clone()) { // Assuming payload is Value
            if payload_val.get("command") == Some(&Value::String("authenticate".to_string())) {
                if let Some(creds_val) = payload_val.get("credentials") {
                    if let Ok(creds) = serde_json::from_value(creds_val.clone()) {
                        return Some(creds);
                    }
                }
            }
         // --- End Use --- 
        }
    }
    None
}

/// Creates a successful response message based on the MCPResponse structure from types.rs
pub fn create_success_response(message: &MCPMessage, payload: &Value) -> MCPResponse {
    // Serialize payload to bytes
    let payload_bytes = match serde_json::to_vec(payload) { // Use to_vec for Vec<u8>
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to serialize success payload: {}", e);
            // Return an error response instead?
            // For now, returning an empty payload and logging error.
            Vec::new()
        }
    };

    MCPResponse {
        protocol_version: message.version.version_string(), // Use version.version_string()
        message_id: message.id.0.clone(), // Use id.0
        status: ResponseStatus::Success, // Use ResponseStatus enum
        payload: payload_bytes, // Use Vec<u8>
        metadata: MessageMetadata::default(), // Keep or enhance metadata
        error_message: None, // No error message for success
    }
}

/// Creates an error response message based on the MCPResponse structure from types.rs
pub fn create_error_response(message: &MCPMessage, error: MCPError) -> Option<MCPResponse> {
    error!(message_id = %message.id.0, error = %error, "Creating error response");
    let error_string = error.to_string();
    let error_details = json!({ "message": error_string, "code": error.code_str() }); // Example details

    Some(MCPResponse {
        protocol_version: message.version.version_string(), // Add correct field name
        message_id: message.id.clone(), // Use correct message_id field
        status: ResponseStatus::Error,
        payload: serde_json::to_vec(&error_details).unwrap_or_default(),
        error_message: Some(error_string),
        metadata: MessageMetadata::default(), // Add appropriate metadata
    })
} 