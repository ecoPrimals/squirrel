// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Session management for MCP clients
//!
//! This module handles session creation, message sending/receiving, event processing, and task management.

use crate::error::{MCPError, Result};
use crate::message::{Message, MessageType};
use crate::session::Session;
use crate::transport::Transport;
use crate::client::config::ClientConfig;
use crate::client::connection::ConnectionManager;
use crate::client::event::EventHandler;

use futures::future::{AbortHandle, Abortable};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Session manager for MCP clients
pub struct SessionManager {
    /// Configuration
    config: ClientConfig,
    /// Connection manager
    connection_manager: Arc<ConnectionManager>,
    /// Current session information
    session: Arc<RwLock<Option<Session>>>,
    /// Message channel sender
    message_tx: mpsc::Sender<Message>,
    /// Message channel receiver
    message_rx: Arc<RwLock<Option<mpsc::Receiver<Message>>>>,
    /// Event subscription channel
    event_channel: Arc<broadcast::Sender<Option<Message>>>,
    /// Map of pending request IDs to response channels
    pending_requests: Arc<RwLock<HashMap<String, oneshot::Sender<Result<Message>>>>>,
    /// Map of event topics to event handlers
    event_handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    /// Message processing task handle
    message_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Reader task handle
    reader_task: Arc<Mutex<Option<AbortHandle>>>,
    /// Cancellation token for graceful shutdown
    cancellation_token: tokio_util::sync::CancellationToken,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: ClientConfig, connection_manager: Arc<ConnectionManager>) -> Self {
        let (message_tx, message_rx) = mpsc::channel(1000);
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            config,
            connection_manager,
            session: Arc::new(RwLock::new(None)),
            message_tx,
            message_rx: Arc::new(RwLock::new(Some(message_rx))),
            event_channel: Arc::new(event_tx),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            message_task: Arc::new(RwLock::new(None)),
            reader_task: Arc::new(Mutex::new(None)),
            cancellation_token: tokio_util::sync::CancellationToken::new(),
        }
    }

    /// Get the current session
    pub async fn get_session(&self) -> Option<Session> {
        self.session.read().await.clone()
    }

    /// Set the current session
    pub async fn set_session(&self, session: Option<Session>) {
        let mut session_guard = self.session.write().await;
        *session_guard = session;
    }

    /// Start the session by setting up message processing
    pub async fn start(&self) -> Result<()> {
        // Start message processing task
        self.start_message_processing().await?;
        
        // Start reader task
        self.start_reader_task().await?;
        
        Ok(())
    }

    /// Stop the session
    pub async fn stop(&self) -> Result<()> {
        // Cancel all tasks
        self.cancellation_token.cancel();
        
        // Stop message processing task
        let task = {
            let mut task_guard = self.message_task.write().await;
            task_guard.take()
        };
        
        if let Some(task) = task {
            task.abort();
            let _ = task.await;
        }
        
        // Stop reader task
        let reader_handle = {
            let mut reader_guard = self.reader_task.lock().await;
            reader_guard.take()
        };
        
        if let Some(handle) = reader_handle {
            handle.abort();
        }
        
        Ok(())
    }

    /// Send a command and wait for response
    pub async fn send_command(&self, command: &Message) -> Result<Message> {
        // Validate connection
        if !self.connection_manager.is_connected().await {
            return Err(MCPError::ConnectionError("Client not connected".to_string()));
        }

        // Get transport
        let transport = self.connection_manager.get_transport().await
            .ok_or_else(|| MCPError::ConnectionError("No transport available".to_string()))?;

        // Create response channel
        let (response_tx, response_rx) = oneshot::channel();
        let request_id = command.id.clone();

        // Store pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request_id.clone(), response_tx);
        }

        // Send message
        let send_result = transport.send_message(command).await;

        // Handle send error
        if let Err(error) = send_result {
            // Remove pending request
            let mut pending = self.pending_requests.write().await;
            pending.remove(&request_id);
            return Err(error);
        }

        // Wait for response with timeout
        let response = timeout(self.config.request_timeout, response_rx)
            .await
            .map_err(|_| MCPError::TimeoutError("Request timeout".to_string()))?
            .map_err(|_| MCPError::ProtocolError("Response channel closed".to_string()))?;

        response
    }

    /// Send a command with content
    pub async fn send_command_with_content<T>(&self, command_name: &str, content: T) -> Result<Message>
    where
        T: Into<serde_json::Value>,
    {
        let command = Message {
            id: Uuid::new_v4().to_string(),
            message_type: MessageType::Request,
            method: Some(command_name.to_string()),
            params: Some(content.into()),
            result: None,
            error: None,
        };

        self.send_command(&command).await
    }

    /// Send an event
    pub async fn send_event(&self, event: &Message) -> Result<()> {
        // Validate connection
        if !self.connection_manager.is_connected().await {
            return Err(MCPError::ConnectionError("Client not connected".to_string()));
        }

        // Get transport
        let transport = self.connection_manager.get_transport().await
            .ok_or_else(|| MCPError::ConnectionError("No transport available".to_string()))?;

        // Send event
        transport.send_message(event).await?;

        // Publish to event channel
        if let Err(error) = self.event_channel.send(Some(event.clone())) {
            warn!("Failed to publish event to channel: {}", error);
        }

        Ok(())
    }

    /// Send an event with content
    pub async fn send_event_with_content<T>(&self, event_name: &str, content: T) -> Result<()>
    where
        T: Into<serde_json::Value>,
    {
        let event = Message {
            id: Uuid::new_v4().to_string(),
            message_type: MessageType::Notification,
            method: Some(event_name.to_string()),
            params: Some(content.into()),
            result: None,
            error: None,
        };

        self.send_event(&event).await
    }

    /// Register an event handler
    pub async fn register_event_handler(&self, handler: Arc<dyn EventHandler>) -> Result<()> {
        let mut handlers = self.event_handlers.write().await;
        
        for event_type in handler.supported_event_types() {
            handlers.entry(event_type).or_insert_with(Vec::new).push(handler.clone());
        }
        
        Ok(())
    }

    /// Subscribe to events
    pub async fn subscribe_to_events(&self) -> broadcast::Receiver<Option<Message>> {
        self.event_channel.subscribe()
    }

    /// Start message processing task
    async fn start_message_processing(&self) -> Result<()> {
        let message_rx = {
            let mut rx_guard = self.message_rx.write().await;
            rx_guard.take()
        };

        if let Some(rx) = message_rx {
            let pending_requests = Arc::clone(&self.pending_requests);
            let event_handlers = Arc::clone(&self.event_handlers);
            let event_channel = Arc::clone(&self.event_channel);
            let connection_manager = Arc::clone(&self.connection_manager);
            let cancellation_token = self.cancellation_token.clone();

            let task = tokio::spawn(async move {
                process_messages(
                    rx,
                    pending_requests,
                    event_handlers,
                    event_channel,
                    connection_manager,
                    cancellation_token,
                ).await;
            });

            let mut task_guard = self.message_task.write().await;
            *task_guard = Some(task);
        }

        Ok(())
    }

    /// Start reader task
    async fn start_reader_task(&self) -> Result<()> {
        let transport = self.connection_manager.get_transport().await
            .ok_or_else(|| MCPError::ConnectionError("No transport available".to_string()))?;

        let message_tx = self.message_tx.clone();
        let connection_manager = Arc::clone(&self.connection_manager);
        let cancellation_token = self.cancellation_token.clone();

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(async move {
            loop {
                if cancellation_token.is_cancelled() {
                    break;
                }

                match transport.receive_message().await {
                    Ok(message) => {
                        if let Err(error) = message_tx.send(message).await {
                            error!("Failed to send message to processing channel: {}", error);
                            break;
                        }
                    }
                    Err(error) => {
                        error!("Failed to receive message: {}", error);
                        let _ = connection_manager.handle_connection_error(error).await;
                        break;
                    }
                }
            }
        }, abort_registration);

        // Store abort handle
        let mut reader_guard = self.reader_task.lock().await;
        *reader_guard = Some(abort_handle);

        // Start the task
        tokio::spawn(future);

        Ok(())
    }

    /// Process a message
    async fn process_message(&self, message: Message) -> Result<()> {
        match message.message_type {
            MessageType::Response => {
                // Handle response
                let mut pending = self.pending_requests.write().await;
                if let Some(response_tx) = pending.remove(&message.id) {
                    let _ = response_tx.send(Ok(message));
                }
            }
            MessageType::Request => {
                // Handle request (if we support server-initiated requests)
                warn!("Received unexpected request from server: {:?}", message);
            }
            MessageType::Notification => {
                // Handle notification/event
                self.handle_event(message).await?;
            }
        }

        Ok(())
    }

    /// Handle an event message
    async fn handle_event(&self, event: Message) -> Result<()> {
        // Publish to event channel
        if let Err(error) = self.event_channel.send(Some(event.clone())) {
            warn!("Failed to publish event to channel: {}", error);
        }

        // Call registered handlers
        let handlers = self.event_handlers.read().await;
        if let Some(method) = &event.method {
            if let Some(event_handlers) = handlers.get(method) {
                for handler in event_handlers {
                    if let Err(error) = handler.handle_event(&event).await {
                        error!("Event handler error: {}", error);
                    }
                }
            }
        }

        Ok(())
    }
}

