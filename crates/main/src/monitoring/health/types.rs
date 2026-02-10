// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Health monitoring core types and traits
//!
//! This module provides the core types and traits for health monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::HealthState;

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Current health state
    pub state: HealthState,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
    /// Check duration
    pub check_duration: Duration,
    /// Health message
    pub message: String,
    /// Health details
    pub details: HashMap<String, String>,
    /// Check count
    pub check_count: u64,
    /// Consecutive successes
    pub consecutive_successes: u64,
    /// Consecutive failures
    pub consecutive_failures: u64,
}

/// Health check configuration for monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringHealthCheckConfig {
    /// Check interval
    pub interval: Duration,
    /// Check timeout
    pub timeout: Duration,
    /// Failure threshold
    pub failure_threshold: u32,
    /// Success threshold
    pub success_threshold: u32,
    /// Grace period after startup
    pub grace_period: Duration,
}

/// Health snapshot for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// Component health states
    pub component_health: HashMap<String, ComponentHealth>,
    /// Overall system health
    pub system_health: HealthState,
}

impl Default for MonitoringHealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
            grace_period: Duration::from_secs(60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MonitoringHealthCheckConfig as HealthCheckConfig;
    use super::*;

    #[test]
    fn test_health_check_config_default() {
        let config = HealthCheckConfig::default();
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.success_threshold, 2);
        assert_eq!(config.grace_period, Duration::from_secs(60));
    }

    #[test]
    fn test_health_check_config_clone() {
        let config = HealthCheckConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.interval, config.interval);
        assert_eq!(cloned.timeout, config.timeout);
    }

    #[test]
    fn test_health_check_config_serialization() {
        let config = HealthCheckConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HealthCheckConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.interval, config.interval);
        assert_eq!(deserialized.failure_threshold, config.failure_threshold);
    }

    #[test]
    fn test_component_health_creation() {
        let health = ComponentHealth {
            name: "test-component".to_string(),
            state: HealthState::Healthy,
            last_check: Utc::now(),
            check_duration: Duration::from_millis(100),
            message: "All systems operational".to_string(),
            details: HashMap::new(),
            check_count: 10,
            consecutive_successes: 10,
            consecutive_failures: 0,
        };

        assert_eq!(health.name, "test-component");
        assert_eq!(health.check_count, 10);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_component_health_with_details() {
        let mut details = HashMap::new();
        details.insert("cpu".to_string(), "45%".to_string());
        details.insert("memory".to_string(), "60%".to_string());

        let health = ComponentHealth {
            name: "resource-monitor".to_string(),
            state: HealthState::Warning,
            last_check: Utc::now(),
            check_duration: Duration::from_millis(50),
            message: "High resource usage".to_string(),
            details,
            check_count: 100,
            consecutive_successes: 0,
            consecutive_failures: 2,
        };

        assert_eq!(health.details.len(), 2);
        assert_eq!(health.details.get("cpu").unwrap(), "45%");
        assert!(matches!(health.state, HealthState::Warning));
    }

    #[test]
    fn test_component_health_serialization() {
        let health = ComponentHealth {
            name: "api-server".to_string(),
            state: HealthState::Healthy,
            last_check: Utc::now(),
            check_duration: Duration::from_millis(25),
            message: "OK".to_string(),
            details: HashMap::new(),
            check_count: 1000,
            consecutive_successes: 50,
            consecutive_failures: 0,
        };

        let json = serde_json::to_string(&health).unwrap();
        let deserialized: ComponentHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "api-server");
        assert_eq!(deserialized.check_count, 1000);
    }

    #[test]
    fn test_health_snapshot_creation() {
        let mut component_health = HashMap::new();
        component_health.insert(
            "component1".to_string(),
            ComponentHealth {
                name: "component1".to_string(),
                state: HealthState::Healthy,
                last_check: Utc::now(),
                check_duration: Duration::from_millis(10),
                message: "OK".to_string(),
                details: HashMap::new(),
                check_count: 5,
                consecutive_successes: 5,
                consecutive_failures: 0,
            },
        );

        let snapshot = HealthSnapshot {
            timestamp: Utc::now(),
            component_health,
            system_health: HealthState::Healthy,
        };

        assert_eq!(snapshot.component_health.len(), 1);
        assert!(matches!(snapshot.system_health, HealthState::Healthy));
    }

    #[test]
    fn test_health_snapshot_serialization() {
        let snapshot = HealthSnapshot {
            timestamp: Utc::now(),
            component_health: HashMap::new(),
            system_health: HealthState::Warning,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: HealthSnapshot = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized.system_health, HealthState::Warning));
    }
}
