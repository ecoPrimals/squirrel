//! Event handling and pub/sub functionality for plugins
//!
//! This module provides event handling and communication between plugins.

use std::collections::HashMap;

use crate::error::{PluginError, PluginResult};
use crate::utils::{current_timestamp_iso, generate_listener_id, safe_lock};
use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

/// Event data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event ID
    pub id: String,
    /// Event type/name
    pub event_type: String,
    /// Event source plugin
    pub source: String,
    /// Event timestamp
    pub timestamp: String,
    /// Event payload
    pub payload: serde_json::Value,
    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Event {
    /// Create a new event
    pub fn new(event_type: String, payload: serde_json::Value) -> Self {
        Self {
            id: crate::utils::generate_uuid(),
            event_type,
            source: "plugin".to_string(),
            timestamp: current_timestamp_iso(),
            payload,
            metadata: HashMap::new(),
        }
    }

    /// Create a new event with custom source
    pub fn with_source(event_type: String, payload: serde_json::Value, source: String) -> Self {
        Self {
            id: crate::utils::generate_uuid(),
            event_type,
            source,
            timestamp: current_timestamp_iso(),
            payload,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get event type
    pub fn get_type(&self) -> &str {
        &self.event_type
    }

    /// Get event data
    pub fn get_data(&self) -> &serde_json::Value {
        &self.payload
    }

    /// Get event timestamp
    pub fn get_timestamp(&self) -> PluginResult<u64> {
        // Parse ISO timestamp back to milliseconds if needed
        Ok(crate::utils::current_timestamp())
    }

    /// Get event source
    pub fn get_source(&self) -> &str {
        &self.source
    }

    /// Get event metadata
    pub fn get_metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.metadata
    }
}

/// Event listener trait
pub trait EventListener: Send + Sync + std::fmt::Debug {
    /// Handle an event
    fn handle_event(
        &self,
        event: Event,
    ) -> Pin<Box<dyn Future<Output = PluginResult<()>> + Send + '_>>;
}

/// Simple event listener implementation
#[derive(Debug)]
pub struct SimpleEventListener<F>
where
    F: Fn(Event) -> Pin<Box<dyn Future<Output = PluginResult<()>> + Send + 'static>>
        + Send
        + Sync
        + std::fmt::Debug,
{
    handler: F,
}

impl<F> SimpleEventListener<F>
where
    F: Fn(Event) -> Pin<Box<dyn Future<Output = PluginResult<()>> + Send + 'static>>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Create a new simple event listener
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> EventListener for SimpleEventListener<F>
where
    F: Fn(Event) -> Pin<Box<dyn Future<Output = PluginResult<()>> + Send + 'static>>
        + Send
        + Sync
        + std::fmt::Debug,
{
    fn handle_event(
        &self,
        event: Event,
    ) -> Pin<Box<dyn Future<Output = PluginResult<()>> + Send + '_>> {
        (self.handler)(event)
    }
}

/// Wrapper for event listeners with IDs for proper removal
#[derive(Debug)]
struct ListenerEntry {
    id: String,
    listener: Box<dyn EventListener + Send + Sync>,
}

