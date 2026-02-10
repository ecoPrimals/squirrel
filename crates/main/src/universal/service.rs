// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
    Authentication {
        methods: Vec<String>,
    },
    Authorization {
        features: Vec<String>,
    },
    Security {
        level: String,
        features: Vec<String>,
    },
    Encryption {
        algorithms: Vec<String>,
    },
    Auditing {
        capabilities: Vec<String>,
    },
    Custom {
        name: String,
        description: String,
        metadata: HashMap<String, Value>,
    },
}

/// Service endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub capabilities: Vec<ServiceCapability>,
    pub health_status: String,
}

/// Universal service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration {
    pub service_id: String,
    pub service_name: String,
    pub endpoints: Vec<ServiceEndpoint>,
    pub capabilities: Vec<ServiceCapability>,
    pub metadata: HashMap<String, String>,
}
