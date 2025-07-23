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
