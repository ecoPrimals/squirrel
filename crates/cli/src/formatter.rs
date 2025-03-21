//! Output formatter for CLI commands
//!
//! This module provides functionality for formatting command output in different formats
//! such as plain text, JSON, and YAML. It is used by CLI commands to provide consistent
//! output formatting.

use serde::Serialize;
use thiserror::Error;

/// Errors related to output formatting
#[derive(Debug, Error)]
pub enum FormatterError {
    /// Error serializing to JSON
    #[error("JSON serialization error: {0}")]
    JsonError(String),
    
    /// Error serializing to YAML
    #[error("YAML serialization error: {0}")]
    YamlError(String),
    
    /// Error with text formatting
    #[error("Text formatting error: {0}")]
    TextError(String),
}

/// Result type for formatter operations
pub type FormatterResult<T> = Result<T, FormatterError>;

/// Available output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text format
    Text,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

impl OutputFormat {
    /// Parse an output format from a string
    ///
    /// # Arguments
    ///
    /// * `format` - The format string to parse
    ///
    /// # Returns
    ///
    /// The parsed output format, or None if the format is not recognized
    pub fn from_format_str(format: &str) -> Option<Self> {
        match format.to_lowercase().as_str() {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            "yaml" => Some(Self::Yaml),
            _ => None,
        }
    }
}

/// OutputFormatter handles formatting output in different formats
#[derive(Debug, Clone)]
pub struct OutputFormatter {
    /// The current output format
    format: OutputFormat,
}

