//! Storage Provider Management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::universal::PrimalCapability;
use crate::universal_primal_ecosystem::DiscoveredPrimal;

// ============================================================================
// STORAGE PROVIDER TYPES
// ============================================================================

/// Discovered storage provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProvider {
    /// Provider identifier
    pub provider_id: String,

    /// Provider metadata
    pub metadata: StorageProviderMetadata,

    /// Available capabilities
    pub capabilities: Vec<PrimalCapability>,

    /// Current health and performance
    pub health: StorageProviderHealth,

    /// AI-generated routing score
    pub routing_score: f64,
}

/// Storage provider metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProviderMetadata {
    /// Provider name
    pub name: String,

    /// Provider version
    pub version: String,

    /// Supported protocols
    pub protocols: Vec<String>,

    /// Geographic regions
    pub regions: Vec<String>,

    /// Compliance certifications
    pub compliance: Vec<String>,
}

/// Storage provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProviderHealth {
    /// Overall health score (0.0 - 1.0)
    pub health_score: f64,

    /// Current latency (ms)
    pub current_latency_ms: f64,

    /// Current throughput (MB/s)
    pub current_throughput_mbps: f64,

    /// Availability percentage
    pub availability_percent: f64,

    /// Last health check
    pub last_check: DateTime<Utc>,
}

impl StorageProvider {
    /// Create storage provider from discovered primal
    pub fn from_discovered_primal(primal: &DiscoveredPrimal) -> Self {
        Self {
            provider_id: primal.instance_id.clone(),
            metadata: StorageProviderMetadata {
                name: primal.id.clone(),
                version: "unknown".to_string(),
                protocols: vec!["http".to_string()],
                regions: vec!["local".to_string()],
                compliance: Vec::new(),
            },
            capabilities: vec![crate::universal::PrimalCapability::Custom {
                name: "storage".to_string(),
                attributes: std::collections::HashMap::new(),
            }],
            health: StorageProviderHealth {
                health_score: 1.0,
                current_latency_ms: 50.0,
                current_throughput_mbps: 100.0,
                availability_percent: 99.9,
                last_check: Utc::now(),
            },
            routing_score: 0.8, // Base score, would be calculated by AI
        }
    }
}
