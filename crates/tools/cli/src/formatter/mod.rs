// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use colored::*;
use prettytable::{Cell, Row, Table};
use serde::Serialize;
use thiserror::Error;

/// Error type for formatter operations
#[derive(Debug, Error)]
pub enum FormatterError {
    /// Serialization error (JSON/YAML)
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Unknown formatter requested
    #[error("Unknown formatter: {0}")]
    UnknownFormat(String),
}

impl From<serde_json::Error> for FormatterError {
    fn from(err: serde_json::Error) -> Self {
        FormatterError::Serialization(err.to_string())
    }
}

impl From<serde_yml::Error> for FormatterError {
    fn from(err: serde_yml::Error) -> Self {
        FormatterError::Serialization(err.to_string())
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    /// Plain text output format
    Text,
    /// JSON output format
    Json,
    /// YAML output format
    Yaml,
    /// Table output format
    Table,
}

/// Concrete formatter type that wraps different formatter implementations
#[derive(Debug, Clone)]
pub enum Formatter {
    /// Text formatter implementation
    Text(TextFormatter),
    /// JSON formatter implementation
    Json(JsonFormatter),
    /// YAML formatter implementation
    Yaml(YamlFormatter),
}

/// Helper methods for Formatter
impl Formatter {
    /// Format data into a string
    pub fn format<T: Serialize + std::fmt::Debug>(
        &self,
        data: T,
    ) -> Result<String, FormatterError> {
        match self {
            Formatter::Text(f) => f.format(data),
            Formatter::Json(f) => f.format(data),
            Formatter::Yaml(f) => f.format(data),
        }
    }

    /// Format an error into a string
    pub fn format_error(&self, error: &dyn std::error::Error) -> String {
        match self {
            Formatter::Text(f) => f.format_error(error),
            Formatter::Json(f) => f.format_error(error),
            Formatter::Yaml(f) => f.format_error(error),
        }
    }

    /// Format data as a table
    pub fn format_table(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        match self {
            Formatter::Text(f) => f.format_table(headers, rows),
            Formatter::Json(f) => f.format_table(headers, rows),
            Formatter::Yaml(f) => f.format_table(headers, rows),
        }
    }
}

/// Text output formatter
#[derive(Debug, Clone)]
pub struct TextFormatter;

impl TextFormatter {
    /// Create a new text formatter
    pub fn new() -> Self {
        Self
    }

    /// Format data into a string
    pub fn format<T: Serialize + std::fmt::Debug>(
        &self,
        data: T,
    ) -> Result<String, FormatterError> {
        Ok(format!("{:#?}", data))
    }

    /// Format an error into a string
    pub fn format_error(&self, error: &dyn std::error::Error) -> String {
        format!("Error: {}", error.to_string().red())
    }

    /// Format data as a table
    pub fn format_table(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        let mut table = Table::new();
        table.add_row(Row::from_iter(headers.iter().map(|&h| Cell::new(h))));

        for row in rows {
            table.add_row(Row::from_iter(row.iter().map(|cell| Cell::new(cell))));
        }

        table.to_string()
    }
}

// TextFormatter Default implementation
impl Default for TextFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON output formatter
#[derive(Debug, Clone)]
pub struct JsonFormatter;

impl JsonFormatter {
    /// Create a new JSON formatter
    pub fn new() -> Self {
        Self
    }

    /// Format data into a string
    pub fn format<T: Serialize>(&self, data: T) -> Result<String, FormatterError> {
        Ok(serde_json::to_string_pretty(&data)?)
    }

    /// Format an error into a string
    pub fn format_error(&self, error: &dyn std::error::Error) -> String {
        serde_json::json!({
            "error": {
                "message": error.to_string(),
                "source": error.source().map(|e| e.to_string()),
            }
        })
        .to_string()
    }

