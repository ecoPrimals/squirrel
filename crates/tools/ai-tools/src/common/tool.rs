//! Tools for AI chat interfaces
//!
//! This module defines the common tool types used across different AI providers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A tool that can be used by the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// The type of tool
    #[serde(rename = "type")]
    pub tool_type: ToolType,

    /// The function specification for function tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionDefinition>,
}

/// A tool call made by the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// The ID of the tool call
    pub id: String,

    /// The type of tool
    #[serde(rename = "type")]
    pub tool_type: ToolType,

    /// The function call
    pub function: FunctionCall,
}

/// A function call made by the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// The name of the function
    pub name: String,

    /// The arguments passed to the function
    pub arguments: String,
}

impl Tool {
    /// Create a new function tool
    pub fn function(function: FunctionDefinition) -> Self {
        Self {
            tool_type: ToolType::Function,
            function: Some(function),
        }
    }
}

/// The type of tool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    /// A function that can be called
    Function,
}

/// A function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function
    pub name: String,

    /// A description of what the function does
    pub description: String,

    /// The parameters schema
    pub parameters: ParameterSchema,
}

/// A parameter schema in JSON Schema format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// The type of the schema
    #[serde(rename = "type")]
    pub schema_type: String,

    /// The properties of the schema (for object types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, PropertySchema>>,

    /// Required properties (for object types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Item schema (for array types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<PropertySchema>>,
}

impl ParameterSchema {
    /// Create a new object schema
    pub fn object() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: Some(Vec::new()),
            items: None,
        }
    }

    /// Add a property to the schema
    pub fn with_property(
        mut self,
        name: impl Into<String>,
        schema: PropertySchema,
        required: bool,
    ) -> Self {
        let name_str = name.into();
        if let Some(ref mut props) = self.properties {
            props.insert(name_str.clone(), schema);
        } else {
            let mut props = HashMap::new();
            props.insert(name_str.clone(), schema);
            self.properties = Some(props);
        }
        if required {
            if let Some(ref mut req) = self.required {
                req.push(name_str);
            } else {
                self.required = Some(vec![name_str]);
            }
        }
        self
    }
}

/// A property schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    /// The type of the property
    #[serde(rename = "type")]
    pub schema_type: String,

    /// Description of the property
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Items schema (for array types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<PropertySchema>>,

    /// Enum values (for enum types)
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Properties (for object types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, PropertySchema>>,

    /// Required properties (for object types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl PropertySchema {
    /// Create a new string property
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            schema_type: "string".to_string(),
            description: Some(description.into()),
            items: None,
            enum_values: None,
            properties: None,
            required: None,
        }
    }

    /// Create a new number property
    pub fn number(description: impl Into<String>) -> Self {
        Self {
            schema_type: "number".to_string(),
            description: Some(description.into()),
            items: None,
            enum_values: None,
            properties: None,
            required: None,
        }
    }

    /// Create a new integer property
    pub fn integer(description: impl Into<String>) -> Self {
        Self {
            schema_type: "integer".to_string(),
            description: Some(description.into()),
            items: None,
            enum_values: None,
            properties: None,
            required: None,
        }
    }

    /// Create a new boolean property
    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            schema_type: "boolean".to_string(),
            description: Some(description.into()),
            items: None,
            enum_values: None,
            properties: None,
            required: None,
        }
    }

    /// Create a new array property
    pub fn array(description: impl Into<String>, item_schema: PropertySchema) -> Self {
        Self {
            schema_type: "array".to_string(),
            description: Some(description.into()),
            items: Some(Box::new(item_schema)),
            enum_values: None,
            properties: None,
            required: None,
        }
    }

    /// Create a new enum property
    pub fn enum_type(description: impl Into<String>, values: Vec<String>) -> Self {
        Self {
            schema_type: "string".to_string(),
            description: Some(description.into()),
            items: None,
            enum_values: Some(values),
            properties: None,
            required: None,
        }
    }
}
