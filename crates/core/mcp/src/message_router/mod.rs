// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Message Router Module
//! 
//! This module provides the implementation of a message router that directs
//! messages to appropriate handlers based on message type and content.
//! It supports registering multiple handlers for different message types,
//! as well as priority-based message handling.

use std::collections::HashMap;
use std::sync::Arc;
use std::future::Future;
use tokio::sync::RwLock;
use tracing::error;

use crate::message::Message;

/// Result type for message handler operations
pub type MessageHandlerResult = crate::error::Result<Option<Message>>;

/// Errors that can occur during message routing
#[derive(Debug, Clone)]
pub enum MessageRouterError {
    /// No handler found for message type
    NoHandlerFound(String),
    
    /// Message validation failed
    ValidationFailed(String),
    
    /// Configuration error
    ConfigurationError(String),
    
    /// Handler execution error
    HandlerError(String),
}

impl std::error::Error for MessageRouterError {}

impl std::fmt::Display for MessageRouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHandlerFound(message_type) => {
                write!(f, "No handler found for message type: {message_type}")
            }
            Self::ValidationFailed(msg) => {
                write!(f, "Message validation failed: {msg}")
            }
            Self::ConfigurationError(msg) => {
                write!(f, "Message router configuration error: {msg}")
            }
            Self::HandlerError(msg) => {
                write!(f, "Handler execution error: {msg}")
            }
        }
    }
}

/// Priority levels for message handlers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HandlerPriority {
    /// Lowest priority handlers
    Low = 0,
    
    /// Medium priority handlers
    Medium = 1,
    
    /// High priority handlers
    High = 2,
    
    /// System-level critical handlers
    System = 3,
}

impl Default for HandlerPriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Trait for handling messages asynchronously
pub trait AsyncMessageHandler: Send + Sync {
    /// Handle a message and optionally return a response
    fn handle_message(&self, message: Message) -> impl Future<Output = MessageHandlerResult> + Send;
}

/// Handler trait for processing messages
pub trait MessageHandler: Send + Sync + std::fmt::Debug {
    /// Handle a message and optionally return a response
    /// This method needs to be async.
    fn handle_message(&self, message: Message) -> impl Future<Output = MessageHandlerResult> + Send;

    /// Get the message types this handler can process
    fn supported_message_types(&self) -> Vec<String>;

    /// Get the priority of this handler
    fn priority(&self) -> HandlerPriority {
        HandlerPriority::default()
    }

    /// Get a unique identifier for this handler
    fn id(&self) -> Option<String> {
        None
    }

    /// Check if this handler can handle a specific message
    fn can_handle(&self, message: &Message) -> bool {
        self.supported_message_types().contains(&message.message_type.to_string())
    }
}

/// Configuration for the message router
#[derive(Debug, Clone)]
pub struct MessageRouterConfig {
    /// Whether to continue processing after a handler returns a response
    pub continue_after_response: bool,
    /// Maximum number of handlers that can be called for a single message
    pub max_handler_calls: usize,
    /// Whether to validate messages before routing
    pub validate_messages: bool,
}

impl Default for MessageRouterConfig {
    fn default() -> Self {
        Self {
            continue_after_response: false,
            max_handler_calls: 10,
            validate_messages: true,
        }
    }
}

/// The router for dispatching messages to appropriate handlers
#[derive(Debug)]
pub struct MessageRouter {
    /// Handlers organized by message type and priority
    handlers: Arc<RwLock<HashMap<String, HashMap<HandlerPriority, Vec<Arc<dyn MessageHandler + Send + Sync>>>>>>,
    /// Router configuration
    config: MessageRouterConfig,
}

