// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal health status types.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// Health status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub status: HealthState,

    /// Detailed health information
    pub details: HashMap<String, HealthDetail>,

    /// Timestamp of the health check
    pub timestamp: DateTime<Utc>,

    /// Time taken for the health check
    pub duration: Duration,
}

/// Health state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HealthState {
    /// Primal is healthy
    Healthy,
    /// Primal is degraded but functional
    Degraded,
    /// Primal is unhealthy
    Unhealthy,
    /// Health status is unknown
    #[default]
    Unknown,
}

/// Health detail structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDetail {
    /// Status of this specific component
    pub status: HealthState,

    /// Human-readable message
    pub message: String,

    /// Additional data
    pub data: HashMap<String, serde_json::Value>,
}

/// Primal health status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalHealth {
    /// Primal is healthy and operational
    Healthy,
    /// Primal is degraded but operational
    Degraded {
        /// List of issues causing degradation
        issues: Vec<String>,
    },
    /// Primal is unhealthy and not operational
    Unhealthy {
        /// Reason why the primal is unhealthy
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_health_state_default() {
        let state = HealthState::default();
        assert_eq!(state, HealthState::Unknown);
    }

    #[test]
    fn test_health_state_serde() {
        for state in [
            HealthState::Healthy,
            HealthState::Degraded,
            HealthState::Unhealthy,
            HealthState::Unknown,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: HealthState = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, state);
        }
    }

    #[test]
    fn test_health_detail_serde() {
        let detail = HealthDetail {
            status: HealthState::Healthy,
            message: "All systems go".to_string(),
            data: {
                let mut m = HashMap::new();
                m.insert("uptime".to_string(), serde_json::json!(3600));
                m
            },
        };
        let json = serde_json::to_string(&detail).unwrap();
        let deserialized: HealthDetail = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, HealthState::Healthy);
        assert_eq!(deserialized.message, "All systems go");
        assert_eq!(
            deserialized.data.get("uptime").unwrap(),
            &serde_json::json!(3600)
        );
    }

    #[test]
    fn test_health_status_serde() {
        let status = HealthStatus {
            status: HealthState::Degraded,
            details: {
                let mut m = HashMap::new();
                m.insert(
                    "db".to_string(),
                    HealthDetail {
                        status: HealthState::Unhealthy,
                        message: "Connection lost".to_string(),
                        data: HashMap::new(),
                    },
                );
                m
            },
            timestamp: chrono::Utc::now(),
            duration: chrono::Duration::milliseconds(150),
        };
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, HealthState::Degraded);
        assert!(deserialized.details.contains_key("db"));
    }

    #[test]
    fn test_primal_health_healthy_serde() {
        let health = PrimalHealth::Healthy;
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: PrimalHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, PrimalHealth::Healthy);
    }

    #[test]
    fn test_primal_health_degraded_serde() {
        let health = PrimalHealth::Degraded {
            issues: vec!["slow disk".to_string(), "high memory".to_string()],
        };
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: PrimalHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, health);
    }

    #[test]
    fn test_primal_health_unhealthy_serde() {
        let health = PrimalHealth::Unhealthy {
            reason: "disk full".to_string(),
        };
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: PrimalHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, health);
    }

    #[test]
    fn test_primal_health_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PrimalHealth::Healthy);
        set.insert(PrimalHealth::Degraded {
            issues: vec!["a".to_string()],
        });
        set.insert(PrimalHealth::Unhealthy {
            reason: "b".to_string(),
        });
        assert_eq!(set.len(), 3);
    }
}
