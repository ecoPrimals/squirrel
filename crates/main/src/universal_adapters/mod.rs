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
    pub name: String,
    pub category: ServiceCategory,
    pub version: String,
    pub description: String,
    pub maintainer: String,
    pub protocols: Vec<String>,
}

/// Extensible service categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCategory {
    /// Computational services (`ToadStool`, custom compute)
    Compute { specialties: Vec<String> },
    /// Storage services (`NestGate`, custom storage)
    Storage { types: Vec<String> },
    /// Security services (`BearDog`, custom security)
    Security { domains: Vec<String> },
    /// Orchestration services (Songbird, custom orchestration)
    Orchestration { scopes: Vec<String> },
    /// AI/ML services (AI primals, custom models)
    Intelligence { modalities: Vec<String> },
    /// Community-defined custom categories
    Custom {
        category: String,
        subcategories: Vec<String>,
    },
}

/// Universal capability system - extensible for any service type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCapability {
    /// Security capabilities (BearDog-style)
    Security {
        functions: Vec<String>,
        compliance: Vec<String>,
        trust_levels: Vec<String>,
    },
    /// Orchestration capabilities (Songbird-style)
    Coordination {
        patterns: Vec<String>,
        consistency: String,
        fault_tolerance: String,
    },
    /// Data management capabilities (NestGate-style)
    DataManagement {
        operations: Vec<String>,
        consistency: String,
        durability: String,
    },
    /// Computation capabilities (ToadStool-style)
    Computation {
        types: Vec<String>,
        resources: HashMap<String, serde_json::Value>,
        constraints: Vec<String>,
    },
    /// AI capabilities (Squirrel and others)
    ArtificialIntelligence {
        models: Vec<String>,
        tasks: Vec<String>,
        interfaces: Vec<String>,
    },
    /// Community extensible capabilities
    Custom {
        domain: String,
        capability: String,
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub name: String,
    pub url: String,
    pub protocol: String,
    pub port: Option<u16>,
    pub path: Option<String>,
}

/// Resource specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu_cores: Option<u32>,
    pub memory_gb: Option<u32>,
    pub storage_gb: Option<u64>,
    pub network_bandwidth: Option<u64>,
    pub custom_resources: HashMap<String, serde_json::Value>,
}

/// Integration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPreferences {
    pub preferred_protocols: Vec<String>,
    pub retry_policy: String,
    pub timeout_seconds: u32,
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
    pub request_id: String,
    pub operation: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: HashMap<String, serde_json::Value>,
    pub requester: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Universal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    pub request_id: String,
    pub status: ResponseStatus,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, serde_json::Value>,
    pub processing_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error { code: String, message: String },
    Partial { completed: usize, total: usize },
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub healthy: bool,
    pub message: Option<String>,
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
