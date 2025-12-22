//! Primal response types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Universal response structure from primal services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request ID this response corresponds to
    pub request_id: Uuid,
    /// Type of response being returned
    pub response_type: PrimalResponseType,
    /// Response payload data
    pub payload: HashMap<String, serde_json::Value>,
    /// Timestamp when response was created
    pub timestamp: DateTime<Utc>,
    /// Whether the request was successful
    pub success: bool,
    /// Error message if request failed
    pub error_message: Option<String>,
    /// Additional metadata about the response
    pub metadata: Option<HashMap<String, String>>,
}

/// Types of responses that can be returned from primals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalResponseType {
    /// Authentication response
    Authentication,
    /// Encryption response
    Encryption,
    /// Decryption response
    Decryption,
    /// Authorization response
    Authorization,
    /// Audit response
    Audit,
    /// Threat detection response
    ThreatDetection,
    /// Health check response
    HealthCheck,
    /// Storage response
    Storage,
    /// Retrieval response
    Retrieval,
    /// Compute response
    Compute,
    /// AI inference response
    Inference,
    /// Custom response type
    Custom(String),
}
