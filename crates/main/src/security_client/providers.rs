// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security Provider Management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::{ComplianceFramework, TrustLevel};
use crate::universal::PrimalCapability;
use crate::universal_primal_ecosystem::DiscoveredPrimal;

// ============================================================================
// SECURITY PROVIDER TYPES
// ============================================================================

/// Discovered security provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProvider {
    /// Provider identifier
    pub provider_id: String,

    /// Provider metadata
    pub metadata: SecurityProviderMetadata,

    /// Available capabilities
    pub capabilities: Vec<PrimalCapability>,

    /// Current health and performance
    pub health: SecurityProviderHealth,

    /// AI-generated routing score
    pub routing_score: f64,

    /// Trust level
    pub trust_level: TrustLevel,
}

/// Security provider metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProviderMetadata {
    /// Provider name
    pub name: String,

    /// Provider version
    pub version: String,

    /// Supported standards
    pub standards: Vec<String>,

    /// Certifications
    pub certifications: Vec<String>,

    /// Geographic regions
    pub regions: Vec<String>,

    /// Compliance attestations
    pub compliance: Vec<ComplianceFramework>,
}

/// Security provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProviderHealth {
    /// Overall health score (0.0 - 1.0)
    pub health_score: f64,

    /// Response time (ms)
    pub response_time_ms: f64,

    /// Availability percentage
    pub availability_percent: f64,

    /// Security incident count (last 24h)
    pub incident_count: u32,

    /// Last security assessment
    pub last_assessment: DateTime<Utc>,

    /// Last health check
    pub last_check: DateTime<Utc>,
}

impl SecurityProvider {
    /// Create security provider from discovered primal
    #[must_use]
    pub fn from_discovered_primal(primal: &DiscoveredPrimal) -> Self {
        Self {
            provider_id: primal.instance_id.clone(),
            metadata: SecurityProviderMetadata {
                name: primal.id.clone(),
                version: "unknown".to_string(),
                standards: vec!["oauth2".to_string(), "openid".to_string()],
                certifications: vec!["soc2".to_string()],
                regions: vec!["local".to_string()],
                compliance: vec![ComplianceFramework::Soc2],
            },
            capabilities: vec![crate::universal::PrimalCapability::Custom {
                name: "security".to_string(),
                description: "Security provider capability".to_string(),
                metadata: std::collections::HashMap::new(),
                attributes: std::collections::HashMap::new(),
            }],
            health: SecurityProviderHealth {
                health_score: 1.0,
                response_time_ms: 50.0,
                availability_percent: 99.9,
                incident_count: 0,
                last_assessment: Utc::now(),
                last_check: Utc::now(),
            },
            routing_score: 0.9, // Base score, would be calculated by AI
            trust_level: TrustLevel::High,
        }
    }
}
