// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Compute Provider Management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::universal::PrimalCapability;
use crate::universal_primal_ecosystem::DiscoveredPrimal;

// ============================================================================
// COMPUTE PROVIDER TYPES
// ============================================================================

/// Discovered compute provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProvider {
    /// Provider identifier
    pub provider_id: String,

    /// Provider metadata
    pub metadata: ComputeProviderMetadata,

    /// Available capabilities
    pub capabilities: Vec<PrimalCapability>,

    /// Current health and performance
    pub health: ComputeProviderHealth,

    /// AI-generated routing score
    pub routing_score: f64,
}

/// Compute provider metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProviderMetadata {
    /// Provider name
    pub name: String,

    /// Provider version
    pub version: String,

    /// Supported architectures
    pub architectures: Vec<String>,

    /// Available frameworks
    pub frameworks: Vec<String>,

    /// Geographic regions
    pub regions: Vec<String>,

    /// Compliance certifications
    pub compliance: Vec<String>,
}

/// Compute provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProviderHealth {
    /// Overall health score (0.0 - 1.0)
    pub health_score: f64,

    /// Current CPU load
    pub cpu_load: f64,

    /// Current memory usage
    pub memory_usage: f64,

    /// Queue length
    pub queue_length: u32,

    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,

    /// Last health check
    pub last_check: DateTime<Utc>,
}

impl ComputeProvider {
    /// Create compute provider from discovered primal
    #[must_use]
    pub fn from_discovered_primal(primal: &DiscoveredPrimal) -> Self {
        Self {
            provider_id: primal.instance_id.clone(),
            metadata: ComputeProviderMetadata {
                name: primal.id.clone(),
                version: "unknown".to_string(),
                architectures: vec!["x86_64".to_string()],
                frameworks: vec!["pytorch".to_string(), "tensorflow".to_string()],
                regions: vec!["local".to_string()],
                compliance: Vec::new(),
            },
            capabilities: vec![crate::universal::PrimalCapability::Custom {
                name: "compute".to_string(),
                description: "Compute provider capability".to_string(),
                metadata: std::collections::HashMap::new(),
                attributes: std::collections::HashMap::new(),
            }],
            health: ComputeProviderHealth {
                health_score: 1.0,
                cpu_load: 0.5,
                memory_usage: 0.4,
                queue_length: 0,
                avg_execution_time_ms: 1000.0,
                last_check: Utc::now(),
            },
            routing_score: 0.8, // Base score, would be calculated by AI
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use universal_patterns::traits::{
        NetworkLocation, PrimalContext, PrimalHealth, PrimalType, SecurityLevel,
    };

    fn make_discovered_primal() -> DiscoveredPrimal {
        DiscoveredPrimal {
            id: "test-compute-primal".to_string(),
            instance_id: "instance-001".to_string(),
            primal_type: PrimalType::Custom("compute".to_string()),
            capabilities: vec![],
            endpoint: "http://localhost:9000".to_string(),
            health: PrimalHealth::Healthy,
            context: PrimalContext {
                user_id: "test-user".to_string(),
                device_id: "test-device".to_string(),
                session_id: "test-session".to_string(),
                network_location: NetworkLocation {
                    ip_address: "127.0.0.1".to_string(),
                    subnet: None,
                    network_id: None,
                    geo_location: None,
                },
                security_level: SecurityLevel::Standard,
                metadata: std::collections::HashMap::default(),
            },
            port_info: None,
        }
    }

    #[test]
    fn test_compute_provider_from_discovered_primal() {
        let primal = make_discovered_primal();
        let provider = ComputeProvider::from_discovered_primal(&primal);

        assert_eq!(provider.provider_id, "instance-001");
        assert_eq!(provider.metadata.name, "test-compute-primal");
        assert_eq!(provider.metadata.version, "unknown");
        assert!(!provider.metadata.architectures.is_empty());
        assert!(!provider.metadata.frameworks.is_empty());
        assert_eq!(provider.metadata.regions, vec!["local".to_string()]);
        assert!(provider.metadata.compliance.is_empty());
        assert_eq!(provider.capabilities.len(), 1);
        assert!((provider.health.health_score - 1.0).abs() < f64::EPSILON);
        assert!((provider.routing_score - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_provider_health_fields() {
        let primal = make_discovered_primal();
        let provider = ComputeProvider::from_discovered_primal(&primal);

        assert!((provider.health.cpu_load - 0.5).abs() < f64::EPSILON);
        assert!((provider.health.memory_usage - 0.4).abs() < f64::EPSILON);
        assert_eq!(provider.health.queue_length, 0);
        assert!((provider.health.avg_execution_time_ms - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_provider_serde() {
        let primal = make_discovered_primal();
        let provider = ComputeProvider::from_discovered_primal(&primal);

        let json = serde_json::to_string(&provider).expect("should succeed");
        let deserialized: ComputeProvider = serde_json::from_str(&json).expect("should succeed");

        assert_eq!(deserialized.provider_id, provider.provider_id);
        assert_eq!(deserialized.metadata.name, provider.metadata.name);
        assert!((deserialized.routing_score - provider.routing_score).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_provider_metadata_serde() {
        let metadata = ComputeProviderMetadata {
            name: "test-provider".to_string(),
            version: "1.0.0".to_string(),
            architectures: vec!["x86_64".to_string(), "aarch64".to_string()],
            frameworks: vec!["pytorch".to_string()],
            regions: vec!["us-east-1".to_string()],
            compliance: vec!["SOC2".to_string()],
        };

        let json = serde_json::to_string(&metadata).expect("should succeed");
        let deserialized: ComputeProviderMetadata =
            serde_json::from_str(&json).expect("should succeed");

        assert_eq!(deserialized.name, "test-provider");
        assert_eq!(deserialized.architectures.len(), 2);
        assert_eq!(deserialized.compliance.len(), 1);
    }

    #[test]
    fn test_compute_provider_health_serde() {
        let health = ComputeProviderHealth {
            health_score: 0.95,
            cpu_load: 0.3,
            memory_usage: 0.7,
            queue_length: 5,
            avg_execution_time_ms: 250.0,
            last_check: Utc::now(),
        };

        let json = serde_json::to_string(&health).expect("should succeed");
        let deserialized: ComputeProviderHealth =
            serde_json::from_str(&json).expect("should succeed");

        assert!((deserialized.health_score - 0.95).abs() < f64::EPSILON);
        assert_eq!(deserialized.queue_length, 5);
    }
}