/// Event bus for pub/sub functionality
#[derive(Debug)]
pub struct EventBus {
    /// Event listeners organized by event type with their IDs
    listeners: std::sync::Mutex<HashMap<String, Vec<ListenerEntry>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            listeners: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Get the global event bus instance
    pub fn global() -> &'static EventBus {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<EventBus> = OnceLock::new();
        INSTANCE.get_or_init(|| EventBus::new())
    }

    /// Subscribe to an event
    pub fn subscribe(
        &self,
        event_type: &str,
        listener: Box<dyn EventListener + Send + Sync>,
    ) -> PluginResult<String> {
        let listener_id = generate_listener_id();
        let mut listeners = safe_lock(&self.listeners, "listeners")?;
        
        let entry = ListenerEntry {
            id: listener_id.clone(),
            listener,
        };
        
        listeners
            .entry(event_type.to_string())
            .or_default()
            .push(entry);
            
        Ok(listener_id)
    }

    /// Publish an event
    pub async fn publish(&self, event: Event) -> PluginResult<()> {
        // Get listeners and handle the event in a single lock acquisition
        let listeners_guard = safe_lock(&self.listeners, "listeners")?;

        if let Some(listeners) = listeners_guard.get(&event.event_type) {
            // Handle event for each listener entry
            for entry in listeners {
                if let Err(e) = entry.listener.handle_event(event.clone()).await {
                    eprintln!("Error handling event for listener {}: {:?}", entry.id, e);
                }
            }
        }

        Ok(())
    }

    /// Unsubscribe from an event
    pub fn unsubscribe(&self, event_type: &str, listener_id: &str) -> PluginResult<()> {
        let mut listeners = safe_lock(&self.listeners, "listeners")?;
        
        if let Some(event_listeners) = listeners.get_mut(event_type) {
            // Find and remove the listener with the matching ID
            let original_len = event_listeners.len();
            event_listeners.retain(|entry| entry.id != listener_id);
            
            // Check if a listener was actually removed
            if event_listeners.len() == original_len {
                return Err(PluginError::ResourceNotFound {
                    resource: format!("Listener with ID '{}' for event type '{}'", listener_id, event_type)
                });
            }
            
            // If no listeners remain for this event type, remove the entry entirely
            if event_listeners.is_empty() {
                listeners.remove(event_type);
            }
        } else {
            return Err(PluginError::ResourceNotFound {
                resource: format!("Event listeners for type '{}'", event_type)
            });
        }
        
        Ok(())
    }

    /// Get all event types
    pub fn list_event_types(&self) -> Vec<String> {
        match safe_lock(&self.listeners, "listeners") {
            Ok(listeners) => listeners.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Get listener count for an event type
    pub fn get_listener_count(&self, event_type: &str) -> usize {
        match safe_lock(&self.listeners, "listeners") {
            Ok(listeners) => listeners.get(event_type).map(|v| v.len()).unwrap_or(0),
            Err(_) => 0,
        }
    }
}

/// Helper macro for creating event listeners
#[macro_export]
macro_rules! event_listener {
    ($handler:expr) => {
        SimpleEventListener::new(|event| Box::pin(async move { $handler(event).await }))
    };
}

/// Plugin event constants
pub mod event_types {
    /// Plugin lifecycle events
    /// Event type emitted when a plugin has been successfully initialized
    pub const PLUGIN_INITIALIZED: &str = "plugin.initialized";

    /// Event type emitted when a plugin has started execution
    pub const PLUGIN_STARTED: &str = "plugin.started";

    /// Event type emitted when a plugin has stopped execution
    pub const PLUGIN_STOPPED: &str = "plugin.stopped";

    /// Event type emitted when a plugin encounters an error
    pub const PLUGIN_ERROR: &str = "plugin.error";

    /// Command events
    /// Event type emitted when a command has been successfully executed
    pub const COMMAND_EXECUTED: &str = "command.executed";

    /// Event type emitted when a command execution fails
    pub const COMMAND_FAILED: &str = "command.failed";

    /// System events
    pub const SYSTEM_READY: &str = "system.ready";
    pub const SYSTEM_SHUTDOWN: &str = "system.shutdown";

    /// Custom events
    /// Custom event type for user-defined events
    pub const CUSTOM_EVENT: &str = "custom.event";
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;

    #[derive(Debug)]
    struct TestListener {
        received_events: Arc<TokioMutex<Vec<Event>>>,
    }

    impl TestListener {
        fn new() -> (Self, Arc<TokioMutex<Vec<Event>>>) {
            let events = Arc::new(TokioMutex::new(Vec::new()));
            let listener = Self {
                received_events: events.clone(),
            };
            (listener, events)
        }
    }

    impl EventListener for TestListener {
        fn handle_event(
            &self,
            event: Event,
        ) -> Pin<Box<dyn Future<Output = PluginResult<()>> + Send + '_>> {
            async move {
                let mut events = self.received_events.lock().await;
                events.push(event);
                Ok(())
            }
            .boxed()
        }
    }

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new();
        let (listener, received_events) = TestListener::new();

        // Subscribe to events
        bus.subscribe("test.event", Box::new(listener)).unwrap();

        // Create and publish event
        let event = Event::new(
            "test.event".to_string(),
            serde_json::json!({"message": "Hello, World!"}),
        );

        bus.publish(event).await.unwrap();

        // Verify event was received
        let events = received_events.lock().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test.event");
    }

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            "test.event".to_string(),
            serde_json::json!({"key": "value"}),
        );

        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.source, "plugin");
        assert!(!event.id.is_empty());
        assert_eq!(event.payload, serde_json::json!({"key": "value"}));
    }

    #[test]
    fn test_event_with_metadata() {
        let event = Event::new(
            "test.event".to_string(),
            serde_json::json!({"key": "value"}),
        )
        .with_metadata("priority".to_string(), serde_json::json!("high"));

        assert_eq!(
            event.metadata.get("priority"),
            Some(&serde_json::json!("high"))
        );
    }
}
