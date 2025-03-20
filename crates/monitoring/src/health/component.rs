use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::health::status::Status;

/// Health information for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Name of the component
    pub name: String,
    /// Health status of the component
    pub status: Status,
    /// Detailed message about the health status
    pub message: String,
    /// Timestamp when the health was last checked
    pub timestamp: u64,
    /// Additional metadata about the component health
    pub metadata: Option<serde_json::Value>,
}

impl ComponentHealth {
    /// Create a new component health
    #[must_use] pub fn new(name: String, status: Status, message: String) -> Self {
        Self {
            name,
            status,
            message,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: None,
        }
    }

    /// Add metadata to the component health
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
} 