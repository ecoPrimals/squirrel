// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP client implementation
//!
//! Provides a client for the Machine Context Protocol, enabling
//! communication with MCP servers.

use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::mcp::protocol::{MCPError, MCPMessage, MCPMessageType, MCPResult};

/// Callback for notification handlers
pub type NotificationCallback = Box<dyn Fn(&str, &MCPMessage) -> MCPResult<()> + Send + Sync>;

/// MCP client for communicating with MCP servers
pub struct MCPClient {
    /// Server host
    host: String,

    /// Server port
    port: u16,

    /// Connection timeout
    timeout: Option<Duration>,

    /// Active connection
    connection: Arc<Mutex<Option<TcpStream>>>,

    /// Subscriptions by topic
    subscriptions: Arc<Mutex<HashMap<String, HashSet<Uuid>>>>,

    /// Notification handlers
    notification_handlers: Arc<Mutex<HashMap<Uuid, NotificationCallback>>>,

    /// Notification listener thread handle
    listener_thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,

    /// Running flag for notification listener
    running: Arc<Mutex<bool>>,
}

impl MCPClient {
    /// Create a new MCP client
    ///
    /// # Arguments
    ///
    /// * `host` - Server host
    /// * `port` - Server port
    ///
    /// # Returns
    ///
    /// A new MCP client
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            timeout: Some(Duration::from_secs(30)),
            connection: Arc::new(Mutex::new(None)),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            notification_handlers: Arc::new(Mutex::new(HashMap::new())),
            listener_thread: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Set the connection timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Connection timeout duration
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Connect to the MCP server
    ///
    /// # Arguments
    ///
    /// * `timeout` - Optional timeout override for this connection attempt
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn connect(&mut self, timeout: Option<Duration>) -> MCPResult<()> {
        let address = format!("{}:{}", self.host, self.port);
        debug!("Connecting to MCP server at {}", address);

        let timeout_to_use = timeout.or(self.timeout).unwrap_or(Duration::from_secs(30));

        let parsed_address = address.parse().map_err(|e| {
            MCPError::ConnectionError(format!("Failed to parse address {}: {}", address, e))
        })?;
        let stream = TcpStream::connect_timeout(&parsed_address, timeout_to_use).map_err(|e| {
            MCPError::ConnectionError(format!("Failed to connect to {}: {}", address, e))
        })?;

        // Set read timeout
        stream.set_read_timeout(Some(timeout_to_use))?;

        // Store connection
        {
            let mut connection = self
                .connection
                .lock()
                .map_err(|_| MCPError::ConnectionError("Connection mutex poisoned".to_string()))?;
            *connection = Some(stream.try_clone()?);
        }

        // Start notification listener
        self.start_notification_listener(stream)?;

        info!("Connected to MCP server at {}", address);
        Ok(())
    }

