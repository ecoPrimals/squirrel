//! Event system for the Squirrel project
//!
//! This module provides the event system functionality, which allows
//! different components of the system to communicate through events.
//! Events can be emitted, handled, and processed by various parts of the system.

use std::fmt;
use serde::{Serialize, Deserialize};
use time::OffsetDateTime;
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::fmt::Debug;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;
use uuid;

/// Errors that can occur during event processing
#[derive(Debug, Error)]
pub enum EventError {
    /// Error when event type is invalid
    #[error("Invalid event type: {0}")]
    InvalidType(String),
    /// Error during event handler execution
    #[error("Handler error: {0}")]
    HandlerError(String),
}

/// Result type for event operations
pub type Result<T> = std::result::Result<T, EventError>;

/// Types of events that can occur in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// System startup event
    SystemStartup,
    /// System shutdown event
    SystemShutdown,
    /// Metric collection event
    MetricCollected,
    /// Alert trigger event
    AlertTriggered,
    /// Health check completion event
    HealthCheckCompleted,
}

impl std::str::FromStr for EventType {
    type Err = EventError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "system" => Ok(Self::SystemStartup),
            "command" | "info" => Ok(Self::MetricCollected),
            "state" | "debug" => Ok(Self::HealthCheckCompleted),
            "error" => Ok(Self::AlertTriggered),
            "warning" | "trace" => Ok(Self::SystemShutdown),
            _ => Err(EventError::InvalidType(s.to_string())),
        }
    }
}

/// Event data that can be attached to an event
pub enum EventData {
    /// String data
    String(String),
    /// JSON data
    Json(Value),
    /// Binary data
    Binary(Vec<u8>),
}

/// Event data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier for the event
    pub id: String,
    /// Type of event
    pub event_type: EventType,
    /// Event metadata
    pub metadata: EventMetadata,
    /// Event payload
    pub payload: Value,
}

/// Builder for creating new events
pub struct EventBuilder {
    event_type: EventType,
    data: Value,
    metadata: EventMetadata,
}

impl Event {
    /// Creates a new event
    ///
    /// # Arguments
    /// * `event_type` - Type of event
    /// * `payload` - Event data
    /// * `metadata` - Optional event metadata
    #[must_use] pub fn new(
        event_type: EventType,
        payload: Value,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        let mut event_metadata = EventMetadata::new();
        
        if let Some(labels) = metadata {
            event_metadata.labels = labels;
        }
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            payload,
            metadata: event_metadata,
        }
    }

    /// Adds metadata to the event
    ///
    /// # Arguments
    /// * `metadata` - Metadata to add
    #[must_use]
    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Adds a correlation ID to the event
    ///
    /// # Arguments
    /// * `correlation_id` - ID to correlate related events
    #[must_use]
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.metadata.correlation_id = Some(correlation_id);
        self
    }

    /// Adds a label to the event
    ///
    /// # Arguments
    /// * `key` - Label key
    /// * `value` - Label value
    #[must_use]
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.metadata.labels.insert(key, value);
        self
    }

    /// Get the event type
    #[must_use = "This returns the event type which may be needed for conditional handling"]
    pub const fn event_type(&self) -> &EventType {
        &self.event_type
    }

    /// Get the timestamp
    #[must_use = "This returns the event timestamp which may be needed for time-based processing"]
    pub const fn timestamp(&self) -> OffsetDateTime {
        self.metadata.timestamp
    }

    /// Get the data
    #[must_use = "This returns the event data which contains the payload information"]
    pub const fn data(&self) -> &Value {
        &self.payload
    }

    /// Get the metadata
    #[must_use = "This returns the event metadata which contains important contextual information"]
    pub const fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}

impl EventBuilder {
    /// Create a new event builder
    #[must_use = "This returns a new event builder that should be used to create events"]
    pub fn new() -> Self {
        Self {
            event_type: EventType::SystemStartup,
            data: Value::Null,
            metadata: EventMetadata::new(),
        }
    }

    /// Set the event type
    #[must_use]
    pub const fn event_type(mut self, event_type: EventType) -> Self {
        self.event_type = event_type;
        self
    }

    /// Set the data
    #[must_use]
    pub fn data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }

    /// Set the metadata
    #[must_use]
    pub fn metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the correlation ID
    #[must_use]
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.metadata.correlation_id = Some(correlation_id);
        self
    }

    /// Add a label to the metadata
    #[must_use]
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.metadata.labels.insert(key, value);
        self
    }

    /// Build the event
    #[must_use = "This returns the built event which should be used or emitted"]
    pub fn build(self) -> Event {
        Event {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: self.event_type,
            payload: self.data,
            metadata: self.metadata,
        }
    }
}

impl Default for EventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemStartup => write!(f, "System Startup"),
            Self::SystemShutdown => write!(f, "System Shutdown"),
            Self::MetricCollected => write!(f, "Metric Collected"),
            Self::AlertTriggered => write!(f, "Alert Triggered"),
            Self::HealthCheckCompleted => write!(f, "Health Check Completed"),
        }
    }
}

impl fmt::Display for EventData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "String({s})"),
            Self::Json(j) => write!(f, "Json({j})"),
            Self::Binary(b) => write!(f, "Binary({} bytes)", b.len()),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Event {{ id: {}, type: {}, timestamp: {} }}",
            self.id, self.event_type, self.metadata.timestamp
        )
    }
}

