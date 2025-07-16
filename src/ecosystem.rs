//! Ecosystem Integration for Squirrel AI Primal
//!
//! This module implements the EcosystemServiceRegistration standard that enables
//! Squirrel to be dynamically discovered and integrated by other primals in the ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::primal_provider::SquirrelPrimalProvider;
use crate::universal::*;

/// Ecosystem service registration for Squirrel AI primal
///
/// This struct follows the standardized format defined in the ecosystem API
/// standardization guide, enabling seamless integration with Songbird orchestration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Unique service identifier: "primal-squirrel-{instance}"
    pub service_id: String,

    /// Primal type from standardized enum
    pub primal_type: EcosystemPrimalType,

    /// Associated biome identifier (if applicable)
    pub biome_id: Option<String>,

    /// Service capabilities (standardized format)
    pub capabilities: ServiceCapabilities,

    /// API endpoints (standardized format)
    pub endpoints: ServiceEndpoints,

    /// Resource requirements
    pub resource_requirements: ResourceSpec,

    /// Security configuration
    pub security_config: SecurityConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Registration timestamp
    pub registered_at: DateTime<Utc>,

    /// Service version
    pub version: String,
}

/// Standardized primal types for ecosystem integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EcosystemPrimalType {
    ToadStool,
    Songbird,
    BearDog,
    NestGate,
    Squirrel,
    BiomeOS,
}

/// Service capabilities in standardized format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    /// Core capabilities (required)
    pub core: Vec<String>,
    /// Extended capabilities (optional)
    pub extended: Vec<String>,
    /// Cross-primal integrations supported
    pub integrations: Vec<String>,
}

/// Service endpoints in standardized format
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
    /// MCP protocol endpoint
    pub mcp: String,
    /// AI coordination endpoint
    pub ai_coordination: String,
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU requirements (in cores)
    pub cpu_cores: f64,
    /// Memory requirements (in MB)
    pub memory_mb: u64,
    /// Disk space requirements (in MB)
    pub disk_mb: u64,
    /// Network bandwidth requirements (in Mbps)
    pub network_mbps: u64,
    /// GPU requirements (optional)
    pub gpu: Option<GpuSpec>,
}

/// GPU specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSpec {
    /// GPU memory requirements (in MB)
    pub memory_mb: u64,
    /// GPU compute capability requirement
    pub compute_capability: String,
    /// Number of GPUs required
    pub count: u32,
}

/// Security configuration for the service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication required
    pub auth_required: bool,
    /// Supported authentication methods
    pub auth_methods: Vec<String>,
    /// Encryption requirements
    pub encryption: EncryptionConfig,
    /// Access control configuration
    pub access_control: AccessControlConfig,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// TLS/SSL required
    pub tls_required: bool,
    /// Minimum TLS version
    pub min_tls_version: String,
    /// Supported cipher suites
    pub cipher_suites: Vec<String>,
    /// Certificate validation required
    pub cert_validation: bool,
}

/// Access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// Role-based access control enabled
    pub rbac_enabled: bool,
    /// Supported roles
    pub roles: Vec<String>,
    /// Permission model
    pub permissions: Vec<String>,
    /// IP whitelist (optional)
    pub ip_whitelist: Option<Vec<String>>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint path
    pub endpoint: String,
    /// Check interval in seconds
    pub interval_seconds: u64,
    /// Timeout for health checks in seconds
    pub timeout_seconds: u64,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,
}

/// Standardized ecosystem request format
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

    /// Request metadata
    pub metadata: HashMap<String, String>,

    /// Request timestamp
    pub timestamp: DateTime<Utc>,

    /// Request priority
    pub priority: RequestPriority,

    /// Request timeout
    pub timeout_seconds: u64,
}

/// Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RequestPriority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// Standardized ecosystem response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemResponse {
    /// Request identifier this response corresponds to
    pub request_id: Uuid,

    /// Response status
    pub status: EcosystemResponseStatus,

    /// Response payload
    pub payload: serde_json::Value,

    /// Response metadata
    pub metadata: HashMap<String, String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Response status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemResponseStatus {
    Success,
    Error { code: String, message: String },
    Processing,
    Timeout,
    Unauthorized,
    Forbidden,
    NotFound,
    RateLimited,
}

