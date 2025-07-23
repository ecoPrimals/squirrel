//! Shared types for ecosystem integration
//!
//! This module contains all the standardized types used across the ecoPrimals
//! ecosystem for communication through the Songbird service mesh.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Standardized request format for all ecosystem communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRequest {
    /// Unique request identifier
    pub request_id: Uuid,

    /// Source service identifier
    pub source_service: String,

    /// Target service identifier
    pub target_service: String,

    /// Request operation
    pub operation: String,

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
        /// Error code identifier for categorization
        code: String,
        /// Human-readable error message description
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

    /// User/service identity
    pub identity: String,

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

    /// Operation to perform
    pub operation: String,

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
    /// User identifier
    pub user_id: String,

    /// Device identifier
    pub device_id: String,

    /// Session identifier
    pub session_id: String,

    /// Network location information
    pub network_location: NetworkLocation,

    /// Security level
    pub security_level: SecurityLevel,

    /// Biome identifier (if applicable)
    pub biome_id: Option<String>,

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
    /// ToadStool compute platform
    ToadStool,
    /// Songbird service mesh
    Songbird,
    /// BearDog security framework
    BearDog,
    /// NestGate storage system
    NestGate,
    /// Squirrel AI platform
    Squirrel,
    /// biomeOS orchestration platform
    BiomeOS,
    /// Any primal that provides the required capabilities (for capability-based discovery)
    Any,
}

impl PrimalType {
    /// Get string representation
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
        /// Supported CPU architectures (e.g., "x86_64", "aarch64")
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
    /// Required capabilities (used when primal_type is Any)
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

    /// Songbird endpoint
    pub songbird_endpoint: Option<String>,

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
    /// Service identifier
    pub service_id: String,

    /// Primal type
    pub primal_type: PrimalType,

    /// Associated biome identifier (if applicable)
    pub biome_id: Option<String>,

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
            source_service: "unknown".to_string(),
            target_service: "unknown".to_string(),
            operation: "unknown".to_string(),
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
            identity: "anonymous".to_string(),
            permissions: vec![],
            security_level: SecurityLevel::Public,
        }
    }
}

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            user_id: "system".to_string(),
            device_id: "unknown".to_string(),
            session_id: Uuid::new_v4().to_string(),
            network_location: NetworkLocation::default(),
            security_level: SecurityLevel::Internal,
            biome_id: None,
            metadata: HashMap::new(),
        }
    }
}
