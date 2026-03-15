// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

// Define helper structs for serialization/deserialization

use crate::error::ProtocolError;
use crate::protocol::adapter_wire;
use crate::protocol::types::{MCPMessage, MessageId, SecurityMetadata, EncryptionInfo, SecurityLevel};
// BearDog handles security: // use crate::security::types::{EncryptionInfo, SecurityLevel, SecurityMetadata};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;
// Import Base64 engine
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use std::collections::HashMap;
use crate::protocol::adapter_wire::{WireProtocolVersion, WireFormatError};


// --- Helper Structs Definition ---
#[derive(Deserialize, Debug)]
pub(crate) struct MCPMessageDefinitionHelper {
    pub(crate) id: String, // Use String directly as MessageId has Serialize/Deserialize issues
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) version: adapter_wire::WireProtocolVersion, // Use WireProtocolVersion
    #[serde(rename = "type_")]
    pub(crate) type_: String, // Deserialize as string first
    pub(crate) payload: Value,
    pub(crate) metadata: Value, // Store metadata as Value
    #[serde(default)]
    pub(crate) security: Option<SecurityMetadataHelper>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct SecurityMetadataHelper {
    pub(crate) security_level: Option<String>,
    pub(crate) encryption_info: Option<EncryptionInfoHelper>,
    pub(crate) signature: Option<String>,
    pub(crate) auth_token: Option<String>,
    pub(crate) permissions: Option<Vec<String>>,
    pub(crate) roles: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct EncryptionInfoHelper {
    pub(crate) format: String, // Deserialize as string first
    pub(crate) key_id: Option<String>,
    pub(crate) iv: Option<String>,
    pub(crate) aad: Option<String>,
}

// -- TryFrom Implementations for Helpers --

impl TryFrom<MCPMessageDefinitionHelper> for MCPMessage {
    type Error = ProtocolError;

    fn try_from(helper: MCPMessageDefinitionHelper) -> std::result::Result<Self, Self::Error> {
        let security_metadata: SecurityMetadata = helper
            .security
            .map(SecurityMetadata::try_from)
            .transpose()? // Convert Option<Result<T, E>> -> Result<Option<T>, E>
            .unwrap_or_default();

        Ok(MCPMessage {
            id: MessageId(helper.id), // Assuming helper.id is String
            timestamp: helper.timestamp,
            version: helper.version.try_into()?, // Propagates ProtocolError
            type_: helper.type_.try_into()?,     // Propagates ProtocolError
            payload: helper.payload,
            metadata: Some(helper.metadata), // Wrap in Some()
            security: security_metadata,
            trace_id: None,
        })
    }
}


impl TryFrom<EncryptionInfoHelper> for EncryptionInfo { 
    type Error = ProtocolError;

    fn try_from(helper: EncryptionInfoHelper) -> std::result::Result<Self, Self::Error> {
        // Decode Base64 strings for iv and aad
        let iv_bytes = helper.iv.map(|s| BASE64_STANDARD.decode(s)).transpose()
            .map_err(|e| ProtocolError::ValidationFailed(format!("Failed to decode base64 iv: {}", e)))?;
        let aad_bytes = helper.aad.map(|s| BASE64_STANDARD.decode(s)).transpose()
            .map_err(|e| ProtocolError::ValidationFailed(format!("Failed to decode base64 aad: {}", e)))?;
            
        Ok(EncryptionInfo {
            format: helper.format.to_string(), // Convert directly to string
            key_id: helper.key_id,
            iv: iv_bytes,
            aad: aad_bytes,
        })
    }
}


impl TryFrom<SecurityMetadataHelper> for SecurityMetadata {
    type Error = ProtocolError;

    fn try_from(helper: SecurityMetadataHelper) -> std::result::Result<Self, Self::Error> {
        Ok(SecurityMetadata {
            security_level: SecurityLevel::try_from(helper.security_level)
                .unwrap_or(SecurityLevel::Low),
            encryption_info: helper.encryption_info,
            signature: helper.signature,
            auth_token: helper.auth_token,
            permissions: helper.permissions.unwrap_or_default(),
            roles: helper.roles,
        })
    }
} 