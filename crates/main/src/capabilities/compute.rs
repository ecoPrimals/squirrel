// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Compute capability (job execution, task processing)

use crate::error::PrimalError;
// Native async traits (Rust 1.75+) - no async_trait needed!
use serde::{Deserialize, Serialize};

/// Request to execute a compute job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequest {
    /// Job ID
    pub job_id: String,

    /// Job type or task name
    pub job_type: String,

    /// Input data
    pub input: serde_json::Value,

    /// Resource requirements
    pub resources: Option<ResourceRequirements>,
}

/// Resource requirements for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores
    pub cpu: f32,

    /// Memory in MB
    pub memory_mb: u64,

    /// GPU required
    pub gpu: bool,
}

/// Response from compute execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResponse {
    /// Job ID
    pub job_id: String,

    /// Job status
    pub status: JobStatus,

    /// Output data (if completed)
    pub output: Option<serde_json::Value>,

    /// Error message (if failed)
    pub error: Option<String>,
}

/// Job execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Capability for compute/job execution
///
/// Typically provided by Toadstool or other compute providers.

pub trait ComputeCapability: Send + Sync {
    /// Submit a compute job
    async fn submit_job(&self, request: ComputeRequest) -> Result<ComputeResponse, PrimalError>;

    /// Get job status
    async fn get_job_status(&self, job_id: String) -> Result<ComputeResponse, PrimalError>;

    /// Cancel a job
    async fn cancel_job(&self, job_id: String) -> Result<(), PrimalError>;
}
