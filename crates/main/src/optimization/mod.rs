pub mod zero_copy;

// Re-export commonly used optimization utilities
pub use zero_copy::*;

/// Module for performance optimization utilities
///
/// This module provides various optimization patterns and utilities:
/// - Zero-copy string handling
/// - Efficient collection operations
/// - Buffer pooling and management
/// - Message optimization
/// - Performance monitoring
///
/// These optimizations are designed to reduce memory allocations,
/// minimize cloning operations, and improve overall system performance.
pub mod optimization {
    pub use super::zero_copy::*;
}

// Version information for optimization features
pub const OPTIMIZATION_VERSION: &str = "1.0.0";

/// Check if optimizations are enabled
pub fn optimizations_enabled() -> bool {
    cfg!(feature = "optimizations") || true // Default to enabled
}

/// Get optimization system information
pub fn get_optimization_info() -> OptimizationInfo {
    OptimizationInfo {
        version: OPTIMIZATION_VERSION.to_string(),
        enabled: optimizations_enabled(),
        features: vec![
            "zero_copy_strings".to_string(),
            "efficient_collections".to_string(),
            "buffer_pooling".to_string(),
            "message_optimization".to_string(),
            "performance_monitoring".to_string(),
        ],
    }
}

/// Optimization system information
#[derive(Debug, Clone)]
pub struct OptimizationInfo {
    pub version: String,
    pub enabled: bool,
    pub features: Vec<String>,
}
