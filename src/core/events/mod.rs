//! Event system for the Squirrel project
//!
//! This module provides the event system functionality, which allows
//! different components of the system to communicate through events.
//! Events can be emitted, handled, and processed by various parts of the system.

use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::core::error::{Error, EventError, Result, SquirrelError};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::future::Future;
use std::pin::Pin;

/// The type of event
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// The payload associated with the event
    pub payload: serde_json::Value,
    /// Metadata associated with the event
    pub metadata: HashMap<String, String>,
}

/// Builder for creating new events
pub struct EventBuilder {
    event_type: String,
    source: String,
    payload: serde_json::Value,
    metadata: HashMap<String, String>,
}

impl Event {
    /// Create a new event builder
    pub fn builder() -> EventBuilder {
        EventBuilder::new()
    }

    /// Get the event type
    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get the source
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the payload
    pub fn payload(&self) -> &serde_json::Value {
        &self.payload
    }

    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

impl EventBuilder {
    /// Create a new event builder
    pub fn new() -> Self {
        Self {
            event_type: String::new(),
            source: String::new(),
            payload: serde_json::Value::Null,
            metadata: HashMap::new(),
        }
    }

    /// Set the event type
    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = event_type.into();
        self
    }

    /// Set the source
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set the payload
    pub fn payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the event
    pub fn build(self) -> Event {
        Event {
            event_type: self.event_type.into(),
            timestamp: Utc::now(),
            source: self.source,
            payload: self.payload,
            metadata: self.metadata,
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
            "Event(type={}, source={}, timestamp={}, metadata={})",
            self.event_type,
            self.source,
            self.timestamp,
            self.metadata.len()
        )
    }
}

pub trait EventHandler: Send + Sync {
    fn as_async(&self) -> &dyn EventHandlerAsync;
}

pub trait EventHandlerAsync: Send + Sync {
    /// Handle an event
    fn handle<'a>(&'a self, event: &'a Event) -> Pin<Box<dyn Future<Output = std::result::Result<(), EventError>> + Send + 'a>>;

    /// Check if this handler can handle the given event type
    fn can_handle<'a>(&'a self, event_type: &'a EventType) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>;
}

pub trait EventProcessor: Send + Sync {
    fn as_async(&self) -> &dyn EventProcessorAsync;
}

pub trait EventProcessorAsync: Send + Sync {
    /// Process an event
    fn process<'a>(&'a self, event: &'a Event) -> Pin<Box<dyn Future<Output = std::result::Result<(), EventError>> + Send + 'a>>;
}

pub trait EventEmitter: Send + Sync {
    fn as_async(&self) -> &dyn EventEmitterAsync;
}

pub trait EventEmitterAsync: Send + Sync {
    /// Emit an event
    fn emit<'a>(&'a self, event: Event) -> Pin<Box<dyn Future<Output = std::result::Result<(), EventError>> + Send + 'a>>;
}

pub struct EventBus {
    handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,
    processors: Arc<RwLock<Vec<Arc<dyn EventProcessor>>>>,
    emitters: Arc<RwLock<Vec<Arc<dyn EventEmitter>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
            processors: Arc::new(RwLock::new(Vec::new())),
            emitters: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) {
        self.handlers.write().await.push(handler);
    }

    pub async fn register_processor(&self, processor: Arc<dyn EventProcessor>) {
        self.processors.write().await.push(processor);
    }

    pub async fn register_emitter(&self, emitter: Arc<dyn EventEmitter>) {
        self.emitters.write().await.push(emitter);
    }

    /// Process an event through the event bus
    pub async fn process_event(&self, event: Event) -> std::result::Result<(), EventError> {
        // Check handlers
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            let handler_async = handler.as_async();
            if handler_async.can_handle(&event.event_type).await {
                handler_async.handle(&event).await?;
            }
        }
        
        // Process through processors
        let processors = self.processors.read().await;
        for processor in processors.iter() {
            processor.as_async().process(&event).await?;
        }
        
        // Emit through emitters
        let emitters = self.emitters.read().await;
        for emitter in emitters.iter() {
            emitter.as_async().emit(event.clone()).await?;
        }
        
        Ok(())
    }
}

/// Initialize the event system
pub async fn initialize() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Initialize event system
    Ok(())
}

/// Shutdown the event system
pub async fn shutdown() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Cleanup event system resources
    Ok(())
} 