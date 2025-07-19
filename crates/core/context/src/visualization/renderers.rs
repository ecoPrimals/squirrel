//! Visualization Renderers
//!
//! This module provides different renderers for various output formats (JSON, HTML, Terminal, Markdown).

use crate::error::Result;
use serde_json::Value;

/// JSON renderer for visualization data
#[derive(Debug)]
pub struct JsonRenderer;

impl JsonRenderer {
    /// Create a new JSON renderer
    pub fn new() -> Self {
        Self
    }

    /// Render data as JSON
    pub async fn render(&self, data: &Value) -> Result<String> {
        serde_json::to_string_pretty(data)
            .map_err(|e| crate::error::ContextError::SerializationError(e.to_string()))
    }
}

/// Terminal renderer for visualization data
#[derive(Debug)]
pub struct TerminalRenderer;

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

/// HTML renderer for visualization data
#[derive(Debug)]
pub struct HtmlRenderer;

impl HtmlRenderer {
    /// Create a new HTML renderer
    pub fn new() -> Self {
        Self
    }

    /// Render data as HTML
    pub async fn render(&self, data: &Value) -> Result<String> {
        let html = format_as_html(data);
        Ok(html)
    }
}

/// Markdown renderer for visualization data
#[derive(Debug)]
pub struct MarkdownRenderer;

impl MarkdownRenderer {
    /// Create a new Markdown renderer
    pub fn new() -> Self {
        Self
    }

    /// Render data as Markdown
    pub async fn render(&self, data: &Value) -> Result<String> {
        let markdown = format_as_markdown(data);
        Ok(markdown)
    }
}

// Helper functions for terminal formatting
fn format_for_terminal(value: &Value, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);

    match value {
        Value::Null => format!("{}null", indent_str),
        Value::Bool(b) => format!("{}{}", indent_str, b),
        Value::Number(n) => format!("{}{}", indent_str, n),
        Value::String(s) => format!("{}\"{}\"", indent_str, s),
        Value::Array(arr) => {
            let mut result = format!("{}[\n", indent_str);
            for (i, item) in arr.iter().enumerate() {
                result.push_str(&format_for_terminal(item, indent + 1));
                if i < arr.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&format!("{}]", indent_str));
            result
        }
        Value::Object(obj) => {
            let mut result = format!("{}{{\n", indent_str);
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
            result.push_str(&format!("{}}}", indent_str));
            result
        }
    }
}

// Helper functions for HTML formatting
fn format_as_html(value: &Value) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<title>Visualization</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
    html.push_str("pre { background-color: #f5f5f5; padding: 10px; border-radius: 5px; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("<h1>Visualization Data</h1>\n");
    html.push_str("<pre>");
    html.push_str(
        &serde_json::to_string_pretty(value)
            .unwrap_or_else(|_| "Error formatting data".to_string()),
    );
    html.push_str("</pre>\n");
    html.push_str("</body>\n</html>");
    html
}

// Helper functions for Markdown formatting
fn format_as_markdown(value: &Value) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Visualization Data\n\n");
    markdown.push_str("```json\n");
    markdown.push_str(
        &serde_json::to_string_pretty(value)
            .unwrap_or_else(|_| "Error formatting data".to_string()),
    );
    markdown.push_str("\n```\n");
    markdown
}
