// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
    pub items: Option<Box<Self>>,

    /// Enum values (for enum types)
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Properties (for object types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Self>>,

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
    pub fn array(description: impl Into<String>, item_schema: Self) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_function_constructor() {
        let func = FunctionDefinition {
            name: "test_fn".to_string(),
            description: "A test function".to_string(),
            parameters: ParameterSchema::object(),
        };
        let tool = Tool::function(func);
        assert!(matches!(tool.tool_type, ToolType::Function));
        assert!(tool.function.is_some());
        assert_eq!(tool.function.unwrap().name, "test_fn");
    }

    #[test]
    fn test_tool_type_serde() {
        let tt = ToolType::Function;
        let json = serde_json::to_string(&tt).expect("serialize");
        assert_eq!(json, "\"function\"");
        let deser: ToolType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser, ToolType::Function);
    }

    #[test]
    fn test_parameter_schema_object() {
        let schema = ParameterSchema::object();
        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties.is_some());
        assert!(schema.required.is_some());
        assert!(schema.items.is_none());
    }

    #[test]
    fn test_parameter_schema_with_property() {
        let schema = ParameterSchema::object()
            .with_property("name", PropertySchema::string("The name"), true)
            .with_property("age", PropertySchema::integer("The age"), false);
        let props = schema.properties.unwrap();
        assert_eq!(props.len(), 2);
        assert!(props.contains_key("name"));
        assert!(props.contains_key("age"));
        let req = schema.required.unwrap();
        assert_eq!(req.len(), 1);
        assert!(req.contains(&"name".to_string()));
    }

    #[test]
    fn test_property_schema_types() {
        let s = PropertySchema::string("desc");
        assert_eq!(s.schema_type, "string");
        assert_eq!(s.description.as_deref(), Some("desc"));

        let n = PropertySchema::number("num");
        assert_eq!(n.schema_type, "number");

        let i = PropertySchema::integer("int");
        assert_eq!(i.schema_type, "integer");

        let b = PropertySchema::boolean("bool");
        assert_eq!(b.schema_type, "boolean");
    }

    #[test]
    fn test_property_schema_array() {
        let arr = PropertySchema::array("items", PropertySchema::string("item"));
        assert_eq!(arr.schema_type, "array");
        assert!(arr.items.is_some());
    }

    #[test]
    fn test_property_schema_enum() {
        let e = PropertySchema::enum_type("pick", vec!["a".to_string(), "b".to_string()]);
        assert_eq!(e.schema_type, "string");
        assert_eq!(e.enum_values.unwrap().len(), 2);
    }

    #[test]
    fn test_tool_serde_roundtrip() {
        let tool = Tool::function(FunctionDefinition {
            name: "search".to_string(),
            description: "Search for items".to_string(),
            parameters: ParameterSchema::object().with_property(
                "query",
                PropertySchema::string("search query"),
                true,
            ),
        });
        let json = serde_json::to_string(&tool).expect("serialize");
        let deser: Tool = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.function.unwrap().name, "search");
    }

    #[test]
    fn test_tool_call_serde() {
        let call = ToolCall {
            id: "call-1".to_string(),
            tool_type: ToolType::Function,
            function: FunctionCall {
                name: "search".to_string(),
                arguments: r#"{"query":"test"}"#.to_string(),
            },
        };
        let json = serde_json::to_string(&call).expect("serialize");
        let deser: ToolCall = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.id, "call-1");
        assert_eq!(deser.function.name, "search");
    }
}
