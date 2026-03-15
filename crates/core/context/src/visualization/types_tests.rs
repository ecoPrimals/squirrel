// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for visualization types

use super::types::*;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_visualization_type_variants() {
    let types = vec![
        VisualizationType::ContextState,
        VisualizationType::RuleDependencyGraph,
        VisualizationType::Timeline,
        VisualizationType::MetricsDashboard,
        VisualizationType::StateDiff,
        VisualizationType::PerformanceHeatmap,
        VisualizationType::InteractiveGraph,
        VisualizationType::Custom("test".to_string()),
    ];

    for viz_type in types {
        let debug_str = format!("{:?}", viz_type);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_visualization_type_serialization() {
    let viz_type = VisualizationType::ContextState;
    let serialized = serde_json::to_string(&viz_type).expect("Should serialize");
    let deserialized: VisualizationType =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized, viz_type);
}

#[test]
fn test_visualization_type_custom() {
    let viz_type = VisualizationType::Custom("my_custom_viz".to_string());
    let serialized = serde_json::to_string(&viz_type).expect("Should serialize");
    assert!(serialized.contains("my_custom_viz"));
}

#[test]
fn test_visualization_request_creation() {
    let request = VisualizationRequest {
        visualization_type: VisualizationType::ContextState,
        config: create_test_config(),
        data: json!({"key": "value"}),
        metadata: HashMap::new(),
        title: Some("Test Visualization".to_string()),
        description: Some("A test visualization".to_string()),
    };

    assert_eq!(request.visualization_type, VisualizationType::ContextState);
    assert!(request.title.is_some());
}

#[test]
fn test_visualization_request_serialization() {
    let request = create_test_request();
    let serialized = serde_json::to_string(&request).expect("Should serialize");
    let deserialized: VisualizationRequest =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.visualization_type, request.visualization_type);
}

#[test]
fn test_visualization_response_creation() {
    let response = VisualizationResponse {
        visualization_id: "viz-123".to_string(),
        visualization_type: VisualizationType::MetricsDashboard,
        format: "html".to_string(),
        content: "<div>Test</div>".to_string(),
        metadata: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(response.visualization_id, "viz-123");
    assert_eq!(response.format, "html");
}

#[test]
fn test_visualization_response_serialization() {
    let response = create_test_response();
    let serialized = serde_json::to_string(&response).expect("Should serialize");
    let deserialized: VisualizationResponse =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.visualization_id, response.visualization_id);
}

#[test]
fn test_visualization_config_default() {
    let config = create_test_config();
    assert_eq!(config.format, "json");
    assert!(config.interactive);
}

#[test]
fn test_visualization_config_serialization() {
    let config = create_test_config();
    let serialized = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: VisualizationConfig =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.format, config.format);
}

#[test]
fn test_visualization_theme_default() {
    let theme = create_test_theme();
    assert_eq!(theme.primary_color, "#007bff");
    assert_eq!(theme.font_size, 14);
    assert!(!theme.dark_mode);
}

#[test]
fn test_visualization_theme_serialization() {
    let theme = create_test_theme();
    let serialized = serde_json::to_string(&theme).expect("Should serialize");
    let deserialized: VisualizationTheme =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.primary_color, theme.primary_color);
}

#[test]
fn test_visualization_layout() {
    let layout = create_test_layout();
    assert_eq!(layout.width, 800);
    assert_eq!(layout.height, 600);
}

#[test]
fn test_visualization_layout_serialization() {
    let layout = create_test_layout();
    let serialized = serde_json::to_string(&layout).expect("Should serialize");
    let deserialized: VisualizationLayout =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.width, layout.width);
    assert_eq!(deserialized.height, layout.height);
}

#[test]
fn test_margin_config() {
    let margin = MarginConfig {
        top: 10,
        right: 20,
        bottom: 30,
        left: 40,
    };

    let serialized = serde_json::to_string(&margin).expect("Should serialize");
    let deserialized: MarginConfig = serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.top, 10);
    assert_eq!(deserialized.left, 40);
}

#[test]
fn test_padding_config() {
    let padding = PaddingConfig {
        top: 5,
        right: 10,
        bottom: 15,
        left: 20,
    };

    let serialized = serde_json::to_string(&padding).expect("Should serialize");
    let deserialized: PaddingConfig =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.top, 5);
    assert_eq!(deserialized.right, 10);
}

#[test]
fn test_animation_config() {
    let animation = AnimationConfig {
        enabled: true,
        duration: 500,
        easing: "ease-in-out".to_string(),
        delay: 0,
    };

    let serialized = serde_json::to_string(&animation).expect("Should serialize");
    let deserialized: AnimationConfig =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.duration, 500);
    assert_eq!(deserialized.easing, "ease-in-out");
}

#[test]
fn test_export_config() {
    let export = ExportConfig {
        formats: vec!["png".to_string(), "svg".to_string()],
        default_format: "png".to_string(),
        quality: QualityConfig {
            image_quality: 90,
            dpi: 300,
            compression: 80,
        },
    };

    let serialized = serde_json::to_string(&export).expect("Should serialize");
    let deserialized: ExportConfig = serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.formats.len(), 2);
    assert_eq!(deserialized.quality, 90);
}

#[test]
fn test_data_point() {
    let point = DataPoint {
        x: 10.0,
        y: 20.0,
        z: Some(30.0),
        value: json!(25.0),
        label: Some("Point A".to_string()),
        color: None,
        size: None,
    };

    let serialized = serde_json::to_string(&point).expect("Should serialize");
    let deserialized: DataPoint = serde_json::from_str(&serialized).expect("Should deserialize");

    assert!(deserialized.z.is_some());
    assert_eq!(deserialized.label, Some("Point A".to_string()));
}

