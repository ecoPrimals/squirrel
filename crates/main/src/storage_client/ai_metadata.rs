//! AI-Enhanced Storage Metadata and Intelligence

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// AI METADATA TYPES
// ============================================================================

/// AI-first metadata for intelligent storage decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIStorageMetadata {
    /// Confidence in storage provider selection
    pub provider_confidence: f64,
    
    /// Predicted access patterns
    pub access_patterns: Vec<AccessPattern>,
    
    /// Cost optimization suggestions
    pub cost_optimizations: Vec<String>,
    
    /// Performance predictions
    pub performance_predictions: PerformancePrediction,
}

/// Predicted access patterns for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Pattern type
    pub pattern_type: String,
    
    /// Confidence in prediction
    pub confidence: f64,
    
    /// Suggested optimizations
    pub optimizations: Vec<String>,
}

/// Performance predictions for storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    /// Expected latency (ms)
    pub expected_latency_ms: f64,
    
    /// Expected throughput (MB/s)  
    pub expected_throughput_mbps: f64,
    
    /// Confidence in predictions
    pub confidence: f64,
}

impl Default for AIStorageMetadata {
    fn default() -> Self {
        Self {
            provider_confidence: 0.8,
            access_patterns: Vec::new(),
            cost_optimizations: Vec::new(),
            performance_predictions: PerformancePrediction {
                expected_latency_ms: 100.0,
                expected_throughput_mbps: 100.0,
                confidence: 0.7,
            },
        }
    }
} 