// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

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
    pub request_id: Uuid,
    pub source_primal: String,
    pub target_capability: String,
    pub operation: String,
    pub payload: Value,
    pub context: PrimalContext,
    pub timestamp: DateTime<Utc>,
    pub timeout_ms: Option<u64>,
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
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
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
    pub request_id: Uuid,
    pub response_id: Uuid,
    pub status: ResponseStatus,
    pub success: bool,
    pub data: Option<Value>,
    pub payload: Value,
    pub timestamp: DateTime<Utc>,
    pub processing_time_ms: Option<u64>,
    pub duration: Option<String>,
    pub error: Option<String>,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Response status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error,
    Timeout,
    NotFound,
}

/// Ecosystem-level request for cross-service communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRequest {
    pub request_id: Uuid,
    pub source_service: String,
    pub target_service: String,
    pub operation: String,
    pub payload: Value,
    pub security_context: UniversalSecurityContext,
    pub timestamp: DateTime<Utc>,
}

/// Ecosystem-level response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemResponse {
    pub request_id: Uuid,
    pub response_id: Uuid,
    pub status: ResponseStatus,
    pub success: bool,
    pub payload: Value,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
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
