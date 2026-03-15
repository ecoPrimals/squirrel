// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: Visualization mechanics licensed under ORC
// Copyright (C) 2026 DataScienceBioLab

//! Tests for visualization manager

use super::manager::*;
use super::types::*;
use super::VisualizationSystemConfig;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_visualization_manager_new() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create visualization manager");

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_created, 0);
}

#[tokio::test]
async fn test_visualization_manager_create() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    assert!(!response.visualization_id.is_empty());
    assert_eq!(response.format, "json");
}

#[tokio::test]
async fn test_visualization_manager_get_active() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let _ = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    let active_list = manager.get_active_visualizations().await;

    assert!(!active_list.is_empty());
}

#[tokio::test]
async fn test_visualization_manager_update() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    let new_data = json!({"updated": "data"});
    manager
        .update_visualization(&response.visualization_id, new_data)
        .await
        .expect("Should update visualization");

    let stats = manager.get_stats().await;
    assert!(stats.total_updated > 0);
}

#[tokio::test]
async fn test_visualization_manager_delete() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    manager
        .delete_visualization(&response.visualization_id)
        .await
        .expect("Should delete visualization");

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_deleted, 1);
}

#[tokio::test]
async fn test_visualization_manager_list_active() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    // Create multiple visualizations
    for _ in 0..3 {
        let request = create_test_request();
        manager
            .create_visualization(request)
            .await
            .expect("Should create visualization");
    }

    let list = manager.get_active_visualizations().await;
    assert_eq!(list.len(), 3);
}

#[tokio::test]
async fn test_visualization_manager_render() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    let rendered = manager
        .render_visualization(&response.visualization_id, "json")
        .await
        .expect("Should render visualization");

    assert!(!rendered.is_empty());
}

#[tokio::test]
async fn test_visualization_manager_stats() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_created, 1);
}

#[tokio::test]
async fn test_visualization_manager_cache() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    // First render - cache miss
    let _ = manager
        .render_visualization(&response.visualization_id, "json")
        .await
        .expect("Should render");

    // Second render - cache hit
    let _ = manager
        .render_visualization(&response.visualization_id, "json")
        .await
        .expect("Should render");

    let stats = manager.get_stats().await;
    assert!(stats.cache_hits > 0 || stats.cache_misses > 0);
}

#[tokio::test]
async fn test_visualization_manager_history() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    manager
        .update_visualization(&response.visualization_id, json!({"new": "data"}))
        .await
        .expect("Should update");

    let history = manager.get_history().await;
    assert!(!history.is_empty());
}

#[tokio::test]
async fn test_visualization_manager_multiple_renderings() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    // Render multiple formats (each format is a cache miss on first render)
    for format in ["json", "html", "terminal"] {
        manager
            .render_visualization(&response.visualization_id, format)
            .await
            .expect("Should render");
    }

    let stats = manager.get_stats().await;
    assert!(stats.total_renderings >= 3);
}

#[test]
fn test_active_visualization_creation() {
    let viz = ActiveVisualization {
        id: "viz-123".to_string(),
        visualization_type: VisualizationType::ContextState,
        config: create_test_config(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        data: json!({"test": "data"}),
        metadata: HashMap::new(),
    };

    assert_eq!(viz.id, "viz-123");
}

#[test]
fn test_visualization_action_variants() {
    let actions = vec![
        VisualizationAction::Created,
        VisualizationAction::Updated,
        VisualizationAction::Deleted,
        VisualizationAction::Rendered,
        VisualizationAction::Cached,
    ];

    for action in actions {
        let debug_str = format!("{:?}", action);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_visualization_action_serialization() {
    let action = VisualizationAction::Created;
    let serialized = serde_json::to_string(&action).expect("Should serialize");
    let deserialized: VisualizationAction =
        serde_json::from_str(&serialized).expect("Should deserialize");

    match deserialized {
        VisualizationAction::Created => {}
        _ => panic!("Unexpected deserialization result"),
    }
}

#[test]
fn test_visualization_manager_stats_default() {
    let stats = VisualizationManagerStats {
        total_created: 0,
        total_updated: 0,
        total_deleted: 0,
        total_renderings: 0,
        cache_hits: 0,
        cache_misses: 0,
        average_render_time_ms: 0.0,
        last_updated: Utc::now(),
    };

    assert_eq!(stats.total_created, 0);
    assert_eq!(stats.average_render_time_ms, 0.0);
}

#[test]
fn test_visualization_manager_stats_serialization() {
    let stats = VisualizationManagerStats {
        total_created: 10,
        total_updated: 5,
        total_deleted: 2,
        total_renderings: 20,
        cache_hits: 15,
        cache_misses: 5,
        average_render_time_ms: 12.5,
        last_updated: Utc::now(),
    };

    let serialized = serde_json::to_string(&stats).expect("Should serialize");
    let deserialized: VisualizationManagerStats =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.total_created, 10);
    assert_eq!(deserialized.cache_hits, 15);
}

#[test]
fn test_visualization_history_entry() {
    let entry = VisualizationHistoryEntry {
        id: "viz-123".to_string(),
        action: VisualizationAction::Created,
        timestamp: Utc::now(),
        data: json!({"info": "test"}),
    };

    assert_eq!(entry.id, "viz-123");
}

#[tokio::test]
async fn test_multiple_format_renders() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    let formats = vec!["json", "html", "markdown", "terminal"];
    for format in formats {
        let rendered = manager
            .render_visualization(&response.visualization_id, format)
            .await
            .expect(&format!("Should render in {} format", format));

        assert!(!rendered.is_empty());
    }
}

#[tokio::test]
async fn test_visualization_manager_render_invalid_format() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let request = create_test_request();
    let response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    let result = manager
        .render_visualization(&response.visualization_id, "invalid_format")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_visualization_manager_render_nonexistent() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let result = manager
        .render_visualization("nonexistent-viz-id", "json")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_visualization_manager_update_nonexistent() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let result = manager
        .update_visualization("nonexistent-viz-id", json!({"x": 1}))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_visualization_manager_start_stop() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    manager.start().await.expect("Should start");
    manager.stop().await.expect("Should stop");
}