impl MessageRouter {
    /// Create a new message router with default configuration
    #[must_use] pub fn new() -> Self {
        Self::with_config(MessageRouterConfig::default())
    }

    /// Create a new message router with custom configuration
    #[must_use] pub fn with_config(config: MessageRouterConfig) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register a message handler for specific message types
    ///
    /// # Arguments
    /// * `handler` - The handler to register
    ///
    /// # Errors
    /// * Returns `MCPError::MessageRouter` if the handler doesn't support any message types
    pub async fn register_handler<T>(&self, handler: Arc<T>) -> crate::error::Result<()> 
    where 
        T: MessageHandler + Send + Sync + 'static
    {
        let message_types = handler.supported_message_types();
        
        // Check if handler supports any message types
        if message_types.is_empty() {
            return Err(crate::error::MCPError::MessageRouter(
                MessageRouterError::ConfigurationError("Handler does not support any message types".to_string())
            ).into());
        }

        // Acquire the lock once and modify
        let mut handlers_guard = self.handlers.write().await;
        
        // Register handler for each supported message type
        for message_type in message_types {
            let handlers_map = handlers_guard
                .entry(message_type)
                .or_insert_with(HashMap::new);
            
            let handler_priority = handler.priority();
            
            handlers_map
                .entry(handler_priority)
                .or_insert_with(Vec::new)
                .push(handler.clone());
        }

        drop(handlers_guard);
        Ok(())
    }

    /// Unregister a message handler
    ///
    /// # Arguments
    /// * `handler` - The handler to unregister
    ///
    /// # Errors
    /// * Returns `MCPError::MessageRouter` if the handler cannot be unregistered
    pub async fn unregister_handler<T>(&self, handler: &Arc<T>) -> crate::error::Result<()> 
    where 
        T: MessageHandler + Send + Sync + 'static
    {
        let message_types = handler.supported_message_types();
        let handler_id = handler.id();
        
        // Get write lock for handlers map
        let mut handlers_guard = self.handlers.write().await;
        
        // Process each message type the handler supports
        for message_type in message_types {
            if let Some(handlers_map) = handlers_guard.get_mut(&message_type) {
                // Only attempt to remove handlers with matching ID if handler has an ID
                if let Some(handler_id) = &handler_id {
                    // Get a cloned ID for the closure
                    let handler_id_clone = handler_id.clone();
                    
                    // Remove handlers with matching ID from all priority levels
                    for handlers_vec in handlers_map.values_mut() {
                        handlers_vec.retain(|h| {
                            h.id().map_or(true, |id| id != handler_id_clone)
                        });
                    }
                }
                
                // Remove any empty priority maps
                handlers_map.retain(|_, handlers_vec| !handlers_vec.is_empty());
            }
        }
        
        // Clean up message types that have no handlers
        handlers_guard.retain(|_, handlers_map| !handlers_map.is_empty());
        
        drop(handlers_guard);
        Ok(())
    }

    /// Route a message to appropriate handlers based on message type
    ///
    /// This method routes a message to registered handlers based on the message type.
    /// Handlers are tried in priority order (highest first) until one returns a response
    /// or all have been tried.
    ///
    /// # Arguments
    /// * `message` - The message to route
    ///
    /// # Returns
    /// * `Ok(Some(Message))` - If a handler processed the message and returned a response
    /// * `Ok(None)` - If a handler processed the message but didn't return a response
    /// * `Err(MCPError)` - If an error occurred during routing
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * Message validation fails (`MCPError::MessageRouter` with `ValidationFailed` variant)
    /// * No handler is found for the message type (`MCPError::MessageRouter` with `NoHandlerFound` variant)
    /// * A handler error occurs that can't be handled internally
    pub async fn route_message(&self, message: &Message) -> MessageHandlerResult {
        // Validate the message first
        self.validate_message(message).await?;
        
        // Get the message type as a string
        let message_type_str = message.message_type.to_string();
        let continue_after_response = self.config.continue_after_response;
        
        let handlers = self.handlers.read().await;
        
        // Variable to store the highest priority response
        let mut highest_priority_response: Option<Message> = None;
        let mut any_handler_called = false;
        
        // Try standard and wildcard handlers
        for lookup_key in [&message_type_str, "any"] {
            if let Some(handlers_map) = handlers.get(lookup_key) {
                // Sort priorities (highest first)
                let mut priorities: Vec<_> = handlers_map.keys().collect();
                priorities.sort_by(|a, b| b.cmp(a));
                
                // Try handlers in priority order
                'priority_loop: for priority in priorities {
                    if let Some(handlers_vec) = handlers_map.get(priority) {
                        for handler in handlers_vec {
                            any_handler_called = true;
                            
                            // Try with this handler, but with a timeout
                            let handle_future = handler.handle_message(message.clone());
                            match tokio::time::timeout(std::time::Duration::from_secs(5), handle_future).await {
                                Ok(Ok(Some(response))) => {
                                    // Store the response if no higher priority response is stored yet
                                    if highest_priority_response.is_none() {
                                        highest_priority_response = Some(response);
                                    }
                                    
                                    // If we're not continuing after response, break out of processing
                                    if !continue_after_response {
                                        break 'priority_loop;
                                    }
                                }
                                Ok(Ok(None)) => continue, // Handler chose not to handle
                                Ok(Err(e)) => {
                                    error!("Handler error for message '{}': {}", message.id, e);
                                    continue; // Try the next handler
                                }
                                Err(_) => {
                                    // Timeout occurred
                                    error!("Handler timed out for message '{}'", message.id);
                                    continue; // Try the next handler
                                }
                            }
                        }
                    }
                }
                
                // If we found a response and we're not continuing, stop here
                if highest_priority_response.is_some() && !continue_after_response {
                    break;
                }
            }
        }
        
        drop(handlers);
        
        // If any handler was called and we have a response, return it
        if any_handler_called && highest_priority_response.is_some() {
            return Ok(highest_priority_response);
        }
        
        // If we're here, no handler actually handled the message
        Err(crate::error::MCPError::MessageRouter(
            MessageRouterError::NoHandlerFound(format!("No handler found for message type: {}", message_type_str))
        ).into())
    }

    /// Validate the message structure before routing
    ///
    /// # Arguments
    /// * `message` - The message to validate
    ///
    /// # Errors
    /// * Returns `MCPError::MessageRouter` with `ValidationFailed` variant if validation fails
    async fn validate_message(&self, message: &Message) -> crate::error::Result<()> {
        // Check if ID is empty
        if message.id.is_empty() {
            return Err(crate::error::MCPError::MessageRouter(
                MessageRouterError::ValidationFailed("Message ID cannot be empty".to_string())
            ).into());
        }
        
        // More validations could be added here
        
        Ok(())
    }

    /// Get all registered message types
    pub async fn get_registered_message_types(&self) -> Vec<String> {
        let handlers_guard = self.handlers.read().await;
        let message_types = handlers_guard.keys().cloned().collect();
        drop(handlers_guard);
        message_types
    }

    /// Get handler count for a specific message type
    pub async fn get_handler_count(&self, message_type: &str) -> usize {
        let handlers_guard = self.handlers.read().await;
        let count = handlers_guard
            .get(message_type)
            .map_or(0, |handlers_map| 
                handlers_map.values()
                    .map(Vec::len)
                    .sum()
            );
        drop(handlers_guard);
        count
    }
}

