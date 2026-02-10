// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Scaling configuration for MCP services, workflows, and compositions.
//!
//! This module provides the canonical `ScalingConfig` used across the MCP subsystem
//! for auto-scaling behavior, instance limits, and scaling metrics.

use serde::{Deserialize, Serialize};

/// Scaling configuration
///
/// Controls auto-scaling behavior for MCP services, workflows, and service compositions.
/// Supports both horizontal scaling (instance count) and metric-based scaling decisions.
///
/// # Examples
///
/// ```rust
/// use squirrel_mcp::config::ScalingConfig;
///
/// let config = ScalingConfig {
///     auto_scaling: true,
///     min_instances: 1,
///     max_instances: 10,
///     metrics: vec!["cpu_usage".to_string(), "memory_usage".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScalingConfig {
    /// Auto-scaling enabled
    ///
    /// When true, the system will automatically adjust instance count based on metrics.
    /// When false, instance count remains fixed at `min_instances`.
    pub auto_scaling: bool,
    
    /// Minimum number of instances
    ///
    /// The system will never scale below this number, even if metrics indicate low load.
    /// Ensures minimum availability and redundancy.
    pub min_instances: u32,
    
    /// Maximum number of instances
    ///
    /// The system will never scale above this number, even if metrics indicate high load.
    /// Prevents resource exhaustion and runaway costs.
    pub max_instances: u32,
    
    /// Scaling metrics
    ///
    /// List of metric names to monitor for scaling decisions.
    /// Common metrics include:
    /// - "cpu_usage"
    /// - "memory_usage"
    /// - "request_rate"
    /// - "response_time"
    /// - "error_rate"
    pub metrics: Vec<String>,
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            auto_scaling: false,
            min_instances: 1,
            max_instances: 1,
            metrics: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = ScalingConfig::default();
        assert!(!config.auto_scaling);
        assert_eq!(config.min_instances, 1);
        assert_eq!(config.max_instances, 1);
        assert!(config.metrics.is_empty());
    }

    #[test]
    fn test_serde() {
        let config = ScalingConfig {
            auto_scaling: true,
            min_instances: 2,
            max_instances: 10,
            metrics: vec!["cpu".to_string(), "memory".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ScalingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}