impl EcosystemServiceRegistration {
    /// Create a new ecosystem service registration for Squirrel
    pub fn new(provider: &SquirrelPrimalProvider) -> Self {
        let endpoints = provider.endpoints();

        Self {
            service_id: format!("primal-squirrel-{}", provider.instance_id()),
            primal_type: EcosystemPrimalType::Squirrel,
            biome_id: None, // Will be set during registration if applicable
            capabilities: ServiceCapabilities {
                core: vec![
                    "ai_coordination".to_string(),
                    "mcp_protocol".to_string(),
                    "session_management".to_string(),
                ],
                extended: vec![
                    "context_awareness".to_string(),
                    "ecosystem_intelligence".to_string(),
                    "tool_orchestration".to_string(),
                ],
                integrations: vec![
                    "songbird".to_string(),
                    "biomeos".to_string(),
                    "beardog".to_string(),
                    "nestgate".to_string(),
                    "toadstool".to_string(),
                ],
            },
            endpoints: ServiceEndpoints {
                health: endpoints.health,
                metrics: endpoints.metrics,
                admin: endpoints.admin,
                websocket: endpoints.websocket,
                mcp: endpoints.mcp,
                ai_coordination: endpoints.ai_coordination,
            },
            resource_requirements: ResourceSpec {
                cpu_cores: 2.0,
                memory_mb: 4096,
                disk_mb: 1024,
                network_mbps: 100,
                gpu: None, // Squirrel doesn't require GPU by default
            },
            security_config: SecurityConfig {
                auth_required: true,
                auth_methods: vec![
                    "bearer_token".to_string(),
                    "api_key".to_string(),
                    "oauth2".to_string(),
                ],
                encryption: EncryptionConfig {
                    tls_required: true,
                    min_tls_version: "1.2".to_string(),
                    cipher_suites: vec![
                        "TLS_AES_256_GCM_SHA384".to_string(),
                        "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                    ],
                    cert_validation: true,
                },
                access_control: AccessControlConfig {
                    rbac_enabled: true,
                    roles: vec![
                        "admin".to_string(),
                        "user".to_string(),
                        "service".to_string(),
                    ],
                    permissions: vec![
                        "read".to_string(),
                        "write".to_string(),
                        "execute".to_string(),
                        "admin".to_string(),
                    ],
                    ip_whitelist: None,
                },
            },
            health_check: HealthCheckConfig {
                endpoint: "/health".to_string(),
                interval_seconds: 30,
                timeout_seconds: 5,
                failure_threshold: 3,
                success_threshold: 2,
            },
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("primal_version".to_string(), crate::VERSION.to_string());
                metadata.insert(
                    "context_user_id".to_string(),
                    provider.context().user_id.clone(),
                );
                metadata.insert(
                    "context_device_id".to_string(),
                    provider.context().device_id.clone(),
                );
                metadata.insert(
                    "security_level".to_string(),
                    format!("{:?}", provider.context().security_level),
                );
                metadata
            },
            registered_at: Utc::now(),
            version: crate::VERSION.to_string(),
        }
    }

    /// Update the biome ID for this registration
    pub fn with_biome_id(mut self, biome_id: String) -> Self {
        self.biome_id = Some(biome_id);
        self
    }

    /// Update resource requirements
    pub fn with_resource_requirements(mut self, requirements: ResourceSpec) -> Self {
        self.resource_requirements = requirements;
        self
    }

    /// Add custom metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Validate the registration
    pub fn validate(&self) -> Result<(), String> {
        if self.service_id.is_empty() {
            return Err("Service ID cannot be empty".to_string());
        }

        if self.capabilities.core.is_empty() {
            return Err("Core capabilities cannot be empty".to_string());
        }

        if self.endpoints.health.is_empty() {
            return Err("Health endpoint cannot be empty".to_string());
        }

        if self.resource_requirements.cpu_cores <= 0.0 {
            return Err("CPU cores must be greater than 0".to_string());
        }

        if self.resource_requirements.memory_mb == 0 {
            return Err("Memory requirement must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Convert to JSON for registration
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Create from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl EcosystemPrimalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EcosystemPrimalType::ToadStool => "toadstool",
            EcosystemPrimalType::Songbird => "songbird",
            EcosystemPrimalType::BearDog => "beardog",
            EcosystemPrimalType::NestGate => "nestgate",
            EcosystemPrimalType::Squirrel => "squirrel",
            EcosystemPrimalType::BiomeOS => "biomeos",
        }
    }
}

impl Default for ResourceSpec {
    fn default() -> Self {
        Self {
            cpu_cores: 1.0,
            memory_mb: 2048,
            disk_mb: 512,
            network_mbps: 10,
            gpu: None,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_required: true,
            auth_methods: vec!["bearer_token".to_string()],
            encryption: EncryptionConfig::default(),
            access_control: AccessControlConfig::default(),
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            tls_required: true,
            min_tls_version: "1.2".to_string(),
            cipher_suites: vec!["TLS_AES_256_GCM_SHA384".to_string()],
            cert_validation: true,
        }
    }
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            rbac_enabled: true,
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string()],
            ip_whitelist: None,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval_seconds: 30,
            timeout_seconds: 5,
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}
