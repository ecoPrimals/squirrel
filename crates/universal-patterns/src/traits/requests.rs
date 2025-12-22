//! Primal request types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Universal request structure for primal services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Unique identifier for this request
    pub id: Uuid,
    /// Type of request being made
    pub request_type: PrimalRequestType,
    /// Request payload data
    pub payload: HashMap<String, serde_json::Value>,
    /// Timestamp when request was created
    pub timestamp: DateTime<Utc>,
    /// User context making the request
    pub context: Option<String>,
    /// Priority level for request processing
    pub priority: Option<u8>,
    /// Security classification of the request
    pub security_level: Option<String>,
}

/// Types of requests that can be made to primals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalRequestType {
    /// Authentication request
    Authenticate,
    /// Encryption request
    Encrypt,
    /// Decryption request
    Decrypt,
    /// Authorization check request
    Authorize,
    /// Audit logging request
    AuditLog,
    /// Threat detection request
    ThreatDetection,
    /// Health check request
    HealthCheck,
    /// Store data request
    Store,
    /// Retrieve data request
    Retrieve,
    /// Compute request
    Compute,
    /// AI inference request
    Infer,
    /// Custom request type
    Custom(String),
}