    /// Start the notification listener thread
    ///
    /// # Arguments
    ///
    /// * `stream` - TCP stream clone for the listener
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    fn start_notification_listener(&self, stream: TcpStream) -> MCPResult<()> {
        // Set running flag
        {
            let mut running = self.running.lock().map_err(|_| {
                MCPError::ConnectionError("Running flag mutex poisoned".to_string())
            })?;
            *running = true;
        }

        // Clone needed references for the thread
        let subscriptions = Arc::clone(&self.subscriptions);
        let notification_handlers = Arc::clone(&self.notification_handlers);
        let running = Arc::clone(&self.running);

        // Start listener thread
        let thread = thread::spawn(move || {
            let mut reader = BufReader::new(stream);

            while match running.lock() {
                Ok(guard) => *guard,
                Err(_) => {
                    error!("Running flag mutex poisoned, exiting listener thread");
                    false
                }
            } {
                let mut line = String::new();

                // Read a line from the server
                match reader.read_line(&mut line) {
                    Ok(0) => {
                        // End of stream, server disconnected
                        debug!("Server disconnected (EOF)");
                        break;
                    }
                    Ok(_) => {
                        // Process message
                        match MCPMessage::from_json(line.trim()) {
                            Ok(message) => {
                                // Only process notifications
                                if message.message_type == MCPMessageType::Notification {
                                    let topic = message.command.clone();

                                    // Find handlers for this topic
                                    let handlers = match subscriptions.lock() {
                                        Ok(subs) => {
                                            if let Some(handler_ids) = subs.get(&topic) {
                                                handler_ids.clone()
                                            } else {
                                                HashSet::new()
                                            }
                                        }
                                        Err(_) => {
                                            error!("Subscriptions mutex poisoned, skipping notification");
                                            continue;
                                        }
                                    };

                                    // Call handlers
                                    for handler_id in handlers {
                                        match notification_handlers.lock() {
                                            Ok(handlers) => {
                                                if let Some(handler) = handlers.get(&handler_id) {
                                                    if let Err(e) = handler(&topic, &message) {
                                                        error!(
                                                            "Error in notification handler: {}",
                                                            e
                                                        );
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                error!("Notification handlers mutex poisoned, skipping handler");
                                                continue;
                                            }
                                        }
                                    }
                                } else {
                                    debug!(
                                        "Ignored non-notification message: {:?}",
                                        message.message_type
                                    );
                                }
                            }
                            Err(e) => {
                                error!("Error parsing notification: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading from server: {}", e);
                        break;
                    }
                }
            }

            debug!("Notification listener thread exiting");
        });

        // Store thread handle
        let mut listener_thread = self
            .listener_thread
            .lock()
            .map_err(|_| MCPError::ConnectionError("Listener thread mutex poisoned".to_string()))?;
        *listener_thread = Some(thread);

        Ok(())
    }

    /// Disconnect from the MCP server
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn disconnect(&self) -> MCPResult<()> {
        // Stop notification listener
        {
            match self.running.lock() {
                Ok(mut running) => {
                    *running = false;
                }
                Err(_) => {
                    error!("Running flag mutex poisoned during disconnect");
                    return Err(MCPError::ConnectionError(
                        "Running flag mutex poisoned".to_string(),
                    ));
                }
            }
        }

        // Wait for listener thread to exit
        {
            match self.listener_thread.lock() {
                Ok(mut listener_thread) => {
                    if let Some(thread) = listener_thread.take() {
                        if let Err(e) = thread.join() {
                            error!("Error joining notification listener thread: {:?}", e);
                        }
                    }
                }
                Err(_) => {
                    error!("Listener thread mutex poisoned during disconnect");
                    return Err(MCPError::ConnectionError(
                        "Listener thread mutex poisoned".to_string(),
                    ));
                }
            }
        }

        // Close connection
        match self.connection.lock() {
            Ok(mut connection) => {
                if connection.is_some() {
                    debug!("Disconnecting from MCP server");

                    // Attempt to unsubscribe from all topics
                    match self.subscriptions.lock() {
                        Ok(subscriptions) => {
                            for topic in subscriptions.keys() {
                                if let Err(e) = self.send_unsubscribe_message(topic) {
                                    warn!("Error unsubscribing from topic {}: {}", topic, e);
                                }
                            }
                        }
                        Err(_) => {
                            error!("Subscriptions mutex poisoned during disconnect");
                        }
                    }

                    // Clear subscriptions
                    match self.subscriptions.lock() {
                        Ok(mut subscriptions) => {
                            subscriptions.clear();
                        }
                        Err(_) => {
                            error!("Subscriptions mutex poisoned when clearing");
                        }
                    }

                    // Close connection
                    *connection = None;
                    info!("Disconnected from MCP server");
                }
            }
            Err(_) => {
                error!("Connection mutex poisoned during disconnect");
                return Err(MCPError::ConnectionError(
                    "Connection mutex poisoned".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Check if connected to the MCP server
    ///
    /// # Returns
    ///
    /// True if connected, false otherwise
    pub fn is_connected(&self) -> bool {
        match self.connection.lock() {
            Ok(connection) => connection.is_some(),
            Err(_) => {
                error!("Connection mutex poisoned when checking connection status");
                false
            }
        }
    }

    /// Send a command to the MCP server
    ///
    /// # Arguments
    ///
    /// * `command` - Command name
    /// * `args` - Command arguments as JSON
    ///
    /// # Returns
    ///
    /// A Result containing the server response or an error
    pub fn send_command(
        &self,
        command: &str,
        args: Option<serde_json::Value>,
    ) -> MCPResult<MCPMessage> {
        // Ensure connected
        if !self.is_connected() {
            return Err(MCPError::ConnectionError(
                "Not connected to MCP server".to_string(),
            ));
        }

        // Generate a unique message ID
        let id = Uuid::new_v4().to_string();

        // Create request message
        let request = MCPMessage::new_request(id, command.to_string(), args);

        // Send request
        let response = self.send_message(&request)?;

        Ok(response)
    }

    /// Send a notification to the MCP server
    ///
    /// # Arguments
    ///
    /// * `topic` - Notification topic
    /// * `payload` - Notification payload as JSON
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn send_notification(
        &self,
        topic: &str,
        payload: Option<serde_json::Value>,
    ) -> MCPResult<()> {
        // Ensure connected
        if !self.is_connected() {
            return Err(MCPError::ConnectionError(
                "Not connected to MCP server".to_string(),
            ));
        }

        // Generate a unique message ID
        let id = Uuid::new_v4().to_string();

        // Create notification message
        let notification = MCPMessage::new_notification(id, topic.to_string(), payload);

        // Send notification (no response expected)
        self.send_raw_message(&notification)?;

        Ok(())
    }

    /// Subscribe to a topic
    ///
    /// # Arguments
    ///
    /// * `topic` - Topic to subscribe to
    /// * `callback` - Notification handler callback
    ///
    /// # Returns
    ///
    /// A Result containing the subscription ID or an error
    pub fn subscribe<F>(&self, topic: &str, callback: F) -> MCPResult<Uuid>
    where
        F: Fn(&str, &MCPMessage) -> MCPResult<()> + Send + Sync + 'static,
    {
        // Ensure connected
        if !self.is_connected() {
            return Err(MCPError::ConnectionError(
                "Not connected to MCP server".to_string(),
            ));
        }

        // Generate subscription ID
        let subscription_id = Uuid::new_v4();

        // Register handler
        {
            let mut handlers = self.notification_handlers.lock().map_err(|_| {
                MCPError::ConnectionError("Notification handlers mutex poisoned".to_string())
            })?;
            handlers.insert(subscription_id, Box::new(callback));
        }

        // Register subscription
        {
            let mut subscriptions = self.subscriptions.lock().map_err(|_| {
                MCPError::ConnectionError("Subscriptions mutex poisoned".to_string())
            })?;
            subscriptions
                .entry(topic.to_string())
                .or_default()
                .insert(subscription_id);
        }

        // Send subscribe message to server
        self.send_subscribe_message(topic)?;

        debug!("Subscribed to topic: {}", topic);
        Ok(subscription_id)
    }

    /// Unsubscribe from a topic
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - Subscription ID to unsubscribe
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn unsubscribe(&self, subscription_id: Uuid) -> MCPResult<()> {
        // Ensure connected
        if !self.is_connected() {
            return Err(MCPError::ConnectionError(
                "Not connected to MCP server".to_string(),
            ));
        }

        // Find topics for this subscription ID
        let topics_to_unsubscribe = {
            let subscriptions = self.subscriptions.lock().map_err(|_| {
                MCPError::ConnectionError("Subscriptions mutex poisoned".to_string())
            })?;
            subscriptions
                .iter()
                .filter_map(|(topic, ids)| {
                    if ids.contains(&subscription_id) {
                        Some(topic.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
        };

        // Remove subscription ID from topics
        {
            let mut subscriptions = self.subscriptions.lock().map_err(|_| {
                MCPError::ConnectionError("Subscriptions mutex poisoned".to_string())
            })?;
            for topic in &topics_to_unsubscribe {
                if let Some(ids) = subscriptions.get_mut(topic) {
                    ids.remove(&subscription_id);

                    // If no more subscriptions for this topic, send unsubscribe message
                    if ids.is_empty() {
                        self.send_unsubscribe_message(topic)?;
                        subscriptions.remove(topic);
                    }
                }
            }
        }

        // Remove handler
        {
            let mut handlers = self.notification_handlers.lock().map_err(|_| {
                MCPError::ConnectionError("Notification handlers mutex poisoned".to_string())
            })?;
            handlers.remove(&subscription_id);
        }

        debug!("Unsubscribed from topics: {:?}", topics_to_unsubscribe);
        Ok(())
    }

    /// Send a subscribe message to the server
    ///
    /// # Arguments
    ///
    /// * `topic` - Topic to subscribe to
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    fn send_subscribe_message(&self, topic: &str) -> MCPResult<()> {
        self.send_command(
            "subscribe",
            Some(json!({
                "topic": topic
            })),
        )?;
        Ok(())
    }

    /// Send an unsubscribe message to the server
    ///
    /// # Arguments
    ///
    /// * `topic` - Topic to unsubscribe from
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    fn send_unsubscribe_message(&self, topic: &str) -> MCPResult<()> {
        // This is a best-effort call, but we don't want it to fail the overall unsubscribe
        if let Err(e) = self.send_command(
            "unsubscribe",
            Some(json!({
                "topic": topic
            })),
        ) {
            warn!(
                "Error sending unsubscribe command for topic {}: {}",
                topic, e
            );
        }
        Ok(())
    }

    /// Run in interactive mode
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn run_interactive(&self) -> MCPResult<()> {
        // Ensure connected
        if !self.is_connected() {
            return Err(MCPError::ConnectionError(
                "Not connected to MCP server".to_string(),
            ));
        }

        println!("MCP Interactive Mode");
        println!("Type 'exit' or 'quit' to exit");
        println!("Type 'help' for available commands");
        println!("Format: <command> [JSON args]");

        // Read commands from stdin
        let stdin = std::io::stdin();
        let mut lines = stdin.lock().lines();

        loop {
            print!("> ");
            std::io::stdout().flush()?;

            if let Some(line) = lines.next() {
                let line = line?;
                let line = line.trim();

                // Exit command
                if line == "exit" || line == "quit" {
                    break;
                }

                // Help command
                if line == "help" {
                    println!("Available commands:");
                    println!("  <command> [JSON args] - Execute command");
                    println!("  subscribe <topic>     - Subscribe to topic");
                    println!("  unsubscribe <topic>   - Unsubscribe from topic");
                    println!("  notify <topic> [JSON] - Send notification");
                    println!("  help                  - Show this help");
                    println!("  exit, quit            - Exit interactive mode");
                    continue;
                }

                // Parse command and args
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.is_empty() {
                    continue;
                }

                let command = parts[0];

                // Handle special commands
                match command {
                    "subscribe" => {
                        if parts.len() > 1 {
                            let topic = parts[1];

                            // Subscribe to topic with a handler that prints notifications
                            match self.subscribe(topic, |topic, msg| {
                                println!("Notification received from topic {}: {:?}", topic, msg);
                                Ok(())
                            }) {
                                Ok(_) => println!("Subscribed to topic: {}", topic),
                                Err(e) => eprintln!("Error subscribing to topic: {}", e),
                            }
                        } else {
                            eprintln!("Missing topic. Usage: subscribe <topic>");
                        }
                        continue;
                    }
                    "unsubscribe" => {
                        if parts.len() > 1 {
                            let topic = parts[1];

                            // Find all subscription IDs for this topic
                            let subscription_ids = {
                                match self.subscriptions.lock() {
                                    Ok(subscriptions) => {
                                        if let Some(ids) = subscriptions.get(topic) {
                                            ids.clone()
                                        } else {
                                            eprintln!("Not subscribed to topic: {}", topic);
                                            continue;
                                        }
                                    }
                                    Err(_) => {
                                        eprintln!("Subscriptions mutex poisoned");
                                        continue;
                                    }
                                }
                            };

                            // Unsubscribe from all
                            for id in subscription_ids {
                                match self.unsubscribe(id) {
                                    Ok(_) => println!("Unsubscribed from topic: {}", topic),
                                    Err(e) => eprintln!("Error unsubscribing from topic: {}", e),
                                }
                            }
                        } else {
                            eprintln!("Missing topic. Usage: unsubscribe <topic>");
                        }
                        continue;
                    }
                    "notify" => {
                        if parts.len() < 2 {
                            eprintln!("Missing topic. Usage: notify <topic> [JSON]");
                            continue;
                        }

                        let notify_parts: Vec<&str> = parts[1].splitn(2, ' ').collect();
                        let topic = notify_parts[0];

                        // Parse JSON payload if provided
                        let payload = if notify_parts.len() > 1 {
                            match serde_json::from_str(notify_parts[1]) {
                                Ok(json) => Some(json),
                                Err(e) => {
                                    eprintln!("Error parsing JSON payload: {}", e);
                                    continue;
                                }
                            }
                        } else {
                            None
                        };

                        // Send notification
                        match self.send_notification(topic, payload) {
                            Ok(_) => println!("Notification sent to topic: {}", topic),
                            Err(e) => eprintln!("Error sending notification: {}", e),
                        }
                        continue;
                    }
                    _ => {}
                }

                // Parse JSON args if provided
                let args = if parts.len() > 1 {
                    match serde_json::from_str(parts[1]) {
                        Ok(json) => Some(json),
                        Err(e) => {
                            eprintln!("Error parsing JSON args: {}", e);
                            continue;
                        }
                    }
                } else {
                    None
                };

                // Send command
                match self.send_command(command, args) {
                    Ok(response) => {
                        // Pretty print response
                        match serde_json::to_string_pretty(&response) {
                            Ok(json) => println!("{}", json),
                            Err(e) => eprintln!("Error formatting response: {}", e),
                        }
                    }
                    Err(e) => {
                        eprintln!("Error sending command: {}", e);
                    }
                }
            }
        }

        println!("Exiting interactive mode");
        Ok(())
    }

    /// Send a message to the MCP server and receive a response
    ///
    /// # Arguments
    ///
    /// * `message` - Message to send
    ///
    /// # Returns
    ///
    /// A Result containing the server response or an error
    fn send_message(&self, message: &MCPMessage) -> MCPResult<MCPMessage> {
        // Send the message
        self.send_raw_message(message)?;

        // Read response
        let connection = self
            .connection
            .lock()
            .map_err(|_| MCPError::ConnectionError("Connection mutex poisoned".to_string()))?;
        if let Some(stream) = &*connection {
            let mut reader = BufReader::new(stream);
            let mut response_json = String::new();
            reader.read_line(&mut response_json)?;

            // Parse response
            let response = MCPMessage::from_json(&response_json)?;

            // Check if error response
            if response.message_type == MCPMessageType::Error {
                if let Some(error) = &response.error {
                    return Err(MCPError::CommandError(error.clone()));
                }
            }

            Ok(response)
        } else {
            Err(MCPError::ConnectionError(
                "Not connected to MCP server".to_string(),
            ))
        }
    }

    /// Send a raw message to the MCP server without reading a response
    ///
    /// # Arguments
    ///
    /// * `message` - Message to send
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    fn send_raw_message(&self, message: &MCPMessage) -> MCPResult<()> {
        // Convert message to JSON
        let message_json = message.to_json()?;

        // Send message
        let connection = self
            .connection
            .lock()
            .map_err(|_| MCPError::ConnectionError("Connection mutex poisoned".to_string()))?;
        if let Some(stream) = &*connection {
            let mut writer = BufWriter::new(stream);
            writeln!(writer, "{}", message_json)?;
            writer.flush()?;
            Ok(())
        } else {
            Err(MCPError::ConnectionError(
                "Not connected to MCP server".to_string(),
            ))
        }
    }
}

impl Drop for MCPClient {
    fn drop(&mut self) {
        // Ensure disconnected when dropped
        if self.is_connected() {
            if let Err(e) = self.disconnect() {
                warn!("Error disconnecting from MCP server: {}", e);
            }
        }
    }
}
