//! Ecosystem type definitions and configurations
//!
//! This module contains all type definitions for ecosystem integration,
//! including service registration, primal types, capabilities, and
//! configuration structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ecosystem service registration for Squirrel AI primal
///
/// This struct follows the standardized format for service discovery
/// and registration within the ecoPrimals ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Unique service identifier: "primal-squirrel-{instance}"
    pub service_id: String,

    /// Primal type from standardized enum
    pub primal_type: EcosystemPrimalType,

    /// Service name
    pub name: String,

    /// Service description
    pub description: String,

    /// Associated biome identifier (if applicable)
    pub biome_id: Option<String>,

    /// Service version
    pub version: String,

    /// Service capabilities (standardized format)
    pub capabilities: ServiceCapabilities,

    /// API endpoints (standardized format)
    pub endpoints: ServiceEndpoints,

    /// Service dependencies
    pub dependencies: Vec<String>,

    /// Service tags
    pub tags: Vec<String>,

    /// Primal provider info
    pub primal_provider: Option<String>,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Security configuration
    pub security_config: SecurityConfig,

    /// Resource requirements
    pub resource_requirements: ResourceSpec,

    /// Service metadata
    pub metadata: HashMap<String, String>,

    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
}

/// Standardized primal types for ecosystem integration
///
/// **DEPRECATED**: This enum violates primal sovereignty by hardcoding all primal names.
/// Use `CapabilityRegistry` and `PrimalCapability` for capability-based discovery instead.
///
/// ## Why This is Deprecated
///
/// - **Compile-time coupling**: Changes to the ecosystem require recompiling all primals
/// - **Sovereignty violation**: Each primal should only know itself
/// - **Scalability**: Cannot add new primals without code changes
/// - **Evolution**: Cannot evolve primal names or capabilities
///
/// ## Migration Path
///
/// ```ignore
/// // OLD (hardcoded):
/// let primal_type = EcosystemPrimalType::Songbird;
/// let endpoint = primal_type.default_endpoint();
///
/// // NEW (capability-based):
/// use crate::capability_registry::{CapabilityRegistry, PrimalCapability};
/// let registry = CapabilityRegistry::new(Default::default());
/// let primals = registry.discover_by_capability(&PrimalCapability::ServiceMesh).await?;
/// let endpoint = &primals[0].endpoint;
/// ```
#[deprecated(
    since = "0.1.0",
    note = "Use CapabilityRegistry for capability-based discovery instead of hardcoded primal types"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EcosystemPrimalType {
    ToadStool,
    Songbird,
    BearDog,
    NestGate,
    Squirrel,
    BiomeOS,
}

impl EcosystemPrimalType {
    /// Convert to string representation
    ///
    /// **DEPRECATED**: Use capability-based discovery instead.
    #[must_use]
    #[deprecated(since = "0.1.0", note = "Use CapabilityRegistry instead")]
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

    /// Get environment variable name for this primal's endpoint
    ///
    /// **DEPRECATED**: Use generic environment variables like `SERVICE_MESH_ENDPOINT`.
    ///
    /// # Migration Example
    /// ```ignore
    /// // OLD:
    /// let primal = EcosystemPrimalType::Songbird;
    /// let env_name = primal.env_name(); // "SONGBIRD"
    /// let endpoint = std::env::var(format!("{}_ENDPOINT", env_name))?;
    ///
    /// // NEW:
    /// let endpoint = std::env::var("SERVICE_MESH_ENDPOINT")?; // Capability-agnostic
    /// ```
    #[must_use]
    #[deprecated(
        since = "0.1.0",
        note = "Use generic env vars like SERVICE_MESH_ENDPOINT"
    )]
    pub fn env_name(&self) -> &'static str {
        match self {
            EcosystemPrimalType::ToadStool => "TOADSTOOL",
            EcosystemPrimalType::Songbird => "SONGBIRD",
            EcosystemPrimalType::BearDog => "BEARDOG",
            EcosystemPrimalType::NestGate => "NESTGATE",
            EcosystemPrimalType::Squirrel => "SQUIRREL",
            EcosystemPrimalType::BiomeOS => "BIOMEOS",
        }
    }

    /// Get service name for service discovery
    ///
    /// **DEPRECATED**: Use capability-based discovery instead.
    ///
    /// # Migration Example
    /// ```ignore
    /// // OLD:
    /// let primal = EcosystemPrimalType::Songbird;
    /// let service_name = primal.service_name(); // "songbird"
    /// let url = format!("http://consul:8500/{}", service_name);
    ///
    /// // NEW:
    /// use crate::capability_registry::{CapabilityRegistry, PrimalCapability};
    /// let registry = CapabilityRegistry::new(Default::default());
    /// let primals = registry.discover_by_capability(&PrimalCapability::ServiceMesh).await?;
    /// let endpoint = &primals[0].endpoint; // Discovered endpoint
    /// ```
    #[must_use]
    #[deprecated(since = "0.1.0", note = "Use CapabilityRegistry for discovery")]
    pub fn service_name(&self) -> &'static str {
        self.as_str()
    }

    /// Parse from string
    ///
    /// **DEPRECATED**: Use capability-based discovery instead.
    #[deprecated(since = "0.1.0", note = "Use CapabilityRegistry for discovery")]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "toadstool" => Ok(EcosystemPrimalType::ToadStool),
            "songbird" => Ok(EcosystemPrimalType::Songbird),
            "beardog" => Ok(EcosystemPrimalType::BearDog),
            "nestgate" => Ok(EcosystemPrimalType::NestGate),
            "squirrel" => Ok(EcosystemPrimalType::Squirrel),
            "biomeos" => Ok(EcosystemPrimalType::BiomeOS),
            _ => Err(format!("Unknown primal type: {s}")),
        }
    }
}

/// Service capabilities with proper Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    pub core: Vec<String>,
    pub extended: Vec<String>,
    pub integrations: Vec<String>,
}

/// Service endpoints with proper Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    pub primary: String,
    pub secondary: Vec<String>,
    pub health: Option<String>,
}

/// Health check configuration with Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub failure_threshold: u32,
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU requirements
    pub cpu: String,
    /// Memory requirements
    pub memory: String,
    /// Storage requirements
    pub storage: String,
    /// Network requirements
    pub network: String,
    /// GPU requirements (optional)
    pub gpu: Option<String>,
}

/// Security configuration for ecosystem integration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication requirements
    pub auth_required: bool,
    /// Encryption level required
    pub encryption_level: String,
    /// Access control level
    pub access_level: String,
    /// Security policies to enforce
    pub policies: Vec<String>,
    /// Audit requirements
    pub audit_enabled: bool,
    /// Security level
    pub security_level: String,
}

/// Resource requirements specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum CPU cores required
    pub min_cpu: Option<u32>,
    /// Maximum CPU cores allowed
    pub max_cpu: Option<u32>,
    /// Minimum memory in MB
    pub min_memory_mb: Option<u64>,
    /// Maximum memory in MB
    pub max_memory_mb: Option<u64>,
    /// Storage requirements in GB
    pub storage_gb: Option<u64>,
}

/// Ecosystem configuration for service registration and discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemConfig {
    /// Service identifier
    pub service_id: String,
    /// Service mesh endpoint for registration
    pub service_mesh_endpoint: Option<String>,
    /// Enable automatic health monitoring
    pub enable_health_monitoring: bool,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Enable automatic service discovery
    pub enable_service_discovery: bool,
    /// Service discovery interval in seconds
    pub discovery_interval_secs: u64,
    /// Registry configuration
    pub registry_config: crate::ecosystem::registry::RegistryConfig,
}
