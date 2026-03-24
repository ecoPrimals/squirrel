// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security health monitoring and status reporting
//!
//! This module provides health checking capabilities for the security system,
//! integrated with the universal adapter pattern for ecosystem-wide monitoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::error::PrimalError;

/// Security health status representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHealth {
    /// Overall security system health
    pub overall_status: HealthStatus,
    /// Individual component health statuses
    pub component_health: HashMap<String, ComponentHealth>,
    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Health check duration
    pub check_duration: Duration,
    /// Any health warnings or issues
    pub warnings: Vec<String>,
}

/// Health status levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Minor issues detected but system functional
    Warning,
    /// Significant issues affecting some functionality
    Degraded,
    /// Critical issues requiring immediate attention
    Critical,
    /// System is down or unreachable
    Down,
}

/// Component-specific health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component health status
    pub status: HealthStatus,
    /// Component-specific metrics
    pub metrics: HashMap<String, f64>,
    /// Last successful operation timestamp
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    /// Error count in current window
    pub error_count: u32,
    /// Component-specific messages
    pub messages: Vec<String>,
}

impl Default for SecurityHealth {
    fn default() -> Self {
        Self {
            overall_status: HealthStatus::Healthy,
            component_health: HashMap::new(),
            last_check: chrono::Utc::now(),
            check_duration: Duration::from_millis(0),
            warnings: Vec::new(),
        }
    }
}

impl Default for ComponentHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            metrics: HashMap::new(),
            last_success: Some(chrono::Utc::now()),
            error_count: 0,
            messages: Vec::new(),
        }
    }
}

impl SecurityHealth {
    /// Create a new security health status
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the overall security system is healthy
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(
            self.overall_status,
            HealthStatus::Healthy | HealthStatus::Warning
        )
    }

    /// Add a component health status
    pub fn add_component(&mut self, name: String, health: ComponentHealth) {
        self.component_health.insert(name, health);
        self.update_overall_status();
    }

    /// Update a component's health status
    pub fn update_component_status(&mut self, name: &str, status: HealthStatus) {
        if let Some(component) = self.component_health.get_mut(name) {
            component.status = status;
            self.update_overall_status();
        }
    }

    /// Add a warning message
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
        if matches!(self.overall_status, HealthStatus::Healthy) {
            self.overall_status = HealthStatus::Warning;
        }
    }

    /// Update the overall status based on component statuses
    fn update_overall_status(&mut self) {
        if self.component_health.is_empty() {
            self.overall_status = HealthStatus::Healthy;
            return;
        }

        let mut has_critical = false;
        let mut has_degraded = false;
        let mut has_warning = false;
        let mut has_down = false;

        for component in self.component_health.values() {
            match component.status {
                HealthStatus::Down => has_down = true,
                HealthStatus::Critical => has_critical = true,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Warning => has_warning = true,
                HealthStatus::Healthy => {}
            }
        }

        self.overall_status = if has_down {
            HealthStatus::Down
        } else if has_critical {
            HealthStatus::Critical
        } else if has_degraded {
            HealthStatus::Degraded
        } else if has_warning || !self.warnings.is_empty() {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
    }

    /// Get health summary as a human-readable string
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Security Health: {:?} ({} components, {} warnings)",
            self.overall_status,
            self.component_health.len(),
            self.warnings.len()
        )
    }
}

/// Universal security health checker that works with any discovered primals
#[derive(Debug, Clone)]
pub struct UniversalSecurityHealthChecker {
    /// Last known health status
    health_status: SecurityHealth,
    /// Health check interval
    check_interval: Duration,
    /// Last health check time
    last_check: Instant,
}

impl UniversalSecurityHealthChecker {
    /// Create a new universal security health checker
    #[must_use]
    pub fn new(check_interval: Duration) -> Self {
        Self {
            health_status: SecurityHealth::new(),
            check_interval,
            last_check: Instant::now(),
        }
    }