impl OutputFormatter {
    /// Create a new output formatter with the specified format
    ///
    /// # Arguments
    ///
    /// * `format` - The output format to use
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }
    
    /// Create a new output formatter from a format string
    ///
    /// # Arguments
    ///
    /// * `format` - Format string ("text", "json", or "yaml")
    ///
    /// # Returns
    ///
    /// A Result containing the formatter or an error if the format is not recognized
    pub fn from_format_str(format: &str) -> FormatterResult<Self> {
        match OutputFormat::from_format_str(format) {
            Some(format) => Ok(Self { format }),
            None => Err(FormatterError::TextError(format!("Unknown output format: {}", format))),
        }
    }
    
    /// Create a new output formatter with plain text format
    pub fn text() -> Self {
        Self::new(OutputFormat::Text)
    }
    
    /// Create a new output formatter with JSON format
    pub fn json() -> Self {
        Self::new(OutputFormat::Json)
    }
    
    /// Create a new output formatter with YAML format
    pub fn yaml() -> Self {
        Self::new(OutputFormat::Yaml)
    }
    
    /// Change the output format
    ///
    /// # Arguments
    ///
    /// * `format` - The new output format to use
    pub fn set_format(&mut self, format: OutputFormat) {
        self.format = format;
    }
    
    /// Get the current output format
    pub fn format(&self) -> OutputFormat {
        self.format
    }
    
    /// Format a serializable value according to the current output format
    ///
    /// # Arguments
    ///
    /// * `value` - The value to format
    ///
    /// # Returns
    ///
    /// A string containing the formatted output
    pub fn format_value<T: Serialize>(&self, value: &T) -> FormatterResult<String> {
        match self.format {
            OutputFormat::Text => self.format_text(value),
            OutputFormat::Json => self.format_json(value),
            OutputFormat::Yaml => self.format_yaml(value),
        }
    }
    
    /// Format a string table
    ///
    /// # Arguments
    ///
    /// * `headers` - The table headers
    /// * `rows` - The table rows
    ///
    /// # Returns
    ///
    /// A string containing the formatted table
    pub fn format_table(&self, headers: &[&str], rows: &[Vec<String>]) -> FormatterResult<String> {
        match self.format {
            OutputFormat::Text => {
                let mut result = String::new();
                let mut col_widths = vec![0; headers.len()];
                
                // Calculate column widths
                for (i, header) in headers.iter().enumerate() {
                    col_widths[i] = header.len();
                }
                
                for row in rows {
                    for (i, cell) in row.iter().enumerate() {
                        if i < col_widths.len() {
                            col_widths[i] = col_widths[i].max(cell.len());
                        }
                    }
                }
                
                // Add headers
                for (i, header) in headers.iter().enumerate() {
                    if i > 0 {
                        result.push_str(" | ");
                    }
                    result.push_str(&format!("{:width$}", header, width = col_widths[i]));
                }
                result.push('\n');
                
                // Add header separator
                for (i, width) in col_widths.iter().enumerate() {
                    if i > 0 {
                        result.push_str("-+-");
                    }
                    result.push_str(&"-".repeat(*width));
                }
                result.push('\n');
                
                // Add rows
                for row in rows {
                    for (i, cell) in row.iter().enumerate() {
                        if i > 0 {
                            result.push_str(" | ");
                        }
                        if i < col_widths.len() {
                            result.push_str(&format!("{:width$}", cell, width = col_widths[i]));
                        } else {
                            result.push_str(cell);
                        }
                    }
                    result.push('\n');
                }
                
                Ok(result)
            },
            OutputFormat::Json => {
                let mut table_data = Vec::new();
                for row in rows {
                    let mut row_data = std::collections::HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        if i < row.len() {
                            row_data.insert(*header, &row[i]);
                        }
                    }
                    table_data.push(row_data);
                }
                
                serde_json::to_string_pretty(&table_data)
                    .map_err(|e| FormatterError::JsonError(e.to_string()))
            },
            OutputFormat::Yaml => {
                let mut table_data = Vec::new();
                for row in rows {
                    let mut row_data = std::collections::HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        if i < row.len() {
                            row_data.insert(*header, &row[i]);
                        }
                    }
                    table_data.push(row_data);
                }
                
                serde_yaml::to_string(&table_data)
                    .map_err(|e| FormatterError::YamlError(e.to_string()))
            },
        }
    }
    
    /// Format structured data as JSON
    ///
    /// # Arguments
    ///
    /// * `value` - The value to format
    ///
    /// # Returns
    ///
    /// A string containing the formatted JSON
    fn format_json<T: Serialize>(&self, value: &T) -> FormatterResult<String> {
        serde_json::to_string_pretty(value)
            .map_err(|e| FormatterError::JsonError(e.to_string()))
    }
    
    /// Format structured data as YAML
    ///
    /// # Arguments
    ///
    /// * `value` - The value to format
    ///
    /// # Returns
    ///
    /// A string containing the formatted YAML
    fn format_yaml<T: Serialize>(&self, value: &T) -> FormatterResult<String> {
        serde_yaml::to_string(value)
            .map_err(|e| FormatterError::YamlError(e.to_string()))
    }
    
    /// Format structured data as plain text
    ///
    /// # Arguments
    ///
    /// * `value` - The value to format
    ///
    /// # Returns
    ///
    /// A string containing the formatted text
    fn format_text<T: Serialize>(&self, value: &T) -> FormatterResult<String> {
        // For simple values, try to convert directly to string
        match serde_json::to_value(value) {
            Ok(serde_json::Value::String(s)) => Ok(s),
            Ok(serde_json::Value::Number(n)) => Ok(n.to_string()),
            Ok(serde_json::Value::Bool(b)) => Ok(b.to_string()),
            Ok(serde_json::Value::Null) => Ok("null".to_string()),
            // For complex values, pretty-print the structure
            Ok(serde_json::Value::Object(obj)) => {
                let mut result = String::new();
                for (key, value) in obj {
                    result.push_str(&format!("{}: {}\n", key, value));
                }
                Ok(result)
            },
            Ok(serde_json::Value::Array(arr)) => {
                let mut result = String::new();
                for (i, value) in arr.iter().enumerate() {
                    result.push_str(&format!("{}. {}\n", i + 1, value));
                }
                Ok(result)
            },
            Err(e) => Err(FormatterError::TextError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_text_format() {
        let formatter = OutputFormatter::text();
        let value = json!({
            "name": "John Doe",
            "age": 30,
            "email": "john@example.com"
        });
        
        let result = formatter.format_value(&value).unwrap();
        assert!(result.contains("name: \"John Doe\""));
        assert!(result.contains("age: 30"));
        assert!(result.contains("email: \"john@example.com\""));
    }
    
    #[test]
    fn test_json_format() {
        let formatter = OutputFormatter::json();
        let value = json!({
            "name": "John Doe",
            "age": 30,
            "email": "john@example.com"
        });
        
        let result = formatter.format_value(&value).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["name"], "John Doe");
        assert_eq!(parsed["age"], 30);
        assert_eq!(parsed["email"], "john@example.com");
    }
    
    #[test]
    fn test_yaml_format() {
        let formatter = OutputFormatter::yaml();
        let value = json!({
            "name": "John Doe",
            "age": 30,
            "email": "john@example.com"
        });
        
        let result = formatter.format_value(&value).unwrap();
        assert!(result.contains("name: John Doe"));
        assert!(result.contains("age: 30"));
        assert!(result.contains("email: john@example.com"));
    }
    
    #[test]
    fn test_table_format() {
        let formatter = OutputFormatter::text();
        let headers = &["Name", "Age", "Email"];
        let rows = &[
            vec!["John Doe".to_string(), "30".to_string(), "john@example.com".to_string()],
            vec!["Jane Smith".to_string(), "25".to_string(), "jane@example.com".to_string()],
        ];
        
        let result = formatter.format_table(headers, rows).unwrap();
        assert!(result.contains("Name"));
        assert!(result.contains("Age"));
        assert!(result.contains("Email"));
        assert!(result.contains("John Doe"));
        assert!(result.contains("30"));
        assert!(result.contains("john@example.com"));
        assert!(result.contains("Jane Smith"));
        assert!(result.contains("25"));
        assert!(result.contains("jane@example.com"));
    }
    
    #[test]
    fn test_from_str() {
        let formatter = OutputFormatter::from_format_str("text").unwrap();
        assert_eq!(formatter.format(), OutputFormat::Text);
        
        let formatter = OutputFormatter::from_format_str("json").unwrap();
        assert_eq!(formatter.format(), OutputFormat::Json);
        
        let formatter = OutputFormatter::from_format_str("yaml").unwrap();
        assert_eq!(formatter.format(), OutputFormat::Yaml);
        
        assert!(OutputFormatter::from_format_str("invalid").is_err());
    }
} 