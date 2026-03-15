// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tool parameter definitions
//!
//! This module contains parameter types for tool capabilities.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt;

/// Tool capability parameter type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// String parameter
    String,
    /// Number parameter
    Number,
    /// Boolean parameter
    Boolean,
    /// Object parameter
    Object,
    /// Array parameter
    Array,
    /// Any type parameter
    Any,
}

impl fmt::Display for ParameterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Boolean => write!(f, "boolean"),
            Self::Object => write!(f, "object"),
            Self::Array => write!(f, "array"),
            Self::Any => write!(f, "any"),
        }
    }
}

/// Tool capability parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Parameter type
    pub parameter_type: ParameterType,
    /// Whether the parameter is required
    pub required: bool,
}

/// Tool capability return type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnType {
    /// Return type description
    pub description: String,
    /// Return type schema
    pub schema: JsonValue,
}

