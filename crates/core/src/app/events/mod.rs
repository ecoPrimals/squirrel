//! Event system for the Squirrel project
//!
//! This module provides the event system functionality, which allows
//! different components of the system to communicate through events.
//! Events can be emitted, handled, and processed by various parts of the system.

use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::core::error::{Result as CoreResult, Error as CoreError, EventError};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::future::Future;
use std::pin::Pin;
use uuid;
use serde_json::Value;
use thiserror;
use crate::core::error::Error as SquirrelError;

/// The type of event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    Core,
    Protocol,
    Context,
    Security,
    Monitoring,
    Custom(u32),
}

impl From<String> for EventType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "system" => Self::System,
            "command" => Self::Command,
            "state" => Self::State,
            "error" => Self::Error,
            "warning" => Self::Warning,
            "info" => Self::Info,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            _ => Self::Custom(s.parse::<u32>().unwrap()),
        }
    }
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
    pub event_type: EventType,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
}

/// Builder for creating new events
pub struct EventBuilder {
    event_type: EventType,
    data: Value,
    metadata: EventMetadata,
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
        self.metadata.timestamp
    }

    /// Get the data
    pub fn data(&self) -> &Value {
        &self.payload
    }

    /// Get the metadata
    pub fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    pub fn new(event_type: EventType, data: Value, metadata: EventMetadata) -> Self {
        Self {
            event_type,
            payload: data,
            metadata,
        }
    }

    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

impl EventBuilder {
    /// Create a new event builder
    pub fn new() -> Self {
        Self {
            event_type: EventType::Core,
            data: serde_json::Value::Null,
            metadata: EventMetadata {
                timestamp: Utc::now(),
                source: String::new(),
                correlation_id: None,
            },
        }
    }

    /// Set the event type
    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.event_type = event_type;
        self
    }

    /// Set the data
    pub fn data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }

    /// Set the metadata
    pub fn metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the event
    pub fn build(self) -> Event {
        self.into()
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::Core => write!(f, "Core"),
            EventType::Protocol => write!(f, "Protocol"),
            EventType::Context => write!(f, "Context"),
            EventType::Security => write!(f, "Security"),
            EventType::Monitoring => write!(f, "Monitoring"),
            EventType::Custom(id) => write!(f, "Custom({})", id),
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
            "Event(type={}, timestamp={}, metadata={})",
            self.event_type,
            self.metadata.timestamp,
            self.metadata.correlation_id.as_ref().map_or("None", |id| id)
        )
    }
}

pub trait EventHandler: Send + Sync {
    fn id(&self) -> String;
    async fn handle(&self, event: Event) -> CoreResult<()>;
}

pub trait EventProcessor: Send + Sync {
    fn as_async(&self) -> &dyn EventProcessorAsync;
}

pub trait EventProcessorAsync: Send + Sync {
    /// Process an event
    fn process<'a>(&'a self, event: &'a Event) -> Pin<Box<dyn Future<Output = std::result::Result<(), EventError>> + Send + 'a>>;
}

pub trait EventEmitter: Send + Sync {
    async fn emit(&self, event: Event) -> CoreResult<()>;
    async fn subscribe(&self, event_type: EventType, handler: Box<dyn EventHandler>) -> CoreResult<()>;
    async fn unsubscribe(&self, event_type: EventType, handler_id: String) -> CoreResult<()>;
}

pub struct EventBus {
    events: Arc<RwLock<Vec<Event>>>,
    subscribers: Arc<RwLock<Vec<Box<dyn Fn(&Event) + Send + Sync>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn publish(&self, event: Event) -> CoreResult<()> {
        let mut events = self.events.write().await;
        events.push(event.clone());
        
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.iter() {
            subscriber(&event);
        }
        
        Ok(())
    }

    pub async fn subscribe<F>(&self, handler: F) -> CoreResult<()>
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(Box::new(handler));
        Ok(())
    }

    pub async fn get_events(&self) -> CoreResult<Vec<Event>> {
        let events = self.events.read().await;
        Ok(events.clone())
    }

    pub async fn clear_events(&self) -> CoreResult<()> {
        let mut events = self.events.write().await;
        events.clear();
        Ok(())
    }
}

/// Initialize the event system
pub async fn initialize() -> Result<()> {
    // TODO: Initialize event system
    Ok(())
}

/// Shutdown the event system
pub async fn shutdown() -> Result<()> {
    // TODO: Cleanup event system resources
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<EventError> for SquirrelError {
    fn from(err: EventError) -> Self {
        match err {
            EventError::Io(e) => SquirrelError::Io(e),
            EventError::Json(e) => SquirrelError::Json(e),
            EventError::Other(e) => SquirrelError::Other(e),
        }
    }
}

pub type Result<T> = std::result::Result<T, EventError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub correlation_id: Option<String>,
}

pub struct DefaultEventEmitter {
    handlers: Arc<RwLock<HashMap<EventType, Vec<Box<dyn EventHandler>>>>>,
}

impl DefaultEventEmitter {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventEmitter for DefaultEventEmitter {
    async fn emit(&self, event: Event) -> CoreResult<()> {
        let handlers = self.handlers.read().await;
        if let Some(type_handlers) = handlers.get(&event.event_type) {
            for handler in type_handlers {
                handler.handle(event.clone()).await?;
            }
        }
        Ok(())
    }

    async fn subscribe(&self, event_type: EventType, handler: Box<dyn EventHandler>) -> CoreResult<()> {
        let mut handlers = self.handlers.write().await;
        handlers.entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
        Ok(())
    }

    async fn unsubscribe(&self, event_type: EventType, handler_id: String) -> CoreResult<()> {
        let mut handlers = self.handlers.write().await;
        if let Some(type_handlers) = handlers.get_mut(&event_type) {
            type_handlers.retain(|h| h.id() != handler_id);
        }
        Ok(())
    }
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    fn id(&self) -> String;
    async fn handle(&self, event: Event) -> CoreResult<()>;
} 