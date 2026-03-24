// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Service capability and registration types
//!
//! This module defines types for describing service capabilities
//! and managing universal service registrations.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Service capability enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCapability {
    /// Authentication capability with supported methods
    Authentication {
        /// Supported authentication methods (e.g., "oauth2", "jwt")
        methods: Vec<String>,
    },
    /// Authorization capability with feature flags
    Authorization {
        /// List of authorization features enabled
        features: Vec<String>,
    },
    /// Security capability with level and features
    Security {
        /// Security level (e.g., "high", "medium", "low")
        level: String,
        /// Security features enabled
        features: Vec<String>,
    },
    /// Encryption capability with algorithm support
    Encryption {
        /// Supported encryption algorithms
        algorithms: Vec<String>,
    },
    /// Auditing capability for compliance tracking
    Auditing {
        /// Auditing capabilities (e.g., "log", "audit_trail")
        capabilities: Vec<String>,
    },
    /// Custom capability with arbitrary metadata
    Custom {
        /// Custom capability name
        name: String,
        /// Human-readable description
        description: String,
        /// Additional metadata as key-value pairs
        metadata: HashMap<String, Value>,
    },
}

/// Service endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Unique endpoint identifier
    pub id: String,
    /// Human-readable endpoint name
    pub name: String,
    /// Endpoint URL for service access
    pub url: String,
    /// Capabilities offered at this endpoint
    pub capabilities: Vec<ServiceCapability>,
    /// Current health status (e.g., "healthy", "degraded")
    pub health_status: String,
}

/// Universal service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration {
    /// Unique service identifier
    pub service_id: String,
    /// Human-readable service name
    pub service_name: String,
    /// List of service endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Aggregate capabilities of the service
    pub capabilities: Vec<ServiceCapability>,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>,
}
