// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Primal Adapters for Squirrel AI Coordinator
//!
//! Implements truly universal, capability-based adapters for all primal integration.
//! Follows the Universal Primal Architecture Standard for agnostic, extensible design.
//!
//! ## Architecture: Capability-First, Name-Agnostic
//!
//! > "Systems should discover and integrate based on what they can do, not what they're called"
//!
//! Instead of hardcoded primal names, we use capability-based discovery to find services
//! that can fulfill specific roles in the AI coordination workflow.

// Native async traits (Rust 1.75+) - no async_trait needed!
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod compute_adapter;
pub mod orchestration_adapter;
pub mod registry;
pub mod security_adapter;
pub mod storage_adapter;
pub mod types_modernized;

#[cfg(test)]
mod adapter_integration_tests;
#[cfg(test)]
mod mod_tests;

// Re-export the universal types
pub use registry::{ServiceMatcher, UniversalServiceRegistry};

/// Universal Service Registration - All services must implement this pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration {
    /// Unique service identifier
    pub service_id: Uuid,
    /// Service metadata
    pub metadata: ServiceMetadata,
    /// Capabilities this service provides
    pub capabilities: Vec<ServiceCapability>,
    /// API endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Resource specifications
    pub resources: ResourceSpec,
    /// Integration preferences
    pub integration: IntegrationPreferences,
    /// Custom extension data
    pub extensions: HashMap<String, serde_json::Value>,
    /// Registration timestamp
    pub registration_timestamp: chrono::DateTime<chrono::Utc>,
    /// Service version
    pub service_version: String,
    /// Instance identifier
    pub instance_id: String,
    /// Priority for load balancing
    pub priority: u8,
}

/// Service metadata with open categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Service display name
    pub name: String,
    /// Service category for discovery
    pub category: ServiceCategory,
    /// Semantic version string
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Maintainer identifier
    pub maintainer: String,
    /// Supported protocol names
    pub protocols: Vec<String>,
}

/// Extensible service categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCategory {
    /// Computational services (`ToadStool`, custom compute)
    Compute {
        /// Compute specialty identifiers
        specialties: Vec<String>,
    },
    /// Storage services (`NestGate`, custom storage)
    Storage {
        /// Storage type identifiers
        types: Vec<String>,
    },
    /// Security services (`BearDog`, custom security)
    Security {
        /// Security domain identifiers
        domains: Vec<String>,
    },
    /// Orchestration services (Songbird, custom orchestration)
    Orchestration {
        /// Orchestration scope identifiers
        scopes: Vec<String>,
    },
    /// AI/ML services (AI primals, custom models)
    Intelligence {
        /// Modality identifiers (e.g., text, vision)
        modalities: Vec<String>,
    },
    /// Community-defined custom categories
    Custom {
        /// Custom category name
        category: String,
        /// Subcategory identifiers
        subcategories: Vec<String>,
    },
}

/// Universal capability system - extensible for any service type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCapability {
    /// Security capabilities (BearDog-style)
    Security {
        /// Security function identifiers
        functions: Vec<String>,
        /// Compliance standards supported
        compliance: Vec<String>,
        /// Trust level identifiers
        trust_levels: Vec<String>,
    },
    /// Orchestration capabilities (Songbird-style)
    Coordination {
        /// Coordination pattern names
        patterns: Vec<String>,
        /// Consistency model
        consistency: String,
        /// Fault tolerance strategy
        fault_tolerance: String,
    },
    /// Data management capabilities (NestGate-style)
    DataManagement {
        /// Supported operations
        operations: Vec<String>,
        /// Consistency model
        consistency: String,
        /// Durability guarantee
        durability: String,
    },
    /// Computation capabilities (ToadStool-style)
    Computation {
        /// Computation type identifiers
        types: Vec<String>,
        /// Resource requirements
        resources: HashMap<String, serde_json::Value>,
        /// Execution constraints
        constraints: Vec<String>,
    },
    /// AI capabilities (Squirrel and others)
    ArtificialIntelligence {
        /// Supported model identifiers
        models: Vec<String>,
        /// Supported task types
        tasks: Vec<String>,
        /// Interface types (e.g., MCP, REST)
        interfaces: Vec<String>,
    },
    /// Community extensible capabilities
    Custom {
        /// Capability domain
        domain: String,
        /// Capability name
        capability: String,
        /// Capability parameters
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Endpoint name or identifier
    pub name: String,
    /// Full URL for the endpoint
    pub url: String,
    /// Protocol (e.g., http, grpc)
    pub protocol: String,
    /// Optional port number
    pub port: Option<u16>,
    /// Optional path suffix
    pub path: Option<String>,
}

/// Resource specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU cores available
    pub cpu_cores: Option<u32>,
    /// Memory in gigabytes
    pub memory_gb: Option<u32>,
    /// Storage in gigabytes
    pub storage_gb: Option<u64>,
    /// Network bandwidth in bytes per second
    pub network_bandwidth: Option<u64>,
    /// Custom resource specifications
    pub custom_resources: HashMap<String, serde_json::Value>,
}

/// Integration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPreferences {
    /// Preferred protocol names in order
    pub preferred_protocols: Vec<String>,
    /// Retry policy identifier
    pub retry_policy: String,
    /// Request timeout in seconds
    pub timeout_seconds: u32,
    /// Weight for load balancing (higher = more traffic)
    pub load_balancing_weight: u8,
}

/// Universal Service Provider trait - all services implement this
pub trait UniversalServiceProvider: Send + Sync {
    /// Get the capabilities this service provides
    fn get_capabilities(&self) -> Vec<ServiceCapability>;

    /// Handle a universal request
    async fn handle_request(
        &self,
        request: UniversalRequest,
    ) -> Result<UniversalResponse, crate::error::PrimalError>;

    /// Get service registration information
    fn get_registration(&self) -> UniversalServiceRegistration;

    /// Health check for the service
    async fn health_check(&self) -> Result<ServiceHealth, crate::error::PrimalError>;
}

/// Universal request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    /// Unique request identifier
    pub request_id: String,
    /// Operation name to invoke
    pub operation: String,
    /// Operation parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Request context for propagation
    pub context: HashMap<String, serde_json::Value>,
    /// Requester identifier
    pub requester: String,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Universal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    /// Request ID for correlation
    pub request_id: String,
    /// Response status
    pub status: ResponseStatus,
    /// Response payload data
    pub data: serde_json::Value,
    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Request completed successfully
    Success,
    /// Request failed with error
    Error {
        /// Error code
        code: String,
        /// Error message
        message: String,
    },
    /// Partial completion (e.g., streaming)
    Partial {
        /// Number of items completed
        completed: usize,
        /// Total items expected
        total: usize,
    },
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Whether the service is healthy
    pub healthy: bool,
    /// Optional status message
    pub message: Option<String>,
    /// Health metrics
    pub metrics: HashMap<String, serde_json::Value>,
}

impl UniversalResponse {
    /// Create a successful response
    #[must_use]
    pub fn success(request_id: String, data: serde_json::Value) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Success,
            data,
            metadata: HashMap::new(),
            processing_time_ms: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(request_id: String, code: &str, message: &str) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Error {
                code: code.to_string(),
                message: message.to_string(),
            },
            data: serde_json::Value::Null,
            metadata: HashMap::new(),
            processing_time_ms: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}
