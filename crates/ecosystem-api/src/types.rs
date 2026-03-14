// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Shared types for ecosystem integration
//!
//! This module contains all the standardized types used across the ecoPrimals
//! ecosystem for communication through the Songbird service mesh.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

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
pub enum ResponseStatus {
    /// Request completed successfully
    Success,
    /// Request failed with error
    Error {
        /// Error code identifier for categorization (`Arc<str>` for O(1) clone)
        code: Arc<str>,
        /// Human-readable error message description (unique per instance)
        message: String,
    },
    /// Request timed out
    Timeout,
    /// Target service is unavailable
    ServiceUnavailable,
}

/// Security context for all requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Authentication token
    pub auth_token: Option<String>,

    /// User/service identity (`Arc<str>` for O(1) clone when shared)
    pub identity: Arc<str>,

    /// Permissions/capabilities
    pub permissions: Vec<String>,

    /// Security level required
    pub security_level: SecurityLevel,
}

/// Security level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Publicly accessible
    Public,
    /// Internal ecosystem services only
    Internal,
    /// Restricted access
    Restricted,
    /// Confidential access
    Confidential,
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

/// Context for primal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalContext {
    /// User identifier (`Arc<str>` for O(1) clone when shared)
    pub user_id: Arc<str>,

    /// Device identifier (`Arc<str>` for O(1) clone when shared)
    pub device_id: Arc<str>,

    /// Session identifier (`Arc<str>` for O(1) clone when shared)
    pub session_id: Arc<str>,

    /// Network location information
    pub network_location: NetworkLocation,

    /// Security level
    pub security_level: SecurityLevel,

    /// Biome identifier (if applicable) (`Arc<str>` for O(1) clone when shared)
    pub biome_id: Option<Arc<str>>,

    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

/// Network location information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkLocation {
    /// IP address
    pub ip_address: Option<String>,

    /// Geographic region
    pub region: Option<String>,

    /// Availability zone
    pub zone: Option<String>,

    /// Network segment
    pub segment: Option<String>,
}

/// Standardized primal types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    /// `ToadStool` compute platform
    ToadStool,
    /// Songbird service mesh
    Songbird,
    /// `BearDog` security framework
    BearDog,
    /// `NestGate` storage system
    NestGate,
    /// Squirrel AI platform
    Squirrel,
    /// biomeOS orchestration platform
    BiomeOS,
    /// Any primal that provides the required capabilities (for capability-based discovery)
    Any,
}

impl PrimalType {
    /// Get string representation (for serialization/backward compatibility)
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimalType::ToadStool => "toadstool",
            PrimalType::Songbird => "songbird",
            PrimalType::BearDog => "beardog",
            PrimalType::NestGate => "nestgate",
            PrimalType::Squirrel => "squirrel",
            PrimalType::BiomeOS => "biomeos",
            PrimalType::Any => "any", // Capability-based discovery
        }
    }

    /// Get capability for discovery (use when discovering OTHER primals by capability)
    ///
    /// Returns the capability constant for capability-based discovery.
    #[must_use]
    pub fn capability(&self) -> &'static str {
        match self {
            PrimalType::ToadStool => "compute",
            PrimalType::Songbird => "service-mesh",
            PrimalType::BearDog => "security",
            PrimalType::NestGate => "storage",
            PrimalType::Squirrel => "squirrel", // Self-identity
            PrimalType::BiomeOS => "ecosystem",
            PrimalType::Any => "any",
        }
    }
}