/// A composite message handler that delegates to multiple inner handlers
#[derive(Debug)]
pub struct CompositeHandler {
    /// Inner handlers
    handlers: Vec<Arc<dyn MessageHandler + Send + Sync>>,
    /// Supported message types (union of all inner handlers)
    message_types: Vec<String>,
    /// Handler priority
    priority: HandlerPriority,
}

impl CompositeHandler {
    /// Create a new composite handler with the specified priority
    #[must_use] pub fn new(priority: HandlerPriority) -> Self {
        Self {
            handlers: Vec::new(),
            message_types: Vec::new(),
            priority,
        }
    }

    /// Add a handler to the composite
    pub fn add_handler(&mut self, handler: Arc<dyn MessageHandler + Send + Sync>) {
        // Add new supported message types
        for message_type in handler.supported_message_types() {
            if !self.message_types.contains(&message_type) {
                self.message_types.push(message_type);
            }
        }
        
        self.handlers.push(handler);
    }
}

impl AsyncMessageHandler for CompositeHandler {
    fn handle_message(&self, message: Message) -> impl Future<Output = MessageHandlerResult> + Send {
        let handlers = self.handlers.clone();
        async move {
            // Try each handler in sequence
            for handler in &handlers {
                if handler.can_handle(&message) {
                    let handle_future = handler.handle_message(message.clone());
                    match tokio::time::timeout(std::time::Duration::from_secs(5), handle_future).await {
                        Ok(Ok(Some(response))) => return Ok(Some(response)),
                        Ok(Ok(None)) => continue, // Try the next handler
                        Ok(Err(e)) => {
                            error!("Handler error in composite: {}", e);
                            continue;
                        },
                        Err(_) => {
                            error!("Handler timed out in composite");
                            continue; // Try the next handler
                        }
                    }
                }
            }
            
            // No handler processed the message
            Ok(None)
        }
    }
}

