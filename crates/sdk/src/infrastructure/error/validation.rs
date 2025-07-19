//! Validation errors and helper functions for the Squirrel Plugin SDK

use super::core::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Validation error for plugin configuration and parameters
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    /// Field is required but missing
    #[error("Required field '{field}' is missing")]
    RequiredField { field: String },

    /// Invalid field value
    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue { field: String, reason: String },

    /// Invalid format
    #[error("Invalid format for field '{field}': expected {expected}, got {actual}")]
    InvalidFormat {
        field: String,
        expected: String,
        actual: String,
    },

    /// Value out of range
    #[error("Value for field '{field}' out of range: {value} (expected {min}-{max})")]
    OutOfRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },
}

impl From<ValidationError> for PluginError {
    fn from(error: ValidationError) -> Self {
        match error {
            ValidationError::RequiredField { field } => {
                PluginError::MissingParameter { parameter: field }
            }
            ValidationError::InvalidValue { field, reason } => PluginError::InvalidParameter {
                name: field,
                reason,
            },
            ValidationError::InvalidFormat {
                field,
                expected,
                actual,
            } => PluginError::InvalidParameter {
                name: field,
                reason: format!("expected {}, got {}", expected, actual),
            },
            ValidationError::OutOfRange {
                field,
                value,
                min,
                max,
            } => PluginError::InvalidParameter {
                name: field,
                reason: format!("value {} out of range ({}-{})", value, min, max),
            },
        }
    }
}

/// Helper function to validate required string parameter
pub fn validate_required_string(params: &serde_json::Value, field: &str) -> PluginResult<String> {
    params
        .get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| crate::missing_param!(field))
}

/// Helper function to validate optional string parameter
pub fn validate_optional_string(
    params: &serde_json::Value,
    field: &str,
) -> PluginResult<Option<String>> {
    match params.get(field) {
        Some(v) => {
            if v.is_null() {
                Ok(None)
            } else {
                v.as_str()
                    .map(|s| Some(s.to_string()))
                    .ok_or_else(|| crate::param_error!(field, "expected string or null"))
            }
        }
        None => Ok(None),
    }
}

/// Helper function to validate required number parameter
pub fn validate_required_number(params: &serde_json::Value, field: &str) -> PluginResult<f64> {
    params
        .get(field)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| crate::missing_param!(field))
}

/// Helper function to validate boolean parameter
pub fn validate_boolean(
    params: &serde_json::Value,
    field: &str,
    default: bool,
) -> PluginResult<bool> {
    Ok(params
        .get(field)
        .and_then(|v| v.as_bool())
        .unwrap_or(default))
}

/// Helper function to validate array parameter
pub fn validate_array(
    params: &serde_json::Value,
    field: &str,
) -> PluginResult<Vec<serde_json::Value>> {
    params
        .get(field)
        .and_then(|v| v.as_array())
        .map(|arr| arr.clone())
        .ok_or_else(|| crate::param_error!(field, "expected array"))
}

/// Helper function to validate object parameter
pub fn validate_object(
    params: &serde_json::Value,
    field: &str,
) -> PluginResult<serde_json::Map<String, serde_json::Value>> {
    params
        .get(field)
        .and_then(|v| v.as_object())
        .map(|obj| obj.clone())
        .ok_or_else(|| crate::param_error!(field, "expected object"))
}

/// Helper function to validate string length
pub fn validate_string_length(
    value: &str,
    field: &str,
    min: usize,
    max: usize,
) -> PluginResult<()> {
    let len = value.len();
    if len < min || len > max {
        return Err(crate::param_error!(
            field,
            format!("length {} not in range {}-{}", len, min, max)
        ));
    }
    Ok(())
}

/// Helper function to validate numeric range
pub fn validate_numeric_range(value: f64, field: &str, min: f64, max: f64) -> PluginResult<()> {
    if value < min || value > max {
        return Err(crate::param_error!(
            field,
            format!("value {} not in range {}-{}", value, min, max)
        ));
    }
    Ok(())
}

/// Helper function to validate integer range
pub fn validate_integer_range(value: i64, field: &str, min: i64, max: i64) -> PluginResult<()> {
    if value < min || value > max {
        return Err(crate::param_error!(
            field,
            format!("value {} not in range {}-{}", value, min, max)
        ));
    }
    Ok(())
}

/// Helper function to validate enum value
pub fn validate_enum_value<T: AsRef<str>>(
    value: &str,
    field: &str,
    valid_values: &[T],
) -> PluginResult<String> {
    if valid_values.iter().any(|v| v.as_ref() == value) {
        Ok(value.to_string())
    } else {
        let valid_list = valid_values
            .iter()
            .map(|v| v.as_ref())
            .collect::<Vec<_>>()
            .join(", ");
        Err(crate::param_error!(
            field,
            format!("invalid value '{}', valid values: {}", value, valid_list)
        ))
    }
}

/// Helper function to validate URL format
pub fn validate_url(value: &str, field: &str) -> PluginResult<String> {
    if value.starts_with("http://") || value.starts_with("https://") {
        Ok(value.to_string())
    } else {
        Err(crate::param_error!(
            field,
            "must be a valid URL starting with http:// or https://"
        ))
    }
}

/// Helper function to validate email format
pub fn validate_email(value: &str, field: &str) -> PluginResult<String> {
    if value.contains('@') && value.len() > 3 {
        Ok(value.to_string())
    } else {
        Err(crate::param_error!(field, "must be a valid email address"))
    }
}

/// Helper function to validate non-empty string
pub fn validate_non_empty_string(value: &str, field: &str) -> PluginResult<String> {
    if value.is_empty() {
        Err(crate::param_error!(field, "cannot be empty"))
    } else {
        Ok(value.to_string())
    }
}

/// Helper function to validate array length
pub fn validate_array_length(
    arr: &[serde_json::Value],
    field: &str,
    min: usize,
    max: usize,
) -> PluginResult<()> {
    let len = arr.len();
    if len < min || len > max {
        return Err(crate::param_error!(
            field,
            format!("array length {} not in range {}-{}", len, min, max)
        ));
    }
    Ok(())
}

/// Helper function to validate required fields in object
pub fn validate_required_fields(
    obj: &serde_json::Map<String, serde_json::Value>,
    required_fields: &[&str],
) -> PluginResult<()> {
    for field in required_fields {
        if !obj.contains_key(*field) {
            return Err(crate::missing_param!(field));
        }
    }
    Ok(())
}
