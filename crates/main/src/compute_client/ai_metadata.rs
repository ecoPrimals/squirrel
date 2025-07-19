//! AI Metadata and Intelligence Types for Compute Client

use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// AI METADATA TYPES
// ============================================================================

/// AI-first metadata for intelligent compute decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIComputeMetadata {
    /// Confidence in provider selection
    pub provider_confidence: f64,
    
    /// Workload predictions
    pub workload_predictions: Vec<WorkloadPrediction>,
    
    /// Cost optimization suggestions
    pub cost_optimizations: Vec<String>,
    
    /// Performance predictions
    pub performance_predictions: ComputePerformancePrediction,
}

/// Workload prediction for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadPrediction {
    /// Workload type
    pub workload_type: String,
    
    /// Confidence in prediction
    pub confidence: f64,
    
    /// Resource utilization prediction
    pub resource_utilization: super::types::ResourceUtilization,
    
    /// Suggested optimizations
    pub optimizations: Vec<String>,
}

/// Performance predictions for compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePerformancePrediction {
    /// Expected execution time
    pub expected_execution_time: Duration,
    
    /// Expected cost
    pub expected_cost: f64,
    
    /// Expected resource efficiency
    pub resource_efficiency: f64,
    
    /// Confidence in predictions
    pub confidence: f64,
}

impl Default for AIComputeMetadata {
    fn default() -> Self {
        Self {
            provider_confidence: 0.8,
            workload_predictions: Vec::new(),
            cost_optimizations: Vec::new(),
            performance_predictions: ComputePerformancePrediction {
                expected_execution_time: Duration::from_secs(60),
                expected_cost: 0.10,
                resource_efficiency: 0.8,
                confidence: 0.7,
            },
        }
    }
} 