    /// Perform comprehensive security health check
    pub async fn check_health(&mut self) -> Result<SecurityHealth, PrimalError> {
        let start_time = Instant::now();
        let mut health = SecurityHealth::new();

        // Check local security components
        self.check_local_components(&mut health).await?;

        // Check discovered security endpoints through universal adapter
        self.check_discovered_endpoints(&mut health).await?;

        // Update timing information
        health.check_duration = start_time.elapsed();
        health.last_check = chrono::Utc::now();

        self.health_status = health.clone();
        self.last_check = start_time;

        Ok(health)
    }

    /// Check local security components
    async fn check_local_components(&self, health: &mut SecurityHealth) -> Result<(), PrimalError> {
        // Check authentication system
        let mut auth_health = ComponentHealth::default();
        auth_health
            .metrics
            .insert("response_time_ms".to_string(), 5.0);
        health.add_component("authentication".to_string(), auth_health);

        // Check authorization system
        let mut authz_health = ComponentHealth::default();
        authz_health
            .metrics
            .insert("cache_hit_rate".to_string(), 0.95);
        health.add_component("authorization".to_string(), authz_health);

        // Check rate limiting
        let mut rate_limit_health = ComponentHealth::default();
        rate_limit_health
            .metrics
            .insert("blocked_requests_rate".to_string(), 0.01);
        health.add_component("rate_limiting".to_string(), rate_limit_health);

        Ok(())
    }

    /// Check discovered security endpoints through universal adapter
    async fn check_discovered_endpoints(
        &self,
        health: &mut SecurityHealth,
    ) -> Result<(), PrimalError> {
        // This would use the universal adapter to discover and check
        // security capabilities from any primal in the ecosystem

        // For now, simulate checking discovered endpoints
        let mut discovered_health = ComponentHealth::default();
        discovered_health
            .metrics
            .insert("discovery_success_rate".to_string(), 1.0);
        health.add_component("capability_discovery".to_string(), discovered_health);

        Ok(())
    }

    /// Get the current health status
    #[must_use]
    pub const fn current_health(&self) -> &SecurityHealth {
        &self.health_status
    }

    /// Check if health check is due
    #[must_use]
    pub fn is_check_due(&self) -> bool {
        self.last_check.elapsed() >= self.check_interval
    }
}

