//! Health check functionality for dashboard
//!
//! This module provides health checking and reporting capabilities for the dashboard.

use serde::{Serialize, Deserialize};

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is functioning normally
    Ok,
    /// System is functioning with warnings
    Warning,
    /// System is in a critical state
    Critical,
    /// System status is unknown
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Name of the component being checked
    pub name: String,
    /// Current health status
    pub status: HealthStatus,
    /// Status details (e.g., error message or current value)
    pub details: String,
}

impl HealthCheck {
    /// Create a new health check result
    pub fn new(name: impl Into<String>, status: HealthStatus, details: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status,
            details: details.into(),
        }
    }
    
    /// Create a new health check with OK status
    pub fn ok(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self::new(name, HealthStatus::Ok, details)
    }
    
    /// Create a new health check with Warning status
    pub fn warning(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self::new(name, HealthStatus::Warning, details)
    }
    
    /// Create a new health check with Critical status
    pub fn critical(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self::new(name, HealthStatus::Critical, details)
    }
    
    /// Create a new health check with Unknown status
    pub fn unknown(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self::new(name, HealthStatus::Unknown, details)
    }
} 