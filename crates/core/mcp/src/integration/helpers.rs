// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Helper functions for the integration module.

use crate::error::MCPError;
use crate::protocol::types::{MCPMessage, MessageType};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::protocol::types::AuthCredentials;
use crate::types::{MCPResponse, ResponseStatus, MessageMetadata};
 // Added direct import
use serde_json::{Value, json};
use tracing::{error, debug};

/// Helper function to determine if an error should trigger circuit breaking
pub const fn is_circuit_breaking_error(error: &MCPError) -> bool {
    matches!(error, 
        MCPError::Connection(_) | 
        MCPError::Network(_) |
        MCPError::Transport(_)
    )
}

/// Helper function to extract credentials (assuming payload structure or metadata)
pub fn extract_credentials(message: &MCPMessage) -> Option<AuthCredentials> {
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
    debug!(message_id = %message.id.0, "Creating success response");
    // Keep the payload as Value, wrap in a Vec for the MCPResponse field
    let response_payload = vec![payload.clone()];

    MCPResponse {
        protocol_version: message.version.version_string(),
        message_id: message.id.clone(),
        status: ResponseStatus::Success,
        payload: response_payload,
        metadata: MessageMetadata::default(),
        error_message: None,
    }
}

/// Creates an error response message based on the MCPResponse structure from types.rs
pub fn create_error_response(message: &MCPMessage, error: MCPError) -> MCPResponse {
    error!(message_id = %message.id.0, error = %error, "Creating error response");
    let error_string = error.to_string();
    // Create the error details as a Value
    let error_details_value = json!({ "message": error_string.clone(), "code": error.code_str() }); 

    // Return MCPResponse directly, removed Some()
    MCPResponse {
        protocol_version: message.version.version_string(), // Add correct field name
        message_id: message.id.clone(), // Use id directly, it's already MessageId
        status: ResponseStatus::Error,
        // Wrap the Value in a vec![] for the payload field
        payload: vec![error_details_value], 
        error_message: Some(error_string),
        metadata: MessageMetadata::default(), // Add appropriate metadata
    }
} 