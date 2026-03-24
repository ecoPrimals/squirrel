// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Validation errors and helper functions for the Squirrel Plugin SDK

use super::core::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Validation error for plugin configuration and parameters
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    /// Field is required but missing
    #[error("Required field '{field}' is missing")]
    RequiredField {
        /// The name of the required field that is missing
        field: String,
    },

    /// Invalid field value
    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue {
        /// The field name that has an invalid value
        field: String,
        /// The reason why the value is invalid
        reason: String,
    },

    /// Invalid format
    #[error("Invalid format for field '{field}': expected {expected}, got {actual}")]
    InvalidFormat {
        /// The field name that has an invalid format
        field: String,
        /// The expected format for the field
        expected: String,
        /// The actual format that was received
        actual: String,
    },

    /// Value out of range
    #[error("Value for field '{field}' out of range: {value} (expected {min}-{max})")]
    OutOfRange {
        /// The field name that has a value out of range
        field: String,
        /// The value that is out of range
        value: String,
        /// The minimum allowed value
        min: String,
        /// The maximum allowed value
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
        .cloned()
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
        .cloned()
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

#[cfg(test)]
mod tests {
    #![allow(deprecated)]
    use super::*;

    fn sample_params() -> serde_json::Value {
        serde_json::json!({
            "name": "test",
            "count": 42,
            "enabled": true,
            "items": [1, 2, 3],
            "config": {"key": "value"},
            "url": "https://example.com",
            "email": "user@example.com",
            "nullable": null
        })
    }

    #[test]
    fn test_validation_error_required_field() {
        let err = ValidationError::RequiredField {
            field: "name".into(),
        };
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn test_validation_error_invalid_value() {
        let err = ValidationError::InvalidValue {
            field: "age".into(),
            reason: "must be positive".into(),
        };
        assert!(err.to_string().contains("age"));
        assert!(err.to_string().contains("must be positive"));
    }

    #[test]
    fn test_validation_error_invalid_format() {
        let err = ValidationError::InvalidFormat {
            field: "date".into(),
            expected: "YYYY-MM-DD".into(),
            actual: "01/01/2026".into(),
        };
        assert!(err.to_string().contains("date"));
        assert!(err.to_string().contains("YYYY-MM-DD"));
    }

    #[test]
    fn test_validation_error_out_of_range() {
        let err = ValidationError::OutOfRange {
            field: "age".into(),
            value: "150".into(),
            min: "0".into(),
            max: "120".into(),
        };
        assert!(err.to_string().contains("150"));
        assert!(err.to_string().contains('0'));
        assert!(err.to_string().contains("120"));
    }

    #[test]
    fn test_validation_error_serde() {
        let err = ValidationError::RequiredField {
            field: "name".into(),
        };
        let json = serde_json::to_string(&err).expect("should succeed");
        let deserialized: ValidationError = serde_json::from_str(&json).expect("should succeed");
        assert!(deserialized.to_string().contains("name"));
    }

    #[test]
    fn test_validation_error_to_plugin_error_required_field() {
        let ve = ValidationError::RequiredField {
            field: "name".into(),
        };
        let pe: PluginError = ve.into();
        match pe {
            PluginError::MissingParameter { parameter } => assert_eq!(parameter, "name"),
            _ => unreachable!("Expected MissingParameter"),
        }
    }

    #[test]
    fn test_validation_error_to_plugin_error_invalid_value() {
        let ve = ValidationError::InvalidValue {
            field: "age".into(),
            reason: "negative".into(),
        };
        let pe: PluginError = ve.into();
        match pe {
            PluginError::InvalidParameter { name, reason } => {
                assert_eq!(name, "age");
                assert_eq!(reason, "negative");
            }
            _ => unreachable!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_validation_error_to_plugin_error_invalid_format() {
        let ve = ValidationError::InvalidFormat {
            field: "date".into(),
            expected: "ISO".into(),
            actual: "US".into(),
        };
        let pe: PluginError = ve.into();
        match pe {
            PluginError::InvalidParameter { name, reason } => {
                assert_eq!(name, "date");
                assert!(reason.contains("expected ISO"));
                assert!(reason.contains("got US"));
            }
            _ => unreachable!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_validation_error_to_plugin_error_out_of_range() {
        let ve = ValidationError::OutOfRange {
            field: "x".into(),
            value: "200".into(),
            min: "0".into(),
            max: "100".into(),
        };
        let pe: PluginError = ve.into();
        match pe {
            PluginError::InvalidParameter { name, reason } => {
                assert_eq!(name, "x");
                assert!(reason.contains("200"));
                assert!(
                    reason.contains("0-100") || (reason.contains('0') && reason.contains("100"))
                );
            }
            _ => unreachable!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_validate_required_string_ok() {
        let params = sample_params();
        let result = validate_required_string(&params, "name");
        assert_eq!(result.expect("should succeed"), "test");
    }

    #[test]
    fn test_validate_required_string_missing() {
        let params = sample_params();
        assert!(validate_required_string(&params, "nonexistent").is_err());
    }

    #[test]
    fn test_validate_optional_string_present() {
        let params = sample_params();
        let result = validate_optional_string(&params, "name");
        assert_eq!(result.expect("should succeed"), Some("test".to_string()));
    }

    #[test]
    fn test_validate_optional_string_missing() {
        let params = sample_params();
        let result = validate_optional_string(&params, "nonexistent");
        assert_eq!(result.expect("should succeed"), None);
    }

    #[test]
    fn test_validate_optional_string_null() {
        let params = sample_params();
        let result = validate_optional_string(&params, "nullable");
        assert_eq!(result.expect("should succeed"), None);
    }

    #[test]
    fn test_validate_optional_string_wrong_type() {
        let params = sample_params();
        assert!(validate_optional_string(&params, "count").is_err());
    }

    #[test]
    fn test_validate_required_number_ok() {
        let params = sample_params();
        let result = validate_required_number(&params, "count").expect("should succeed");
        assert!((result - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_validate_required_number_missing() {
        let params = sample_params();
        assert!(validate_required_number(&params, "nonexistent").is_err());
    }

    #[test]
    fn test_validate_boolean_present() {
        let params = sample_params();
        assert!(validate_boolean(&params, "enabled", false).expect("should succeed"));
    }

    #[test]
    fn test_validate_boolean_missing_uses_default() {
        let params = sample_params();
        assert!(validate_boolean(&params, "nonexistent", true).expect("should succeed"));
        assert!(!validate_boolean(&params, "nonexistent", false).expect("should succeed"));
    }

    #[test]
    fn test_validate_array_ok() {
        let params = sample_params();
        let arr = validate_array(&params, "items").expect("should succeed");
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_validate_array_missing() {
        let params = sample_params();
        assert!(validate_array(&params, "nonexistent").is_err());
    }

    #[test]
    fn test_validate_object_ok() {
        let params = sample_params();
        let obj = validate_object(&params, "config").expect("should succeed");
        assert_eq!(obj.len(), 1);
        assert_eq!(obj["key"], serde_json::json!("value"));
    }

    #[test]
    fn test_validate_object_missing() {
        let params = sample_params();
        assert!(validate_object(&params, "nonexistent").is_err());
    }

    #[test]
    fn test_validate_string_length_ok() {
        assert!(validate_string_length("hello", "field", 1, 10).is_ok());
    }

    #[test]
    fn test_validate_string_length_too_short() {
        assert!(validate_string_length("", "field", 1, 10).is_err());
    }

    #[test]
    fn test_validate_string_length_too_long() {
        assert!(validate_string_length("hello world!", "field", 1, 5).is_err());
    }

    #[test]
    fn test_validate_numeric_range_ok() {
        assert!(validate_numeric_range(5.0, "field", 0.0, 10.0).is_ok());
    }

    #[test]
    fn test_validate_numeric_range_out() {
        assert!(validate_numeric_range(15.0, "field", 0.0, 10.0).is_err());
        assert!(validate_numeric_range(-1.0, "field", 0.0, 10.0).is_err());
    }

    #[test]
    fn test_validate_integer_range_ok() {
        assert!(validate_integer_range(5, "field", 0, 10).is_ok());
    }

    #[test]
    fn test_validate_integer_range_out() {
        assert!(validate_integer_range(15, "field", 0, 10).is_err());
        assert!(validate_integer_range(-1, "field", 0, 10).is_err());
    }

    #[test]
    fn test_validate_enum_value_ok() {
        let result = validate_enum_value("red", "color", &["red", "green", "blue"]);
        assert_eq!(result.expect("should succeed"), "red");
    }

    #[test]
    fn test_validate_enum_value_invalid() {
        let result = validate_enum_value("purple", "color", &["red", "green", "blue"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_url_ok() {
        assert!(validate_url("https://example.com", "url").is_ok());
        assert!(validate_url("http://localhost:3000", "url").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("ftp://example.com", "url").is_err());
        assert!(validate_url("not-a-url", "url").is_err());
    }

    #[test]
    fn test_validate_email_ok() {
        assert!(validate_email("user@example.com", "email").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("no-at-sign", "email").is_err());
        assert!(validate_email("a@b", "email").is_err()); // too short (len <= 3)
    }

    #[test]
    fn test_validate_non_empty_string_ok() {
        assert_eq!(
            validate_non_empty_string("hello", "field").expect("should succeed"),
            "hello"
        );
    }

    #[test]
    fn test_validate_non_empty_string_empty() {
        assert!(validate_non_empty_string("", "field").is_err());
    }

    #[test]
    fn test_validate_array_length_ok() {
        let items = vec![
            serde_json::json!(1),
            serde_json::json!(2),
            serde_json::json!(3),
        ];
        assert!(validate_array_length(&items, "arr", 1, 5).is_ok());
    }

    #[test]
    fn test_validate_array_length_too_short() {
        let items = vec![serde_json::json!(1)];
        assert!(validate_array_length(&items, "arr", 3, 10).is_err());
    }

    #[test]
    fn test_validate_array_length_too_long() {
        let items = vec![
            serde_json::json!(1),
            serde_json::json!(2),
            serde_json::json!(3),
        ];
        assert!(validate_array_length(&items, "arr", 1, 2).is_err());
    }

    #[test]
    fn test_validate_required_fields_ok() {
        let mut map = serde_json::Map::new();
        map.insert("name".to_string(), serde_json::json!("test"));
        map.insert("age".to_string(), serde_json::json!(25));
        assert!(validate_required_fields(&map, &["name", "age"]).is_ok());
    }

    #[test]
    fn test_validate_required_fields_missing() {
        let mut map = serde_json::Map::new();
        map.insert("name".to_string(), serde_json::json!("test"));
        assert!(validate_required_fields(&map, &["name", "age"]).is_err());
    }
}