#[test]
fn test_visualization_manager_stats_new() {
    let stats = VisualizationManagerStats::new();
    assert_eq!(stats.total_created, 0);
    assert_eq!(stats.total_updated, 0);
    assert_eq!(stats.total_deleted, 0);
    assert_eq!(stats.total_renderings, 0);
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
    assert_eq!(stats.average_render_time_ms, 0.0);
}

#[test]
fn test_visualization_manager_stats_cache_hit_rate() {
    let mut stats = VisualizationManagerStats::new();
    assert_eq!(stats.cache_hit_rate(), 0.0);

    stats.cache_hits = 3;
    stats.cache_misses = 7;
    assert!((stats.cache_hit_rate() - 0.3).abs() < 0.001);

    stats.cache_hits = 5;
    stats.cache_misses = 5;
    assert!((stats.cache_hit_rate() - 0.5).abs() < 0.001);
}

#[tokio::test]
async fn test_visualization_metadata() {
    let config = VisualizationSystemConfig::default();
    let manager = VisualizationManager::new(Arc::new(config))
        .await
        .expect("Should create manager");

    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), json!("value1"));
    metadata.insert("key2".to_string(), json!(42));

    let request = VisualizationRequest {
        visualization_type: VisualizationType::MetricsDashboard,
        config: create_test_config(),
        data: json!({}),
        metadata,
        title: Some("Test with Metadata".to_string()),
        description: None,
    };

    let _response = manager
        .create_visualization(request)
        .await
        .expect("Should create visualization");

    let active_vizs = manager.get_active_visualizations().await;
    assert!(!active_vizs.is_empty());
    assert_eq!(active_vizs[0].metadata.len(), 2);
}

// Helper functions
fn create_test_request() -> VisualizationRequest {
    VisualizationRequest {
        visualization_type: VisualizationType::ContextState,
        config: create_test_config(),
        data: json!({"test": "data"}),
        metadata: HashMap::new(),
        title: Some("Test Visualization".to_string()),
        description: Some("A test visualization".to_string()),
    }
}

fn create_test_config() -> VisualizationConfig {
    VisualizationConfig {
        format: "json".to_string(),
        theme: create_test_theme(),
        layout: create_test_layout(),
        interactive: false,
        animation: AnimationConfig {
            enabled: false,
            duration: 0,
            easing: "none".to_string(),
            delay: 0,
        },
        export: ExportConfig {
            formats: vec![],
            default_format: "json".to_string(),
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
        primary_color: "#000000".to_string(),
        secondary_color: "#ffffff".to_string(),
        background_color: "#f0f0f0".to_string(),
        text_color: "#333333".to_string(),
        border_color: "#cccccc".to_string(),
        font_family: "monospace".to_string(),
        font_size: 12,
        color_palette: vec!["#000000".to_string()],
        dark_mode: false,
    }
}

fn create_test_layout() -> VisualizationLayout {
    VisualizationLayout {
        width: 800,
        height: 600,
        margin: MarginConfig {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        },
        padding: PaddingConfig {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        },
        grid: GridConfig {
            enabled: false,
            color: "#000000".to_string(),
            line_width: 1,
            spacing: 10,
        },
        responsive: false,
    }
}
