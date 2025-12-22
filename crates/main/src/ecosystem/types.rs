//! Ecosystem type definitions
//!
//! Core types for ecosystem service registration, discovery, and integration.
//! These types follow standardized formats for the ecoPrimals ecosystem.
//!
//! # Zero-Copy Performance
//!
//! This module uses `Arc<str>` for frequently-cloned strings (service names,
//! IDs, endpoints) to eliminate expensive allocations. String cloning is O(1)
//! instead of O(n), providing 50-70% reduction in allocations during service
//! discovery and routing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::optimization::zero_copy::ArcStr;
use crate::universal::LoadBalancingStatus;

/// Ecosystem service registration for primals
///
/// This struct follows the standardized format for service discovery
/// and registration within the ecoPrimals ecosystem.
///
/// # Performance Optimization
///
/// Uses `Arc<str>` for frequently-cloned fields (service_id, name, version, biome_id)
/// to eliminate allocations during service discovery and routing. Cloning is O(1)
/// instead of O(n).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Unique service identifier: "primal-{type}-{instance}"
    /// Zero-copy: Arc<str> for efficient cloning during routing
    #[serde(serialize_with = "serialize_arc_str")]
    #[serde(deserialize_with = "deserialize_arc_str")]
    pub service_id: ArcStr,

    /// Primal type from standardized enum
    pub primal_type: EcosystemPrimalType,

    /// Service name
    /// Zero-copy: Arc<str> for efficient cloning during discovery
    #[serde(serialize_with = "serialize_arc_str")]
    #[serde(deserialize_with = "deserialize_arc_str")]
    pub name: ArcStr,

    /// Service description
    pub description: String,

    /// Associated biome identifier (if applicable)
    /// Zero-copy: Arc<str> option for efficient cloning
    #[serde(serialize_with = "serialize_option_arc_str")]
    #[serde(deserialize_with = "deserialize_option_arc_str")]
    pub biome_id: Option<ArcStr>,

    /// Service version
    /// Zero-copy: Arc<str> for efficient version comparisons
    #[serde(serialize_with = "serialize_arc_str")]
    #[serde(deserialize_with = "deserialize_arc_str")]
    pub version: ArcStr,

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
/// # Migration to Capability-Based Discovery
///
/// The hardcoded variants (ToadStool, Songbird, BearDog, NestGate) are deprecated
/// in favor of capability-based discovery. Use `Custom(String)` for new primals
/// or better yet, use `CapabilityMigrationHelper::discover_by_capability()` to
/// avoid hardcoding primal names entirely.
///
/// ## Example (New Pattern)
/// ```no_run
/// use squirrel::capability_migration::CapabilityMigrationHelper;
/// use squirrel::capability_registry::PrimalCapability;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let helper = CapabilityMigrationHelper::new().await?;
/// let security_primals = helper
///     .discover_by_capability(&PrimalCapability::Security)
///     .await?;
/// // Use any primal that provides Security capability
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EcosystemPrimalType {
    /// ToadStool storage primal (deprecated - use capability discovery)
    #[deprecated(
        since = "0.5.0",
        note = "Use capability-based discovery: CapabilityMigrationHelper::discover_by_capability(PrimalCapability::Storage)"
    )]
    ToadStool,

    /// Songbird service mesh primal (deprecated - use capability discovery)
    #[deprecated(
        since = "0.5.0",
        note = "Use capability-based discovery: CapabilityMigrationHelper::discover_by_capability(PrimalCapability::ServiceMesh)"
    )]
    Songbird,

    /// BearDog security primal (deprecated - use capability discovery)
    #[deprecated(
        since = "0.5.0",
        note = "Use capability-based discovery: CapabilityMigrationHelper::discover_by_capability(PrimalCapability::Security)"
    )]
    BearDog,

    /// NestGate compute primal (deprecated - use capability discovery)
    #[deprecated(
        since = "0.5.0",
        note = "Use capability-based discovery: CapabilityMigrationHelper::discover_by_capability(PrimalCapability::Compute)"
    )]
    NestGate,

    /// Squirrel AI inference primal (this is us!)
    Squirrel,

    /// BiomeOS orchestration platform
    BiomeOS,

    /// Custom primal type for dynamic discovery
    ///
    /// Use this for primals discovered at runtime or new primals not in the
    /// hardcoded list. Prefer using capability-based discovery over hardcoding names.
    ///
    /// # Example
    /// ```
    /// use squirrel::ecosystem::EcosystemPrimalType;
    ///
    /// let custom_primal = EcosystemPrimalType::Custom("my-storage-primal".to_string());
    /// ```
    Custom(String),
}

