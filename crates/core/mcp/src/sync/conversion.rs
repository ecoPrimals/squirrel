// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Conversion functions for sync protocol types (StateChange <-> ProtoContextChange)
//! and JSON-RPC sync transport.

use crate::MCPError;
use crate::sync::json_rpc_types::{ProtoContextChange, ProtoOperationType, SyncRequest, SyncResponse};
use crate::sync::state::{StateChange, StateOperation as InternalStateOperation};
use chrono::{Utc, TimeZone};
use uuid::Uuid;

/// Converts a Rust StateChange object to a Protocol Buffer ContextChange object
///
/// This function handles serialization of the Rust `StateChange` object to the Protocol Buffer format
/// used for network transmission. It converts the operation type, serializes JSON data and metadata
/// to binary, and handles timestamp conversion.
///
/// # Arguments
/// * `change` - The StateChange to convert
///
/// # Returns
/// * `Ok(ProtoContextChange)` if conversion was successful
/// * `Err(MCPError::Serialization)` if JSON serialization fails
pub(crate) fn state_change_to_proto(
    change: &StateChange,
) -> std::result::Result<ProtoContextChange, MCPError> {
    // Map the operation type
    let operation_type = match change.operation {
        InternalStateOperation::Create => ProtoOperationType::Create as i32,
        InternalStateOperation::Update => ProtoOperationType::Update as i32,
        InternalStateOperation::Delete => ProtoOperationType::Delete as i32,
        InternalStateOperation::Sync => ProtoOperationType::Unspecified as i32,
    };

    // Convert data to bytes
    let data_bytes = serde_json::to_vec(&change.data)
        .map_err(|e| MCPError::InvalidArgument(format!("Failed to serialize data: {}", e)))?;

    // Convert metadata to bytes if available
    let metadata_bytes = match &change.metadata {
        Some(metadata) => serde_json::to_vec(metadata)
            .map_err(|e| MCPError::InvalidArgument(format!("Failed to serialize metadata: {}", e)))?,
        None => Vec::new(),
    };

    // Build the proto change
    Ok(ProtoContextChange {
        operation_type,
        context_id: change.context_id.to_string(),
        name: change.name.clone().unwrap_or_default(),
        parent_id: change.parent_id.map_or_else(String::new, |id| id.to_string()),
        created_at_unix_secs: change.timestamp.timestamp(),
        updated_at_unix_secs: change.timestamp.timestamp(),
        data: data_bytes,
        metadata: metadata_bytes,
    })
}

/// Converts a Protocol Buffer ContextChange object to a Rust StateChange object
///
/// This function handles deserialization of the Protocol Buffer format to the Rust `StateChange`
/// object used internally. It parses the operation type, deserializes JSON data and metadata.
///
/// # Arguments
/// * `proto` - The ProtoContextChange to convert
///
/// # Returns
/// * `Ok(StateChange)` if conversion was successful
/// * `Err(MCPError::Deserialization)` if parsing of any values fails
pub(crate) fn proto_to_state_change(
    proto: ProtoContextChange,
) -> std::result::Result<StateChange, MCPError> {
    // Parse the context ID
    let context_id = Uuid::parse_str(&proto.context_id)
        .map_err(|e| MCPError::InvalidArgument(format!("Invalid context ID: {}", e)))?;

    // Parse the parent ID if present
    let parent_id = if !proto.parent_id.is_empty() {
        Some(
            Uuid::parse_str(&proto.parent_id)
                .map_err(|e| MCPError::InvalidArgument(format!("Invalid parent ID: {}", e)))?,
        )
    } else {
        None
    };

    // Parse the operation type
    let operation = match proto.operation_type {
        x if x == ProtoOperationType::Create as i32 => InternalStateOperation::Create,
        x if x == ProtoOperationType::Update as i32 => InternalStateOperation::Update,
        x if x == ProtoOperationType::Delete as i32 => InternalStateOperation::Delete,
        _ => InternalStateOperation::Sync,
    };

    // Parse the data
    let data = if !proto.data.is_empty() {
        serde_json::from_slice(&proto.data)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid data format: {}", e)))?
    } else {
        serde_json::Value::Null
    };

    // Parse the metadata if present
    let metadata = if !proto.metadata.is_empty() {
        Some(
            serde_json::from_slice(&proto.metadata)
                .map_err(|e| MCPError::InvalidArgument(format!("Invalid metadata format: {}", e)))?,
        )
    } else {
        None
    };

    // Use the timestamps from the proto
    let timestamp = if proto.created_at_unix_secs > 0 {
        match Utc.timestamp_opt(proto.created_at_unix_secs, 0) {
            chrono::LocalResult::Single(ts) => ts,
            _ => Utc::now(),
        }
    } else {
        Utc::now()
    };

    // Build the state change
    Ok(StateChange {
        id: Uuid::new_v4(),
        context_id,
        operation,
        data,
        timestamp,
        version: 0, // Version will be assigned by the state manager
        name: if proto.name.is_empty() {
            None
        } else {
            Some(proto.name)
        },
        metadata,
        parent_id,
    })
}

/// JSON-RPC sync call over Unix socket
///
/// # Arguments
/// * `socket_path` - Path to the sync server Unix socket (e.g., "/tmp/mcp-sync.sock")
/// * `request` - The sync request to send
///
/// # Returns
/// A Result containing the SyncResponse, or an error
pub(crate) async fn sync_json_rpc_call(
    socket_path: &str,
    request: &SyncRequest,
) -> std::result::Result<SyncResponse, MCPError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    let stream = UnixStream::connect(socket_path).await.map_err(|e| {
        MCPError::Connection(crate::error::ConnectionError::ConnectionFailed(format!(
            "Failed to connect to sync server at {}: {}",
            socket_path, e
        )))
    })?;

    let json_rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sync",
        "params": serde_json::to_value(request).map_err(|e| MCPError::Serialization(e.to_string()))?
    });

    let request_bytes =
        serde_json::to_vec(&json_rpc_request).map_err(|e| MCPError::Serialization(e.to_string()))?;

    let (mut reader, mut writer) = stream.into_split();
    writer
        .write_all(&request_bytes)
        .await
        .map_err(|e| MCPError::Io(e.to_string()))?;
    writer
        .flush()
        .await
        .map_err(|e| MCPError::Io(e.to_string()))?;
    drop(writer);

    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .await
        .map_err(|e| MCPError::Io(e.to_string()))?;

    let response: serde_json::Value = serde_json::from_slice(&buf)
        .map_err(|e| MCPError::Deserialization(format!("Invalid JSON-RPC response: {}", e)))?;

    let result = response.get("result").ok_or_else(|| {
        if let Some(err) = response.get("error") {
            let msg = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            MCPError::Remote(msg.to_string())
        } else {
            MCPError::Deserialization("Missing result in JSON-RPC response".to_string())
        }
    })?;

    let sync_response: SyncResponse = serde_json::from_value(result.clone())
        .map_err(|e| MCPError::Deserialization(format!("Invalid sync response: {}", e)))?;

    Ok(sync_response)
}
