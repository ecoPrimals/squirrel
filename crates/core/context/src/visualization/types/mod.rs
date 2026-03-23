// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Visualization Type System
//!
//! Organized into logical modules for better maintainability:
//! - `core`: Core visualization types and structures
//! - `config`: Configuration structures and settings
//! - `theme`: Theme and layout types
//! - `display`: Display implementations and conversions

#![allow(
    dead_code,
    reason = "Visualization type system expanded for planned UI features"
)]

pub mod config;
pub mod core;
pub mod display;
pub mod theme;

// Re-export commonly used types for backward compatibility
pub use config::*;
pub use core::*;
pub use theme::*;

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
            VisualizationType::from_str("context_state").unwrap(),
            VisualizationType::ContextState
        );
        assert_eq!(
            VisualizationType::from_str("timeline").unwrap(),
            VisualizationType::Timeline
        );
        assert_eq!(
            VisualizationType::from_str("metrics_dashboard").unwrap(),
            VisualizationType::MetricsDashboard
        );
        assert_eq!(
            VisualizationType::from_str("custom_thing").unwrap(),
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
            let parsed = VisualizationType::from_str(&s).unwrap();
            assert_eq!(parsed, vt);
        }
    }

    #[test]
    fn test_visualization_type_serde() {
        let vt = VisualizationType::MetricsDashboard;
        let json = serde_json::to_string(&vt).unwrap();
        let deserialized: VisualizationType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, vt);
    }

    // -- Theme defaults --

    #[test]
    fn test_visualization_theme_default() {
        let theme = VisualizationTheme::default();
        assert_eq!(theme.primary_color, "#3498db");
        assert_eq!(theme.font_size, 14);
        assert!(!theme.dark_mode);
        assert!(!theme.color_palette.is_empty());
    }

    #[test]
    fn test_visualization_theme_serde() {
        let theme = VisualizationTheme::default();
        let json = serde_json::to_string(&theme).unwrap();
        let deserialized: VisualizationTheme = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.primary_color, theme.primary_color);
        assert_eq!(deserialized.font_size, theme.font_size);
    }

    #[test]
    fn test_visualization_layout_default() {
        let layout = VisualizationLayout::default();
        assert_eq!(layout.width, 800);
        assert_eq!(layout.height, 600);
        assert!(layout.responsive);
    }

    #[test]
    fn test_margin_config_default() {
        let margin = MarginConfig::default();
        assert_eq!(margin.top, 20);
        assert_eq!(margin.right, 20);
        assert_eq!(margin.bottom, 20);
        assert_eq!(margin.left, 20);
    }

    #[test]
    fn test_padding_config_default() {
        let padding = PaddingConfig::default();
        assert_eq!(padding.top, 10);
        assert_eq!(padding.right, 10);
        assert_eq!(padding.bottom, 10);
        assert_eq!(padding.left, 10);
    }

    // -- Config defaults --

    #[test]
    fn test_visualization_config_default() {
        let config = VisualizationConfig::default();
        assert_eq!(config.format, "html");
        assert!(config.interactive);
        assert!(config.custom_options.is_empty());
    }

    #[test]
    fn test_grid_config_default() {
        let grid = GridConfig::default();
        assert!(grid.enabled);
        assert_eq!(grid.color, "#f0f0f0");
        assert_eq!(grid.line_width, 1);
        assert_eq!(grid.spacing, 20);
    }

    #[test]
    fn test_animation_config_default() {
        let anim = AnimationConfig::default();
        assert!(anim.enabled);
        assert_eq!(anim.duration, 1000);
        assert_eq!(anim.easing, "ease-in-out");
        assert_eq!(anim.delay, 0);
    }

    #[test]
    fn test_export_config_default() {
        let export = ExportConfig::default();
        assert_eq!(export.default_format, "png");
        assert!(export.formats.contains(&"png".to_string()));
        assert!(export.formats.contains(&"svg".to_string()));
        assert!(export.formats.contains(&"pdf".to_string()));
        assert!(export.formats.contains(&"json".to_string()));
    }

    #[test]
    fn test_quality_config_default() {
        let quality = QualityConfig::default();
        assert_eq!(quality.image_quality, 95);
        assert_eq!(quality.dpi, 300);
        assert_eq!(quality.compression, 80);
    }

    #[test]
    fn test_quality_config_serde() {
        let quality = QualityConfig::default();
        let json = serde_json::to_string(&quality).unwrap();
        let deserialized: QualityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, quality);
    }

    // -- Core types serde --

    #[test]
    fn test_data_point_serde() {
        let dp = DataPoint {
            x: 1.0,
            y: 2.0,
            z: Some(3.0),
            value: serde_json::json!(42),
            label: Some("point".into()),
            color: Some("#ff0000".into()),
            size: Some(5.0),
        };
        let json = serde_json::to_string(&dp).unwrap();
        let deserialized: DataPoint = serde_json::from_str(&json).unwrap();
        assert!((deserialized.x - 1.0).abs() < 1e-9);
        assert!((deserialized.y - 2.0).abs() < 1e-9);
        assert_eq!(deserialized.z, Some(3.0));
    }

    #[test]
    fn test_visualization_series_serde() {
        let series = VisualizationSeries {
            name: "series1".into(),
            data: vec![],
            color: Some("#00ff00".into()),
            series_type: "line".into(),
            metadata: std::collections::HashMap::new(),
        };
        let json = serde_json::to_string(&series).unwrap();
        let deserialized: VisualizationSeries = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "series1");
        assert_eq!(deserialized.series_type, "line");
    }

    #[test]
    fn test_interactive_element_serde() {
        let elem = InteractiveElement {
            element_type: "button".into(),
            id: "btn-1".into(),
            properties: std::collections::HashMap::new(),
            event_handlers: std::collections::HashMap::new(),
        };
        let json = serde_json::to_string(&elem).unwrap();
        let deserialized: InteractiveElement = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.element_type, "button");
        assert_eq!(deserialized.id, "btn-1");
    }

    #[test]
    fn test_filter_config_serde() {
        let filter = FilterConfig {
            filter_type: "range".into(),
            parameters: std::collections::HashMap::new(),
            enabled: true,
        };
        let json = serde_json::to_string(&filter).unwrap();
        let deserialized: FilterConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.filter_type, "range");
        assert!(deserialized.enabled);
    }
}