/// Standardized capability system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    // Compute capabilities (ToadStool)
    /// Container runtime support
    ContainerRuntime {
        /// Supported container orchestration platforms (e.g., "kubernetes", "docker-compose")
        orchestrators: Vec<String>,
    },
    /// Serverless execution support
    ServerlessExecution {
        /// Supported programming languages for serverless functions
        languages: Vec<String>,
    },
    /// GPU acceleration support
    GpuAcceleration {
        /// Whether CUDA acceleration is supported
        cuda_support: bool,
    },
    /// Native execution support
    NativeExecution {
        /// Supported CPU architectures (e.g., "`x86_64`", "aarch64")
        architectures: Vec<String>,
    },
    /// WebAssembly execution support
    WasmExecution {
        /// Whether WebAssembly System Interface (WASI) is supported
        wasi_support: bool,
    },

    // Security capabilities (BearDog)
    /// Authentication methods
    Authentication {
        /// Supported authentication methods (e.g., "oauth2", "jwt", "ldap")
        methods: Vec<String>,
    },
    /// Encryption algorithms
    Encryption {
        /// Supported encryption algorithms (e.g., "AES-256", "RSA-2048")
        algorithms: Vec<String>,
    },
    /// Key management
    KeyManagement {
        /// Whether Hardware Security Module (HSM) is supported
        hsm_support: bool,
    },
    /// Threat detection
    ThreatDetection {
        /// Whether machine learning-based threat detection is enabled
        ml_enabled: bool,
    },
    /// Compliance frameworks
    Compliance {
        /// Supported compliance frameworks (e.g., "SOX", "GDPR", "HIPAA")
        frameworks: Vec<String>,
    },

    // Storage capabilities (NestGate)
    /// File system support
    FileSystem {
        /// Whether ZFS file system is supported
        supports_zfs: bool,
    },
    /// Object storage
    ObjectStorage {
        /// Supported object storage backends (e.g., "s3", "azure-blob", "gcs")
        backends: Vec<String>,
    },
    /// Data replication
    DataReplication {
        /// Data consistency model (e.g., "eventual", "strong", "sequential")
        consistency: String,
    },
    /// Volume management
    VolumeManagement {
        /// Supported volume management protocols (e.g., "nfs", "iscsi", "ceph")
        protocols: Vec<String>,
    },
    /// Backup and restore
    BackupRestore {
        /// Whether incremental backup is supported
        incremental: bool,
    },

    // Network capabilities (Songbird)
    /// Service discovery
    ServiceDiscovery {
        /// Supported service discovery protocols (e.g., "dns", "consul", "etcd")
        protocols: Vec<String>,
    },
    /// Network routing
    NetworkRouting {
        /// Supported network routing protocols (e.g., "bgp", "ospf", "http")
        protocols: Vec<String>,
    },
    /// Load balancing
    LoadBalancing {
        /// Supported load balancing algorithms (e.g., "round-robin", "weighted", "least-conn")
        algorithms: Vec<String>,
    },
    /// Circuit breaking
    CircuitBreaking {
        /// Whether circuit breaking is enabled for fault tolerance
        enabled: bool,
    },

    // AI capabilities (Squirrel)
    /// Model inference
    ModelInference {
        /// Supported AI models (e.g., "gpt-4", "claude", "llama2")
        models: Vec<String>,
    },
    /// Agent framework
    AgentFramework {
        /// Whether Model Context Protocol (MCP) is supported
        mcp_support: bool,
    },
    /// Machine learning
    MachineLearning {
        /// Whether model training is supported (vs inference only)
        training_support: bool,
    },
    /// Natural language processing
    NaturalLanguage {
        /// Supported natural languages (e.g., "en", "es", "fr", "de")
        languages: Vec<String>,
    },

    // OS capabilities (biomeOS)
    /// Orchestration
    Orchestration {
        /// Supported primal components (e.g., "toadstool", "beardog", "songbird")
        primals: Vec<String>,
    },
    /// Manifests
    Manifests {
        /// Supported manifest formats (e.g., "yaml", "json", "toml")
        formats: Vec<String>,
    },
    /// Deployment
    Deployment {
        /// Supported deployment strategies (e.g., "rolling", "canary", "blue-green")
        strategies: Vec<String>,
    },
    /// Monitoring
    Monitoring {
        /// Supported monitoring metrics (e.g., "prometheus", "grafana", "jaeger")
        metrics: Vec<String>,
    },
}

/// Dependency on another primal's capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Type of primal (or Any for capability-based discovery)
    pub primal_type: PrimalType,
    /// Human-readable name for the dependency
    pub name: String,
    /// Required capabilities (used when `primal_type` is Any)
    pub capabilities: Vec<String>,
    /// Whether this dependency is required for operation
    pub required: bool,
    /// Minimum version requirement
    pub min_version: Option<String>,
}

/// Health status for all primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    /// Health status
    pub status: HealthStatus,

    /// Primal version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Resource usage
    pub resource_usage: ResourceUsage,

    /// Capabilities currently online
    pub capabilities_online: Vec<String>,

    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,

    /// Memory usage in bytes
    pub memory_bytes: u64,

    /// Disk usage in bytes
    pub disk_bytes: u64,

    /// Network usage in bytes per second
    pub network_bytes_per_sec: u64,
}