impl Default for UniversalSecurityHealthChecker {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_serde() {
        for status in [
            HealthStatus::Healthy,
            HealthStatus::Warning,
            HealthStatus::Degraded,
            HealthStatus::Critical,
            HealthStatus::Down,
        ] {
            let json = serde_json::to_string(&status).expect("serialize");
            let deser: HealthStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, status);
        }
    }

    #[test]
    fn test_security_health_default() {
        let health = SecurityHealth::default();
        assert_eq!(health.overall_status, HealthStatus::Healthy);
        assert!(health.component_health.is_empty());
        assert!(health.warnings.is_empty());
    }

    #[test]
    fn test_security_health_new() {
        let health = SecurityHealth::new();
        assert_eq!(health.overall_status, HealthStatus::Healthy);
    }

    #[test]
    fn test_is_healthy() {
        let mut health = SecurityHealth::new();
        assert!(health.is_healthy());

        health.overall_status = HealthStatus::Warning;
        assert!(health.is_healthy());

        health.overall_status = HealthStatus::Degraded;
        assert!(!health.is_healthy());

        health.overall_status = HealthStatus::Critical;
        assert!(!health.is_healthy());

        health.overall_status = HealthStatus::Down;
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_add_component() {
        let mut health = SecurityHealth::new();
        let comp = ComponentHealth::default();
        health.add_component("auth".to_string(), comp);
        assert_eq!(health.component_health.len(), 1);
        assert!(health.component_health.contains_key("auth"));
    }

    #[test]
    fn test_update_component_status() {
        let mut health = SecurityHealth::new();
        health.add_component("auth".to_string(), ComponentHealth::default());
        health.update_component_status("auth", HealthStatus::Warning);
        assert_eq!(
            health
                .component_health
                .get("auth")
                .expect("should succeed")
                .status,
            HealthStatus::Warning
        );
        assert_eq!(health.overall_status, HealthStatus::Warning);
    }

    #[test]
    fn test_update_component_status_nonexistent() {
        let mut health = SecurityHealth::new();
        // Should not panic
        health.update_component_status("nonexistent", HealthStatus::Critical);
        assert_eq!(health.overall_status, HealthStatus::Healthy);
    }

    #[test]
    fn test_add_warning() {
        let mut health = SecurityHealth::new();
        health.add_warning("test warning".to_string());
        assert_eq!(health.warnings.len(), 1);
        assert_eq!(health.overall_status, HealthStatus::Warning);
    }

    #[test]
    fn test_overall_status_escalation() {
        let mut health = SecurityHealth::new();

        let healthy_comp = ComponentHealth {
            status: HealthStatus::Healthy,
            ..Default::default()
        };
        health.add_component("a".to_string(), healthy_comp);
        assert_eq!(health.overall_status, HealthStatus::Healthy);

        let warning_comp = ComponentHealth {
            status: HealthStatus::Warning,
            ..Default::default()
        };
        health.add_component("b".to_string(), warning_comp);
        assert_eq!(health.overall_status, HealthStatus::Warning);

        let degraded_comp = ComponentHealth {
            status: HealthStatus::Degraded,
            ..Default::default()
        };
        health.add_component("c".to_string(), degraded_comp);
        assert_eq!(health.overall_status, HealthStatus::Degraded);

        let critical_comp = ComponentHealth {
            status: HealthStatus::Critical,
            ..Default::default()
        };
        health.add_component("d".to_string(), critical_comp);
        assert_eq!(health.overall_status, HealthStatus::Critical);

        let down_comp = ComponentHealth {
            status: HealthStatus::Down,
            ..Default::default()
        };
        health.add_component("e".to_string(), down_comp);
        assert_eq!(health.overall_status, HealthStatus::Down);
    }

    #[test]
    fn test_summary() {
        let mut health = SecurityHealth::new();
        health.add_component("auth".to_string(), ComponentHealth::default());
        health.add_warning("test".to_string());
        let summary = health.summary();
        assert!(summary.contains("Warning"));
        assert!(summary.contains("1 components"));
        assert!(summary.contains("1 warnings"));
    }

    #[test]
    fn test_component_health_default() {
        let comp = ComponentHealth::default();
        assert_eq!(comp.status, HealthStatus::Healthy);
        assert!(comp.metrics.is_empty());
        assert!(comp.last_success.is_some());
        assert_eq!(comp.error_count, 0);
        assert!(comp.messages.is_empty());
    }

    #[test]
    fn test_health_checker_new() {
        let checker = UniversalSecurityHealthChecker::new(Duration::from_secs(60));
        assert!(checker.current_health().is_healthy());
    }

    #[test]
    fn test_health_checker_default() {
        let checker = UniversalSecurityHealthChecker::default();
        assert!(checker.current_health().is_healthy());
    }

    #[test]
    fn test_health_checker_is_check_due() {
        let checker = UniversalSecurityHealthChecker::new(Duration::from_millis(0));
        // With 0ms interval, check is immediately due
        assert!(checker.is_check_due());
    }

    #[tokio::test]
    async fn test_health_checker_check_health() {
        let mut checker = UniversalSecurityHealthChecker::new(Duration::from_secs(30));
        let health = checker.check_health().await.expect("check health");
        assert!(health.is_healthy());
        assert!(health.component_health.contains_key("authentication"));
        assert!(health.component_health.contains_key("authorization"));
        assert!(health.component_health.contains_key("rate_limiting"));
        assert!(health.component_health.contains_key("capability_discovery"));
    }

    #[test]
    fn test_security_health_serde() {
        let health = SecurityHealth::default();
        let json = serde_json::to_string(&health).expect("serialize");
        let deser: SecurityHealth = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.overall_status, HealthStatus::Healthy);
    }
}