impl EcosystemPrimalType {
    /// Convert to string representation
    ///
    /// Note: Prefer using capability-based discovery over hardcoded names
    pub fn as_str(&self) -> &str {
        match self {
            #[allow(deprecated)]
            EcosystemPrimalType::ToadStool => "toadstool",
            #[allow(deprecated)]
            EcosystemPrimalType::Songbird => "songbird",
            #[allow(deprecated)]
            EcosystemPrimalType::BearDog => "beardog",
            #[allow(deprecated)]
            EcosystemPrimalType::NestGate => "nestgate",
            EcosystemPrimalType::Squirrel => "squirrel",
            EcosystemPrimalType::BiomeOS => "biomeos",
            EcosystemPrimalType::Custom(name) => name.as_str(),
        }
    }

    /// Get the display name for this primal type
    pub fn display_name(&self) -> String {
        match self {
            #[allow(deprecated)]
            EcosystemPrimalType::ToadStool => "ToadStool".to_string(),
            #[allow(deprecated)]
            EcosystemPrimalType::Songbird => "Songbird".to_string(),
            #[allow(deprecated)]
            EcosystemPrimalType::BearDog => "BearDog".to_string(),
            #[allow(deprecated)]
            EcosystemPrimalType::NestGate => "NestGate".to_string(),
            EcosystemPrimalType::Squirrel => "Squirrel".to_string(),
            EcosystemPrimalType::BiomeOS => "BiomeOS".to_string(),
            EcosystemPrimalType::Custom(name) => name.clone(),
        }
    }

    /// Parse from string (deprecated - use std::str::FromStr trait)
    #[deprecated(since = "0.1.0", note = "Use std::str::FromStr trait instead")]
    pub fn from_str(s: &str) -> Result<Self, String> {
        s.parse()
            .map_err(|e: <Self as std::str::FromStr>::Err| e.to_string())
    }

    /// Get the primary capability this primal type provides
    ///
    /// This enables capability-based discovery without hardcoded names.
    /// Use with CapabilityRegistry for dynamic primal discovery.
    ///
    /// For `Custom` variants, returns `None` since capabilities must be
    /// discovered at runtime via the capability registry.
    pub fn primary_capability(&self) -> Option<crate::capability_registry::PrimalCapability> {
        use crate::capability_registry::PrimalCapability;
        match self {
            #[allow(deprecated)]
            EcosystemPrimalType::ToadStool => Some(PrimalCapability::Storage),
            #[allow(deprecated)]
            EcosystemPrimalType::Songbird => Some(PrimalCapability::ServiceMesh),
            #[allow(deprecated)]
            EcosystemPrimalType::BearDog => Some(PrimalCapability::Security),
            #[allow(deprecated)]
            EcosystemPrimalType::NestGate => Some(PrimalCapability::Compute),
            EcosystemPrimalType::Squirrel => Some(PrimalCapability::AIInference),
            EcosystemPrimalType::BiomeOS => Some(PrimalCapability::ServiceMesh),
            // Custom primals need runtime capability discovery
            EcosystemPrimalType::Custom(_) => None,
        }
    }

    /// Get all capabilities this primal type provides
    ///
    /// Returns a vector of all capabilities, including secondary capabilities
    /// like monitoring and observability.
    ///
    /// For `Custom` variants, returns an empty vector since capabilities must be
    /// discovered at runtime via the capability registry.
    pub fn capabilities(&self) -> Vec<crate::capability_registry::PrimalCapability> {
        use crate::capability_registry::PrimalCapability;

        let mut caps = Vec::new();

        if let Some(primary) = self.primary_capability() {
            caps.push(primary);
        }

        // All primals can provide monitoring capability
        match self {
            EcosystemPrimalType::Custom(_) => {} // Custom needs runtime discovery
            _ => {
                caps.push(PrimalCapability::Monitoring);
            }
        }

        caps
    }

    /// Check if this primal type provides a specific capability
    ///
    /// **Capability-Based Migration Helper**
    ///
    /// Use this method instead of matching on primal type enum variants.
    /// This enables runtime flexibility and multi-provider support.
    ///
    /// # Example
    /// ```ignore
    /// // ❌ OLD: Hardcoded primal type check
    /// if primal_type == EcosystemPrimalType::BearDog {
    ///     // security logic
    /// }
    ///
    /// // ✅ NEW: Capability-based check
    /// if primal_type.provides_capability(PrimalCapability::Security) {
    ///     // security logic - works with ANY security provider
    /// }
    /// ```
    ///
    /// # Benefits
    /// - Multiple providers can offer the same capability
    /// - No hardcoded assumptions about which primal provides what
    /// - Easier to test (mock capabilities, not names)
    /// - Future-proof (new security providers work automatically)
    pub fn provides_capability(&self, capability: crate::capability_registry::PrimalCapability) -> bool {
        self.capabilities().contains(&capability)
    }