/// Primal endpoints information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Primary endpoint
    pub primary: String,

    /// Health check endpoint
    pub health: String,

    /// Metrics endpoint
    pub metrics: Option<String>,

    /// Admin endpoint
    pub admin: Option<String>,

    /// WebSocket endpoint
    pub websocket: Option<String>,

    /// Service mesh endpoint
    pub service_mesh: String,
}

/// Dynamic port information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Port number
    pub port: u16,

    /// Protocol
    pub protocol: String,

    /// Assigned by Songbird
    pub assigned_by: String,

    /// Assignment timestamp
    pub assigned_at: DateTime<Utc>,

    /// Lease duration
    pub lease_duration: std::time::Duration,
}

/// Service mesh status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceMeshStatus {
    /// Is connected to service mesh
    pub connected: bool,

    /// Service mesh endpoint (capability-based, not primal-specific)
    pub service_mesh_endpoint: Option<String>,

    /// Registration timestamp
    pub registration_time: Option<DateTime<Utc>>,

    /// Last heartbeat timestamp
    pub last_heartbeat: Option<DateTime<Utc>>,

    /// Service mesh metadata
    pub metadata: HashMap<String, String>,
}

/// Service capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    /// Core capabilities (required)
    pub core: Vec<String>,

    /// Extended capabilities (optional)
    pub extended: Vec<String>,

    /// Cross-primal integrations supported
    pub integrations: Vec<String>,
}

/// Service endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    /// Health check endpoint
    pub health: String,

    /// Metrics endpoint
    pub metrics: String,

    /// Admin/management endpoint
    pub admin: String,

    /// WebSocket endpoint (if supported)
    pub websocket: Option<String>,
}

/// Resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU cores required
    pub cpu_cores: Option<f64>,

    /// Memory in MB required
    pub memory_mb: Option<u64>,

    /// Disk space in MB required
    pub disk_mb: Option<u64>,

    /// Network bandwidth in Mbps required
    pub network_bandwidth_mbps: Option<u64>,

    /// GPU count required
    pub gpu_count: Option<u32>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check path
    pub path: String,

    /// Check interval in seconds
    pub interval_seconds: u64,

    /// Timeout in seconds
    pub timeout_seconds: u64,

    /// Number of retries
    pub retries: u32,

    /// Initial delay in seconds
    pub initial_delay_seconds: u64,
}

