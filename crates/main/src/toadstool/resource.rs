//! Resource allocation and tracking
//!
//! This module defines types for managing resource allocations
//! and tracking resource usage.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Resource allocation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub job_id: String,
    pub node_id: String,
    pub resources: AllocatedResources,
    pub allocated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: AllocationStatus,
}

/// Allocated resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedResources {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu_units: u32,
    pub storage_gb: u32,
}

/// Allocation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStatus {
    Active,
    Released,
    Expired,
    Failed,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub gpu_usage: f64,
    pub network_usage: f64,
    pub peak_memory: u32,
    pub peak_cpu: f64,
}
