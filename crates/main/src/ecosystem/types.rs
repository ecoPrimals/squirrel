// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
//! Ecosystem type definitions and configurations
//!
//! This module contains all type definitions for ecosystem integration,
//! including service registration, primal types, capabilities, and
//! configuration structures.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use universal_constants::primal_names;

/// Standardized primal types for ecosystem integration (canonical definition)
#[deprecated(
    since = "0.2.0",
    note = "Use CapabilityDomain for capability-based discovery instead of hardcoded primal types"
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
    #[must_use]
    #[deprecated(since = "0.2.0", note = "Use CapabilityDomain for discovery")]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ToadStool => primal_names::TOADSTOOL,
            Self::Songbird => primal_names::SONGBIRD,
            Self::BearDog => primal_names::BEARDOG,
            Self::NestGate => primal_names::NESTGATE,
            Self::Squirrel => primal_names::SQUIRREL,
            Self::BiomeOS => primal_names::BIOMEOS,
        }
    }
    /// Prefix for `{PREFIX}_ENDPOINT`-style configuration, derived from [`Self::capability`].
    #[must_use]
    #[deprecated(since = "0.2.0", note = "Use CapabilityDomain::endpoint_env_prefix")]
    pub fn endpoint_env_prefix(&self) -> String {
        CapabilityIdentifier::new(self.capability()).endpoint_env_prefix()
    }
    #[must_use]
    #[deprecated(since = "0.2.0", note = "Use CapabilityDomain for discovery")]
    pub const fn service_name(&self) -> &'static str {
        self.as_str()
    }
    #[must_use]
    pub const fn capability(&self) -> &'static str {
        use universal_constants::capabilities;
        match self {
            Self::ToadStool => capabilities::COMPUTE_CAPABILITY,
            Self::Songbird => capabilities::SERVICE_MESH_CAPABILITY,
            Self::BearDog => capabilities::SECURITY_CAPABILITY,
            Self::NestGate => capabilities::STORAGE_CAPABILITY,
            Self::Squirrel => capabilities::SELF_PRIMAL_NAME,
            Self::BiomeOS => capabilities::ECOSYSTEM_CAPABILITY,
        }
    }
}

#[expect(deprecated, reason = "implements FromStr for deprecated enum (serde compat)")]
impl std::str::FromStr for EcosystemPrimalType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            primal_names::TOADSTOOL => Ok(Self::ToadStool),
            primal_names::SONGBIRD => Ok(Self::Songbird),
            primal_names::BEARDOG => Ok(Self::BearDog),
            primal_names::NESTGATE => Ok(Self::NestGate),
            primal_names::SQUIRREL => Ok(Self::Squirrel),
            primal_names::BIOMEOS => Ok(Self::BiomeOS),
            _ => Err(format!("Unknown primal type: {s}")),
        }
    }
}

#[expect(deprecated, reason = "implements Display for deprecated enum (logging compat)")]
impl std::fmt::Display for EcosystemPrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Runtime capability identifier — primary routing key for ecosystem discovery.
///
/// Accepts any string-based capability domain (e.g. `"security"`, `"storage"`,
/// `"ai.coordination"`). Prefer [`CapabilityDomain`] when routing discovery queries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapabilityIdentifier(Arc<str>);

impl CapabilityIdentifier {
    /// Create a capability identifier from a string-based capability domain.
    #[must_use]
    pub fn new(capability: impl AsRef<str>) -> Self {
        Self(Arc::from(capability.as_ref()))
    }

    /// Create from a [`CapabilityDomain`].
    #[must_use]
    pub fn from_domain(domain: &CapabilityDomain) -> Self {
        Self(Arc::from(domain.as_str()))
    }

    /// View this identifier as a [`CapabilityDomain`] for discovery routing.
    #[must_use]
    pub fn as_domain(&self) -> CapabilityDomain {
        CapabilityDomain::new(self.as_str())
    }

    /// Get the capability domain string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Prefix for `{PREFIX}_ENDPOINT`-style configuration.
    ///
    /// Hyphens in capability strings become underscores, then the result is uppercased
    /// (for example `service-mesh` → `SERVICE_MESH`).
    #[must_use]
    pub fn endpoint_env_prefix(&self) -> String {
        self.as_str().replace('-', "_").to_uppercase()
    }
}

/// Capability domain for ecosystem routing — what Squirrel needs, not who provides it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityDomain(CapabilityIdentifier);

impl CapabilityDomain {
    /// Create a capability domain from a string (e.g. `"security"`, `"storage"`).
    #[must_use]
    pub fn new(domain: impl AsRef<str>) -> Self {
        Self(CapabilityIdentifier::new(domain))
    }

