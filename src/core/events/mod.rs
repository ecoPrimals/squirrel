//! Event system for the Squirrel project
//!
//! This module provides the event system functionality, which allows
//! different components of the system to communicate through events.
//! Events can be emitted, handled, and processed by various parts of the system.

use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::core::error::{Error, EventError};

/// The type of event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// System events
    System,
    /// Command events
    Command,
    /// State events
    State,
    /// Error events
    Error,
    /// Custom events
    Custom(String),
}

/// Event data that can be attached to an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    /// String data
    String(String),
    /// JSON data
    Json(serde_json::Value),
    /// Binary data
    Binary(Vec<u8>),
}

/// An event in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// The type of event
    pub event_type: EventType,
    /// The timestamp when the event was created
    pub timestamp: DateTime<Utc>,
    /// The source of the event
    pub source: String,
    /// The data associated with the event
    pub data: Option<EventData>,
}

/// Builder for creating new events
pub struct EventBuilder {
    event_type: EventType,
    source: String,
    data: Option<EventData>,
}

impl Event {
    /// Create a new event builder
    pub fn builder() -> EventBuilder {
        EventBuilder {
            event_type: EventType::System,
            source: String::new(),
            data: None,
        }
    }

    /// Get the event type
    pub fn event_type(&self) -> EventType {
        self.event_type
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get the source
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the data
    pub fn data(&self) -> Option<&EventData> {
        self.data.as_ref()
    }
}

impl EventBuilder {
    /// Set the event type
    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.event_type = event_type;
        self
    }

    /// Set the source
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set the data
    pub fn data(mut self, data: EventData) -> Self {
        self.data = Some(data);
        self
    }

    /// Build the event
    pub fn build(self) -> Event {
        Event {
            event_type: self.event_type,
            timestamp: Utc::now(),
            source: self.source,
            data: self.data,
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::System => write!(f, "System"),
            EventType::Command => write!(f, "Command"),
            EventType::State => write!(f, "State"),
            EventType::Error => write!(f, "Error"),
            EventType::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

impl fmt::Display for EventData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventData::String(s) => write!(f, "String({})", s),
            EventData::Json(j) => write!(f, "Json({})", j),
            EventData::Binary(b) => write!(f, "Binary({} bytes)", b.len()),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Event(type={}, source={}, timestamp={}, data={})",
            self.event_type,
            self.source,
            self.timestamp,
            self.data.as_ref().map_or("None".to_string(), |d| d.to_string())
        )
    }
}

/// Trait for event handlers
pub trait EventHandler: Send + Sync {
    /// Handle an event
    fn handle_event(&self, event: &Event) -> Result<(), Error>;
}

/// Trait for event processors
pub trait EventProcessor: Send + Sync {
    /// Process an event
    fn process_event(&self, event: &Event) -> Result<(), Error>;
}

/// Trait for event emitters
pub trait EventEmitter: Send + Sync {
    /// Emit an event
    fn emit_event(&self, event: Event) -> Result<(), Error>;
} 