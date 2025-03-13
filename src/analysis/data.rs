//! Data structures and types for analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a dataset for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// Unique identifier for the dataset
    pub id: String,
    /// Name of the dataset
    pub name: String,
    /// Description of the dataset
    pub description: Option<String>,
    /// Metadata associated with the dataset
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Represents a data point in the dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Unique identifier for the data point
    pub id: String,
    /// Values associated with the data point
    pub values: HashMap<String, serde_json::Value>,
    /// Timestamp of the data point
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Dataset {
    /// Creates a new dataset
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            metadata: HashMap::new(),
        }
    }

    /// Adds metadata to the dataset
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
} 