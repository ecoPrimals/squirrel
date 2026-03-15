// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use crate::error::context_err::ContextError;
use crate::error::{MCPError, Result};
use super::types::{Context, ContextValidation};
use super::helpers::{rule_validator, is_none_or_matches};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Validation logic for contexts
pub struct ValidationEngine {
    validations: Arc<RwLock<HashMap<String, ContextValidation>>>,
}

impl ValidationEngine {
    pub fn new(validations: Arc<RwLock<HashMap<String, ContextValidation>>>) -> Self {
        Self { validations }
    }

    /// Validates a context against registered validation rules
    ///
    /// # Errors
    ///
    /// Returns `ContextError::ValidationError` if the context fails validation.
    pub async fn validate_context(&self, context: &Context) -> Result<()> {
        // Check expiration
        if let Some(expires_at) = context.expires_at {
            if expires_at < Utc::now() {
                return Err(MCPError::Context(
                    ContextError::from("Context has expired")
                ));
            }
        }

        // Get context type from metadata
        let context_type = if let Some(metadata) = &context.metadata {
            metadata
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("default")
        } else {
            "default"
        };

        // Apply registered validations
        let validations = self.validations.read().await;
        if let Some(validation) = validations.get(context_type) {
            // Validate against schema
            if !validation.schema.is_null() {
                Self::validate_json_schema(&validation.schema, &context.data).await?;
            }

            // Apply each validation rule
            for rule in &validation.rules {
                Self::apply_validation_rule(context, rule).await.map_err(|e| e)?;
            }
        }

        Ok(())
    }

    /// Applies a specific validation rule to a context
    ///
    /// # Errors
    ///
    /// Returns `ContextError::ValidationError` if the context fails the rule.
    async fn apply_validation_rule(context: &Context, rule: &str) -> Result<()> {
        // Basic validation for all contexts
        if context.data.is_null() {
            return Err(MCPError::Context(
                ContextError::from("Context data cannot be empty")
            ));
        }

        // Required fields validation
        if rule.starts_with("required:") {
            let field_name = rule.strip_prefix("required:").unwrap_or("");
            if !field_name.is_empty() && context.data.get(field_name).is_none() {
                return Err(MCPError::Context(ContextError::ValidationError(
                    format!("Required field '{field_name}' is missing")
                )));
            }
            return Ok(());
        }

        // Check if the rule function exists and apply it
        if !rule_validator(rule, context) {
            return Err(MCPError::Context(ContextError::ValidationError(
                format!("Rule '{}' failed for context '{}'", rule, context.name)
            )));
        }

        Ok(())
    }

    /// Validate JSON schema for context data
    async fn validate_json_schema(schema: &serde_json::Value, data: &serde_json::Value) -> Result<()> {
        let schema_obj = schema.as_object().ok_or_else(|| {
            MCPError::Context(ContextError::from("Schema must be an object"))
        })?;

        // Validate type
        if let Some(type_value) = schema_obj.get("type") {
            if let Some(expected_type) = type_value.as_str() {
                let actual_type = match data {
                    serde_json::Value::Null => "null",
                    serde_json::Value::Bool(_) => "boolean",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::Object(_) => "object",
                };

                if expected_type != actual_type {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Expected type '{}', got '{}'", expected_type, actual_type
                    ))));
                }
            }
        }

        // Validate object properties
        if let Some(properties) = schema_obj.get("properties") {
            if let Some(data_obj) = data.as_object() {
                if let Some(properties_obj) = properties.as_object() {
                    for (property_name, property_schema) in properties_obj {
                        if let Some(property_value) = data_obj.get(property_name) {
                            // Recursively validate nested properties
                            Self::validate_json_schema(property_schema, property_value).await?;
                        }
                    }
                }
            }
        }

        // Validate required fields
        if let Some(required) = schema_obj.get("required") {
            if let Some(required_array) = required.as_array() {
                if let Some(data_obj) = data.as_object() {
                    for req_field in required_array {
                        if let Some(field_name) = req_field.as_str() {
                            if !data_obj.contains_key(field_name) {
                                return Err(MCPError::Context(ContextError::from(format!(
                                    "Missing required field: {}", field_name
                                ))));
                            }
                        }
                    }
                }
            }
        }

        // Validate string constraints
        if let Some(data_str) = data.as_str() {
            if let Some(min_length) = schema_obj.get("minLength").and_then(|v| v.as_u64()) {
                if data_str.len() < min_length as usize {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "String length {} is less than minimum {}", data_str.len(), min_length
                    ))));
                }
            }
            if let Some(max_length) = schema_obj.get("maxLength").and_then(|v| v.as_u64()) {
                if data_str.len() > max_length as usize {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "String length {} exceeds maximum {}", data_str.len(), max_length
                    ))));
                }
            }
        }

        // Validate numeric constraints
        if let Some(data_num) = data.as_f64() {
            if let Some(minimum) = schema_obj.get("minimum").and_then(|v| v.as_f64()) {
                if data_num < minimum {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Value {} is less than minimum {}", data_num, minimum
                    ))));
                }
            }
            if let Some(maximum) = schema_obj.get("maximum").and_then(|v| v.as_f64()) {
                if data_num > maximum {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Value {} exceeds maximum {}", data_num, maximum
                    ))));
                }
            }
        }

        // Validate array constraints
        if let Some(data_array) = data.as_array() {
            if let Some(min_items) = schema_obj.get("minItems").and_then(|v| v.as_u64()) {
                if data_array.len() < min_items as usize {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Array length {} is less than minimum {}", data_array.len(), min_items
                    ))));
                }
            }
            if let Some(max_items) = schema_obj.get("maxItems").and_then(|v| v.as_u64()) {
                if data_array.len() > max_items as usize {
                    return Err(MCPError::Context(ContextError::from(format!(
                        "Array length {} exceeds maximum {}", data_array.len(), max_items
                    ))));
                }
            }
            
            // Validate array items
            if let Some(items_schema) = schema_obj.get("items") {
                for item in data_array {
                    Self::validate_json_schema(items_schema, item).await?;
                }
            }
        }

        Ok(())
    }
}
