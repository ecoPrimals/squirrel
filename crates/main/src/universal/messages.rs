// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Message types for inter-primal communication
//!
//! This module defines request/response structures for communication
//! between primals within the ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use super::context::{PrimalContext, UniversalSecurityContext};

/// Request sent between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Unique request identifier
    pub request_id: Uuid,
    /// Source primal identifier
    pub source_primal: String,
    /// Target capability to invoke
    pub target_capability: String,
    /// Operation name
    pub operation: String,
    /// Request payload
    pub payload: Value,
    /// Request context
    pub context: PrimalContext,
    /// When the request was created
    pub timestamp: DateTime<Utc>,
    /// Optional timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl PrimalRequest {
    /// Create a new primal request
    pub fn new(
        source_primal: impl Into<String>,
        target_capability: impl Into<String>,
        operation: impl Into<String>,
        payload: Value,
        context: PrimalContext,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            source_primal: source_primal.into(),
            target_capability: target_capability.into(),
            operation: operation.into(),
            payload,
            context,
            timestamp: Utc::now(),
            timeout_ms: None,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the request
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set request timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Check if the request has timed out
    #[must_use]
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout_ms) = self.timeout_ms {
            let elapsed = Utc::now()
                .signed_duration_since(self.timestamp)
                .num_milliseconds();
            elapsed > timeout_ms as i64
        } else {
            false
        }
    }
}

/// Response from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request ID for correlation
    pub request_id: Uuid,
    /// Unique response identifier
    pub response_id: Uuid,
    /// Response status
    pub status: ResponseStatus,
    /// Whether the operation succeeded
    pub success: bool,
    /// Response data when successful
    pub data: Option<Value>,
    /// Response payload
    pub payload: Value,
    /// When the response was created
    pub timestamp: DateTime<Utc>,
    /// Processing time in milliseconds
    pub processing_time_ms: Option<u64>,
    /// Human-readable duration
    pub duration: Option<String>,
    /// Error code or identifier
    pub error: Option<String>,
    /// Error message when failed
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Response status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Request completed successfully
    Success,
    /// Request failed with error
    Error,
    /// Request timed out
    Timeout,
    /// Resource not found
    NotFound,
}

/// Ecosystem-level request for cross-service communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRequest {
    /// Unique request identifier
    pub request_id: Uuid,
    /// Source service identifier
    pub source_service: String,
    /// Target service identifier
    pub target_service: String,
    /// Operation name
    pub operation: String,
    /// Request payload
    pub payload: Value,
    /// Security context for the request
    pub security_context: UniversalSecurityContext,
    /// When the request was created
    pub timestamp: DateTime<Utc>,
}

/// Ecosystem-level response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemResponse {
    /// Request ID for correlation
    pub request_id: Uuid,
    /// Unique response identifier
    pub response_id: Uuid,
    /// Response status
    pub status: ResponseStatus,
    /// Whether the operation succeeded
    pub success: bool,
    /// Response payload
    pub payload: Value,
    /// Error message when failed
    pub error_message: Option<String>,
    /// When the response was created
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Create an ecosystem request
#[must_use]
pub fn create_ecosystem_request(
    source_service: &str,
    target_service: &str,
    operation: &str,
    payload: Value,
    security_context: UniversalSecurityContext,
) -> EcosystemRequest {
    EcosystemRequest {
        request_id: Uuid::new_v4(),
        source_service: source_service.to_string(),
        target_service: target_service.to_string(),
        operation: operation.to_string(),
        payload,
        security_context,
        timestamp: Utc::now(),
    }
}

/// Create a success response
#[must_use]
pub fn create_success_response(request_id: Uuid, payload: Value) -> EcosystemResponse {
    EcosystemResponse {
        request_id,
        response_id: Uuid::new_v4(),
        status: ResponseStatus::Success,
        success: true,
        payload,
        error_message: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    }
}

/// Create an error response
#[must_use]
pub fn create_error_response(request_id: Uuid, error_message: &str) -> EcosystemResponse {
    EcosystemResponse {
        request_id,
        response_id: Uuid::new_v4(),
        status: ResponseStatus::Error,
        success: false,
        payload: Value::Null,
        error_message: Some(error_message.to_string()),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal::context::{create_default_context, create_default_security_context};

    #[test]
    fn primal_request_new_builder_and_timeout() {
        let ctx = create_default_context("u", "d");
        let req = PrimalRequest::new("src", "cap", "op", Value::String("p".into()), ctx)
            .with_metadata("k", "v")
            .with_timeout(60_000);
        assert_eq!(req.source_primal, "src");
        assert_eq!(req.metadata.get("k").map(String::as_str), Some("v"));
        assert_eq!(req.timeout_ms, Some(60_000));
        assert!(!req.is_timed_out());
    }

    #[test]
    fn primal_request_timed_out_after_elapsed() {
        let ctx = create_default_context("u", "d");
        let mut req = PrimalRequest::new("s", "c", "o", Value::Null, ctx).with_timeout(0);
        req.timestamp = Utc::now() - chrono::Duration::milliseconds(2);
        assert!(req.is_timed_out());
    }

    #[test]
    fn ecosystem_request_and_response_helpers() {
        let sec = create_default_security_context("user");
        let req = create_ecosystem_request("a", "b", "op", Value::Bool(true), sec);
        let ok = create_success_response(req.request_id, Value::Number(1.into()));
        assert!(ok.success);
        assert_eq!(ok.status, ResponseStatus::Success);
        let err = create_error_response(req.request_id, "e");
        assert!(!err.success);
        assert_eq!(err.status, ResponseStatus::Error);
    }

    #[test]
    fn primal_and_ecosystem_message_serde() {
        let ctx = create_default_context("u", "d");
        let pr = PrimalRequest::new("s", "c", "o", Value::Null, ctx);
        let pj = serde_json::to_string(&pr).unwrap();
        let _: PrimalRequest = serde_json::from_str(&pj).unwrap();

        let sec = create_default_security_context("u");
        let er = create_ecosystem_request("x", "y", "z", Value::Null, sec);
        let ej = serde_json::to_string(&er).unwrap();
        let _: EcosystemRequest = serde_json::from_str(&ej).unwrap();
    }

    #[test]
    fn response_status_all_variants_roundtrip() {
        for status in [
            ResponseStatus::Success,
            ResponseStatus::Error,
            ResponseStatus::Timeout,
            ResponseStatus::NotFound,
        ] {
            let v = serde_json::to_value(&status).unwrap();
            let back: ResponseStatus = serde_json::from_value(v).unwrap();
            assert_eq!(back, status);
        }
    }
}
