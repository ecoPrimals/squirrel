//! Health status events and reports

use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

use super::types::HealthStatus;
use super::component::ComponentHealth;

/// Event for health status changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatusEvent {
    /// Overall system health status
    pub system_status: HealthStatus,
    /// Component health statuses
    pub component_statuses: HashMap<String, ComponentHealth>,
    /// Timestamp of the event
    pub timestamp: u64,
}

/// Health status change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatusChange {
    /// Component ID
    pub component_id: String,
    /// Previous health status
    pub previous_status: HealthStatus,
    /// New health status
    pub new_status: HealthStatus,
    /// Details about the status change
    pub details: Option<String>,
    /// When the status changed
    pub timestamp: SystemTime,
}

/// Complete health status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatusReport {
    /// When the report was generated
    pub timestamp: SystemTime,
    /// Overall system health status
    pub status: HealthStatus,
    /// Component health statuses
    pub components: HashMap<String, ComponentHealth>,
}

/// Simple component status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    /// Component ID
    pub component_id: String,
    /// Health status
    pub status: HealthStatus,
    /// Additional details about the health status
    pub details: Option<String>,
    /// Last time the health was checked
    pub last_checked: u64,
} 