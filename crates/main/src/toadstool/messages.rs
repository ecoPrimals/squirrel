//! Request and response types for ToadStool communication
//!
//! This module defines message types for interacting with the
//! ToadStool compute primal.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::job::{ComputeJobType, JobStatus, ResourceRequirements};
use super::resource::{AllocatedResources, ResourceUsage};

/// Compute job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJobRequest {
    pub job_type: ComputeJobType,
    pub resource_requirements: ResourceRequirements,
    pub payload: serde_json::Value,
    pub callback_url: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Compute job response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJobResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub queue_position: Option<u32>,
    pub allocated_resources: Option<AllocatedResources>,
}

/// Job result response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResultResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time: Option<Duration>,
    pub resource_usage: Option<ResourceUsage>,
}