/// Process messages from the transport
async fn process_messages(
    mut rx: mpsc::Receiver<Message>,
    pending_requests: Arc<RwLock<HashMap<String, oneshot::Sender<Result<Message>>>>>,
    event_handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    event_channel: Arc<broadcast::Sender<Option<Message>>>,
    connection_manager: Arc<ConnectionManager>,
    cancellation_token: tokio_util::sync::CancellationToken,
) {
    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                break;
            }
            message = rx.recv() => {
                match message {
                    Some(message) => {
                        if let Err(error) = process_message_internal(
                            message,
                            Arc::clone(&pending_requests),
                            Arc::clone(&event_handlers),
                            Arc::clone(&event_channel),
                        ).await {
                            error!("Error processing message: {}", error);
                            let _ = connection_manager.handle_connection_error(error).await;
                        }
                    }
                    None => {
                        debug!("Message channel closed");
                        break;
                    }
                }
            }
        }
    }
}

/// Process a single message
async fn process_message_internal(
    message: Message,
    pending_requests: Arc<RwLock<HashMap<String, oneshot::Sender<Result<Message>>>>>,
    event_handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    event_channel: Arc<broadcast::Sender<Option<Message>>>,
) -> Result<()> {
    match message.message_type {
        MessageType::Response => {
            // Handle response
            let mut pending = pending_requests.write().await;
            if let Some(response_tx) = pending.remove(&message.id) {
                let _ = response_tx.send(Ok(message));
            }
        }
        MessageType::Request => {
            // Handle request (if we support server-initiated requests)
            warn!("Received unexpected request from server: {:?}", message);
        }
        MessageType::Notification => {
            // Handle notification/event
            // Publish to event channel
            if let Err(error) = event_channel.send(Some(message.clone())) {
                warn!("Failed to publish event to channel: {}", error);
            }

            // Call registered handlers
            let handlers = event_handlers.read().await;
            if let Some(method) = &message.method {
                if let Some(event_handlers) = handlers.get(method) {
                    for handler in event_handlers {
                        if let Err(error) = handler.handle_event(&message).await {
                            error!("Event handler error: {}", error);
                        }
                    }
                }
            }
        }
    }

    Ok(())
} 