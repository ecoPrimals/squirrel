//! Core plugin functionality
//!
//! This module contains core plugin functionality that is used by all plugins.

pub mod metadata {
    //! Plugin metadata types
    
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    
    /// Detailed metadata for a plugin resource
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Metadata {
        /// Unique identifier
        pub id: String,
        
        /// Human-readable name
        pub name: String,
        
        /// Resource description
        pub description: String,
        
        /// Resource category (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub category: Option<String>,
        
        /// JSON schema for input (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub input_schema: Option<Value>,
        
        /// JSON schema for output (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub output_schema: Option<Value>,
        
        /// Required permissions (optional)
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub permissions: Vec<String>,
        
        /// Additional attributes (optional)
        #[serde(default, skip_serializing_if = "Value::is_null")]
        pub attributes: Value,
    }
    
    /// Builder for metadata
    #[derive(Debug, Default)]
    pub struct MetadataBuilder {
        id: Option<String>,
        name: Option<String>,
        description: Option<String>,
        category: Option<String>,
        input_schema: Option<Value>,
        output_schema: Option<Value>,
        permissions: Vec<String>,
        attributes: Value,
    }
    
    impl MetadataBuilder {
        /// Create a new metadata builder
        pub fn new() -> Self {
            Self {
                id: None,
                name: None,
                description: None,
                category: None,
                input_schema: None,
                output_schema: None,
                permissions: Vec::new(),
                attributes: Value::Null,
            }
        }
        
        /// Set the ID
        pub fn id(mut self, id: impl Into<String>) -> Self {
            self.id = Some(id.into());
            self
        }
        
        /// Set the name
        pub fn name(mut self, name: impl Into<String>) -> Self {
            self.name = Some(name.into());
            self
        }
        
        /// Set the description
        pub fn description(mut self, description: impl Into<String>) -> Self {
            self.description = Some(description.into());
            self
        }
        
        /// Set the category
        pub fn category(mut self, category: impl Into<String>) -> Self {
            self.category = Some(category.into());
            self
        }
        
        /// Set the input schema
        pub fn input_schema(mut self, schema: Option<Value>) -> Self {
            self.input_schema = schema;
            self
        }
        
        /// Set the output schema
        pub fn output_schema(mut self, schema: Option<Value>) -> Self {
            self.output_schema = schema;
            self
        }
        
        /// Add a permission
        pub fn permission(mut self, permission: impl Into<String>) -> Self {
            self.permissions.push(permission.into());
            self
        }
        
        /// Set the attributes
        pub fn attributes(mut self, attributes: Value) -> Self {
            self.attributes = attributes;
            self
        }
        
        /// Build the metadata
        pub fn build(self) -> Metadata {
            Metadata {
                id: self.id.unwrap_or_else(|| "unknown".to_string()),
                name: self.name.unwrap_or_else(|| "Unknown".to_string()),
                description: self.description.unwrap_or_else(|| "".to_string()),
                category: self.category,
                input_schema: self.input_schema,
                output_schema: self.output_schema,
                permissions: self.permissions,
                attributes: self.attributes,
            }
        }
    }
}

// Re-export from interfaces
pub use squirrel_interfaces::plugins::PluginExecutionContext; 