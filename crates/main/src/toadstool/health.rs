//! Health monitoring for ToadStool integration
//!
//! This module defines types for tracking the health status
//! of the ToadStool compute cluster.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Health status for ToadStool integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub compute_nodes_healthy: u32,
    pub compute_nodes_total: u32,
    pub active_jobs: u32,
    pub queued_jobs: u32,
    pub resource_utilization: f64,
}

impl Default for HealthStatus {
    fn default() -> Self {
        use chrono::Utc;
        Self {
            status: "initializing".to_string(),
            timestamp: Utc::now(),
            compute_nodes_healthy: 0,
            compute_nodes_total: 0,
            active_jobs: 0,
            queued_jobs: 0,
            resource_utilization: 0.0,
        }
    }
}
