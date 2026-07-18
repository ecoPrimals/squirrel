// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Visualization Type System
//!
//! Organized into logical modules for better maintainability:
//! - `core`: Core visualization types and structures
//! - `config`: Configuration structures and settings
//! - `theme`: Theme and layout types
//! - `display`: Display implementations and conversions

pub mod config;
pub mod core;
pub mod display;

pub use config::*;
pub use core::*;

// Display trait implementations are in the display module but don't need re-export
// as they're automatically available through trait imports

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // -- VisualizationType tests --

    #[test]
    fn test_visualization_type_display() {
        assert_eq!(VisualizationType::ContextState.to_string(), "context_state");
        assert_eq!(
            VisualizationType::RuleDependencyGraph.to_string(),
            "rule_dependency_graph"
        );
        assert_eq!(VisualizationType::Timeline.to_string(), "timeline");
        assert_eq!(
            VisualizationType::MetricsDashboard.to_string(),
            "metrics_dashboard"
        );
        assert_eq!(VisualizationType::StateDiff.to_string(), "state_diff");
        assert_eq!(
            VisualizationType::PerformanceHeatmap.to_string(),
            "performance_heatmap"
        );
        assert_eq!(
            VisualizationType::InteractiveGraph.to_string(),
            "interactive_graph"
        );
        assert_eq!(
            VisualizationType::Custom("my_viz".into()).to_string(),
            "my_viz"
        );
    }

    #[test]
    fn test_visualization_type_from_str() {
        assert_eq!(
            VisualizationType::from_str("context_state").expect("should succeed"),
            VisualizationType::ContextState
        );
        assert_eq!(
            VisualizationType::from_str("timeline").expect("should succeed"),
            VisualizationType::Timeline
        );
        assert_eq!(
            VisualizationType::from_str("metrics_dashboard").expect("should succeed"),
            VisualizationType::MetricsDashboard
        );
        assert_eq!(
            VisualizationType::from_str("custom_thing").expect("should succeed"),
            VisualizationType::Custom("custom_thing".into())
        );
    }

    #[test]
    fn test_visualization_type_roundtrip() {
        let types = vec![
            VisualizationType::ContextState,
            VisualizationType::RuleDependencyGraph,
            VisualizationType::Timeline,
            VisualizationType::MetricsDashboard,
            VisualizationType::StateDiff,
            VisualizationType::PerformanceHeatmap,
            VisualizationType::InteractiveGraph,
        ];
        for vt in types {
            let s = vt.to_string();
            let parsed = VisualizationType::from_str(&s).expect("should succeed");
            assert_eq!(parsed, vt);
        }
    }

    #[test]
    fn test_visualization_type_serde() {
        let vt = VisualizationType::MetricsDashboard;
        let json = serde_json::to_string(&vt).expect("should succeed");
        let deserialized: VisualizationType = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized, vt);
    }

    // -- Config defaults --

    #[test]
    fn test_visualization_config_default() {
        let config = VisualizationConfig::default();
        assert_eq!(config.format, "json");
        assert!(config.custom_options.is_empty());
    }

}