impl MessageHandler for CompositeHandler {
    fn handle_message(&self, message: Message) -> impl Future<Output = MessageHandlerResult> + Send {
        let handlers = self.handlers.clone();
        async move {
            // Iterate through handlers and try to handle the message
            // This needs more sophisticated logic based on priority, configuration etc.
            for handler in &handlers {
                // Check if the handler supports the type before calling
                if handler.supported_message_types().contains(&message.message_type.to_string()) {
                    let handle_future = handler.handle_message(message.clone());
                    match tokio::time::timeout(std::time::Duration::from_secs(5), handle_future).await {
                        Ok(Ok(Some(response))) => return Ok(Some(response)), // Return first response
                        Ok(Ok(None)) => continue, // Try next handler
                        Ok(Err(e)) => return Err(e), // Propagate error
                        Err(_) => {
                            error!("Handler timed out in composite");
                            continue; // Try the next handler
                        }
                    }
                }
            }
            // If no handler produced a response
            Ok(None)
        }
    }

    fn supported_message_types(&self) -> Vec<String> {
        self.message_types.clone()
    }
    
    fn priority(&self) -> HandlerPriority {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crate::message::MessageBuilder;

    /// Mock handler for testing
    #[derive(Debug)]
    struct MockHandler {
        message_types: Vec<String>,
        priority: HandlerPriority,
        response: Option<Message>,
        call_count: Arc<AtomicUsize>,
    }
    
    impl MockHandler {
        fn new(
            message_types: Vec<String>,
            priority: HandlerPriority,
            response: Option<Message>,
        ) -> Self {
            Self {
                message_types,
                priority,
                response,
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }
        
        fn get_call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }
    
    impl AsyncMessageHandler for MockHandler {
        fn handle_message(&self, _message: Message) -> impl Future<Output = MessageHandlerResult> + Send {
            let call_count = self.call_count.clone();
            let response = self.response.clone();
            async move {
                call_count.fetch_add(1, Ordering::SeqCst);
                Ok(response)
            }
        }
    }
    
    impl MessageHandler for MockHandler {
        fn supported_message_types(&self) -> Vec<String> {
            self.message_types.clone()
        }
        
        fn priority(&self) -> HandlerPriority {
            self.priority
        }
        
        fn handle_message(&self, _message: Message) -> impl Future<Output = MessageHandlerResult> + Send {
            let call_count = self.call_count.clone();
            let response = self.response.clone();
            async move {
                call_count.fetch_add(1, Ordering::SeqCst);
                Ok(response)
            }
        }
    }
    
    #[tokio::test]
    async fn test_register_handler() {
        let router = MessageRouter::new();
        let handler = Arc::new(MockHandler::new(
            vec!["test-type".to_string()],
            HandlerPriority::Medium,
            None,
        ));
        
        let result = router.register_handler(handler.clone()).await;
        assert!(result.is_ok());
        
        let count = router.get_handler_count("test-type").await;
        assert_eq!(count, 1);
    }
    
    #[tokio::test]
    async fn test_route_message_no_handler() {
        let router = MessageRouter::new();
        let message = MessageBuilder::new()
            .with_message_type("command")
            .with_content_str("test content")
            .with_source("test")
            .with_destination("test")
            .build();
        
        let result = router.route_message(&message).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_route_message_with_handler() {
        let router = MessageRouter::new();
        let response = MessageBuilder::new()
            .with_message_type("response")
            .with_content_str("response content")
            .with_source("test")
            .with_destination("test")
            .build();
        
        let handler = Arc::new(MockHandler::new(
            vec!["request".to_string()],
            HandlerPriority::Medium,
            Some(response.clone()),
        ));
        
        let _ = router.register_handler(handler).await;
        
        let message = MessageBuilder::new()
            .with_message_type("request")
            .with_content_str("test content")
            .with_source("test")
            .with_destination("test")
            .build();
        
        let result = router.route_message(&message).await;
        assert!(result.is_ok());
        
        let received_response = result.unwrap();
        assert!(received_response.is_some());
        assert_eq!(received_response.unwrap().content, "response content");
    }
    
    #[tokio::test]
    async fn test_handler_priority() {
        let router = MessageRouter::new();
        
        // Create handlers with different priorities
        let low_priority_handler = Arc::new(MockHandler::new(
            vec!["notification".to_string()],
            HandlerPriority::Low,
            Some(MessageBuilder::new()
                .with_message_type("response")
                .with_content_str("low priority")
                .with_source("test")
                .with_destination("test")
                .build()),
        ));
        
        let high_priority_handler = Arc::new(MockHandler::new(
            vec!["notification".to_string()],
            HandlerPriority::High,
            Some(MessageBuilder::new()
                .with_message_type("response")
                .with_content_str("high priority")
                .with_source("test")
                .with_destination("test")
                .build()),
        ));
        
        // Register handlers
        let _ = router.register_handler(low_priority_handler.clone()).await;
        let _ = router.register_handler(high_priority_handler.clone()).await;
        
        // Create a message
        let message = MessageBuilder::new()
            .with_message_type("notification")
            .with_content_str("test content")
            .with_source("test")
            .with_destination("test")
            .build();
        
        // Route the message
        let result = router.route_message(&message).await;
        assert!(result.is_ok());
        
        let received_response = result.unwrap();
        assert!(received_response.is_some());
        
        // Should get the high priority handler's response
        assert_eq!(received_response.unwrap().content, "high priority");
        
        // High priority handler should have been called, but not the low priority one
        assert_eq!(high_priority_handler.get_call_count(), 1);
        assert_eq!(low_priority_handler.get_call_count(), 0);
    }
    
    #[tokio::test]
    async fn test_continue_after_response() {
        let mut config = MessageRouterConfig::default();
        config.continue_after_response = true;
        let router = MessageRouter::with_config(config);
        
        // Create handlers with different priorities
        let low_priority_handler = Arc::new(MockHandler::new(
            vec!["notification".to_string()],
            HandlerPriority::Low,
            Some(MessageBuilder::new()
                .with_message_type("response")
                .with_content_str("low priority")
                .with_source("test")
                .with_destination("test")
                .build()),
        ));
        
        let high_priority_handler = Arc::new(MockHandler::new(
            vec!["notification".to_string()],
            HandlerPriority::High,
            Some(MessageBuilder::new()
                .with_message_type("response")
                .with_content_str("high priority")
                .with_source("test")
                .with_destination("test")
                .build()),
        ));
        
        // Register handlers
        match router.register_handler(low_priority_handler.clone()).await {
            Ok(_) => println!("Low priority handler registered successfully"),
            Err(e) => panic!("Failed to register low priority handler: {}", e),
        }
        
        match router.register_handler(high_priority_handler.clone()).await {
            Ok(_) => println!("High priority handler registered successfully"),
            Err(e) => panic!("Failed to register high priority handler: {}", e),
        }
        
        // Verify both handlers are registered
        let handler_count = router.get_handler_count("notification").await;
        println!("Handler count for 'notification': {}", handler_count);
        assert_eq!(handler_count, 2, "Expected 2 handlers to be registered");
        
        // Create a message with the correct type that matches what handlers were registered for
        let message = MessageBuilder::new()
            .with_message_type("notification")
            .with_content_str("test content")
            .with_source("test")
            .with_destination("test")
            .build();
        
        println!("Message type: {}", message.message_type);
        println!("Starting route_message call...");
        
        // Route the message
        let result = router.route_message(&message).await;
        match &result {
            Ok(response) => println!("Got response: {:?}", response),
            Err(e) => println!("Error from route_message: {}", e),
        }
        assert!(result.is_ok(), "Expected route_message to return Ok, got: {:?}", result);
        
        let received_response = result.unwrap();
        assert!(received_response.is_some(), "Expected a response from a handler");
        
        // Should still get the high priority handler's response
        let response_content = received_response.unwrap().content;
        println!("Response content: {}", response_content);
        assert_eq!(response_content, "high priority", "Expected high priority response");
        
        // Both handlers should have been called
        let high_count = high_priority_handler.get_call_count();
        let low_count = low_priority_handler.get_call_count();
        println!("High priority handler call count: {}", high_count);
        println!("Low priority handler call count: {}", low_count);
        assert_eq!(high_count, 1, "High priority handler should be called exactly once");
        assert_eq!(low_count, 1, "Low priority handler should be called exactly once");
    }
    
    #[tokio::test]
    async fn test_composite_handler() {
        let mut composite = CompositeHandler::new(HandlerPriority::Medium);
        
        let handler1 = Arc::new(MockHandler::new(
            vec!["request".to_string()],
            HandlerPriority::Medium,
            Some(MessageBuilder::new()
                .with_message_type("response")
                .with_content_str("handler1 response")
                .with_source("test")
                .with_destination("test")
                .build()),
        ));
        
        let handler2 = Arc::new(MockHandler::new(
            vec!["notification".to_string()],
            HandlerPriority::Medium,
            Some(MessageBuilder::new()
                .with_message_type("response")
                .with_content_str("handler2 response")
                .with_source("test")
                .with_destination("test")
                .build()),
        ));
        
        composite.add_handler(handler1.clone());
        composite.add_handler(handler2.clone());
        
        // Verify supported message types
        let types = composite.supported_message_types();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"request".to_string()));
        assert!(types.contains(&"notification".to_string()));
        
        // Test request message
        let message1 = MessageBuilder::new()
            .with_message_type("request")
            .with_content_str("test content")
            .with_source("test")
            .with_destination("test")
            .build();
        
        // Use the MessageHandler trait implementation specifically
        let result1 = MessageHandler::handle_message(&composite, message1).await;
        assert!(result1.is_ok());
        
        let response1 = result1.unwrap();
        assert!(response1.is_some());
        assert_eq!(response1.unwrap().content, "handler1 response");
        
        // Test notification message
        let message2 = MessageBuilder::new()
            .with_message_type("notification")
            .with_content_str("test content")
            .with_source("test")
            .with_destination("test")
            .build();
        
        // Use the MessageHandler trait implementation specifically
        let result2 = MessageHandler::handle_message(&composite, message2).await;
        assert!(result2.is_ok());
        
        let response2 = result2.unwrap();
        assert!(response2.is_some());
        assert_eq!(response2.unwrap().content, "handler2 response");
        
        // Test unknown message type (but still valid in the system)
        let message3 = MessageBuilder::new()
            .with_message_type("error")
            .with_content_str("test content")
            .with_source("test")
            .with_destination("test")
            .build();
        
        // Use the MessageHandler trait implementation specifically
        let result3 = MessageHandler::handle_message(&composite, message3).await;
        assert!(result3.is_ok());
        
        let response3 = result3.unwrap();
        assert!(response3.is_none());
    }
} 