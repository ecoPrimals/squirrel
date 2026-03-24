// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_request_serde() {
        let request = PrimalRequest {
            id: Uuid::new_v4(),
            request_type: PrimalRequestType::Authenticate,
            payload: {
                let mut m = HashMap::new();
                m.insert("username".to_string(), serde_json::json!("admin"));
                m
            },
            timestamp: Utc::now(),
            context: Some("user-session-1".to_string()),
            priority: Some(5),
            security_level: Some("high".to_string()),
        };
        let json = serde_json::to_string(&request).expect("should succeed");
        let deserialized: PrimalRequest = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.id, request.id);
        assert_eq!(deserialized.request_type, PrimalRequestType::Authenticate);
        assert_eq!(deserialized.priority, Some(5));
    }

    #[test]
    fn test_primal_request_type_serde() {
        let types = vec![
            PrimalRequestType::Authenticate,
            PrimalRequestType::Encrypt,
            PrimalRequestType::Decrypt,
            PrimalRequestType::Authorize,
            PrimalRequestType::AuditLog,
            PrimalRequestType::ThreatDetection,
            PrimalRequestType::HealthCheck,
            PrimalRequestType::Store,
            PrimalRequestType::Retrieve,
            PrimalRequestType::Compute,
            PrimalRequestType::Infer,
            PrimalRequestType::Custom("my-op".to_string()),
        ];
        for rt in types {
            let json = serde_json::to_string(&rt).expect("should succeed");
            let deserialized: PrimalRequestType =
                serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, rt);
        }
    }

    #[test]
    fn test_primal_request_minimal() {
        let request = PrimalRequest {
            id: Uuid::nil(),
            request_type: PrimalRequestType::HealthCheck,
            payload: HashMap::new(),
            timestamp: Utc::now(),
            context: None,
            priority: None,
            security_level: None,
        };
        let json = serde_json::to_string(&request).expect("should succeed");
        let deserialized: PrimalRequest = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.id, Uuid::nil());
        assert!(deserialized.context.is_none());
    }
}