/// Ecosystem service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Service identifier (`Arc<str>` for O(1) clone when shared)
    pub service_id: Arc<str>,

    /// Primal type
    pub primal_type: PrimalType,

    /// Associated biome identifier (if applicable) (`Arc<str>` for O(1) clone when shared)
    pub biome_id: Option<Arc<str>>,

    /// Service capabilities
    pub capabilities: ServiceCapabilities,

    /// API endpoints
    pub endpoints: ServiceEndpoints,

    /// Resource requirements
    pub resource_requirements: ResourceSpec,

    /// Security configuration
    pub security_config: SecurityConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication method
    pub auth_method: String,

    /// TLS enabled
    pub tls_enabled: bool,

    /// Mutual TLS required
    pub mtls_required: bool,

    /// Trust domain
    pub trust_domain: String,

    /// Security level
    pub security_level: SecurityLevel,

    /// Crypto lock enabled
    pub crypto_lock_enabled: bool,
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

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            auth_token: None,
            identity: Arc::from("anonymous"),
            permissions: vec![],
            security_level: SecurityLevel::Public,
        }
    }
}

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            user_id: Arc::from("system"),
            device_id: Arc::from("unknown"),
            session_id: Arc::from(Uuid::new_v4().to_string()),
            network_location: NetworkLocation::default(),
            security_level: SecurityLevel::Internal,
            biome_id: None,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== PrimalType Tests ==========

    #[test]
    fn test_primal_type_as_str() {
        assert_eq!(PrimalType::ToadStool.as_str(), "toadstool");
        assert_eq!(PrimalType::Songbird.as_str(), "songbird");
        assert_eq!(PrimalType::BearDog.as_str(), "beardog");
        assert_eq!(PrimalType::NestGate.as_str(), "nestgate");
        assert_eq!(PrimalType::Squirrel.as_str(), "squirrel");
        assert_eq!(PrimalType::BiomeOS.as_str(), "biomeos");
        assert_eq!(PrimalType::Any.as_str(), "any");
    }

    #[test]
    fn test_primal_type_serde_roundtrip() {
        for pt in &[
            PrimalType::ToadStool,
            PrimalType::Songbird,
            PrimalType::BearDog,
            PrimalType::NestGate,
            PrimalType::Squirrel,
            PrimalType::BiomeOS,
            PrimalType::Any,
        ] {
            let json = serde_json::to_string(pt).unwrap();
            let deser: PrimalType = serde_json::from_str(&json).unwrap();
            assert_eq!(*pt, deser);
        }
    }

    // ========== SecurityLevel Tests ==========

    #[test]
    fn test_security_level_serde_roundtrip() {
        for level in &[
            SecurityLevel::Public,
            SecurityLevel::Internal,
            SecurityLevel::Restricted,
            SecurityLevel::Confidential,
        ] {
            let json = serde_json::to_string(level).unwrap();
            let deser: SecurityLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(*level, deser);
        }
    }

    // ========== ResponseStatus Tests ==========

    #[test]
    fn test_response_status_serde() {
        let success = ResponseStatus::Success;
        let json = serde_json::to_string(&success).unwrap();
        let deser: ResponseStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(success, deser);

        let error = ResponseStatus::Error {
            code: Arc::from("E001"),
            message: "Something went wrong".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        let deser: ResponseStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deser);

        let timeout = ResponseStatus::Timeout;
        let json = serde_json::to_string(&timeout).unwrap();
        let deser: ResponseStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(timeout, deser);

        let unavailable = ResponseStatus::ServiceUnavailable;
        let json = serde_json::to_string(&unavailable).unwrap();
        let deser: ResponseStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(unavailable, deser);
    }

    // ========== HealthStatus Tests ==========

    #[test]
    fn test_health_status_serde() {
        for status in &[
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unhealthy,
            HealthStatus::Unknown,
        ] {
            let json = serde_json::to_string(status).unwrap();
            let deser: HealthStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*status, deser);
        }
    }

    // ========== Default Implementation Tests ==========

    #[test]
    fn test_primal_response_default() {
        let resp = PrimalResponse::default();
        assert_eq!(resp.status, ResponseStatus::Success);
        assert_eq!(resp.payload, serde_json::Value::Null);
        assert!(resp.metadata.is_empty());
    }

    #[test]
    fn test_ecosystem_request_default() {
        let req = EcosystemRequest::default();
        assert_eq!(req.source_service.as_ref(), "unknown");
        assert_eq!(req.target_service.as_ref(), "unknown");
        assert_eq!(req.operation.as_ref(), "unknown");
        assert_eq!(req.payload, serde_json::Value::Null);
        assert!(req.metadata.is_empty());
    }

    #[test]
    fn test_security_context_default() {
        let ctx = SecurityContext::default();
        assert!(ctx.auth_token.is_none());
        assert_eq!(ctx.identity.as_ref(), "anonymous");
        assert!(ctx.permissions.is_empty());
        assert_eq!(ctx.security_level, SecurityLevel::Public);
    }

    #[test]
    fn test_primal_context_default() {
        let ctx = PrimalContext::default();
        assert_eq!(ctx.user_id.as_ref(), "system");
        assert_eq!(ctx.device_id.as_ref(), "unknown");
        assert!(!ctx.session_id.is_empty());
        assert_eq!(ctx.security_level, SecurityLevel::Internal);
        assert!(ctx.biome_id.is_none());
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_network_location_default() {
        let loc = NetworkLocation::default();
        assert!(loc.ip_address.is_none());
        assert!(loc.region.is_none());
        assert!(loc.zone.is_none());
        assert!(loc.segment.is_none());
    }

    #[test]
    fn test_service_mesh_status_default() {
        let status = ServiceMeshStatus::default();
        assert!(!status.connected);
        assert!(status.service_mesh_endpoint.is_none());
        assert!(status.registration_time.is_none());
        assert!(status.last_heartbeat.is_none());
        assert!(status.metadata.is_empty());
    }

    // ========== Serialization Round-Trip Tests ==========

    #[test]
    fn test_ecosystem_request_serde() {
        let req = EcosystemRequest {
            request_id: Uuid::new_v4(),
            source_service: Arc::from("squirrel"),
            target_service: Arc::from("songbird"),
            operation: Arc::from("discover"),
            payload: serde_json::json!({"key": "value"}),
            security_context: SecurityContext::default(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&req).unwrap();
        let deser: EcosystemRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.source_service.as_ref(), "squirrel");
        assert_eq!(deser.target_service.as_ref(), "songbird");
        assert_eq!(deser.operation.as_ref(), "discover");
    }

    #[test]
    fn test_primal_capability_serde() {
        let cap = PrimalCapability::ModelInference {
            models: vec!["gpt-4".to_string(), "claude".to_string()],
        };
        let json = serde_json::to_string(&cap).unwrap();
        let deser: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deser);
    }

    #[test]
    fn test_primal_dependency_serde() {
        let dep = PrimalDependency {
            primal_type: PrimalType::Any,
            name: "security-provider".to_string(),
            capabilities: vec!["authentication".to_string(), "encryption".to_string()],
            required: true,
            min_version: Some("1.0.0".to_string()),
        };
        let json = serde_json::to_string(&dep).unwrap();
        let deser: PrimalDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(dep, deser);
    }

    #[test]
    fn test_all_primal_capabilities_serde() {
        let caps: Vec<PrimalCapability> = vec![
            PrimalCapability::ContainerRuntime {
                orchestrators: vec!["kubernetes".to_string()],
            },
            PrimalCapability::ServerlessExecution {
                languages: vec!["rust".to_string()],
            },
            PrimalCapability::GpuAcceleration { cuda_support: true },
            PrimalCapability::NativeExecution {
                architectures: vec!["x86_64".to_string()],
            },
            PrimalCapability::WasmExecution { wasi_support: true },
            PrimalCapability::Authentication {
                methods: vec!["oauth2".to_string()],
            },
            PrimalCapability::Encryption {
                algorithms: vec!["AES-256".to_string()],
            },
            PrimalCapability::KeyManagement { hsm_support: false },
            PrimalCapability::ThreatDetection { ml_enabled: true },
            PrimalCapability::Compliance {
                frameworks: vec!["GDPR".to_string()],
            },
            PrimalCapability::FileSystem { supports_zfs: true },
            PrimalCapability::ObjectStorage {
                backends: vec!["s3".to_string()],
            },
            PrimalCapability::DataReplication {
                consistency: "strong".to_string(),
            },
            PrimalCapability::VolumeManagement {
                protocols: vec!["nfs".to_string()],
            },
            PrimalCapability::BackupRestore { incremental: true },
            PrimalCapability::ServiceDiscovery {
                protocols: vec!["dns".to_string()],
            },
            PrimalCapability::NetworkRouting {
                protocols: vec!["http".to_string()],
            },
            PrimalCapability::LoadBalancing {
                algorithms: vec!["round-robin".to_string()],
            },
            PrimalCapability::CircuitBreaking { enabled: true },
            PrimalCapability::ModelInference {
                models: vec!["gpt-4".to_string()],
            },
            PrimalCapability::AgentFramework { mcp_support: true },
            PrimalCapability::MachineLearning {
                training_support: false,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".to_string()],
            },
            PrimalCapability::Orchestration {
                primals: vec!["squirrel".to_string()],
            },
            PrimalCapability::Manifests {
                formats: vec!["yaml".to_string()],
            },
            PrimalCapability::Deployment {
                strategies: vec!["rolling".to_string()],
            },
            PrimalCapability::Monitoring {
                metrics: vec!["prometheus".to_string()],
            },
        ];

        for cap in &caps {
            let json = serde_json::to_string(cap).unwrap();
            let deser: PrimalCapability = serde_json::from_str(&json).unwrap();
            assert_eq!(*cap, deser, "Failed roundtrip for: {:?}", cap);
        }
    }

    // ========== PrimalType Hash Tests ==========

    #[test]
    fn test_primal_type_hash_works_in_collections() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PrimalType::Squirrel);
        set.insert(PrimalType::Songbird);
        set.insert(PrimalType::Squirrel); // duplicate
        assert_eq!(set.len(), 2);
        assert!(set.contains(&PrimalType::Squirrel));
        assert!(set.contains(&PrimalType::Songbird));
    }
}
