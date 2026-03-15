// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Health Checking Module
//!
//! This module provides health checking capabilities for monitoring
//! the status of various system components.

pub mod types;
pub mod component;
pub mod event;
pub mod result;
pub mod subscription;
pub mod checker;

// Re-export commonly used types
pub use types::{HealthStatus, HealthCheckType, SystemHealthReport, ComponentHealthInfo, SystemHealthMetrics, SystemInfo};
pub use component::ComponentHealth;
pub use event::HealthStatusEvent;
pub use result::HealthCheckResult;
pub use subscription::{HealthStatusSubscriber, HealthStatusSubscriberNonBlocking, HealthStatusUpdate};
pub use checker::HealthChecker;
pub use checker::types::{HealthCheck, HealthCheckFn, HealthCheckerConfig};

use crate::observability::ObservabilityResult;

/// Health report container
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthReport {
    /// Overall system status
    pub overall_status: HealthStatus,
    /// Component health statuses  
    pub component_statuses: std::collections::HashMap<String, ComponentHealth>,
    /// Timestamp of the report
    pub timestamp: std::time::SystemTime,
}

/// Create standard health checks for a component
pub fn create_standard_health_checks(
    health_checker: &HealthChecker,
    component_id: &str,
) -> ObservabilityResult<()> {
    // Register a basic liveness check
    health_checker.register_health_check(
        component_id,
        &format!("{}_liveness", component_id),
        &format!("Liveness check for {}", component_id),
        Box::new(|| {
            // Basic liveness check - always return healthy for now
            HealthCheckResult::healthy()
        }),
    )?;

    // Register a readiness check
    health_checker.register_health_check(
        component_id,
        &format!("{}_readiness", component_id),
        &format!("Readiness check for {}", component_id),
        Box::new(|| {
            // Basic readiness check - always return healthy for now
            HealthCheckResult::healthy()
        }),
    )?;

    Ok(())
} 