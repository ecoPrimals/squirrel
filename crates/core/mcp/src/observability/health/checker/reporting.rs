// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Health Check Reporting
//!
//! This module contains functionality for generating health reports
//! and creating status events for subscribers.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::observability::{ObservabilityResult, ObservabilityError};
use crate::observability::health::types::HealthStatus;
use crate::observability::health::component::ComponentHealth;
use crate::observability::health::event::{HealthStatusEvent, HealthStatusReport};

use super::execution::calculate_system_status;
use super::scheduler::execute_with_runtime;

/// Get a JSON report of the current health status
pub fn get_json_report(
    component_health: &RwLock<HashMap<String, ComponentHealth>>
) -> ObservabilityResult<String> {
    execute_with_runtime(async {
        match component_health.read() {
            Ok(component_health_guard) => {
                let overall_status = calculate_system_status(&component_health_guard);
                
                let report = HealthStatusReport {
                    timestamp: SystemTime::now(),
                    status: overall_status,
                    components: component_health_guard.clone(),
                };
                
                serde_json::to_string_pretty(&report).map_err(|e|
                    ObservabilityError::HealthError(format!("Failed to serialize health report: {}", e))
                )
            },
            Err(e) => Err(ObservabilityError::HealthError(format!("Failed to read component health: {}", e))),
        }
    })
}

/// Create a status event synchronously
pub fn create_status_event_sync(
    component_health: &RwLock<HashMap<String, ComponentHealth>>
) -> ObservabilityResult<HealthStatusEvent> {
    // Get component health
    let component_health_data = match component_health.read() {
        Ok(ch) => ch.clone(),
        Err(e) => return Err(ObservabilityError::HealthError(format!("Failed to read component health: {}", e))),
    };
    
    // Calculate system status
    let system_status = calculate_system_status(&component_health_data);
    
    // Get current time as seconds since epoch
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    Ok(HealthStatusEvent {
        system_status,
        component_statuses: component_health_data,
        timestamp,
    })
}

/// Create a status event (async version)
pub async fn create_status_event(
    component_health: &RwLock<HashMap<String, ComponentHealth>>
) -> ObservabilityResult<HealthStatusEvent> {
    create_status_event_sync(component_health)
}

/// Generate a comprehensive health report with all component details
pub fn generate_comprehensive_report(
    component_health: &RwLock<HashMap<String, ComponentHealth>>
) -> ObservabilityResult<HealthStatusReport> {
    match component_health.read() {
        Ok(component_health_guard) => {
            let overall_status = calculate_system_status(&component_health_guard);
            
            Ok(HealthStatusReport {
                timestamp: SystemTime::now(),
                status: overall_status,
                components: component_health_guard.clone(),
            })
        },
        Err(e) => Err(ObservabilityError::HealthError(
            format!("Failed to read component health: {}", e)
        )),
    }
}

/// Get system health status only (without component details)
pub fn get_system_health_status(
    component_health: &RwLock<HashMap<String, ComponentHealth>>
) -> ObservabilityResult<HealthStatus> {
    match component_health.read() {
        Ok(component_health_guard) => {
            let status = calculate_system_status(&component_health_guard);
            Ok(status)
        },
        Err(_) => Err(ObservabilityError::HealthError(
            "Failed to read component health".to_string()
        )),
    }
}

/// Get system health data synchronously
pub fn get_system_health_sync(
    component_health: &RwLock<HashMap<String, ComponentHealth>>
) -> ObservabilityResult<HashMap<String, ComponentHealth>> {
    match component_health.read() {
        Ok(health) => Ok(health.clone()),
        Err(e) => Err(ObservabilityError::HealthError(
            format!("Failed to read component health: {}", e)
        )),
    }
} 