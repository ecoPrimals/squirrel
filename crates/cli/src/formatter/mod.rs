use std::error::Error;
use serde::Serialize;
use colored::*;
use prettytable::{Table, Row, Cell};

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
    Table,
}

/// Concrete formatter type that wraps different formatter implementations
#[derive(Debug, Clone)]
pub enum Formatter {
    Text(TextFormatter),
    Json(JsonFormatter),
    Yaml(YamlFormatter),
}

/// Helper methods for Formatter
impl Formatter {
    /// Format data into a string
    pub fn format<T: Serialize + std::fmt::Debug>(&self, data: T) -> Result<String, Box<dyn Error>> {
        match self {
            Formatter::Text(f) => f.format(data),
            Formatter::Json(f) => f.format(data),
            Formatter::Yaml(f) => f.format(data),
        }
    }
    
    /// Format an error into a string
    pub fn format_error(&self, error: &dyn Error) -> String {
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
    pub fn new() -> Self {
        Self
    }
    
    /// Format data into a string
    pub fn format<T: Serialize + std::fmt::Debug>(&self, data: T) -> Result<String, Box<dyn Error>> {
        Ok(format!("{:#?}", data))
    }

    /// Format an error into a string
    pub fn format_error(&self, error: &dyn Error) -> String {
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

/// JSON output formatter
#[derive(Debug, Clone)]
pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }
    
    /// Format data into a string
    pub fn format<T: Serialize>(&self, data: T) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string_pretty(&data)?)
    }

    /// Format an error into a string
    pub fn format_error(&self, error: &dyn Error) -> String {
        serde_json::json!({
            "error": {
                "message": error.to_string(),
                "source": error.source().map(|e| e.to_string()),
            }
        }).to_string()
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

/// YAML output formatter
#[derive(Debug, Clone)]
pub struct YamlFormatter;

impl YamlFormatter {
    pub fn new() -> Self {
        Self
    }
    
    /// Format data into a string
    pub fn format<T: Serialize>(&self, data: T) -> Result<String, Box<dyn Error>> {
        Ok(serde_yaml::to_string(&data)?)
    }

    /// Format an error into a string
    pub fn format_error(&self, error: &dyn Error) -> String {
        serde_yaml::to_string(&serde_json::json!({
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
            let mut row_data = serde_yaml::Mapping::new();
            for (i, header) in headers.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    row_data.insert(
                        serde_yaml::Value::String(header.to_string()),
                        serde_yaml::Value::String(value.clone()),
                    );
                }
            }
            table_data.push(serde_yaml::Value::Mapping(row_data));
        }
        
        serde_yaml::to_string(&table_data).unwrap_or_default()
    }
}

/// Factory for creating formatters
pub struct FormatterFactory;

impl FormatterFactory {
    pub fn create(format: OutputFormat) -> Formatter {
        match format {
            OutputFormat::Text => Formatter::Text(TextFormatter::new()),
            OutputFormat::Json => Formatter::Json(JsonFormatter::new()),
            OutputFormat::Yaml => Formatter::Yaml(YamlFormatter::new()),
            OutputFormat::Table => Formatter::Text(TextFormatter::new()), // Uses text formatter's table implementation
        }
    }
}

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