    /// Get the domain string used for discovery routing.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get the underlying capability identifier.
    #[must_use]
    pub const fn identifier(&self) -> &CapabilityIdentifier {
        &self.0
    }

    /// Prefix for `{PREFIX}_ENDPOINT`-style configuration.
    #[must_use]
    pub fn endpoint_env_prefix(&self) -> String {
        self.0.endpoint_env_prefix()
    }
}

impl From<&str> for CapabilityDomain {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for CapabilityDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Infer deprecated [`EcosystemPrimalType`] from a capability domain (serde backward compat only).
#[deprecated(
    since = "0.2.0",
    note = "Use CapabilityDomain for routing; do not infer primal types"
)]
#[expect(deprecated, reason = "backward-compat shim for serde deserialization")]
#[must_use]
pub fn infer_primal_type_from_capability(capability: &str) -> EcosystemPrimalType {
    use universal_constants::capabilities as caps;
    match capability {
        c if c == caps::COMPUTE_CAPABILITY => EcosystemPrimalType::ToadStool,
        c if c == caps::SERVICE_MESH_CAPABILITY => EcosystemPrimalType::Songbird,
        c if c == caps::SECURITY_CAPABILITY => EcosystemPrimalType::BearDog,
        c if c == caps::STORAGE_CAPABILITY => EcosystemPrimalType::NestGate,
        c if c == caps::SELF_PRIMAL_NAME => EcosystemPrimalType::Squirrel,
        c if c == caps::ECOSYSTEM_CAPABILITY => EcosystemPrimalType::BiomeOS,
        _ => EcosystemPrimalType::BiomeOS,
    }
}

/// Pick the primary capability domain from a provider's advertised capabilities.
#[must_use]
pub fn primary_capability_from_iter<'a>(
    capabilities: impl IntoIterator<Item = &'a str>,
) -> CapabilityIdentifier {
    use universal_constants::capabilities as caps;
    const DOMAIN_PRIORITY: &[&str] = &[
        caps::SECURITY_CAPABILITY,
        caps::SERVICE_MESH_CAPABILITY,
        caps::STORAGE_CAPABILITY,
        caps::COMPUTE_CAPABILITY,
        caps::ECOSYSTEM_CAPABILITY,
        caps::COORDINATION_CAPABILITY,
        caps::VISUALIZATION_CAPABILITY,
    ];
    let caps: Vec<&str> = capabilities.into_iter().collect();
    for domain in DOMAIN_PRIORITY {
        if caps
            .iter()
            .any(|c| *c == *domain || c.starts_with(&format!("{domain}.")))
        {
            return CapabilityIdentifier::new(domain);
        }
    }
    CapabilityIdentifier::new(caps.first().copied().unwrap_or("unknown"))
}

impl From<&str> for CapabilityIdentifier {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for CapabilityIdentifier {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

impl From<CapabilityDomain> for CapabilityIdentifier {
    fn from(domain: CapabilityDomain) -> Self {
        Self::from_domain(&domain)
    }
}

impl std::fmt::Display for CapabilityIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Well-known capability domain constants for discovery routing.
///
/// Use these as discoverable strings when finding providers by capability.
/// Primals announce and discover by capability domain, not by product name.
pub mod capabilities {
    pub use universal_constants::capabilities::{
        COMPUTE_CAPABILITY as COMPUTE, COORDINATION_CAPABILITY as COORDINATION,
        ECOSYSTEM_CAPABILITY as ECOSYSTEM, SECURITY_CAPABILITY as SECURITY,
        SERVICE_MESH_CAPABILITY as SERVICE_MESH, STORAGE_CAPABILITY as STORAGE,
        VISUALIZATION_CAPABILITY as VISUALIZATION,
    };

    /// AI coordination capability (method-level)
    pub const AI_COORDINATION: &str = "ai.coordination";
    /// Lifecycle management capability (method-level)
    pub const LIFECYCLE: &str = "lifecycle.management";
    /// Capability discovery domain
    pub const DISCOVERY: &str = "discovery";
}

// EcosystemServiceRegistration is defined in registration.rs with Arc<str> zero-copy service_id

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
/// Service capabilities with proper Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    /// Core capabilities
    pub core: Vec<String>,
    /// Extended capabilities
    pub extended: Vec<String>,
    /// Integration capabilities
    pub integrations: Vec<String>,
}

/// Service endpoints with proper Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    /// Primary endpoint URL
    pub primary: String,
    /// Secondary endpoint URLs
    pub secondary: Vec<String>,
    /// Optional health check endpoint
    pub health: Option<String>,
}

