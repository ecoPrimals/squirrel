//! MCP server implementation
//!
//! This module provides the server-side functionality for the Machine Context Protocol (MCP).

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::protocol::{MCPError, MCPMessage, MCPMessageType, MCPResult};
use crate::commands::registry::CommandRegistry;

/// Helper function to safely lock a mutex and convert poison errors to MCPError
fn safe_lock<'a, T>(mutex: &'a Mutex<T>, context: &str) -> MCPResult<std::sync::MutexGuard<'a, T>> {
    mutex.lock().map_err(|e| {
        error!("Mutex poisoned in {}: {}", context, e);
        MCPError::ProtocolError(format!(
            "Internal server error in {}: mutex poisoned",
            context
        ))
    })
}

/// Get default host for the MCP server (environment-aware)
pub fn default_host() -> String {
    use squirrel_mcp_config::core::{network_defaults, DevelopmentConfig, NetworkEndpointConfig};

    let is_production = std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .eq_ignore_ascii_case("production");

    if is_production {
        std::env::var("MCP_HOST")
            .unwrap_or_else(|_| network_defaults::DEFAULT_BIND_HOST.to_string())
    } else {
        let network_config = NetworkEndpointConfig::default();
        let dev_config = DevelopmentConfig::default();
        std::env::var("MCP_HOST").unwrap_or_else(|_| network_config.get_effective_host(&dev_config))
    }
}

