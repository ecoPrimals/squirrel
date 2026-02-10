// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Health Check Types and Configuration
//!
//! This module contains the core types used by the health checking system,
//! including health check definitions and configuration structures.

use std::sync::RwLock;
use std::time::SystemTime;
use std::fmt;

use crate::observability::health::result::HealthCheckResult;

// Re-export for convenience
pub use crate::observability::health::types::HealthCheckType;

/// Function type for health checks
pub type HealthCheckFn = Box<dyn Fn() -> HealthCheckResult + Send + Sync>;

/// Health check definition
///
/// Represents a single health check that can be executed to determine
/// the health status of a component or service.
pub struct HealthCheck {
    /// Unique identifier for the health check
    pub id: String,
    /// Component this check belongs to
    pub component_id: String,
    /// Human-readable name for the check
    pub name: String,
    /// Type of health check
    pub check_type: HealthCheckType,
    /// Function to execute for the health check
    pub check_fn: HealthCheckFn,
    /// Optional interval in seconds for scheduled execution
    pub interval: Option<u64>,
    /// Last run time
    pub last_run: RwLock<Option<SystemTime>>,
    /// Last result
    pub last_result: RwLock<Option<HealthCheckResult>>,
    /// Description of the health check (might be different from name)
    pub description: String,
}

impl fmt::Debug for HealthCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HealthCheck")
            .field("id", &self.id)
            .field("component_id", &self.component_id)
            .field("name", &self.name)
            .field("check_type", &self.check_type)
            .field("interval", &self.interval)
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

impl HealthCheck {
    /// Create a new health check
    pub fn new(
        id: impl Into<String>,
        component_id: impl Into<String>,
        name: impl Into<String>,
        check_type: HealthCheckType,
        check_fn: HealthCheckFn,
        interval: Option<u64>,
    ) -> Self {
        let id_str = id.into();
        let name_str = name.into();
        Self {
            id: id_str,
            component_id: component_id.into(),
            name: name_str.clone(),
            check_type,
            check_fn,
            interval,
            last_run: RwLock::new(None),
            last_result: RwLock::new(None),
            description: name_str,
        }
    }
    
    /// Execute this health check and return the result
    pub fn execute(&self) -> Result<HealthCheckResult, crate::observability::ObservabilityError> {
        let now = SystemTime::now();
        
        // Execute the check function
        let result = (self.check_fn)();
        
        // Update last run time and result
        if let Ok(mut last_run) = self.last_run.write() {
            *last_run = Some(now);
        }
        
        if let Ok(mut last_result) = self.last_result.write() {
            *last_result = Some(result.clone());
        }
        
        Ok(result)
    }
    
    /// Get the ID of this check
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the component ID this check belongs to
    pub fn component(&self) -> &str {
        &self.component_id
    }
    
    /// Get the name of this check
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the description of this check
    pub fn description(&self) -> &str {
        &self.description
    }
    
    /// Create a new health check with dependencies
    pub fn new_with_dependencies(
        id: impl Into<String>,
        component_id: impl Into<String>,
        name: impl Into<String>,
        check_fn: HealthCheckFn,
        _dependencies: Vec<String>,
    ) -> Self {
        Self::new(
            id,
            component_id,
            name,
            HealthCheckType::Comprehensive,
            check_fn,
            None,
        )
    }
    
    /// Create a new basic health check
    pub fn new_basic(
        id: impl Into<String>,
        component_id: impl Into<String>,
        name: impl Into<String>,
        check_fn: HealthCheckFn,
    ) -> Self {
        Self::new(
            id,
            component_id,
            name,
            HealthCheckType::Basic,
            check_fn,
            None,
        )
    }
    
    /// Check if this check is due to run based on its interval
    pub async fn is_due(&self) -> bool {
        if let Some(interval) = self.interval {
            // Check if the interval has elapsed since the last run
            if let Ok(last_run) = self.last_run.read() {
                if let Some(last_run_time) = *last_run {
                    if let Ok(elapsed) = last_run_time.elapsed() {
                        return elapsed.as_secs() >= interval;
                    }
                }
            }
            
            // If we can't determine when it was last run, or it hasn't run yet,
            // consider it due
            true
        } else {
            // No interval specified, so it's not automatically due
            false
        }
    }
    
    /// Get the last run time
    pub fn last_run(&self) -> Option<SystemTime> {
        if let Ok(last_run) = self.last_run.read() {
            *last_run
        } else {
            None
        }
    }
    
    /// Get the last result
    pub fn last_result(&self) -> Option<HealthCheckResult> {
        if let Ok(last_result) = self.last_result.read() {
            last_result.clone()
        } else {
            None
        }
    }
}

/// Configuration for the health checker
#[derive(Debug, Clone)]
pub struct HealthCheckerConfig {
    /// Default check interval in seconds
    pub default_check_interval: u64,
    /// Default check timeout in seconds
    pub default_check_timeout: u64,
    /// Maximum number of status change subscribers
    pub max_subscribers: usize,
}

impl Default for HealthCheckerConfig {
    fn default() -> Self {
        Self {
            default_check_interval: 30,
            default_check_timeout: 5,
            max_subscribers: 100,
        }
    }
} 