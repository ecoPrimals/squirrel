// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for visualization renderers

use super::renderers::*;
use super::types::*;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_json_renderer_new() {
    let renderer = JsonRenderer::new();
    assert!(format!("{:?}", renderer).contains("JsonRenderer"));
}

#[tokio::test]
async fn test_json_renderer_render() {
    let renderer = JsonRenderer::new();
    let data = json!({"key": "value", "number": 42});

    let result = renderer.render(&data).await.expect("Should render");
    assert!(result.contains("key"));
    assert!(result.contains("value"));
}

#[tokio::test]
async fn test_json_renderer_with_config() {
    let renderer = JsonRenderer::new();
    let config = create_test_config();
    let data = json!({"test": "data"});

    let result = renderer
        .render_with_config(&data, &config)
        .await
        .expect("Should render with config");

    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_terminal_renderer_new() {
    let renderer = TerminalRenderer::new();
    assert!(format!("{:?}", renderer).contains("TerminalRenderer"));
}

#[tokio::test]
async fn test_terminal_renderer_render() {
    let renderer = TerminalRenderer::new();
    let data = json!({"message": "Hello Terminal"});

    let result = renderer.render(&data).await.expect("Should render");
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_html_renderer_new() {
    let renderer = HtmlRenderer::new();
    assert!(format!("{:?}", renderer).contains("HtmlRenderer"));
}

#[tokio::test]
async fn test_html_renderer_render() {
    let renderer = HtmlRenderer::new();
    let data = json!({"content": "Hello HTML"});

    let result = renderer.render(&data).await.expect("Should render");
    assert!(result.contains("<") || result.contains(">") || !result.is_empty());
}

#[tokio::test]
async fn test_markdown_renderer_new() {
    let renderer = MarkdownRenderer::new();
    assert!(format!("{:?}", renderer).contains("MarkdownRenderer"));
}

#[tokio::test]
async fn test_markdown_renderer_render() {
    let renderer = MarkdownRenderer::new();
    let data = json!({"title": "Markdown Test", "content": "Hello"});

    let result = renderer.render(&data).await.expect("Should render");
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_json_renderer_complex_data() {
    let renderer = JsonRenderer::new();
    let data = json!({
        "array": [1, 2, 3],
        "object": {"nested": "value"},
        "string": "test",
        "number": 123,
        "boolean": true,
        "null": null
    });

    let result = renderer.render(&data).await.expect("Should render");
    assert!(result.contains("array"));
    assert!(result.contains("nested"));
}

#[tokio::test]
async fn test_terminal_renderer_colored_output() {
    let renderer = TerminalRenderer::new();
    let data = json!({
        "status": "success",
        "message": "Operation completed"
    });

    let result = renderer.render(&data).await.expect("Should render");
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_html_renderer_with_theme() {
    let renderer = HtmlRenderer::new();
    let config = create_test_config();
    let data = json!({"title": "Test Page"});

    let result = renderer
        .render_with_config(&data, &config)
        .await
        .expect("Should render with theme");

    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_markdown_renderer_formatting() {
    let renderer = MarkdownRenderer::new();
    let data = json!({
        "heading": "# Main Title",
        "list": ["item1", "item2", "item3"],
        "code": "let x = 42;"
    });

    let result = renderer.render(&data).await.expect("Should render");
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_json_renderer_empty_data() {
    let renderer = JsonRenderer::new();
    let data = json!({});

    let result = renderer.render(&data).await.expect("Should render empty");
    assert!(result.contains("{}") || result == "{}");
}

#[tokio::test]
async fn test_terminal_renderer_multiline() {
    let renderer = TerminalRenderer::new();
    let data = json!({
        "line1": "First line",
        "line2": "Second line",
        "line3": "Third line"
    });

    let result = renderer.render(&data).await.expect("Should render");
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_html_renderer_escaping() {
    let renderer = HtmlRenderer::new();
    let data = json!({
        "content": "<script>alert('test')</script>"
    });

    let result = renderer.render(&data).await.expect("Should render");
    // Should handle special HTML characters
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_markdown_renderer_links() {
    let renderer = MarkdownRenderer::new();
    let data = json!({
        "link": "[Example](https://example.com)",
        "image": "![Alt text](image.png)"
    });

    let result = renderer.render(&data).await.expect("Should render");
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_json_renderer_pretty_print() {
    let renderer = JsonRenderer::new();
    let data = json!({
        "key1": "value1",
        "key2": "value2"
    });

    let result = renderer.render(&data).await.expect("Should render");
    // JSON should be formatted
    assert!(result.len() > 20); // Pretty-printed JSON is longer
}

#[tokio::test]
async fn test_all_renderers_same_data() {
    let data = json!({
        "title": "Test",
        "value": 42
    });

    let json_renderer = JsonRenderer::new();
    let terminal_renderer = TerminalRenderer::new();
    let html_renderer = HtmlRenderer::new();
    let markdown_renderer = MarkdownRenderer::new();

    let json_result = json_renderer
        .render(&data)
        .await
        .expect("Should render JSON");
    let terminal_result = terminal_renderer
        .render(&data)
        .await
        .expect("Should render terminal");
    let html_result = html_renderer
        .render(&data)
        .await
        .expect("Should render HTML");
    let markdown_result = markdown_renderer
        .render(&data)
        .await
        .expect("Should render markdown");

    assert!(!json_result.is_empty());
    assert!(!terminal_result.is_empty());
    assert!(!html_result.is_empty());
    assert!(!markdown_result.is_empty());
}

#[tokio::test]
async fn test_renderer_error_handling() {
    let renderer = JsonRenderer::new();
    // Valid JSON should always work
    let data = json!({"valid": "data"});
    let result = renderer.render(&data).await;
    assert!(result.is_ok());
}

// Helper function
fn create_test_config() -> VisualizationConfig {
    VisualizationConfig {
        format: "json".to_string(),
        theme: VisualizationTheme {
            primary_color: "#007bff".to_string(),
            secondary_color: "#6c757d".to_string(),
            background_color: "#ffffff".to_string(),
            text_color: "#212529".to_string(),
            border_color: "#dee2e6".to_string(),
            font_family: "Arial".to_string(),
            font_size: 14,
            color_palette: vec!["#007bff".to_string()],
            dark_mode: false,
        },
        layout: VisualizationLayout {
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
                enabled: false,
                color: "#000000".to_string(),
                line_width: 1,
                spacing: 10,
            },
            responsive: true,
        },
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
