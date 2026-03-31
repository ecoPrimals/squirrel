// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Inter-primal API request and response DTOs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::ecosystem::EcosystemPrimalType;

use super::arc_serde::{
    deserialize_arc_str, deserialize_arc_str_map, deserialize_optional_arc_str, serialize_arc_str,
    serialize_arc_str_map, serialize_optional_arc_str,
};
use super::interning::intern_registry_string;

/// Standard API request for inter-primal communication with ``Arc<str>`` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalApiRequest {
    /// Request ID as `Arc<str>` for efficient sharing across async boundaries
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub request_id: Arc<str>,
    /// Source primal type
    pub from_primal: EcosystemPrimalType,
    /// Target primal type
    pub to_primal: EcosystemPrimalType,
    /// Operation name as `Arc<str>` with string interning
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub operation: Arc<str>,
    /// Request payload
    pub payload: serde_json::Value,
    /// Headers with `Arc<str>` keys and values for zero-copy
    #[serde(
        serialize_with = "serialize_arc_str_map",
        deserialize_with = "deserialize_arc_str_map"
    )]
    pub headers: HashMap<Arc<str>, Arc<str>>,
    /// Request timeout
    pub timeout: Duration,
}

impl PrimalApiRequest {
    /// Create new `PrimalApiRequest` with string interning optimization
    #[must_use]
    pub fn new(
        request_id: &str,
        from_primal: EcosystemPrimalType,
        to_primal: EcosystemPrimalType,
        operation: &str,
        payload: serde_json::Value,
        headers: HashMap<&str, &str>,
        timeout: Duration,
    ) -> Self {
        Self {
            request_id: Arc::from(request_id),
            from_primal,
            to_primal,
            operation: intern_registry_string(operation),
            payload,
            headers: headers
                .into_iter()
                .map(|(k, v)| (intern_registry_string(k), Arc::from(v)))
                .collect(),
            timeout,
        }
    }

    /// Efficient header lookup without allocation
    #[must_use]
    pub fn get_header(&self, key: &str) -> Option<&Arc<str>> {
        self.headers
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }
}

/// Standard API response for inter-primal communication with ``Arc<str>`` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalApiResponse {
    /// Request ID as `Arc<str>` for efficient correlation
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub request_id: Arc<str>,
    /// Whether the request succeeded
    pub success: bool,
    /// Response data when successful
    pub data: Option<serde_json::Value>,
    /// Error message as `Arc<str>` for efficient sharing
    #[serde(
        serialize_with = "serialize_optional_arc_str",
        deserialize_with = "deserialize_optional_arc_str"
    )]
    pub error: Option<Arc<str>>,
    /// Headers with `Arc<str>` keys and values for zero-copy
    #[serde(
        serialize_with = "serialize_arc_str_map",
        deserialize_with = "deserialize_arc_str_map"
    )]
    pub headers: HashMap<Arc<str>, Arc<str>>,
    /// Time taken to process the request
    pub processing_time: Duration,
}

impl PrimalApiResponse {
    /// Create new `PrimalApiResponse` with string optimization
    pub fn new(
        request_id: Arc<str>,
        success: bool,
        data: Option<serde_json::Value>,
        error: Option<&str>,
        headers: HashMap<&str, &str>,
        processing_time: Duration,
    ) -> Self {
        Self {
            request_id,
            success,
            data,
            error: error.map(Arc::from),
            headers: headers
                .into_iter()
                .map(|(k, v)| (intern_registry_string(k), Arc::from(v)))
                .collect(),
            processing_time,
        }
    }
}
