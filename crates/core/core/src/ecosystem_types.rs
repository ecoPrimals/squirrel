// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-oriented helper types for ecosystem service metadata (runtime deserialization).

use std::collections::HashMap;

/// Serialized service registration payload (ecosystem registry / Songbird).
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ServiceRegistration {
    /// Stable service identifier.
    pub service_id: String,
    /// Reported primal type name.
    pub primal_type: String,
    /// Primary endpoint URL or socket.
    pub endpoint: String,
    /// Advertised capability ids.
    pub capabilities: Vec<String>,
    /// Dedicated health probe target.
    pub health_endpoint: String,
    /// Arbitrary key/value metadata.
    pub metadata: HashMap<String, String>,
}

/// Service listing entry returned from discovery.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ServiceInfo {
    /// Stable service identifier.
    pub service_id: String,
    /// Reported primal type name.
    pub primal_type: String,
    /// Primary endpoint URL or socket.
    pub endpoint: String,
    /// Advertised capability ids.
    pub capabilities: Vec<String>,
    /// Arbitrary key/value metadata.
    pub metadata: HashMap<String, String>,
}

/// Minimal primal capability snapshot for JSON interchange.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PrimalInfo {
    /// Advertised capability ids.
    pub capabilities: Vec<String>,
    /// Arbitrary key/value metadata.
    pub metadata: HashMap<String, String>,
}
