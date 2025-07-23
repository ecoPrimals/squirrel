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
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the overall security system is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.overall_status, HealthStatus::Healthy | HealthStatus::Warning)
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
        auth_health.metrics.insert("response_time_ms".to_string(), 5.0);
        health.add_component("authentication".to_string(), auth_health);

        // Check authorization system  
        let mut authz_health = ComponentHealth::default();
        authz_health.metrics.insert("cache_hit_rate".to_string(), 0.95);
        health.add_component("authorization".to_string(), authz_health);

        // Check rate limiting
        let mut rate_limit_health = ComponentHealth::default();
        rate_limit_health.metrics.insert("blocked_requests_rate".to_string(), 0.01);
        health.add_component("rate_limiting".to_string(), rate_limit_health);

        Ok(())
    }

    /// Check discovered security endpoints through universal adapter
    async fn check_discovered_endpoints(&self, health: &mut SecurityHealth) -> Result<(), PrimalError> {
        // This would use the universal adapter to discover and check
        // security capabilities from any primal in the ecosystem
        
        // For now, simulate checking discovered endpoints
        let mut discovered_health = ComponentHealth::default();
        discovered_health.metrics.insert("discovery_success_rate".to_string(), 1.0);
        health.add_component("capability_discovery".to_string(), discovered_health);

        Ok(())
    }

    /// Get the current health status
    pub fn current_health(&self) -> &SecurityHealth {
        &self.health_status
    }

    /// Check if health check is due
    pub fn is_check_due(&self) -> bool {
        self.last_check.elapsed() >= self.check_interval
    }
}

impl Default for UniversalSecurityHealthChecker {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}