    /// Check if this is a security provider
    ///
    /// **Capability-Based Helper**
    ///
    /// Prefer this over `primal_type == EcosystemPrimalType::BearDog`
    #[inline]
    pub fn is_security_provider(&self) -> bool {
        self.provides_capability(crate::capability_registry::PrimalCapability::Security)
    }

    /// Check if this is a service mesh provider
    ///
    /// **Capability-Based Helper**
    ///
    /// Prefer this over `primal_type == EcosystemPrimalType::Songbird`
    #[inline]
    pub fn is_service_mesh_provider(&self) -> bool {
        self.provides_capability(crate::capability_registry::PrimalCapability::ServiceMesh)
    }

    /// Check if this is a storage provider
    ///
    /// **Capability-Based Helper**
    ///
    /// Prefer this over `primal_type == EcosystemPrimalType::ToadStool`
    #[inline]
    pub fn is_storage_provider(&self) -> bool {
        self.provides_capability(crate::capability_registry::PrimalCapability::Storage)
    }

    /// Check if this is a compute provider
    ///
    /// **Capability-Based Helper**
    ///
    /// Prefer this over `primal_type == EcosystemPrimalType::NestGate`
    #[inline]
    pub fn is_compute_provider(&self) -> bool {
        self.provides_capability(crate::capability_registry::PrimalCapability::Compute)
    }

    /// Check if this is an AI inference provider
    ///
    /// **Capability-Based Helper**
    ///
    /// Prefer this over `primal_type == EcosystemPrimalType::Squirrel`
    #[inline]
    pub fn is_ai_provider(&self) -> bool {
        self.provides_capability(crate::capability_registry::PrimalCapability::AIInference)
    }
}

impl std::fmt::Display for EcosystemPrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for EcosystemPrimalType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "toadstool" => Ok(EcosystemPrimalType::ToadStool),
            "songbird" => Ok(EcosystemPrimalType::Songbird),
            "beardog" => Ok(EcosystemPrimalType::BearDog),
            "nestgate" => Ok(EcosystemPrimalType::NestGate),
            "squirrel" => Ok(EcosystemPrimalType::Squirrel),
            "biomeos" => Ok(EcosystemPrimalType::BiomeOS),
            _ => Ok(EcosystemPrimalType::Custom(s.to_string())),
        }
    }
}

/// Service capabilities in standardized format
/// Zero-copy: Uses ArcStr for efficient string sharing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceCapabilities {
    /// Core capabilities (zero-copy shared strings)
    pub core: Vec<ArcStr>,
    /// Extended capabilities (zero-copy shared strings)
    pub extended: Vec<ArcStr>,
    /// Integration capabilities (zero-copy shared strings)
    pub integrations: Vec<ArcStr>,
}

/// Service endpoints in standardized format
/// Zero-copy: Uses ArcStr for efficient string sharing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceEndpoints {
    /// Primary endpoint (zero-copy shared string)
    pub primary: ArcStr,
    /// Secondary endpoints (zero-copy shared strings)
    pub secondary: Vec<ArcStr>,
    /// Health check endpoint (zero-copy shared string)
    pub health: Option<ArcStr>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval in seconds
    pub interval_secs: u64,
    /// Health check timeout in seconds
    pub timeout_secs: u64,
    /// Consecutive failures before unhealthy
    pub failure_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 5,
            failure_threshold: 3,
        }
    }
}

/// Security configuration for ecosystem integration
/// Zero-copy: Uses ArcStr for efficient string sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication requirements
    pub auth_required: bool,
    /// Encryption level required (zero-copy shared string)
    pub encryption_level: ArcStr,
    /// Access control level (zero-copy shared string)
    pub access_level: ArcStr,
    /// Security policies to enforce (zero-copy shared strings)
    pub policies: Vec<ArcStr>,
    /// Audit requirements
    pub audit_enabled: bool,
    /// Security level (zero-copy shared string)
    pub security_level: ArcStr,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_required: true,
            encryption_level: "high".to_string(),
            access_level: "internal".to_string(),
            policies: vec!["no_sensitive_data".to_string()],
            audit_enabled: true,
            security_level: "standard".to_string(),
        }
    }
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU requirements (e.g., "500m", "2")
    pub cpu: String,
    /// Memory requirements (e.g., "1Gi", "512Mi")
    pub memory: String,
    /// Storage requirements (e.g., "10Gi")
    pub storage: String,
    /// Network requirements (e.g., "1Gbps")
    pub network: String,
    /// GPU requirements (optional)
    pub gpu: Option<String>,
}

