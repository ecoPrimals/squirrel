// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_response_serde() {
        let response = PrimalResponse {
            request_id: Uuid::new_v4(),
            response_type: PrimalResponseType::Authentication,
            payload: {
                let mut m = HashMap::new();
                m.insert("token".to_string(), serde_json::json!("abc123"));
                m
            },
            timestamp: Utc::now(),
            success: true,
            error_message: None,
            metadata: Some({
                let mut m = HashMap::new();
                m.insert("provider".to_string(), "local".to_string());
                m
            }),
        };
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PrimalResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.request_id, response.request_id);
        assert!(deserialized.success);
        assert!(deserialized.error_message.is_none());
    }

    #[test]
    fn test_primal_response_error() {
        let response = PrimalResponse {
            request_id: Uuid::new_v4(),
            response_type: PrimalResponseType::Encryption,
            payload: HashMap::new(),
            timestamp: Utc::now(),
            success: false,
            error_message: Some("key not found".to_string()),
            metadata: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PrimalResponse = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.success);
        assert_eq!(deserialized.error_message.unwrap(), "key not found");
    }

    #[test]
    fn test_primal_response_type_serde() {
        let types = vec![
            PrimalResponseType::Authentication,
            PrimalResponseType::Encryption,
            PrimalResponseType::Decryption,
            PrimalResponseType::Authorization,
            PrimalResponseType::Audit,
            PrimalResponseType::ThreatDetection,
            PrimalResponseType::HealthCheck,
            PrimalResponseType::Storage,
            PrimalResponseType::Retrieval,
            PrimalResponseType::Compute,
            PrimalResponseType::Inference,
            PrimalResponseType::Custom("stream".to_string()),
        ];
        for rt in types {
            let json = serde_json::to_string(&rt).unwrap();
            let deserialized: PrimalResponseType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, rt);
        }
    }
}