    /// Format data as a table
    pub fn format_table(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        let mut table_data = Vec::new();

        for row in rows {
            let mut row_data = serde_json::Map::new();
            for (i, header) in headers.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    row_data.insert(header.to_string(), serde_json::Value::String(value.clone()));
                }
            }
            table_data.push(row_data);
        }

        serde_json::to_string_pretty(&table_data).unwrap_or_default()
    }
}

// JsonFormatter Default implementation
impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// YAML output formatter
#[derive(Debug, Clone)]
pub struct YamlFormatter;

impl YamlFormatter {
    /// Create a new YAML formatter
    pub fn new() -> Self {
        Self
    }

    /// Format data into a string
    pub fn format<T: Serialize>(&self, data: T) -> Result<String, FormatterError> {
        Ok(serde_yml::to_string(&data)?)
    }

    /// Format an error into a string
    pub fn format_error(&self, error: &dyn std::error::Error) -> String {
        serde_yml::to_string(&serde_json::json!({
            "error": {
                "message": error.to_string(),
                "source": error.source().map(|e| e.to_string()),
            }
        }))
        .unwrap_or_else(|_| format!("Error: {}", error))
    }

    /// Format data as a table
    pub fn format_table(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        let mut table_data = Vec::new();

        for row in rows {
            let mut row_data = serde_yml::Mapping::new();
            for (i, header) in headers.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    row_data.insert(
                        serde_yml::Value::String(header.to_string()),
                        serde_yml::Value::String(value.clone()),
                    );
                }
            }
            table_data.push(serde_yml::Value::Mapping(row_data));
        }

        serde_yml::to_string(&table_data).unwrap_or_default()
    }
}

// YamlFormatter Default implementation
impl Default for YamlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating formatters
#[derive(Debug)]
pub struct Factory;

impl Factory {
    /// Create a formatter from an OutputFormat enum
    pub fn create(format: OutputFormat) -> Formatter {
        match format {
            OutputFormat::Text => Formatter::Text(TextFormatter::new()),
            OutputFormat::Json => Formatter::Json(JsonFormatter::new()),
            OutputFormat::Yaml => Formatter::Yaml(YamlFormatter::new()),
            OutputFormat::Table => Formatter::Text(TextFormatter::new()), // Uses text formatter's table implementation
        }
    }

    /// Create a formatter from a string format name
    pub fn create_formatter(format: &str) -> Result<Formatter, FormatterError> {
        match format.to_lowercase().as_str() {
            "text" => Ok(Formatter::Text(TextFormatter::new())),
            "json" => Ok(Formatter::Json(JsonFormatter::new())),
            "yaml" | "yml" => Ok(Formatter::Yaml(YamlFormatter::new())),
            "table" => Ok(Formatter::Text(TextFormatter::new())), // Uses text formatter's table implementation
            _ => Err(FormatterError::UnknownFormat(format.to_string())),
        }
    }
}

/// Backward compatibility alias for FormatterFactory
pub type FormatterFactory = Factory;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize, Debug)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_text_formatter() {
        let formatter = Formatter::Text(TextFormatter::new());
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let result = formatter.format(data).unwrap();
        assert!(result.contains("test"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_json_formatter() {
        let formatter = Formatter::Json(JsonFormatter::new());
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let result = formatter.format(data).unwrap();
        assert!(result.contains("\"name\": \"test\""));
        assert!(result.contains("\"value\": 42"));
    }

    #[test]
    fn test_yaml_formatter() {
        let formatter = Formatter::Yaml(YamlFormatter::new());
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let result = formatter.format(data).unwrap();
        assert!(result.contains("name: test"));
        assert!(result.contains("value: 42"));
    }

    #[test]
    fn test_table_formatting() {
        let formatter = Formatter::Text(TextFormatter::new());
        let headers = &["Name", "Value"];
        let rows = &[vec!["test".to_string(), "42".to_string()]];

        let result = formatter.format_table(headers, rows);
        assert!(result.contains("Name"));
        assert!(result.contains("Value"));
        assert!(result.contains("test"));
        assert!(result.contains("42"));
    }
}