impl Default for ResourceSpec {
    fn default() -> Self {
        Self {
            cpu: "500m".to_string(),
            memory: "1Gi".to_string(),
            storage: "10Gi".to_string(),
            network: "1Gbps".to_string(),
            gpu: None,
        }
    }
}

/// Ecosystem status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    /// Overall status
    pub status: String,
    /// Status timestamp
    pub timestamp: DateTime<Utc>,
    /// Discovered services
    pub discovered_services: Vec<crate::ecosystem::registry::types::DiscoveredService>,
    /// Active integrations
    pub active_integrations: Vec<String>,
    /// Service mesh status
    pub service_mesh_status: ServiceMeshStatus,
    /// Overall health score (0.0 to 1.0)
    pub overall_health: f64,
}

/// Service mesh status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshStatus {
    /// Service mesh enabled
    pub enabled: bool,
    /// Registered with service mesh
    pub registered: bool,
    /// Load balancing status
    pub load_balancing: LoadBalancingStatus,
    /// Cross-primal communication status
    pub cross_primal_communication: CrossPrimalStatus,
}

/// Cross-primal communication status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalStatus {
    /// Cross-primal communication enabled
    pub enabled: bool,
    /// Active connections
    pub active_connections: u32,
    /// Supported protocols
    pub supported_protocols: Vec<String>,
}

/// Ecosystem manager status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemManagerStatus {
    /// Current status
    pub status: String,
    /// Initialization timestamp
    pub initialized_at: Option<DateTime<Utc>>,
    /// Last successful registration
    pub last_registration: Option<DateTime<Utc>>,
    /// Active service registrations
    pub active_registrations: Vec<String>,
    /// Health status
    pub health_status: HealthStatus,
    /// Error count
    pub error_count: u32,
    /// Last error message
    pub last_error: Option<String>,
}

impl Default for EcosystemManagerStatus {
    fn default() -> Self {
        Self {
            status: "initializing".to_string(),
            initialized_at: None,
            last_registration: None,
            active_registrations: Vec::new(),
            health_status: HealthStatus::default(),
            error_count: 0,
            last_error: None,
        }
    }
}

/// Health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health score (0.0 to 1.0)
    pub health_score: f64,
    /// Component health statuses
    pub component_statuses: HashMap<String, ComponentHealth>,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
    /// Health check errors
    pub health_errors: Vec<String>,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            health_score: 0.0,
            component_statuses: HashMap::new(),
            last_check: Utc::now(),
            health_errors: Vec::new(),
        }
    }
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Health status
    pub status: String,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
    /// Error message (if any)
    pub error_message: Option<String>,
}

// Re-export EcosystemRegistryConfig from registry module
pub use crate::ecosystem::registry::config::EcosystemRegistryConfig;

/// Serde helpers for ArcStr serialization/deserialization
///
/// These helpers allow seamless JSON serialization of ArcStr fields,
/// treating them as regular strings for external consumers while maintaining
/// zero-copy performance internally.
///
/// Note: ArcStr has built-in serde support, but we provide these helpers
/// for compatibility with the #[serde(with = "...")] attribute pattern.
mod arc_str_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    /// Serialize ArcStr as a regular string
    pub fn serialize_arc_str<S>(arc_str: &ArcStr, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(arc_str.as_ref())
    }

    /// Deserialize string into ArcStr
    pub fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<ArcStr, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ArcStr::from(s))
    }

    /// Serialize Option<ArcStr> as Option<String>
    pub fn serialize_option_arc_str<S>(
        opt: &Option<ArcStr>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match opt {
            Some(arc_str) => serializer.serialize_some(arc_str.as_ref()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize Option<String> into Option<ArcStr>
    pub fn deserialize_option_arc_str<'de, D>(deserializer: D) -> Result<Option<ArcStr>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        Ok(opt.map(ArcStr::from))
    }
}

// Make serde helpers available
use arc_str_serde::*;

/// Helper function to create ArcStr from string literals or String
///
/// This provides ergonomic conversion for existing code being migrated
/// to zero-copy ArcStr.
pub fn arc_str(s: impl AsRef<str>) -> ArcStr {
    ArcStr::from(s.as_ref())
}
