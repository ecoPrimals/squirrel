// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Request and response types for ecosystem communication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::context::PrimalContext;
use super::security::SecurityContext;

/// Standardized request format for all ecosystem communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRequest {
    /// Unique request identifier
    pub request_id: Uuid,

    /// Source service identifier (`Arc<str>` for O(1) clone when shared)
    pub source_service: Arc<str>,

    /// Target service identifier (`Arc<str>` for O(1) clone when shared)
    pub target_service: Arc<str>,

    /// Request operation (`Arc<str>` for O(1) clone when shared)
    pub operation: Arc<str>,

    /// Request payload
    pub payload: serde_json::Value,

    /// Security context
    pub security_context: SecurityContext,

    /// Request metadata
    pub metadata: HashMap<String, String>,

    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Standardized response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemResponse {
    /// Request ID this response is for
    pub request_id: Uuid,

    /// Response status
    pub status: ResponseStatus,

    /// Response payload
    pub payload: serde_json::Value,

    /// Response metadata
    pub metadata: HashMap<String, String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// Response status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResponseStatus {
    /// Request completed successfully
    Success,
    /// Request failed with error
    Error {
        /// Error code identifier
        code: Arc<str>,
        /// Human-readable error message
        message: String,
    },
    /// Request timed out
    Timeout,
    /// Target service is unavailable
    ServiceUnavailable,
}

/// Primal request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Request ID
    pub id: Uuid,

    /// Operation to perform (`Arc<str>` for O(1) clone when shared)
    pub operation: Arc<str>,

    /// Request payload
    pub payload: serde_json::Value,

    /// Request context
    pub context: PrimalContext,

    /// Security context
    pub security_context: SecurityContext,

    /// Request metadata
    pub metadata: HashMap<String, String>,

    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Primal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request ID this response is for
    pub request_id: Uuid,

    /// Response status
    pub status: ResponseStatus,

    /// Response payload
    pub payload: serde_json::Value,

    /// Response metadata
    pub metadata: HashMap<String, String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

impl Default for PrimalResponse {
    fn default() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            status: ResponseStatus::Success,
            payload: serde_json::Value::Null,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
}

impl Default for EcosystemRequest {
    fn default() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            source_service: Arc::from("unknown"),
            target_service: Arc::from("unknown"),
            operation: Arc::from("unknown"),
            payload: serde_json::Value::Null,
            security_context: SecurityContext::default(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
}
