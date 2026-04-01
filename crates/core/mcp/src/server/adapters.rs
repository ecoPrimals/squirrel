// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Handler Adapters
//!
//! Adapters for integrating command handlers with message routing.

use serde_json::json;
use std::sync::Arc;

use crate::error::Result;
use crate::message::{Message, MessageBuilder};
use crate::message_router::{MessageRouter, HandlerPriority};
use super::handlers::CommandHandler;

/// Router-based command handler
#[derive(Debug)]
pub struct RouterCommandHandler {
    /// Message router
    router: Arc<MessageRouter>,
}

impl RouterCommandHandler {
    /// Create a new router command handler
    #[must_use]
    pub const fn new(router: Arc<MessageRouter>) -> Self {
        Self { router }
    }
}

impl CommandHandler for RouterCommandHandler {
    fn handle_command<'a>(
        &'a self, 
        command: &'a Message
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Message>>> + Send + 'a>> {
        Box::pin(async move {
            // Use the message router to handle the command
            self.router.route_message(command).await
                .map(|response| {
                    // Map the Option<Message> to Some(Message)
                    Some(response.unwrap_or_else(|| {
                        // Create a default success response when no handler returns a response
                        MessageBuilder::new()
                            .with_message_type("response")
                            .with_correlation_id(command.id.clone())
                            .with_content(json!({"status": "success"}))
                            .build()
                    }))
                })
                .or_else(|err| {
                    // Try to convert err to MCPError if necessary
                    let err_str = err.to_string();
                    
                    // Check if the error message indicates a "no handler found" error
                    if err_str.contains("No handler found for message type") {
                        // Extract the message type from the error string if possible
                        let msg_type = err_str
                            .split("No handler found for message type")
                            .nth(1)
                            .map(|s| s.trim_start_matches(':').trim())
                            .unwrap_or("unknown");
                            
                        tracing::warn!("No handler found for message type: {msg_type}");
                        
                        // Create a default success response when no handler found
                        Ok(Some(MessageBuilder::new()
                            .with_message_type("response")
                            .with_correlation_id(command.id.clone())
                            .with_content(json!({"status": "success", "message": format!("No handler found for {}", msg_type)}))
                            .build()))
                    } else {
                        // Pass through other errors
                        Err(err)
                    }
                })
        })
    }
    
    fn supported_commands(&self) -> Vec<String> {
        // Get all message types supported by the router
        futures::executor::block_on(self.router.get_registered_message_types())
    }

    fn clone_box(&self) -> Box<dyn CommandHandler> {
        Box::new(Self {
            router: self.router.clone()
        })
    }
}

/// Adapter that converts a `CommandHandler` to a `MessageHandler`
#[derive(Debug)]
pub struct CommandHandlerAdapter {
    /// Inner command handler
    handler: Box<dyn CommandHandler>,
    /// Priority
    priority: HandlerPriority,
}

impl CommandHandlerAdapter {
    /// Create a new command handler adapter
    #[must_use]
    pub fn new(handler: Box<dyn CommandHandler>) -> Self {
        Self {
            handler,
            priority: HandlerPriority::Medium,
        }
    }
    
    /// Create a new command handler adapter with custom priority
    #[must_use]
    pub fn with_priority(handler: Box<dyn CommandHandler>, priority: HandlerPriority) -> Self {
        Self {
            handler,
            priority,
        }
    }
}

impl CommandHandler for CommandHandlerAdapter {
    fn handle_command<'a>(
        &'a self, 
        command: &'a Message
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Message>>> + Send + 'a>> {
        Box::pin(async move {
            // Use the command handler to get the response
            self.handler.handle_command(command).await
                .map(|maybe_response| {
                    // If we got a response, use it; otherwise create a default response
                    Some(maybe_response.unwrap_or_else(|| {
                        // Create a default success response
                        MessageBuilder::new()
                            .with_message_type("response")
                            .with_correlation_id(command.id.clone())
                            .with_content(json!({"status": "success"}))
                            .build()
                    }))
                })
        })
    }
    
    fn supported_commands(&self) -> Vec<String> {
        self.handler.supported_commands()
    }

    fn clone_box(&self) -> Box<dyn CommandHandler> {
        Box::new(Self {
            handler: self.handler.clone_box(),
            priority: self.priority,
        })
    }
}

