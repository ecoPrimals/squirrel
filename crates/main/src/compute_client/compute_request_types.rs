// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Compute request and response types.
//!
//! Wire types for compute operations, payloads, results, metrics,
//! and AI-driven workload analysis.  Configuration types live in
//! [`super::types`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use super::types::{ComputeSecurityRequirements, ResourceRequirements};

/// Universal compute request — AI-first design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalComputeRequest {
    /// Unique request identifier
    pub request_id: Uuid,

    /// Operation type
    pub operation: ComputeOperation,

    /// Compute payload
    pub payload: ComputePayload,

    /// Resource requirements
    pub resources: ResourceRequirements,

    /// Security requirements
    pub security: ComputeSecurityRequirements,

    /// AI context for intelligent routing
    pub ai_context: AIComputeContext,

    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Types of compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeOperation {
    /// Execute code
    Execute {
        /// Programming language
        language: String,
        /// Entry point
        entrypoint: String,
    },

    /// Train ML model
    TrainModel {
        /// ML framework
        framework: String,
        /// Model type
        model_type: String,
    },

    /// Run inference
    RunInference {
        /// Model identifier
        model_id: String,
        /// Batch size
        batch_size: u32,
    },

    /// Batch processing
    BatchProcess {
        /// Job type
        job_type: String,
        /// Parallelism level
        parallelism: u32,
    },

    /// Stream processing
    StreamProcess {
        /// Stream source
        stream_source: String,
        /// Processing window
        processing_window: Duration,
    },

    /// Custom workload
    CustomWorkload {
        /// Workload type
        workload_type: String,
        /// Workload configuration
        configuration: HashMap<String, serde_json::Value>,
    },
}

/// Compute payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePayload {
    /// Code or configuration
    pub code: Option<String>,

    /// Input data
    pub input_data: Option<Vec<u8>>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// Configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// AI context for intelligent compute routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIComputeContext {
    /// Expected workload characteristics
    pub workload_characteristics: WorkloadCharacteristics,

    /// Priority level
    pub priority: ComputePriority,

    /// Deadline for completion
    pub deadline: Option<DateTime<Utc>>,

    /// Cost vs performance preference
    pub cost_performance_preference: CostPerformancePreference,
}

/// Workload characteristics for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCharacteristics {
    /// CPU intensity (0.0 - 1.0)
    pub cpu_intensity: f64,

    /// Memory intensity (0.0 - 1.0)
    pub memory_intensity: f64,

    /// I/O intensity (0.0 - 1.0)
    pub io_intensity: f64,

    /// GPU requirement (0.0 - 1.0)
    pub gpu_requirement: f64,

    /// Parallelizability (0.0 - 1.0)
    pub parallelizability: f64,
}

/// Compute priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputePriority {
    /// Low priority — can be delayed
    Low,

    /// Normal priority
    Normal,

    /// High priority — expedited processing
    High,

    /// Critical priority — immediate processing
    Critical,
}

/// Cost vs performance preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostPerformancePreference {
    /// Optimize for minimum cost
    MinimizeCost,

    /// Balance cost and performance
    Balanced,

    /// Optimize for maximum performance
    MaximizePerformance,

    /// Custom weights
    Custom {
        /// Weight for cost (0.0 - 1.0)
        cost_weight: f64,
        /// Weight for performance (0.0 - 1.0)
        performance_weight: f64,
    },
}

/// Universal compute response — AI-first design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalComputeResponse {
    /// Request identifier
    pub request_id: Uuid,

    /// Operation success
    pub success: bool,

    /// Compute results
    pub results: Option<ComputeResults>,

    /// Provider that handled the request
    pub provider_id: String,

    /// Performance metrics
    pub performance: ComputePerformanceMetrics,

    /// AI insights and recommendations
    pub ai_insights: AIComputeInsights,

    /// Error information (if applicable)
    pub error: Option<String>,
}

/// Compute operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResults {
    /// Output data
    pub output_data: Option<Vec<u8>>,

    /// Return code
    pub return_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Result metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Performance metrics for compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePerformanceMetrics {
    /// Total execution time
    pub execution_time: Duration,

    /// Queue wait time
    pub queue_time: Duration,

    /// Resource utilization
    pub resource_utilization: ResourceUtilization,

    /// Cost breakdown
    pub cost_breakdown: CostBreakdown,

    /// Provider health during operation
    pub provider_health: f64,
}

/// Resource utilization prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU utilization (0.0 - 1.0)
    pub cpu_utilization: f64,

    /// Memory utilization (0.0 - 1.0)
    pub memory_utilization: f64,

    /// GPU utilization (0.0 - 1.0)
    pub gpu_utilization: Option<f64>,

    /// Network utilization (0.0 - 1.0)
    pub network_utilization: Option<f64>,
}

/// Cost breakdown for compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// CPU cost
    pub cpu_cost: f64,

    /// Memory cost
    pub memory_cost: f64,

    /// GPU cost (if applicable)
    pub gpu_cost: Option<f64>,

    /// Storage cost
    pub storage_cost: f64,

    /// Network cost
    pub network_cost: f64,

    /// Total cost
    pub total_cost: f64,
}

/// AI insights and recommendations for compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIComputeInsights {
    /// Confidence in operation success
    pub confidence_score: f64,

    /// Performance optimizations
    pub performance_optimizations: Vec<String>,

    /// Cost optimizations
    pub cost_optimizations: Vec<String>,

    /// Alternative providers
    pub alternative_providers: Vec<String>,

    /// Workload analysis
    pub workload_analysis: WorkloadAnalysis,
}

/// Workload analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadAnalysis {
    /// Detected workload patterns
    pub patterns: Vec<String>,

    /// Resource efficiency score
    pub efficiency_score: f64,

    /// Bottleneck analysis
    pub bottlenecks: Vec<String>,

    /// Optimization recommendations
    pub recommendations: Vec<String>,
}