/// Health check configuration with Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Whether health checks are enabled
    pub enabled: bool,
    /// Interval between checks in seconds
    pub interval_secs: u64,
    /// Timeout per check in seconds
    pub timeout_secs: u64,
    /// Failures before marking unhealthy
    pub failure_threshold: u32,
}

/// Resource requirements specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- CapabilityIdentifier ---

    #[test]
    fn capability_identifier_new_and_as_str() {
        let cap = CapabilityIdentifier::new("compute");
        assert_eq!(cap.as_str(), "compute");
    }

    #[test]
    fn capability_identifier_from_str() {
        let cap: CapabilityIdentifier = "storage".into();
        assert_eq!(cap.as_str(), "storage");
    }

    #[test]
    fn capability_identifier_display() {
        let cap = CapabilityIdentifier::new("ai.coordination");
        assert_eq!(format!("{cap}"), "ai.coordination");
    }

    #[test]
    fn capability_identifier_eq_and_hash() {
        let a = CapabilityIdentifier::new("compute");
        let b = CapabilityIdentifier::new("compute");
        let c = CapabilityIdentifier::new("storage");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn capability_identifier_endpoint_env_prefix() {
        assert_eq!(
            CapabilityIdentifier::new("service-mesh").endpoint_env_prefix(),
            "SERVICE_MESH"
        );
        assert_eq!(
            CapabilityIdentifier::new("security").endpoint_env_prefix(),
            "SECURITY"
        );
    }

    #[test]
    fn capability_domain_routing() {
        let domain: CapabilityDomain = "storage".into();
        assert_eq!(domain.as_str(), "storage");
        assert_eq!(domain.endpoint_env_prefix(), "STORAGE");
    }

    #[test]
    fn primary_capability_from_iter_prefers_domain() {
        let cap = primary_capability_from_iter(["ai.query", "security.auth", "health"]);
        assert_eq!(cap.as_str(), "security");
    }

    // --- ServiceCapabilities ---

    #[test]
    fn test_service_capabilities_default() {
        let caps = ServiceCapabilities::default();
        assert!(caps.core.is_empty());
        assert!(caps.extended.is_empty());
        assert!(caps.integrations.is_empty());
    }

    #[test]
    fn test_service_capabilities_serde() {
        let caps = ServiceCapabilities {
            core: vec!["ai_coordination".to_string()],
            extended: vec!["universal_patterns".to_string()],
            integrations: vec!["ecosystem_integration".to_string()],
        };
        let json = serde_json::to_string(&caps).expect("should succeed");
        let deserialized: ServiceCapabilities =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.core, vec!["ai_coordination"]);
        assert_eq!(deserialized.extended, vec!["universal_patterns"]);
    }

    // --- ServiceEndpoints ---

    #[test]
    fn test_service_endpoints_default() {
        let endpoints = ServiceEndpoints::default();
        assert!(endpoints.primary.is_empty());
        assert!(endpoints.secondary.is_empty());
        assert!(endpoints.health.is_none());
    }

    #[test]
    fn test_service_endpoints_serde() {
        // Zero-HTTP: use Unix socket paths; endpoints discovered at runtime
        let endpoints = ServiceEndpoints {
            primary: "unix:///run/user/1000/biomeos/squirrel.sock".to_string(),
            secondary: vec!["unix:///run/user/1000/biomeos/songbird.sock".to_string()],
            health: None, // health via JSON-RPC; discovered at runtime
        };
        let json = serde_json::to_string(&endpoints).expect("should succeed");
        let deserialized: ServiceEndpoints = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(
            deserialized.primary,
            "unix:///run/user/1000/biomeos/squirrel.sock"
        );
        assert!(deserialized.health.is_none());
    }

    // --- HealthCheckConfig ---

    #[test]
    fn test_health_check_config_default() {
        let config = HealthCheckConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.interval_secs, 0);
        assert_eq!(config.timeout_secs, 0);
        assert_eq!(config.failure_threshold, 0);
    }

    #[test]
    fn test_health_check_config_serde() {
        let config = HealthCheckConfig {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 5,
            failure_threshold: 3,
        };
        let json = serde_json::to_string(&config).expect("should succeed");
        let deserialized: HealthCheckConfig = serde_json::from_str(&json).expect("should succeed");
        assert!(deserialized.enabled);
        assert_eq!(deserialized.interval_secs, 30);
        assert_eq!(deserialized.failure_threshold, 3);
    }

    // --- ResourceSpec ---

    #[test]
    fn test_resource_spec_default() {
        let spec = ResourceSpec::default();
        assert!(spec.cpu.is_empty());
        assert!(spec.memory.is_empty());
        assert!(spec.gpu.is_none());
    }

    #[test]
    fn test_resource_spec_serde() {
        let spec = ResourceSpec {
            cpu: "500m".to_string(),
            memory: "1Gi".to_string(),
            storage: "10Gi".to_string(),
            network: "1Gbps".to_string(),
            gpu: Some("nvidia-t4".to_string()),
        };
        let json = serde_json::to_string(&spec).expect("should succeed");
        let deserialized: ResourceSpec = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.cpu, "500m");
        assert_eq!(deserialized.gpu.as_deref(), Some("nvidia-t4"));
    }

    // --- SecurityConfig ---

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(!config.auth_required);
        assert!(config.encryption_level.is_empty());
        assert!(config.policies.is_empty());
        assert!(!config.audit_enabled);
    }

    #[test]
    fn test_security_config_serde() {
        let config = SecurityConfig {
            auth_required: true,
            encryption_level: "tls1.3".to_string(),
            access_level: "admin".to_string(),
            policies: vec!["strict".to_string()],
            audit_enabled: true,
            security_level: "high".to_string(),
        };
        let json = serde_json::to_string(&config).expect("should succeed");
        let deserialized: SecurityConfig = serde_json::from_str(&json).expect("should succeed");
        assert!(deserialized.auth_required);
        assert_eq!(deserialized.encryption_level, "tls1.3");
        assert!(deserialized.audit_enabled);
    }

    // --- ResourceRequirements ---

    #[test]
    fn test_resource_requirements_default() {
        let req = ResourceRequirements::default();
        assert!(req.min_cpu.is_none());
        assert!(req.max_cpu.is_none());
        assert!(req.min_memory_mb.is_none());
        assert!(req.max_memory_mb.is_none());
        assert!(req.storage_gb.is_none());
    }

    #[test]
    fn test_resource_requirements_serde() {
        let req = ResourceRequirements {
            min_cpu: Some(2),
            max_cpu: Some(8),
            min_memory_mb: Some(4096),
            max_memory_mb: Some(16384),
            storage_gb: Some(100),
        };
        let json = serde_json::to_string(&req).expect("should succeed");
        let deserialized: ResourceRequirements =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.min_cpu, Some(2));
        assert_eq!(deserialized.max_cpu, Some(8));
        assert_eq!(deserialized.storage_gb, Some(100));
    }

    // --- EcosystemPrimalType (deprecated but still used in registration) ---

    #[expect(deprecated, reason = "tests deprecated enum backward compat")]
    #[test]
    fn ecosystem_primal_type_from_str_accepts_case_insensitive() {
        assert_eq!(
            "Songbird"
                .parse::<EcosystemPrimalType>()
                .expect("should succeed"),
            EcosystemPrimalType::Songbird
        );
        assert_eq!(
            "NESTGATE"
                .parse::<EcosystemPrimalType>()
                .expect("should succeed"),
            EcosystemPrimalType::NestGate
        );
    }

    #[expect(deprecated, reason = "tests deprecated enum backward compat")]
    #[test]
    fn ecosystem_primal_type_from_str_rejects_unknown() {
        assert!("unknown".parse::<EcosystemPrimalType>().is_err());
    }

    #[expect(deprecated, reason = "tests deprecated enum backward compat")]
    #[test]
    fn ecosystem_primal_type_display_and_capability() {
        let t = EcosystemPrimalType::Squirrel;
        assert_eq!(t.to_string(), "squirrel");
        assert!(!t.capability().is_empty());
    }

    #[expect(deprecated, reason = "tests deprecated enum backward compat")]
    #[test]
    fn capability_identifier_from_domain() {
        let domain = CapabilityDomain::new("security");
        let id = CapabilityIdentifier::from_domain(&domain);
        assert_eq!(id.as_str(), "security");
        assert_eq!(id.as_domain().as_str(), "security");
        let from: CapabilityIdentifier = domain.into();
        assert_eq!(from.as_str(), "security");
    }

    #[expect(deprecated, reason = "tests deprecated enum backward compat")]
    #[test]
    fn ecosystem_primal_type_endpoint_env_prefix() {
        assert_eq!(
            EcosystemPrimalType::BearDog.endpoint_env_prefix(),
            "SECURITY"
        );
        assert_eq!(
            EcosystemPrimalType::Songbird.endpoint_env_prefix(),
            "SERVICE_MESH"
        );
    }
}
