/*!
 * Data models for the Galaxy adapter.
 * 
 * This module defines the core data models used throughout the Galaxy adapter crate.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod tool;
pub mod workflow;
pub mod dataset;
pub mod job;
pub mod history;
pub mod library;

// Re-exports
pub use tool::GalaxyTool;
pub use workflow::GalaxyWorkflow;
pub use dataset::GalaxyDataset;
pub use job::GalaxyJob;
pub use history::GalaxyHistory;
pub use library::GalaxyLibrary;

/// Different ways to reference a Galaxy dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GalaxyDataReference {
    /// Reference by dataset ID
    Id(String),
    
    /// Reference by history and position
    HistoryPosition {
        /// The history ID
        history_id: String,
        
        /// The position in the history
        position: usize,
    },
    
    /// Reference by collection element
    CollectionElement {
        /// The collection ID
        collection_id: String,
        
        /// The element identifier
        element_identifier: String,
    },
    
    /// Reference by library dataset
    LibraryDataset {
        /// The library ID
        library_id: String,
        
        /// The folder ID
        folder_id: String,
        
        /// The dataset ID
        dataset_id: String,
    },
}

/// Common metadata for Galaxy resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    /// The unique identifier for this resource
    pub id: String,
    
    /// The name of this resource
    pub name: String,
    
    /// A description of this resource
    pub description: Option<String>,
    
    /// When this resource was created
    pub create_time: String,
    
    /// When this resource was last updated
    pub update_time: String,
    
    /// Tags for this resource
    pub tags: Vec<String>,
    
    /// Any additional metadata for this resource
    pub additional_metadata: HashMap<String, String>,
}

/// Represents the status of a Galaxy API response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseStatus {
    /// The request was successful
    Success,
    
    /// The request failed
    Error,
    
    /// The request is pending or processing
    Pending,
}

/// Represents a generic API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// The status of this response
    pub status: ResponseStatus,
    
    /// The data returned, if any
    pub data: Option<T>,
    
    /// A message describing the result
    pub message: Option<String>,
    
    /// Error details, if any
    pub error_details: Option<String>,
    
    /// The request ID for tracking
    pub request_id: Option<String>,
}

/// Represents a value for a Galaxy parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    /// A string value
    String(String),
    
    /// A numeric value
    Number(f64),
    
    /// A boolean value
    Boolean(bool),
    
    /// An array of values
    Array(Vec<ParameterValue>),
    
    /// An object with string keys and arbitrary values
    Object(HashMap<String, ParameterValue>),
    
    /// A null value
    Null,
}

impl ParameterValue {
    /// Create a parameter value from a JSON value
    pub fn from_json(value: serde_json::Value) -> crate::error::Result<Self> {
        match value {
            serde_json::Value::String(s) => Ok(ParameterValue::String(s)),
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    Ok(ParameterValue::Number(f))
                } else {
                    Err(crate::error::Error::InvalidInput(format!("Invalid number value: {}", n)))
                }
            },
            serde_json::Value::Bool(b) => Ok(ParameterValue::Boolean(b)),
            serde_json::Value::Array(arr) => {
                let mut values = Vec::with_capacity(arr.len());
                for item in arr {
                    values.push(ParameterValue::from_json(item)?);
                }
                Ok(ParameterValue::Array(values))
            },
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::with_capacity(obj.len());
                for (key, val) in obj {
                    map.insert(key, ParameterValue::from_json(val)?);
                }
                Ok(ParameterValue::Object(map))
            },
            serde_json::Value::Null => Ok(ParameterValue::Null),
        }
    }
}

/// Represents a parameter definition for a Galaxy tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// The name of this parameter
    pub name: String,
    
    /// A label for this parameter
    pub label: Option<String>,
    
    /// A description of this parameter
    pub description: Option<String>,
    
    /// The type of this parameter
    pub type_name: String,
    
    /// Whether this parameter is required
    pub required: bool,
    
    /// The default value, if any
    pub default_value: Option<ParameterValue>,
    
    /// Available options for select parameters
    pub options: Option<Vec<ParameterOption>>,
    
    /// Additional attributes for this parameter
    pub attributes: HashMap<String, ParameterValue>,
}

/// Represents an option for a select parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterOption {
    /// The value of this option
    pub value: String,
    
    /// A label for this option
    pub label: Option<String>,
    
    /// Whether this option is selected by default
    pub selected: bool,
}

/// Parameters for pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    /// The page number (1-based)
    pub page: usize,
    
    /// The number of items per page
    pub items_per_page: usize,
    
    /// Sort field
    pub sort_field: Option<String>,
    
    /// Sort direction (true = descending)
    pub sort_desc: bool,
}

/// A paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The items on this page
    pub items: Vec<T>,
    
    /// The total number of items
    pub total_count: usize,
    
    /// The current page
    pub page: usize,
    
    /// The number of items per page
    pub items_per_page: usize,
    
    /// The total number of pages
    pub total_pages: usize,
}

impl ResourceMetadata {
    /// Create a new resource metadata with the given name
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: None,
            create_time: time::OffsetDateTime::now_utc().to_string(),
            update_time: time::OffsetDateTime::now_utc().to_string(),
            tags: Vec::new(),
            additional_metadata: HashMap::new(),
        }
    }
    
    /// Set the description for this resource
    pub fn with_description(&mut self, description: &str) -> &mut Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Add a tag to this resource
    pub fn add_tag(&mut self, tag: &str) -> &mut Self {
        self.tags.push(tag.to_string());
        self
    }
    
    /// Add a metadata value to this resource
    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.additional_metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Update the timestamp on this resource
    pub fn update_timestamp(&mut self) -> &mut Self {
        self.update_time = time::OffsetDateTime::now_utc().to_string();
        self
    }
}

impl Default for ResourceMetadata {
    fn default() -> Self {
        Self::new("Unnamed Resource")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_metadata() {
        let mut metadata = ResourceMetadata::new("Test Resource");
        
        metadata
            .with_description("A test resource")
            .add_tag("test")
            .add_metadata("creator", "unit-test");
        
        assert_eq!(metadata.name, "Test Resource");
        assert_eq!(metadata.description, Some("A test resource".to_string()));
        assert_eq!(metadata.tags, vec!["test"]);
        assert_eq!(metadata.additional_metadata.get("creator"), Some(&"unit-test".to_string()));
    }
} 