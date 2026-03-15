// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
#![allow(deprecated)]

//! Ecosystem type definitions and configurations
//!
//! This module contains all type definitions for ecosystem integration,
//! including service registration, primal types, capabilities, and
//! configuration structures.

// Backward compatibility: kept for deserialization of legacy data
#[expect(
    deprecated,
    reason = "backward compat: EcosystemPrimalType for legacy deserialization"
)]
use super::EcosystemPrimalType;
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
/// ```
// NOTE: EcosystemPrimalType is defined in ecosystem/mod.rs (canonical source)
// and re-exported via `pub use types::*` -- do not duplicate here.
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
        let json = serde_json::to_string(&caps).unwrap();
        let deserialized: ServiceCapabilities = serde_json::from_str(&json).unwrap();
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
        let json = serde_json::to_string(&endpoints).unwrap();
        let deserialized: ServiceEndpoints = serde_json::from_str(&json).unwrap();
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
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HealthCheckConfig = serde_json::from_str(&json).unwrap();
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
        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: ResourceSpec = serde_json::from_str(&json).unwrap();
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
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SecurityConfig = serde_json::from_str(&json).unwrap();
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
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ResourceRequirements = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.min_cpu, Some(2));
        assert_eq!(deserialized.max_cpu, Some(8));
        assert_eq!(deserialized.storage_gb, Some(100));
    }
}