/// Event processor interface
pub trait EventProcessor: Send + Sync {
    /// Gets the async event processor implementation
    fn as_async(&self) -> &dyn EventProcessorAsync;
}

/// Event processor for asynchronous event processing
pub trait EventProcessorAsync: Send + Sync {
    /// Process an event
    ///
    /// # Errors
    ///
    /// Returns an `EventError` if the processing fails, which can occur due to:
    /// - Invalid event data
    /// - Processing logic failure
    /// - Resource unavailability
    fn process<'a>(&'a self, event: &'a Event) -> Pin<Box<dyn Future<Output = std::result::Result<(), EventError>> + Send + 'a>>;
}

/// Event bus for managing events
pub struct EventBus {
    events: Arc<RwLock<Vec<Event>>>,
    #[allow(clippy::type_complexity)]
    subscribers: Arc<RwLock<Vec<Box<dyn Fn(&Event) + Send + Sync>>>>,
}

impl Debug for EventBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventBus")
            .field("events", &self.events)
            .field("subscribers", &format!("<{count} subscribers>", count = self.subscribers.blocking_read().len()))
            .finish()
    }
}

impl EventBus {
    /// Create a new event bus
    #[must_use = "This returns a new event bus that should be used for event operations"]
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Publish an event to the event bus
    ///
    /// # Errors
    ///
    /// Returns an `EventError` if the event cannot be published
    pub async fn publish(&self, event: Event) -> Result<()> {
        let mut events = self.events.write().await;
        events.push(event.clone());
        
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.iter() {
            subscriber(&event);
        }
        
        Ok(())
    }

    /// Subscribe to events on the event bus
    ///
    /// # Errors
    ///
    /// Returns an `EventError` if the subscription cannot be registered
    pub async fn subscribe<F>(&self, handler: F) -> Result<()>
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(Box::new(handler));
        Ok(())
    }

    /// Get all events from the event bus
    ///
    /// # Errors
    ///
    /// Returns an `EventError` if the events cannot be retrieved
    pub async fn get_events(&self) -> Result<Vec<Event>> {
        let events = self.events.read().await.clone();
        Ok(events)
    }

    /// Clear all events from the event bus
    ///
    /// # Errors
    ///
    /// Returns an `EventError` if the events cannot be cleared
    pub async fn clear_events(&self) -> Result<()> {
        let mut events = self.events.write().await;
        events.clear();
        Ok(())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the event system
///
/// # Errors
///
/// Returns an `EventError` if the event system cannot be initialized
pub const fn initialize() -> Result<()> {
    // TODO: Initialize event system
    Ok(())
}

/// Shutdown the event system
///
/// # Errors
///
/// Returns an `EventError` if the event system cannot be shut down properly
pub const fn shutdown() -> Result<()> {
    // TODO: Cleanup event system resources
    Ok(())
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Time when the event occurred
    pub timestamp: OffsetDateTime,
    /// ID to correlate related events
    pub correlation_id: Option<String>,
    /// Additional event labels
    pub labels: HashMap<String, String>,
}

impl EventMetadata {
    /// Create a new event metadata
    #[must_use = "This returns new event metadata that should be used with events"]
    pub fn new() -> Self {
        Self {
            timestamp: OffsetDateTime::now_utc(),
            correlation_id: None,
            labels: HashMap::new(),
        }
    }
    
    /// Set the correlation ID
    #[must_use = "This method returns self for method chaining and the return value should be used"]
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    /// Add a label
    #[must_use = "This method returns self for method chaining and the return value should be used"]
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Event handler interface
#[async_trait]
pub trait EventHandler: Send + Sync + Debug {
    /// Handles an event
    ///
    /// # Arguments
    /// * `event` - Event to handle
    async fn handle(&self, event: Event) -> Result<()>;
}

/// Configuration for event emitter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEmitterConfig {
    /// Maximum number of events to store
    pub max_events: usize,
    /// Size of event buffer
    pub buffer_size: usize,
}

/// Event emitter interface
#[async_trait]
pub trait EventEmitter: Send + Sync {
    /// Emits an event
    ///
    /// # Arguments
    /// * `event` - Event to emit
    async fn emit(&self, event: Event) -> Result<()>;
}

/// Default implementation of the `EventEmitter` trait
#[derive(Debug)]
pub struct DefaultEventEmitter {
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler + Send + Sync>>>>,
}

impl DefaultEventEmitter {
    /// Create a new default event emitter
    #[must_use = "This returns a new event emitter that should be used for emitting events"]
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Register a new event handler
    ///
    /// # Errors
    ///
    /// Returns an `EventError` if the handler cannot be registered
    pub async fn register_handler<H>(&self, handler: H) -> Result<()>
    where
        H: EventHandler + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().await;
        handlers.push(Box::new(handler));
        Ok(())
    }
}

#[async_trait]
impl EventEmitter for DefaultEventEmitter {
    async fn emit(&self, event: Event) -> Result<()> {
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            handler.handle(event.clone()).await?;
        }
        Ok(())
    }
}

impl Default for DefaultEventEmitter {
    fn default() -> Self {
        Self::new()
    }
} 
