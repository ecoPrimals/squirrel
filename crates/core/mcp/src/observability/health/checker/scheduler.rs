// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Health Check Scheduler
//!
//! This module contains functionality for scheduling and automatically
//! executing health checks at defined intervals.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;

use crate::observability::{ObservabilityResult, ObservabilityError};
use crate::observability::health::component::ComponentHealth;

use super::types::HealthCheck;
use super::execution::{run_health_check, update_component_health_with_result};

/// Start the health check scheduler
pub async fn start_scheduler(
    health_checks: Arc<RwLock<HashMap<String, Arc<HealthCheck>>>>,
    component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
) {
    loop {
        // Execute due health checks
        if let Err(e) = execute_due_health_checks(&health_checks, &component_health).await {
            eprintln!("Error executing health checks: {}", e);
        }
        
        // Sleep for a short interval before checking again
        sleep(Duration::from_secs(10)).await;
    }
}

/// Execute health checks that are due to run
pub async fn execute_due_health_checks(
    health_checks: &RwLock<HashMap<String, Arc<HealthCheck>>>,
    component_health: &RwLock<HashMap<String, ComponentHealth>>,
) -> ObservabilityResult<()> {
    // First, collect all the health checks
    let all_checks = {
        let health_checks_guard = match health_checks.read() {
            Ok(guard) => guard,
            Err(_) => return Err(ObservabilityError::HealthError("Failed to read health checks".to_string())),
        };
        
        health_checks_guard.values().cloned().collect::<Vec<_>>()
    };
    
    // Now check which ones are due (outside the lock)
    let mut checks_to_run = Vec::new();
    for check in all_checks {
        if check.is_due().await {
            checks_to_run.push(check);
        }
    }
    
    // Execute all due checks
    for check in checks_to_run {
        let result = run_health_check(&check).await;
        
        // Update component status based on check result
        let _ = update_component_health_with_result(component_health, &check.component_id, &result);
    }
    
    Ok(())
}

/// Execute checks for a specific component
pub async fn execute_component_checks(
    health_checks: &RwLock<HashMap<String, Arc<HealthCheck>>>,
    component_health: &RwLock<HashMap<String, ComponentHealth>>,
    component_id: &str,
) -> ObservabilityResult<Vec<(String, crate::observability::health::result::HealthCheckResult)>> {
    let checks = {
        let health_checks_guard = match health_checks.read() {
            Ok(guard) => guard,
            Err(_) => return Err(ObservabilityError::HealthError("Failed to read health checks".to_string())),
        };
        
        health_checks_guard
            .values()
            .filter(|check| check.component_id == component_id)
            .cloned()
            .collect::<Vec<_>>()
    };
    
    let mut results = Vec::new();
    
    for check in checks {
        let result = run_health_check(&check).await;
        results.push((check.id.clone(), result.clone()));
        
        // Update component status based on check result
        let _ = update_component_health_with_result(component_health, component_id, &result);
    }
    
    Ok(results)
}

/// Helper function to execute async code safely with runtime handling
pub fn execute_with_runtime<F, T>(f: F) -> ObservabilityResult<T>
where
    F: std::future::Future<Output = ObservabilityResult<T>>,
{
    // Check if we're already in a tokio runtime
    if tokio::runtime::Handle::try_current().is_ok() {
        // We're in a runtime, use block_in_place to run the async code
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(f)
        })
    } else {
        // Not in a runtime, create a minimal one for this operation
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| 
                ObservabilityError::HealthError(format!("Failed to create runtime: {}", e)))?;
        
        rt.block_on(f)
    }
} 