#[test]
fn test_visualization_series() {
    let series = VisualizationSeries {
        name: "Series 1".to_string(),
        data: vec![],
        series_type: "line".to_string(),
        color: Some("#ff0000".to_string()),
        visible: true,
        metadata: HashMap::new(),
    };

    let serialized = serde_json::to_string(&series).expect("Should serialize");
    let deserialized: VisualizationSeries =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.name, "Series 1");
    assert!(deserialized.visible);
}

#[test]
fn test_interactive_element() {
    let element = InteractiveElement {
        id: "element-1".to_string(),
        element_type: "button".to_string(),
        label: "Click Me".to_string(),
        action: "toggle".to_string(),
        parameters: HashMap::new(),
        enabled: true,
    };

    let serialized = serde_json::to_string(&element).expect("Should serialize");
    let deserialized: InteractiveElement =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.id, "element-1");
    assert_eq!(deserialized.label, "Click Me");
}

#[test]
fn test_filter_config() {
    let filter = FilterConfig {
        enabled: true,
        filter_type: "range".to_string(),
        min_value: Some(json!(0)),
        max_value: Some(json!(100)),
        current_value: json!(50),
    };

    let serialized = serde_json::to_string(&filter).expect("Should serialize");
    let deserialized: FilterConfig = serde_json::from_str(&serialized).expect("Should deserialize");

    assert!(deserialized.enabled);
    assert!(deserialized.min_value.is_some());
}

#[test]
fn test_axis_config() {
    let axis = AxisConfig {
        label: "X Axis".to_string(),
        scale: "linear".to_string(),
        min: Some(0.0),
        max: Some(100.0),
        grid: true,
        ticks: create_test_tick_config(),
    };

    let serialized = serde_json::to_string(&axis).expect("Should serialize");
    let deserialized: AxisConfig = serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.label, "X Axis");
    assert!(deserialized.grid);
}

#[test]
fn test_tick_config() {
    let tick = TickConfig {
        count: 10,
        format: "%.2f".to_string(),
        rotation: 0,
    };

    let serialized = serde_json::to_string(&tick).expect("Should serialize");
    let deserialized: TickConfig = serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.count, 10);
    assert_eq!(deserialized.rotation, 0);
}

#[test]
fn test_legend_config() {
    let legend = LegendConfig {
        enabled: true,
        position: "top-right".to_string(),
        orientation: "vertical".to_string(),
    };

    let serialized = serde_json::to_string(&legend).expect("Should serialize");
    let deserialized: LegendConfig = serde_json::from_str(&serialized).expect("Should deserialize");

    assert!(deserialized.enabled);
    assert_eq!(deserialized.position, "top-right");
}

#[test]
fn test_tooltip_config() {
    let tooltip = TooltipConfig {
        enabled: true,
        format: "html".to_string(),
        show_delay_ms: 100,
    };

    let serialized = serde_json::to_string(&tooltip).expect("Should serialize");
    let deserialized: TooltipConfig =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert!(deserialized.enabled);
    assert_eq!(deserialized.show_delay_ms, 100);
}

// Helper functions
fn create_test_config() -> VisualizationConfig {
    VisualizationConfig {
        format: "json".to_string(),
        theme: create_test_theme(),
        layout: create_test_layout(),
        interactive: true,
        animation: AnimationConfig {
            enabled: true,
            duration: 300,
            easing: "ease".to_string(),
            delay: 0,
        },
        export: ExportConfig {
            enabled: false,
            formats: vec![],
            quality: QualityConfig {
                image_quality: 100,
                dpi: 300,
                compression: 80,
            },
        },
        custom_options: HashMap::new(),
    }
}

fn create_test_theme() -> VisualizationTheme {
    VisualizationTheme {
        primary_color: "#007bff".to_string(),
        secondary_color: "#6c757d".to_string(),
        background_color: "#ffffff".to_string(),
        text_color: "#212529".to_string(),
        border_color: "#dee2e6".to_string(),
        font_family: "Arial".to_string(),
        font_size: 14,
        color_palette: vec!["#007bff".to_string(), "#6c757d".to_string()],
        dark_mode: false,
    }
}

fn create_test_layout() -> VisualizationLayout {
    VisualizationLayout {
        width: 800,
        height: 600,
        margin: MarginConfig {
            top: 20,
            right: 20,
            bottom: 20,
            left: 20,
        },
        padding: PaddingConfig {
            top: 10,
            right: 10,
            bottom: 10,
            left: 10,
        },
        grid: GridConfig {
            enabled: true,
            color: "#cccccc".to_string(),
            line_width: 1,
            spacing: 10,
        },
        responsive: true,
    }
}

fn create_test_request() -> VisualizationRequest {
    VisualizationRequest {
        visualization_type: VisualizationType::ContextState,
        config: create_test_config(),
        data: json!({"test": "data"}),
        metadata: HashMap::new(),
        title: Some("Test".to_string()),
        description: None,
    }
}

fn create_test_response() -> VisualizationResponse {
    VisualizationResponse {
        visualization_id: "viz-test".to_string(),
        visualization_type: VisualizationType::ContextState,
        format: "json".to_string(),
        content: "{}".to_string(),
        metadata: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn create_test_tick_config() -> TickConfig {
    TickConfig {
        count: 5,
        format: "%d".to_string(),
        rotation: 0,
    }
}
