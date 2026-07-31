// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Visualization Renderers
//!
//! This module provides different renderers for various output formats (JSON, HTML, Terminal, Markdown).

use crate::error::Result;
use serde_json::Value;

/// JSON renderer for visualization data
#[derive(Debug)]
pub struct JsonRenderer;

impl Default for JsonRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonRenderer {
    /// Create a new JSON renderer
    pub fn new() -> Self {
        Self
    }

    /// Render data as JSON
    pub async fn render(&self, data: &Value) -> Result<String> {
        serde_json::to_string_pretty(data)
            .map_err(|e| crate::error::ContextError::Serialization(e.to_string()))
    }
}

/// Terminal renderer for visualization data
#[derive(Debug)]
pub struct TerminalRenderer;

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalRenderer {
    /// Create a new terminal renderer
    pub fn new() -> Self {
        Self
    }

    /// Render data for terminal output
    pub async fn render(&self, data: &Value) -> Result<String> {
        // Simple terminal rendering - convert to formatted text
        let formatted = format_for_terminal(data, 0);
        Ok(formatted)
    }
}

// Helper functions for terminal formatting
fn format_for_terminal(value: &Value, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);

    match value {
        Value::Null => format!("{indent_str}null"),
        Value::Bool(b) => format!("{indent_str}{b}"),
        Value::Number(n) => format!("{indent_str}{n}"),
        Value::String(s) => format!("{indent_str}\"{s}\""),
        Value::Array(arr) => {
            let mut result = format!("{indent_str}[\n");
            for (i, item) in arr.iter().enumerate() {
                result.push_str(&format_for_terminal(item, indent + 1));
                if i < arr.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&format!("{indent_str}]"));
            result
        }
        Value::Object(obj) => {
            let mut result = format!("{indent_str}{{\n");
            let items: Vec<_> = obj.iter().collect();
            for (i, (key, value)) in items.iter().enumerate() {
                result.push_str(&format!("{}\"{}\": ", "  ".repeat(indent + 1), key));
                let value_str = format_for_terminal(value, 0).trim_start().to_string();
                result.push_str(&value_str);
                if i < items.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&format!("{indent_str}}}"));
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // JsonRenderer tests
    #[tokio::test]
    async fn test_json_renderer_new() {
        let renderer = JsonRenderer::new();
        let _ = format!("{renderer:?}");
    }

    #[tokio::test]
    async fn test_json_renderer_default() {
        let renderer = JsonRenderer;
        let _ = format!("{renderer:?}");
    }

    #[tokio::test]
    async fn test_json_renderer_render_object() {
        let renderer = JsonRenderer::new();
        let data = json!({"key": "value", "count": 42});
        let result = renderer.render(&data).await;
        assert!(result.is_ok());
        let output = result.expect("should succeed");
        assert!(output.contains("key"));
        assert!(output.contains("value"));
        assert!(output.contains("42"));
    }

    #[tokio::test]
    async fn test_json_renderer_render_null() {
        let renderer = JsonRenderer::new();
        let result = renderer.render(&json!(null)).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), "null");
    }

    // TerminalRenderer tests
    #[tokio::test]
    async fn test_terminal_renderer_new() {
        let renderer = TerminalRenderer::new();
        let _ = format!("{renderer:?}");
    }

    #[tokio::test]
    async fn test_terminal_renderer_default() {
        let renderer = TerminalRenderer;
        let _ = format!("{renderer:?}");
    }

    #[tokio::test]
    async fn test_terminal_renderer_render_null() {
        let renderer = TerminalRenderer::new();
        let result = renderer.render(&json!(null)).await;
        assert!(result.is_ok());
        assert!(result.expect("should succeed").contains("null"));
    }

    #[tokio::test]
    async fn test_terminal_renderer_render_bool() {
        let renderer = TerminalRenderer::new();
        let result = renderer.render(&json!(true)).await;
        assert!(result.is_ok());
        assert!(result.expect("should succeed").contains("true"));
    }

    #[tokio::test]
    async fn test_terminal_renderer_render_number() {
        let renderer = TerminalRenderer::new();
        let result = renderer.render(&json!(42)).await;
        assert!(result.is_ok());
        assert!(result.expect("should succeed").contains("42"));
    }

    #[tokio::test]
    async fn test_terminal_renderer_render_string() {
        let renderer = TerminalRenderer::new();
        let result = renderer.render(&json!("hello")).await;
        assert!(result.is_ok());
        assert!(result.expect("should succeed").contains("hello"));
    }

    #[tokio::test]
    async fn test_terminal_renderer_render_array() {
        let renderer = TerminalRenderer::new();
        let result = renderer.render(&json!([1, 2, 3])).await;
        assert!(result.is_ok());
        let output = result.expect("should succeed");
        assert!(output.contains('['));
        assert!(output.contains(']'));
        assert!(output.contains('1'));
    }

    #[tokio::test]
    async fn test_terminal_renderer_render_object() {
        let renderer = TerminalRenderer::new();
        let data = json!({"name": "test", "value": 42});
        let result = renderer.render(&data).await;
        assert!(result.is_ok());
        let output = result.expect("should succeed");
        assert!(output.contains("name"));
        assert!(output.contains("test"));
    }

    #[test]
    fn test_format_for_terminal_nested_object() {
        let data = json!({"outer": {"inner": "value"}});
        let output = format_for_terminal(&data, 0);
        assert!(output.contains("outer"));
        assert!(output.contains("inner"));
        assert!(output.contains("value"));
    }

    #[test]
    fn test_format_for_terminal_with_indent() {
        let data = json!("hello");
        let output = format_for_terminal(&data, 2);
        assert!(output.starts_with("    ")); // 2 levels of indent
    }
}
