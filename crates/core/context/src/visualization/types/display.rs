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