/// Get default port for the MCP server (configurable)
pub fn default_port() -> u16 {
    use squirrel_mcp_config::core::network_defaults;

    std::env::var("MCP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(network_defaults::DEFAULT_MCP_PORT)
}

/// MCP command handler function
pub type MCPCommandHandler = Box<dyn Fn(MCPMessage) -> MCPResult<MCPMessage> + Send + Sync>;

/// MCP Server
///
/// The MCP server handles incoming TCP connections and processes MCP protocol messages.
#[derive(Clone)]
pub struct MCPServer {
    /// Server host
    host: String,

    /// Server port
    port: u16,

    /// Command registry
    command_registry: Option<Arc<CommandRegistry>>,

    /// Custom command handlers
    command_handlers: Arc<Mutex<HashMap<String, MCPCommandHandler>>>,

    /// Active client connections
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,

    /// Topic subscriptions by client
    client_subscriptions: Arc<Mutex<HashMap<String, HashSet<String>>>>,

    /// Client subscriptions by topic
    topic_subscribers: Arc<Mutex<HashMap<String, HashSet<String>>>>,

    /// Server running flag
    running: Arc<Mutex<bool>>,
}

impl MCPServer {
    /// Create a new MCP server
    ///
    /// # Arguments
    ///
    /// * `host` - Optional server host (defaults to 127.0.0.1)
    /// * `port` - Optional server port (defaults to 8778)
    ///
    /// # Returns
    ///
    /// A new `MCPServer` instance
    pub fn new(host: Option<&str>, port: Option<u16>) -> Self {
        Self {
            host: host.map(|h| h.to_string()).unwrap_or_else(default_host),
            port: port.unwrap_or_else(default_port),
            command_registry: None,
            command_handlers: Arc::new(Mutex::new(HashMap::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_subscriptions: Arc::new(Mutex::new(HashMap::new())),
            topic_subscribers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Set the command registry
    ///
    /// # Arguments
    ///
    /// * `registry` - Command registry
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_command_registry(mut self, registry: Arc<CommandRegistry>) -> Self {
        self.command_registry = Some(registry);
        self
    }

    /// Check if a command registry is available
    ///
    /// # Returns
    ///
    /// `true` if a command registry is available, `false` otherwise
    pub fn has_command_registry(&self) -> bool {
        self.command_registry.is_some()
    }

    /// Register a custom command handler
    ///
    /// # Arguments
    ///
    /// * `command` - Command name
    /// * `handler` - Command handler function
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn register_handler<F>(self, command: &str, handler: F) -> Self
    where
        F: Fn(MCPMessage) -> MCPResult<MCPMessage> + Send + Sync + 'static,
    {
        // Add the handler to the map under a lock
        {
            match self.command_handlers.lock() {
                Ok(mut handlers) => {
                    handlers.insert(command.to_string(), Box::new(handler));
                }
                Err(_) => {
                    error!("Failed to acquire lock for command handlers - mutex poisoned");
                    // Continue execution as this is a builder pattern
                }
            }
        }

        self
    }

    /// Start the MCP server
    ///
    /// This method starts the server in a new thread and returns immediately.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub fn start(&self) -> MCPResult<()> {
        let mut running = safe_lock(&self.running, "server start")?;
        if *running {
            return Err(MCPError::ProtocolError(
                "Server already running".to_string(),
            ));
        }

        *running = true;
        drop(running);

        let server = self.clone();
        thread::spawn(move || {
            if let Err(e) = server.run_server() {
                error!("MCP server error: {}", e);
                if let Ok(mut running) = safe_lock(&server.running, "server error cleanup") {
                    *running = false;
                } else {
                    error!("Failed to set running state to false after error");
                }
            }
        });

        info!("MCP server started on {}:{}", self.host, self.port);
        Ok(())
    }

    /// Stop the MCP server
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub fn stop(&self) -> MCPResult<()> {
        let mut running = self
            .running
            .lock()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire running lock".to_string()))?;
        if !*running {
            return Err(MCPError::ProtocolError("Server not running".to_string()));
        }

        *running = false;

        // Disconnect all clients
        let mut clients = self
            .clients
            .lock()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire clients lock".to_string()))?;
        for (_id, client) in clients.drain() {
            if let Ok(client) = client.lock() {
                let _ = client.shutdown(std::net::Shutdown::Both);
            }
        }

        // Clear all subscriptions
        {
            let mut client_subscriptions = self.client_subscriptions.lock().map_err(|_| {
                MCPError::ProtocolError("Failed to acquire client subscriptions lock".to_string())
            })?;
            client_subscriptions.clear();

            let mut topic_subscribers = self.topic_subscribers.lock().map_err(|_| {
                MCPError::ProtocolError("Failed to acquire topic subscribers lock".to_string())
            })?;
            topic_subscribers.clear();
        }

        info!("MCP server stopped");
        Ok(())
    }

    /// Check if the server is running
    ///
    /// # Returns
    ///
    /// `true` if the server is running, `false` otherwise
    pub fn is_running(&self) -> bool {
        safe_lock(&self.running, "is_running check")
            .map(|guard| *guard)
            .unwrap_or_else(|e| {
                warn!("Failed to check server running state: {}", e);
                false
            })
    }

    /// Send notification to all connected clients
    ///
    /// # Arguments
    ///
    /// * `topic` - Notification topic
    /// * `payload` - Optional notification payload
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub fn broadcast_notification(&self, topic: &str, payload: Option<Value>) -> MCPResult<()> {
        let notification =
            MCPMessage::new_notification(Uuid::new_v4().to_string(), topic.to_string(), payload);

        let json = notification.to_json()?;

        // Get subscribers for this topic
        let subscribers = {
            let topic_subscribers = safe_lock(
                &self.topic_subscribers,
                "broadcast_notification - get subscribers",
            )?;
            topic_subscribers
                .get(topic)
                .cloned()
                .unwrap_or_else(HashSet::new)
        };

        // Send notification to each subscriber
        let clients = safe_lock(&self.clients, "broadcast_notification - get clients")?;

        for client_id in subscribers {
            if let Some(stream) = clients.get(&client_id) {
                match safe_lock(stream, "broadcast_notification - client stream") {
                    Ok(mut stream_guard) => {
                        match stream_guard.write_all(format!("{}\n", json).as_bytes()) {
                            Ok(_) => debug!("Notification sent to client {}", client_id),
                            Err(e) => {
                                warn!("Failed to send notification to client {}: {}", client_id, e)
                            }
                        }
                    }
                    Err(e) => warn!("Failed to lock client stream for {}: {}", client_id, e),
                }
            }
        }

        Ok(())
    }

    /// Send notification to a specific client
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client ID
    /// * `topic` - Notification topic
    /// * `payload` - Optional notification payload
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub fn send_notification(
        &self,
        client_id: &str,
        topic: &str,
        payload: Option<Value>,
    ) -> MCPResult<()> {
        let notification =
            MCPMessage::new_notification(Uuid::new_v4().to_string(), topic.to_string(), payload);

        let json = notification.to_json()?;

        // Send notification to the client
        let clients = safe_lock(&self.clients, "send_notification_to_client - get clients")?;
        if let Some(stream) = clients.get(client_id) {
            let mut stream = safe_lock(stream, "send_notification_to_client - client stream")?;
            match stream.write_all(format!("{}\n", json).as_bytes()) {
                Ok(_) => debug!("Notification sent to client {}", client_id),
                Err(e) => {
                    return Err(MCPError::IoError(e));
                }
            }
        } else {
            return Err(MCPError::ProtocolError(format!(
                "Client {} not found",
                client_id
            )));
        }

        Ok(())
    }

    /// Subscribe a client to a topic
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client ID
    /// * `topic` - Topic to subscribe to
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub fn subscribe_client(&self, client_id: &str, topic: &str) -> MCPResult<()> {
        // Add subscription to client
        {
            let mut client_subscriptions = safe_lock(
                &self.client_subscriptions,
                "subscribe_client - client subscriptions",
            )?;
            client_subscriptions
                .entry(client_id.to_string())
                .or_default()
                .insert(topic.to_string());
        }

        // Add client to topic subscribers
        {
            let mut topic_subscribers = safe_lock(
                &self.topic_subscribers,
                "subscribe_client - topic subscribers",
            )?;
            topic_subscribers
                .entry(topic.to_string())
                .or_default()
                .insert(client_id.to_string());
        }

        debug!("Client {} subscribed to topic '{}'", client_id, topic);
        Ok(())
    }

    /// Unsubscribe a client from a topic
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client ID
    /// * `topic` - Topic to unsubscribe from
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub fn unsubscribe_client(&self, client_id: &str, topic: &str) -> MCPResult<()> {
        // Remove subscription from client
        {
            let mut client_subscriptions = safe_lock(
                &self.client_subscriptions,
                "unsubscribe_client - client subscriptions",
            )?;
            if let Some(topics) = client_subscriptions.get_mut(client_id) {
                topics.remove(topic);

                // If no more topics, remove the client entry
                if topics.is_empty() {
                    client_subscriptions.remove(client_id);
                }
            }
        }

        // Remove client from topic subscribers
        {
            let mut topic_subscribers = safe_lock(
                &self.topic_subscribers,
                "unsubscribe_client - topic subscribers",
            )?;
            if let Some(subscribers) = topic_subscribers.get_mut(topic) {
                subscribers.remove(client_id);

                // If no more subscribers, remove the topic entry
                if subscribers.is_empty() {
                    topic_subscribers.remove(topic);
                }
            }
        }

        debug!("Client {} unsubscribed from topic '{}'", client_id, topic);
        Ok(())
    }

    /// Unsubscribe a client from all topics
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client ID
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub fn unsubscribe_client_all(&self, client_id: &str) -> MCPResult<()> {
        // Get all topics the client is subscribed to
        let topics = {
            let client_subscriptions =
                safe_lock(&self.client_subscriptions, "unsubscribe_client_all")?;
            if let Some(topics) = client_subscriptions.get(client_id) {
                topics.clone()
            } else {
                HashSet::new()
            }
        };

        // Unsubscribe from each topic
        for topic in topics {
            self.unsubscribe_client(client_id, &topic)?;
        }

        debug!("Client {} unsubscribed from all topics", client_id);
        Ok(())
    }

    /// Run the server (blocking)
    ///
    /// This method blocks and runs the server until stopped.
    fn run_server(&self) -> MCPResult<()> {
        let address = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&address).map_err(MCPError::IoError)?;

        // Set a timeout to allow checking the running flag
        listener.set_nonblocking(true).map_err(MCPError::IoError)?;

        while self.is_running() {
            match listener.accept() {
                Ok((stream, addr)) => {
                    info!("New MCP client connected: {}", addr);

                    // Reset to blocking mode for the client
                    stream.set_nonblocking(false).map_err(MCPError::IoError)?;

                    // Clone the server for the client thread
                    let server = self.clone();
                    let client_id = Uuid::new_v4().to_string();

                    // Store the client connection
                    {
                        let mut clients = safe_lock(&server.clients, "server loop - store client")?;
                        clients
                            .insert(client_id.clone(), Arc::new(Mutex::new(stream.try_clone()?)));
                    }

                    // Spawn a new thread to handle this client
                    thread::spawn(move || {
                        if let Err(e) = server.handle_client(client_id.clone(), stream) {
                            error!("Error handling client {}: {}", client_id, e);
                        }

                        // Unsubscribe from all topics when disconnected
                        if let Err(e) = server.unsubscribe_client_all(&client_id) {
                            error!("Error unsubscribing client {}: {}", client_id, e);
                        }

                        // Remove the client when disconnected
                        match safe_lock(&server.clients, "client cleanup - remove client") {
                            Ok(mut clients) => {
                                if clients.remove(&client_id).is_some() {
                                    info!("Client {} disconnected", client_id);
                                }
                            }
                            Err(e) => {
                                error!("Failed to remove client {}: {}", client_id, e);
                            }
                        }
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No new connection, sleep a bit
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        Ok(())
    }

    /// Handle client connection
    ///
    /// # Arguments
    ///
    /// * `client_id` - Unique client identifier
    /// * `stream` - Client TCP stream
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    fn handle_client(&self, client_id: String, mut stream: TcpStream) -> MCPResult<()> {
        let mut reader = BufReader::new(stream.try_clone()?);

        let mut line = String::new();
        while self.is_running() {
            line.clear();

            // Read a line from the client
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // End of stream, client disconnected
                    debug!("Client {} disconnected (EOF)", client_id);
                    break;
                }
                Ok(_) => {
                    // Process the message
                    match self.process_message(&client_id, line.trim()) {
                        Ok(Some(response)) => {
                            // Send the response back to the client
                            let mut stream = stream.try_clone()?;
                            stream.write_all(format!("{}\n", response).as_bytes())?;
                        }
                        Ok(None) => {
                            // No response needed (e.g., for notifications)
                        }
                        Err(e) => {
                            error!("Error processing message from client {}: {}", client_id, e);

                            // Try to send an error response
                            let error_msg = MCPMessage::new_error(
                                Uuid::new_v4().to_string(),
                                "error".to_string(),
                                format!("Error: {}", e),
                            );

                            if let Ok(json) = error_msg.to_json() {
                                let _ = stream.write_all(format!("{}\n", json).as_bytes());
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from client {}: {}", client_id, e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Process an incoming message
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client ID
    /// * `message` - Message string (JSON)
    ///
    /// # Returns
    ///
    /// A result containing an optional response message
    fn process_message(&self, client_id: &str, message: &str) -> MCPResult<Option<String>> {
        debug!("Received message from client {}: {}", client_id, message);

        // Parse the message
        let msg = MCPMessage::from_json(message)?;

        match msg.message_type {
            MCPMessageType::Request => {
                // Handle subscribe and unsubscribe commands
                if msg.command == "subscribe" {
                    return self.handle_subscribe(client_id, msg);
                } else if msg.command == "unsubscribe" {
                    return self.handle_unsubscribe(client_id, msg);
                }

                // Handle other request messages
                let response = self.handle_command(msg)?;
                Ok(Some(response.to_json()?))
            }
            MCPMessageType::Notification => {
                // Process notification (no response needed)
                self.handle_notification(client_id.to_string(), msg)?;
                Ok(None)
            }
            MCPMessageType::Response | MCPMessageType::Error => {
                // Server should not receive response or error messages
                Err(MCPError::ProtocolError(
                    "Server received unexpected response or error message".to_string(),
                ))
            }
        }
    }

    /// Handle subscribe request
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client ID
    /// * `request` - Subscribe request message
    ///
    /// # Returns
    ///
    /// A result containing an optional response message
    fn handle_subscribe(&self, client_id: &str, request: MCPMessage) -> MCPResult<Option<String>> {
        // Extract topic from request payload
        let topic = if let Some(payload) = &request.payload {
            if let Some(topic) = payload.get("topic") {
                if let Some(topic_str) = topic.as_str() {
                    topic_str.to_string()
                } else {
                    return Err(MCPError::ProtocolError(
                        "Topic must be a string".to_string(),
                    ));
                }
            } else {
                return Err(MCPError::ProtocolError(
                    "Missing 'topic' field in subscription request".to_string(),
                ));
            }
        } else {
            return Err(MCPError::ProtocolError(
                "Missing payload in subscription request".to_string(),
            ));
        };

        // Subscribe the client to the topic
        self.subscribe_client(client_id, &topic)?;

        // Create response
        let response = MCPMessage::new_response(
            request.id,
            "subscribe".to_string(),
            Some(serde_json::json!({
                "result": "ok",
                "topic": topic
            })),
        );

        Ok(Some(response.to_json()?))
    }

    /// Handle unsubscribe request
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client ID
    /// * `request` - Unsubscribe request message
    ///
    /// # Returns
    ///
    /// A result containing an optional response message
    fn handle_unsubscribe(
        &self,
        client_id: &str,
        request: MCPMessage,
    ) -> MCPResult<Option<String>> {
        // Extract topic from request payload
        let topic = if let Some(payload) = &request.payload {
            if let Some(topic) = payload.get("topic") {
                if let Some(topic_str) = topic.as_str() {
                    topic_str.to_string()
                } else {
                    return Err(MCPError::ProtocolError(
                        "Topic must be a string".to_string(),
                    ));
                }
            } else {
                return Err(MCPError::ProtocolError(
                    "Missing 'topic' field in unsubscription request".to_string(),
                ));
            }
        } else {
            return Err(MCPError::ProtocolError(
                "Missing payload in unsubscription request".to_string(),
            ));
        };

        // Check for special case of unsubscribing from all topics
        if topic == "*" {
            self.unsubscribe_client_all(client_id)?;
        } else {
            // Unsubscribe the client from the topic
            self.unsubscribe_client(client_id, &topic)?;
        }

        // Create response
        let response = MCPMessage::new_response(
            request.id,
            "unsubscribe".to_string(),
            Some(serde_json::json!({
                "result": "ok",
                "topic": topic
            })),
        );

        Ok(Some(response.to_json()?))
    }

    /// Handle a command request
    ///
    /// # Arguments
    ///
    /// * `request` - Request message
    ///
    /// # Returns
    ///
    /// A result containing the response message
    fn handle_command(&self, message: MCPMessage) -> MCPResult<MCPMessage> {
        let command_name = message.command.clone();

        // Check for custom handlers
        {
            let handlers = safe_lock(&self.command_handlers, "handle_command - get handlers")?;
            if let Some(handler) = handlers.get(&command_name) {
                return handler(message);
            }
        }

        // Use the command registry if available
        if let Some(registry) = &self.command_registry {
            // Try to find and execute the command
            let args: Vec<String> = if let Some(payload) = &message.payload {
                self.json_to_args(payload)?
            } else {
                vec![]
            };

            // Try to execute the command using the command registry execute method
            let result = registry.execute(&command_name, &args);

            match result {
                Ok(output) => {
                    // Create a success response
                    let response = MCPMessage::new_response(
                        message.id,
                        message.command,
                        Some(serde_json::json!({"result": output})),
                    );

                    Ok(response)
                }
                Err(e) => {
                    // Create an error response
                    let error = format!("Command execution failed: {}", e);
                    let response = MCPMessage::new_error(message.id, message.command, error);

                    Ok(response)
                }
            }
        } else {
            // Unknown command
            Err(MCPError::ProtocolError(format!(
                "Unknown command: {}",
                command_name
            )))
        }
    }

    /// Handle a notification message
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client ID
    /// * `notification` - Notification message
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    fn handle_notification(&self, client_id: String, notification: MCPMessage) -> MCPResult<()> {
        let topic = notification.command.clone();

        debug!(
            "Received notification '{}' from client {}",
            topic, client_id
        );

        // Get subscribers for this topic (excluding sender)
        let subscribers = {
            let topic_subscribers = safe_lock(
                &self.topic_subscribers,
                "handle_notification - get subscribers",
            )?;
            topic_subscribers
                .get(&topic)
                .map(|subs| {
                    subs.iter()
                        .filter(|id| **id != client_id)
                        .cloned()
                        .collect::<HashSet<String>>()
                })
                .unwrap_or_default()
        };

        // Forward the notification to all subscribers
        if !subscribers.is_empty() {
            let json = notification.to_json()?;
            let clients = safe_lock(&self.clients, "handle_notification - get clients")?;

            for subscriber_id in subscribers {
                if let Some(stream) = clients.get(&subscriber_id) {
                    match safe_lock(stream, "handle_notification - subscriber stream") {
                        Ok(mut stream_guard) => {
                            if let Err(e) = stream_guard.write_all(format!("{}\n", json).as_bytes())
                            {
                                warn!(
                                    "Failed to forward notification to client {}: {}",
                                    subscriber_id, e
                                );
                            }
                        }
                        Err(e) => {
                            warn!("Failed to lock stream for client {}: {}", subscriber_id, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Convert JSON payload to command arguments
    fn json_to_args(&self, payload: &Value) -> MCPResult<Vec<String>> {
        if let Some(args) = payload.get("args") {
            if let Some(args_array) = args.as_array() {
                Ok(args_array
                    .iter()
                    .filter_map(|arg| arg.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>())
            } else {
                // If args is not an array, try to use it as a single string arg
                if let Some(arg_str) = args.as_str() {
                    Ok(vec![arg_str.to_string()])
                } else {
                    Ok(vec![])
                }
            }
        } else {
            // No args field, use empty args
            Ok(vec![])
        }
    }
}
