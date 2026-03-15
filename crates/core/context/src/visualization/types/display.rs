// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Display and conversion implementations

use super::core::VisualizationType;
use std::fmt;
use std::str::FromStr;

impl fmt::Display for VisualizationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VisualizationType::ContextState => write!(f, "context_state"),
            VisualizationType::RuleDependencyGraph => write!(f, "rule_dependency_graph"),
            VisualizationType::Timeline => write!(f, "timeline"),
            VisualizationType::MetricsDashboard => write!(f, "metrics_dashboard"),
            VisualizationType::StateDiff => write!(f, "state_diff"),
            VisualizationType::PerformanceHeatmap => write!(f, "performance_heatmap"),
            VisualizationType::InteractiveGraph => write!(f, "interactive_graph"),
            VisualizationType::Custom(name) => write!(f, "{name}"),
        }
    }
}

impl FromStr for VisualizationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "context_state" => Ok(VisualizationType::ContextState),
            "rule_dependency_graph" => Ok(VisualizationType::RuleDependencyGraph),
            "timeline" => Ok(VisualizationType::Timeline),
            "metrics_dashboard" => Ok(VisualizationType::MetricsDashboard),
            "state_diff" => Ok(VisualizationType::StateDiff),
            "performance_heatmap" => Ok(VisualizationType::PerformanceHeatmap),
            "interactive_graph" => Ok(VisualizationType::InteractiveGraph),
            custom => Ok(VisualizationType::Custom(custom.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_context_state() {
        assert_eq!(VisualizationType::ContextState.to_string(), "context_state");
    }

    #[test]
    fn test_display_rule_dependency_graph() {
        assert_eq!(
            VisualizationType::RuleDependencyGraph.to_string(),
            "rule_dependency_graph"
        );
    }

    #[test]
    fn test_display_timeline() {
        assert_eq!(VisualizationType::Timeline.to_string(), "timeline");
    }

    #[test]
    fn test_display_metrics_dashboard() {
        assert_eq!(
            VisualizationType::MetricsDashboard.to_string(),
            "metrics_dashboard"
        );
    }

    #[test]
    fn test_display_state_diff() {
        assert_eq!(VisualizationType::StateDiff.to_string(), "state_diff");
    }

    #[test]
    fn test_display_performance_heatmap() {
        assert_eq!(
            VisualizationType::PerformanceHeatmap.to_string(),
            "performance_heatmap"
        );
    }

    #[test]
    fn test_display_interactive_graph() {
        assert_eq!(
            VisualizationType::InteractiveGraph.to_string(),
            "interactive_graph"
        );
    }

    #[test]
    fn test_display_custom() {
        assert_eq!(
            VisualizationType::Custom("my_viz".to_string()).to_string(),
            "my_viz"
        );
    }

    #[test]
    fn test_from_str_all_variants() {
        assert_eq!(
            "context_state".parse::<VisualizationType>().unwrap(),
            VisualizationType::ContextState
        );
        assert_eq!(
            "rule_dependency_graph"
                .parse::<VisualizationType>()
                .unwrap(),
            VisualizationType::RuleDependencyGraph
        );
        assert_eq!(
            "timeline".parse::<VisualizationType>().unwrap(),
            VisualizationType::Timeline
        );
        assert_eq!(
            "metrics_dashboard".parse::<VisualizationType>().unwrap(),
            VisualizationType::MetricsDashboard
        );
        assert_eq!(
            "state_diff".parse::<VisualizationType>().unwrap(),
            VisualizationType::StateDiff
        );
        assert_eq!(
            "performance_heatmap".parse::<VisualizationType>().unwrap(),
            VisualizationType::PerformanceHeatmap
        );
        assert_eq!(
            "interactive_graph".parse::<VisualizationType>().unwrap(),
            VisualizationType::InteractiveGraph
        );
    }

    #[test]
    fn test_from_str_custom() {
        let result = "my_custom_viz".parse::<VisualizationType>().unwrap();
        assert_eq!(
            result,
            VisualizationType::Custom("my_custom_viz".to_string())
        );
    }

    #[test]
    fn test_roundtrip_display_from_str() {
        let variants = vec![
            VisualizationType::ContextState,
            VisualizationType::RuleDependencyGraph,
            VisualizationType::Timeline,
            VisualizationType::MetricsDashboard,
            VisualizationType::StateDiff,
            VisualizationType::PerformanceHeatmap,
            VisualizationType::InteractiveGraph,
            VisualizationType::Custom("custom_viz".to_string()),
        ];

        for variant in variants {
            let s = variant.to_string();
            let parsed: VisualizationType = s.parse().unwrap();
            assert_eq!(parsed, variant);
        }
    }
}
