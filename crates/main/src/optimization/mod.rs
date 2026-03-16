// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
pub mod prelude {
    pub use super::zero_copy::*;
}

// Version information for optimization features

/// Version string for the optimization system.
pub const OPTIMIZATION_VERSION: &str = "1.0.0";

/// Check if optimizations are enabled
#[must_use]
pub const fn optimizations_enabled() -> bool {
    true // Optimizations always enabled; feature gate reserved for future use
}

/// Get optimization system information
#[must_use]
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
    /// Version of the optimization system
    pub version: String,
    /// Whether optimizations are enabled
    pub enabled: bool,
    /// List of enabled optimization features
    pub features: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_version() {
        assert_eq!(OPTIMIZATION_VERSION, "1.0.0");
    }

    #[test]
    fn test_optimizations_enabled() {
        assert!(optimizations_enabled());
    }

    #[test]
    fn test_get_optimization_info() {
        let info = get_optimization_info();
        assert_eq!(info.version, "1.0.0");
        assert!(info.enabled);
        assert!(!info.features.is_empty());
    }

    #[test]
    fn test_optimization_info_features() {
        let info = get_optimization_info();
        assert!(info.features.contains(&"zero_copy_strings".to_string()));
        assert!(info.features.contains(&"efficient_collections".to_string()));
        assert!(info.features.contains(&"buffer_pooling".to_string()));
        assert!(info.features.contains(&"message_optimization".to_string()));
        assert!(
            info.features
                .contains(&"performance_monitoring".to_string())
        );
        assert_eq!(info.features.len(), 5);
    }

    #[test]
    fn test_optimization_info_clone() {
        let info = get_optimization_info();
        let cloned = info.clone();
        assert_eq!(cloned.version, info.version);
        assert_eq!(cloned.enabled, info.enabled);
        assert_eq!(cloned.features.len(), info.features.len());
    }
}
