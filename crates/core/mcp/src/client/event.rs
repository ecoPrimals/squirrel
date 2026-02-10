// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Event handling for MCP clients
//!
//! This module provides event handling capabilities including the EventHandler trait
//! and composite event handlers for managing multiple event types.

use crate::error::Result;
use crate::message::Message;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Trait for handling events from MCP servers
pub trait EventHandler: Send + Sync {
    /// Handle an event message
    fn handle_event<'a>(&'a self, event: &'a Message) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
    
    /// Get the event types this handler can process
    fn supported_event_types(&self) -> Vec<String>;
}

/// A composite event handler that manages multiple event handlers
pub struct CompositeEventHandler {
    /// Mapping of event type to handlers
    handlers: HashMap<String, Vec<Arc<dyn EventHandler>>>,
}

impl CompositeEventHandler {
    /// Create a new composite event handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Add a handler for specific event types
    pub fn add_handler(&mut self, event_type: String, handler: Arc<dyn EventHandler>) {
        self.handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Add a handler for all its supported event types
    pub fn add_handler_for_all_types(&mut self, handler: Arc<dyn EventHandler>) {
        for event_type in handler.supported_event_types() {
            self.add_handler(event_type, handler.clone());
        }
    }

    /// Remove all handlers for a specific event type
    pub fn remove_handlers(&mut self, event_type: &str) {
        self.handlers.remove(event_type);
    }

    /// Get the number of handlers for a specific event type
    pub fn handler_count(&self, event_type: &str) -> usize {
        self.handlers.get(event_type).map(|h| h.len()).unwrap_or(0)
    }

    /// Get all registered event types
    pub fn registered_event_types(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

impl Default for CompositeEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler for CompositeEventHandler {
    fn handle_event<'a>(&'a self, event: &'a Message) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(method) = &event.method {
                if let Some(handlers) = self.handlers.get(method) {
                    for handler in handlers {
                        if let Err(error) = handler.handle_event(event).await {
                            tracing::error!("Event handler error for '{}': {}", method, error);
                        }
                    }
                }
            }
            Ok(())
        })
    }

    fn supported_event_types(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

/// A simple event handler that just logs events
pub struct LoggingEventHandler {
    event_types: Vec<String>,
}

impl LoggingEventHandler {
    /// Create a new logging event handler for specific event types
    pub fn new(event_types: Vec<String>) -> Self {
        Self { event_types }
    }

    /// Create a logging event handler for all events
    pub fn for_all_events() -> Self {
        Self {
            event_types: vec!["*".to_string()],
        }
    }
}

impl EventHandler for LoggingEventHandler {
    fn handle_event<'a>(&'a self, event: &'a Message) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(method) = &event.method {
                tracing::info!("Received event '{}': {:?}", method, event);
            } else {
                tracing::info!("Received event: {:?}", event);
            }
            Ok(())
        })
    }

    fn supported_event_types(&self) -> Vec<String> {
        self.event_types.clone()
    }
}

/// A channel-based event handler that publishes events to a channel
pub struct ChannelEventHandler {
    event_type: String,
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

impl ChannelEventHandler {
    /// Create a new channel event handler
    pub fn new(event_type: String, sender: tokio::sync::mpsc::UnboundedSender<Message>) -> Self {
        Self {
            event_type,
            sender,
        }
    }
}

impl EventHandler for ChannelEventHandler {
    fn handle_event<'a>(&'a self, event: &'a Message) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if let Err(error) = self.sender.send(event.clone()) {
                tracing::error!("Failed to send event to channel: {}", error);
            }
            Ok(())
        })
    }

    fn supported_event_types(&self) -> Vec<String> {
        vec![self.event_type.clone()]
    }
}

/// A filtering event handler that only processes events matching certain criteria
pub struct FilteringEventHandler {
    event_types: Vec<String>,
    filter: Box<dyn Fn(&Message) -> bool + Send + Sync>,
    inner_handler: Arc<dyn EventHandler>,
}

impl FilteringEventHandler {
    /// Create a new filtering event handler
    pub fn new<F>(
        event_types: Vec<String>,
        filter: F,
        inner_handler: Arc<dyn EventHandler>,
    ) -> Self
    where
        F: Fn(&Message) -> bool + Send + Sync + 'static,
    {
        Self {
            event_types,
            filter: Box::new(filter),
            inner_handler,
        }
    }
}

impl EventHandler for FilteringEventHandler {
    fn handle_event<'a>(&'a self, event: &'a Message) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if (self.filter)(event) {
                self.inner_handler.handle_event(event).await
            } else {
                Ok(())
            }
        })
    }

    fn supported_event_types(&self) -> Vec<String> {
        self.event_types.clone()
    }
}

/// A batching event handler that collects events and processes them in batches
pub struct BatchingEventHandler {
    event_types: Vec<String>,
    batch_size: usize,
    batch_timeout: tokio::time::Duration,
    inner_handler: Arc<dyn EventHandler>,
    batch_sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

impl BatchingEventHandler {
    /// Create a new batching event handler
    pub fn new(
        event_types: Vec<String>,
        batch_size: usize,
        batch_timeout: tokio::time::Duration,
        inner_handler: Arc<dyn EventHandler>,
    ) -> Self {
        let (batch_sender, mut batch_receiver) = tokio::sync::mpsc::unbounded_channel();
        
        // Spawn batch processing task
        let handler = inner_handler.clone();
        tokio::spawn(async move {
            let mut batch = Vec::new();
            let mut interval = tokio::time::interval(batch_timeout);
            
            loop {
                tokio::select! {
                    message = batch_receiver.recv() => {
                        match message {
                            Some(msg) => {
                                batch.push(msg);
                                if batch.len() >= batch_size {
                                    Self::process_batch(&handler, &mut batch).await;
                                }
                            }
                            None => break,
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            Self::process_batch(&handler, &mut batch).await;
                        }
                    }
                }
            }
        });

        Self {
            event_types,
            batch_size,
            batch_timeout,
            inner_handler,
            batch_sender,
        }
    }

    async fn process_batch(handler: &Arc<dyn EventHandler>, batch: &mut Vec<Message>) {
        for event in batch.drain(..) {
            if let Err(error) = handler.handle_event(&event).await {
                tracing::error!("Batch event handler error: {}", error);
            }
        }
    }
}

impl EventHandler for BatchingEventHandler {
    fn handle_event<'a>(&'a self, event: &'a Message) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if let Err(error) = self.batch_sender.send(event.clone()) {
                tracing::error!("Failed to send event to batch queue: {}", error);
            }
            Ok(())
        })
    }

    fn supported_event_types(&self) -> Vec<String> {
        self.event_types.clone()
    }
} 