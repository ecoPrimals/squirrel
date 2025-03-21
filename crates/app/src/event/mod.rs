use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an event in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier for the event
    pub id: String,
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    /// Type of the event
    pub event_type: String,
    /// Event payload data
    pub data: serde_json::Value,
    /// Additional metadata associated with the event
    pub metadata: serde_json::Value,
}

impl Event {
    /// Creates a new event with the given type and data.
    #[must_use] pub fn new(event_type: String, data: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            data,
            metadata: serde_json::Value::Null,
        }
    }

    /// Adds metadata to the event and returns the modified event.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Represents errors that can occur during event operations.
#[derive(Debug)]
pub enum EventError {
    /// Error occurred during event creation
    Creation(String),
    /// Error occurred during event validation
    Validation(String),
    /// Error occurred during event storage
    Storage(String),
    /// Error occurred during event retrieval
    Retrieval(String),
    /// Other event-related errors
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::Creation(msg) => write!(f, "Event creation error: {msg}"),
            EventError::Validation(msg) => write!(f, "Event validation error: {msg}"),
            EventError::Storage(msg) => write!(f, "Event storage error: {msg}"),
            EventError::Retrieval(msg) => write!(f, "Event retrieval error: {msg}"),
            EventError::Other(e) => write!(f, "Other event error: {e}"),
        }
    }
}

impl std::error::Error for EventError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EventError::Other(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

/// A specialized Result type for event operations.
pub type Result<T> = std::result::Result<T, EventError>; 