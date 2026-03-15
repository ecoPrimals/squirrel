// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Health Check Execution
//!
//! This module contains functionality for executing health checks
//! and updating component status based on results.

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::observability::{ObservabilityResult, ObservabilityError};
use crate::observability::health::types::HealthStatus;
use crate::observability::health::component::ComponentHealth;
use crate::observability::health::result::HealthCheckResult;

use super::types::HealthCheck;

/// Execute a health check and update last run time and result
pub async fn run_health_check(check: &HealthCheck) -> HealthCheckResult {
    // Here we would implement any async context needed
    let result = (check.check_fn)();
    
    // Update last run time
    if let Ok(mut last_run) = check.last_run.write() {
        *last_run = Some(SystemTime::now());
    }
    
    // Update last result
    if let Ok(mut last_result) = check.last_result.write() {
        *last_result = Some(result.clone());
    }
    
    result
}

/// Execute a health check synchronously
pub fn execute_health_check_sync(check: &HealthCheck) -> ObservabilityResult<HealthCheckResult> {
    // Execute the check function
    let result = (check.check_fn)();
    
    // Update last run time
    if let Ok(mut last_run) = check.last_run.write() {
        *last_run = Some(SystemTime::now());
    } else {
        return Err(ObservabilityError::HealthError(
            "Failed to update last run time".to_string(),
        ));
    }
    
    // Update last result
    if let Ok(mut last_result) = check.last_result.write() {
        *last_result = Some(result.clone());
    } else {
        return Err(ObservabilityError::HealthError(
            "Failed to update last result".to_string(),
        ));
    }
    
    Ok(result)
}

/// Update component health based on a check result (internal helper)
pub fn update_component_health_with_result(
    component_health: &RwLock<HashMap<String, ComponentHealth>>,
    component_id: &str, 
    result: &HealthCheckResult
) -> ObservabilityResult<bool> {
    let mut component_health_guard = match component_health.write() {
        Ok(health) => health,
        Err(_) => return Err(ObservabilityError::HealthError("Failed to write component health".to_string())),
    };
    
    // First check if component exists, if not create it
    if !component_health_guard.contains_key(component_id) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        component_health_guard.insert(
            component_id.to_string(),
            ComponentHealth {
                component_id: component_id.to_string(),
                name: format!("Component {}", component_id),
                status: result.status,
                details: result.details.clone(),
                last_checked: now,
                last_status_change: now,
                metadata: HashMap::new(),
                tags: HashSet::new(),
            }
        );
        
        return Ok(true); // Status changed (new component)
    }
    
    if let Some(health) = component_health_guard.get_mut(component_id) {
        // Update status based on check result
        // If current status is worse than the check result, keep it
        // If check result is worse than current status, update it
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let status_changed = health.status != result.status;
        
        match (health.status, result.status()) {
            (HealthStatus::Unhealthy, _) => {
                // Already in worst state, no change needed
                health.last_checked = now;
            },
            (_, HealthStatus::Unhealthy) => {
                // Transition to unhealthy
                health.status = HealthStatus::Unhealthy;
                health.last_checked = now;
                health.last_status_change = now;
                health.details = Some(result.message().to_string());
            },
            (HealthStatus::Degraded, HealthStatus::Degraded) | 
            (HealthStatus::Degraded, HealthStatus::Healthy) | 
            (HealthStatus::Degraded, HealthStatus::Unknown) => {
                // Keep degraded status, just update timestamp
                health.last_checked = now;
            },
            (_, HealthStatus::Degraded) => {
                // Transition to degraded
                health.status = HealthStatus::Degraded;
                health.last_checked = now;
                health.last_status_change = now;
                health.details = Some(result.message().to_string());
            },
            (HealthStatus::Healthy, HealthStatus::Healthy) => {
                // Keep healthy status, just update timestamp
                health.last_checked = now;
            },
            (HealthStatus::Unknown, HealthStatus::Healthy) => {
                // Transition from unknown to healthy
                health.status = HealthStatus::Healthy;
                health.last_checked = now;
                health.last_status_change = now;
                health.details = Some(result.message().to_string());
            },
            _ => {
                // Default, just update timestamp
                health.last_checked = now;
            }
        }
        
        Ok(status_changed)
    } else {
        Err(ObservabilityError::HealthError(
            format!("Component {} not found after insert", component_id)
        ))
    }
}

/// Calculate overall system status from component statuses
pub fn calculate_system_status(component_statuses: &HashMap<String, ComponentHealth>) -> HealthStatus {
    if component_statuses.is_empty() {
        return HealthStatus::Unknown;
    }
    
    let mut has_unhealthy = false;
    let mut has_degraded = false;
    let mut has_healthy = false;
    let mut has_unknown = false;
    
    // Count status types
    for component_health in component_statuses.values() {
        match component_health.status {
            HealthStatus::Unhealthy => has_unhealthy = true,
            HealthStatus::Degraded => has_degraded = true,
            HealthStatus::Healthy => has_healthy = true,
            HealthStatus::Unknown => has_unknown = true,
        }
    }
    
    // Determine overall status based on component statuses
    if has_unhealthy {
        // Any unhealthy component makes the system unhealthy
        HealthStatus::Unhealthy
    } else if has_degraded {
        // Any degraded component (but no unhealthy) makes the system degraded
        HealthStatus::Degraded
    } else if has_healthy {
        // All components are healthy or unknown, with at least one healthy
        if has_unknown {
            // Mix of healthy and unknown - consider degraded
            HealthStatus::Degraded
        } else {
            // All healthy
            HealthStatus::Healthy
        }
    } else {
        // All components are unknown
        HealthStatus::Unknown
    }
} 