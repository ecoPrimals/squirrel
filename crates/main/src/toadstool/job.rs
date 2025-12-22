//! Compute job types and management
//!
//! This module defines types for managing compute jobs, including
//! job specifications, status tracking, and queuing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Compute job specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJob {
    pub job_id: String,
    pub job_type: ComputeJobType,
    pub requester: String,
    pub resource_requirements: ResourceRequirements,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Types of compute jobs supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeJobType {
    /// AI model training
    ModelTraining,
    /// AI model inference
    ModelInference,
    /// Large language model processing
    LLMProcessing,
    /// Computer vision processing
    VisionProcessing,
    /// Neural network optimization
    NetworkOptimization,
    /// Data preprocessing
    DataPreprocessing,
    /// Distributed computing task
    DistributedCompute,
    /// Custom compute task
    Custom(String),
}

/// Job status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is queued waiting for resources
    Queued,
    /// Job is being prepared
    Preparing,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed with error
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Job timed out
    TimedOut,
}

/// Resource requirements for compute jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu_units: u32,
    pub storage_gb: u32,
    pub network_bandwidth: u32,
    pub estimated_duration: Duration,
    pub priority: JobPriority,
}

/// Job priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Queued job with scheduling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedJob {
    pub job: ComputeJob,
    pub queued_at: DateTime<Utc>,
    pub estimated_start: DateTime<Utc>,
    pub queue_position: u32,
}
