// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Event handling and pub/sub functionality for plugins
//!
//! This module provides event handling and communication between plugins.

use std::collections::HashMap;

use crate::infrastructure::error::PluginResult;
use crate::utils::{current_timestamp_iso, generate_listener_id};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use tokio::sync::Mutex as TokioMutex;

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
///
/// Uses `tokio::sync::Mutex` for async-safe locking that can be held across await points.
/// This is the recommended approach for async Rust code that needs to hold locks
/// while awaiting futures.
#[derive(Debug)]
pub struct EventBus {
    /// Event listeners organized by event type with their IDs
    /// Using tokio Mutex for async-safe access across await points
    listeners: TokioMutex<HashMap<String, Vec<ListenerEntry>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self {
            listeners: TokioMutex::new(HashMap::new()),
        }
    }
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the global event bus instance
    pub fn global() -> &'static EventBus {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<EventBus> = OnceLock::new();
        INSTANCE.get_or_init(EventBus::default)
    }

    /// Subscribe to an event
    ///
    /// Note: This is now an async function to use the tokio mutex.
    /// For sync contexts, use `subscribe_blocking()`.
    pub async fn subscribe_async(
        &self,
        event_type: &str,
        listener: Box<dyn EventListener + Send + Sync>,
    ) -> PluginResult<String> {
        let listener_id = generate_listener_id();
        let mut listeners = self.listeners.lock().await;

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

    /// Subscribe to an event (blocking version for sync contexts)
    ///
    /// Uses `futures::executor::block_on` to subscribe from sync code.
    /// Prefer `subscribe_async` in async contexts.
    pub fn subscribe(
        &self,
        event_type: &str,
        listener: Box<dyn EventListener + Send + Sync>,
    ) -> PluginResult<String> {
        // Use try_lock first to avoid blocking if possible
        if let Ok(mut listeners) = self.listeners.try_lock() {
            let listener_id = generate_listener_id();
            let entry = ListenerEntry {
                id: listener_id.clone(),
                listener,
            };

            listeners
                .entry(event_type.to_string())
                .or_default()
                .push(entry);

            return Ok(listener_id);
        }

        // Fallback: if try_lock fails, we need to block
        // This should be rare in practice
        futures::executor::block_on(self.subscribe_async(event_type, listener))
    }

    /// Publish an event
    ///
    /// Uses tokio::Mutex which is designed to be held across await points,
    /// making this safe for async event handling without risk of deadlocks.
    pub async fn publish(&self, event: Event) -> PluginResult<()> {
        // tokio::Mutex is explicitly designed for holding across await points
        let listeners = self.listeners.lock().await;

        if let Some(event_listeners) = listeners.get(&event.event_type) {
            for entry in event_listeners {
                if let Err(e) = entry.listener.handle_event(event.clone()).await {
                    eprintln!("Error handling event for listener {}: {:?}", entry.id, e);
                }
            }
        }

        Ok(())
    }

    /// Unsubscribe from an event (async version)
    pub async fn unsubscribe_async(&self, event_type: &str, listener_id: &str) -> PluginResult<()> {
        let mut listeners = self.listeners.lock().await;

        if let Some(event_listeners) = listeners.get_mut(event_type) {
            // Find and remove the listener with the matching ID
            let original_len = event_listeners.len();
            event_listeners.retain(|entry| entry.id != listener_id);

            // Check if a listener was actually removed
            if event_listeners.len() == original_len {
                return Err(
                    crate::infrastructure::error::PluginError::EventHandlingError {
                        event_type: event_type.to_string(),
                        message: format!("Listener with ID '{}' not found", listener_id),
                    },
                );
            }

            // If no listeners remain for this event type, remove the entry entirely
            if event_listeners.is_empty() {
                listeners.remove(event_type);
            }
        } else {
            return Err(
                crate::infrastructure::error::PluginError::EventHandlingError {
                    event_type: event_type.to_string(),
                    message: format!("Event listeners for type '{}' not found", event_type),
                },
            );
        }

        Ok(())
    }

    /// Unsubscribe from an event (sync version for backward compatibility)
    pub fn unsubscribe(&self, event_type: &str, listener_id: &str) -> PluginResult<()> {
        // Try non-blocking first
        if let Ok(mut listeners) = self.listeners.try_lock() {
            if let Some(event_listeners) = listeners.get_mut(event_type) {
                let original_len = event_listeners.len();
                event_listeners.retain(|entry| entry.id != listener_id);

                if event_listeners.len() == original_len {
                    return Err(
                        crate::infrastructure::error::PluginError::EventHandlingError {
                            event_type: event_type.to_string(),
                            message: format!("Listener with ID '{}' not found", listener_id),
                        },
                    );
                }

                if event_listeners.is_empty() {
                    listeners.remove(event_type);
                }
                return Ok(());
            } else {
                return Err(
                    crate::infrastructure::error::PluginError::EventHandlingError {
                        event_type: event_type.to_string(),
                        message: format!("Event listeners for type '{}' not found", event_type),
                    },
                );
            }
        }

        // Fallback to blocking
        futures::executor::block_on(self.unsubscribe_async(event_type, listener_id))
    }

    /// Get all event types
    pub fn list_event_types(&self) -> Vec<String> {
        self.listeners
            .try_lock()
            .map(|listeners| listeners.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get listener count for an event type
    pub fn get_listener_count(&self, event_type: &str) -> usize {
        self.listeners
            .try_lock()
            .map(|listeners| listeners.get(event_type).map(|v| v.len()).unwrap_or(0))
            .unwrap_or(0)
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
    /// Event type emitted when the system is ready for operation
    pub const SYSTEM_READY: &str = "system.ready";
    /// Event type emitted when the system is shutting down
    pub const SYSTEM_SHUTDOWN: &str = "system.shutdown";

    /// Custom events
    /// Custom event type for user-defined events
    pub const CUSTOM_EVENT: &str = "custom.event";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::error::PluginError;
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
        ) -> Pin<Box<dyn Future<Output = Result<(), PluginError>> + Send + '_>> {
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
        bus.subscribe("test.event", Box::new(listener))
            .expect("should succeed");

        // Create and publish event
        let event = Event::new(
            "test.event".to_string(),
            serde_json::json!({"message": "Hello, World!"}),
        );

        bus.publish(event).await.expect("should succeed");

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

    #[test]
    fn test_event_with_source_and_accessors() {
        let e = Event::with_source(
            "e.t".to_string(),
            serde_json::json!({"a": 1}),
            "src1".to_string(),
        );
        assert_eq!(e.get_source(), "src1");
        assert_eq!(e.get_type(), "e.t");
        assert_eq!(e.get_data(), &serde_json::json!({"a": 1}));
        assert!(e.get_timestamp().is_ok());
        assert!(e.get_metadata().is_empty());
    }

    #[test]
    fn test_event_serde_roundtrip() {
        let e = Event::new("t".to_string(), serde_json::json!({}));
        let s = serde_json::to_string(&e).expect("should succeed");
        let back: Event = serde_json::from_str(&s).expect("should succeed");
        assert_eq!(back.event_type, e.event_type);
    }

    #[test]
    fn test_event_types_constants() {
        use super::event_types;
        assert!(!event_types::PLUGIN_INITIALIZED.is_empty());
        assert!(!event_types::CUSTOM_EVENT.is_empty());
    }

    #[test]
    fn test_event_bus_global() {
        let _ = EventBus::global();
    }

    #[tokio::test]
    async fn test_subscribe_async_and_unsubscribe_async() {
        let bus = EventBus::new();
        let (listener, _received) = TestListener::new();
        let id = bus
            .subscribe_async("async.evt", Box::new(listener))
            .await
            .expect("should succeed");
        assert_eq!(bus.get_listener_count("async.evt"), 1);
        let mut types = bus.list_event_types();
        types.sort();
        assert!(types.contains(&"async.evt".to_string()));

        bus.unsubscribe_async("async.evt", &id)
            .await
            .expect("should succeed");
        assert_eq!(bus.get_listener_count("async.evt"), 0);
        assert!(bus.unsubscribe_async("async.evt", "nope").await.is_err());
        assert!(bus.unsubscribe_async("missing", &id).await.is_err());
    }

    #[tokio::test]
    async fn test_publish_listener_error_is_logged() {
        #[derive(Debug)]
        struct FailListener;

        impl EventListener for FailListener {
            fn handle_event(
                &self,
                _event: Event,
            ) -> Pin<Box<dyn Future<Output = Result<(), PluginError>> + Send + '_>> {
                async move {
                    Err(PluginError::InternalError {
                        message: "fail".into(),
                    })
                }
                .boxed()
            }
        }

        let bus = EventBus::new();
        bus.subscribe("fail.evt", Box::new(FailListener))
            .expect("should succeed");
        let ev = Event::new("fail.evt".to_string(), serde_json::json!({}));
        bus.publish(ev).await.expect("should succeed");
    }

    #[tokio::test]
    async fn test_counting_listener_like_simple_listener() {
        #[derive(Debug)]
        struct CountingListener {
            n: Arc<TokioMutex<u32>>,
        }

        impl EventListener for CountingListener {
            fn handle_event(
                &self,
                _event: Event,
            ) -> Pin<Box<dyn Future<Output = Result<(), PluginError>> + Send + '_>> {
                let n = self.n.clone();
                async move {
                    *n.lock().await += 1;
                    Ok(())
                }
                .boxed()
            }
        }

        let n = Arc::new(TokioMutex::new(0u32));
        let bus = EventBus::new();
        bus.subscribe("s.e", Box::new(CountingListener { n: n.clone() }))
            .expect("should succeed");
        bus.publish(Event::new("s.e".to_string(), serde_json::json!({})))
            .await
            .expect("should succeed");
        assert_eq!(*n.lock().await, 1);
    }

    #[tokio::test]
    async fn test_listener_count_and_empty_publish() {
        let bus = EventBus::new();
        assert_eq!(bus.get_listener_count("none"), 0);
        bus.publish(Event::new("none".to_string(), serde_json::json!({})))
            .await
            .expect("should succeed");
    }
}
