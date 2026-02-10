// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Simplified Wire Format Adapter
//! 
//! Provides essential wire format types for protocol compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// Essential wire format types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireProtocolVersion {
    V0_9,
    V1_0,
    Latest,
}

impl WireProtocolVersion {
    pub fn from_str(s: &str) -> Result<Self, WireFormatError> {
        match s {
            "0.9" => Ok(Self::V0_9),
            "1.0" => Ok(Self::V1_0),
            "latest" => Ok(Self::Latest),
            _ => Err(WireFormatError::InvalidVersion(s.to_string())),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            Self::V0_9 => "0.9".to_string(),
            Self::V1_0 => "1.0".to_string(),
            Self::Latest => "latest".to_string(),
        }
    }
}

impl Default for WireProtocolVersion {
    fn default() -> Self {
        Self::V1_0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
    pub id: String,
    pub version: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Default for WireMessage {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            version: "1.0".to_string(),
            message_type: "command".to_string(),
            payload: serde_json::json!({}),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WireFormatError {
    InvalidVersion(String),
    SerializationFailed(String),
    DeserializationFailed(String),
    InvalidFormat(String),
}

impl std::fmt::Display for WireFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireFormatError::InvalidVersion(v) => write!(f, "Invalid wire protocol version: {}", v),
            WireFormatError::SerializationFailed(e) => write!(f, "Serialization failed: {}", e),
            WireFormatError::DeserializationFailed(e) => write!(f, "Deserialization failed: {}", e),
            WireFormatError::InvalidFormat(e) => write!(f, "Invalid wire format: {}", e),
        }
    }
}

impl std::error::Error for WireFormatError {}

// Wire format trait
pub trait WireFormat {
    fn to_wire(&self) -> Result<WireMessage, WireFormatError>;
    fn from_wire(wire: WireMessage) -> Result<Self, WireFormatError> where Self: Sized;
}

// Domain object trait
pub trait DomainObject {
    fn to_wire_message(&self, version: WireProtocolVersion) -> Result<WireMessage, WireFormatError>;
    fn from_wire_message(wire: WireMessage) -> Result<Self, WireFormatError> where Self: Sized;
}

// Simplified adapter types
pub struct WireFormatAdapter;
pub struct WireFormatConfig {
    pub version: WireProtocolVersion,
    pub compression: bool,
}

impl Default for WireFormatConfig {
    fn default() -> Self {
        Self {
            version: WireProtocolVersion::default(),
            compression: false,
        }
    }
} 