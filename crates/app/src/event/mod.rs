use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub data: serde_json::Value,
    pub metadata: serde_json::Value,
}

impl Event {
    pub fn new(event_type: String, data: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            data,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug)]
pub enum EventError {
    Creation(String),
    Validation(String),
    Storage(String),
    Retrieval(String),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::Creation(msg) => write!(f, "Event creation error: {}", msg),
            EventError::Validation(msg) => write!(f, "Event validation error: {}", msg),
            EventError::Storage(msg) => write!(f, "Event storage error: {}", msg),
            EventError::Retrieval(msg) => write!(f, "Event retrieval error: {}", msg),
            EventError::Other(e) => write!(f, "Other event error: {}", e),
        }
    }
}

impl std::error::Error for EventError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EventError::Creation(_) => None,
            EventError::Validation(_) => None,
            EventError::Storage(_) => None,
            EventError::Retrieval(_) => None,
            EventError::Other(e) => Some(e.as_ref()),
        }
    }
}

impl From<EventError> for crate::core::error::Error {
    fn from(err: EventError) -> Self {
        crate::core::error::Error::Other(Box::new(err))
    }
}

pub type Result<T> = std::result::Result<T, EventError>; 