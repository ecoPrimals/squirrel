//! MCP command for the Squirrel CLI
//!
//! This module implements a command to work with the Machine Context Protocol.

use clap::{Command as ClapCommand, Arg, ArgAction};
use tracing::{debug, info, warn};
use std::time::Duration;
use std::sync::Arc;
use serde_json::Value;

use commands::{Command, CommandResult, CommandError};
use crate::mcp::{MCPClient, MCPServer};

/// Command to work with the Machine Context Protocol
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct MCPCommand;


impl MCPCommand {
    /// Create a new MCP command
    pub fn new() -> Arc<dyn Command> {
        Arc::new(Self)
    }
    
    /// Parse a string into a serde_json::Value
    fn parse_args_to_json(args: &[String]) -> Option<Value> {
        if args.is_empty() {
            return None;
        }
        
        // Join the args and parse as JSON
        let json_str = args.join(" ");
        match serde_json::from_str(&json_str) {
            Ok(value) => Some(value),
            Err(_) => {
                // If not valid JSON, treat as a simple string array
                Some(serde_json::Value::Array(
                    args.iter().map(|s| serde_json::Value::String(s.clone())).collect()
                ))
            }
        }
    }
}

impl Command for MCPCommand {
    fn name(&self) -> &str {
        "mcp"
    }
    
    fn description(&self) -> &str {
        "Interact with the Machine Context Protocol (MCP)"
    }
    
    fn parser(&self) -> ClapCommand {
        let name: &str = "mcp";
        let description: &str = "Interact with the Machine Context Protocol (MCP)";
        
        ClapCommand::new(name)
            .about(description)
            .subcommand(
                ClapCommand::new("server")
                    .about("Start or control an MCP server")
                    .arg(
                        Arg::new("host")
                            .long("host")
                            .help("Host to bind to")
                            .default_value("127.0.0.1")
                    )
                    .arg(
                        Arg::new("port")
                            .long("port")
                            .short('p')
                            .help("Port to listen on")
                            .default_value("8778")
                    )
                    .arg(
                        Arg::new("action")
                            .help("Server action (start/stop/status)")
                            .default_value("start")
                            .value_parser(["start", "stop", "status"])
                    )
            )
            .subcommand(
                ClapCommand::new("client")
                    .about("Connect to an MCP server")
                    .arg(
                        Arg::new("host")
                            .long("host")
                            .help("Server host")
                            .default_value("127.0.0.1")
                    )
                    .arg(
                        Arg::new("port")
                            .long("port")
                            .short('p')
                            .help("Server port")
                            .default_value("8778")
                    )
                    .arg(
                        Arg::new("timeout")
                            .long("timeout")
                            .short('t')
                            .help("Connection timeout in seconds")
                            .default_value("5")
                    )
                    .arg(
                        Arg::new("interactive")
                            .long("interactive")
                            .short('i')
                            .help("Start interactive mode")
                            .action(ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("command")
                            .help("Command to execute")
                            .conflicts_with("interactive")
                    )
                    .arg(
                        Arg::new("args")
                            .help("Command arguments")
                            .num_args(0..)
                            .conflicts_with("interactive")
                    )
            )
            .subcommand(
                ClapCommand::new("subscribe")
                    .about("Subscribe to a topic on an MCP server")
                    .arg(
                        Arg::new("host")
                            .long("host")
                            .help("Server host")
                            .default_value("127.0.0.1")
                    )
                    .arg(
                        Arg::new("port")
                            .long("port")
                            .short('p')
                            .help("Server port")
                            .default_value("8778")
                    )
                    .arg(
                        Arg::new("timeout")
                            .long("timeout")
                            .short('t')
                            .help("Connection timeout in seconds")
                            .default_value("5")
                    )
                    .arg(
                        Arg::new("topic")
                            .help("Topic to subscribe to")
                            .required(true)
                    )
                    .arg(
                        Arg::new("wait")
                            .long("wait")
                            .short('w')
                            .help("Wait for notifications (keep connection open)")
                            .action(ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("count")
                            .long("count")
                            .short('c')
                            .help("Number of notifications to wait for before exiting (0 = unlimited)")
                            .default_value("1")
                            .value_parser(clap::value_parser!(u32))
                    )
            )
            .subcommand(
                ClapCommand::new("publish")
                    .about("Publish a notification to a topic on an MCP server")
                            .arg(
                        Arg::new("host")
                            .long("host")
                            .help("Server host")
                            .default_value("127.0.0.1")
                            )
                            .arg(
                        Arg::new("port")
                            .long("port")
                            .short('p')
                            .help("Server port")
                            .default_value("8778")
                    )
                            .arg(
                        Arg::new("timeout")
                            .long("timeout")
                            .short('t')
                            .help("Connection timeout in seconds")
                            .default_value("5")
                    )
                            .arg(
                        Arg::new("topic")
                            .help("Topic to publish to")
                                    .required(true)
                            )
                            .arg(
                        Arg::new("payload")
                            .help("JSON payload to send")
                            .required(false)
                    )
            )
            .subcommand(
                ClapCommand::new("status")
                    .about("Check MCP server status")
            )
            .subcommand(
                ClapCommand::new("protocol")
                    .about("Show MCP protocol information")
            )
    }
    
    fn execute(&self, _args: &[String]) -> Result<String, CommandError> {
        // This will need to be implemented differently to match the Command trait's requirements
        // For now, we return a temporary message that this needs to be updated
        Ok("MCP command execution needs to be updated to match the Command trait".to_string())
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[allow(dead_code)]
impl MCPCommand {
    /// Handle the server subcommand
    async fn handle_server_command(&self, matches: &clap::ArgMatches) -> CommandResult<serde_json::Value> {
        let host = matches.get_one::<String>("host").unwrap();
        let port_str = matches.get_one::<String>("port").unwrap();
        let port = port_str.parse::<u16>().map_err(|e| {
            CommandError::ValidationError(format!("Invalid port number: {}", e))
        })?;
        
        let action = matches.get_one::<String>("action").unwrap();
        
        match action.as_str() {
            "start" => {
                info!("Starting MCP server on {}:{}", host, port);
                
                // Create and start the server
                let server = {
                    // Get the command registry from context - for the MCP demo, we'll just create a new one
                    // In a full implementation, this would come from the main app's registry
                    let registry = commands::CommandRegistry::new();
                    
                    // Register some basic commands for demo purposes
                    registry.register("version", std::sync::Arc::new(commands::builtin::VersionCommand))
                        .map_err(|e| {
                            CommandError::ValidationError(format!("Failed to register version command: {}", e))
                        })?;
                    
                    let registry_arc = std::sync::Arc::new(registry);
                    
                    // Create the server with the registry
                    MCPServer::new(Some(host), Some(port))
                        .with_command_registry(registry_arc)
                };
                
                match server.start() {
                    Ok(_) => {
                        // Keep the server running for a while
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        
                        if server.is_running() {
                            Ok(serde_json::json!({
                                "status": "running",
                                "host": host,
                                "port": port,
                                "message": format!("MCP server started on {}:{} with command registry", host, port),
                                "command_registry": server.has_command_registry(),
                            }))
                        } else {
                            Err(CommandError::ExecutionError(
                                format!("Failed to start MCP server on {}:{}", host, port)
                            ))
                        }
                    }
                    Err(e) => {
                        Err(CommandError::ExecutionError(
                            format!("Failed to start MCP server: {}", e)
                        ))
                    }
                }
            }
            "stop" => {
                // TODO: Implement server stop functionality
                // This would require access to a globally stored server instance
                Err(CommandError::ExecutionError(
                    "Server stop command not implemented yet".to_string()
                ))
            }
            "status" => {
                // TODO: Implement server status check
                // This would require access to a globally stored server instance
                Ok(serde_json::json!({
                    "message": "MCP server status check not implemented yet"
                }))
            }
            _ => {
                Err(CommandError::ValidationError(
                    format!("Unknown server action: {}", action)
                ))
            }
        }
    }
    
    /// Handle the client subcommand
    async fn handle_client_command(&self, matches: &clap::ArgMatches) -> CommandResult<serde_json::Value> {
        let host = matches.get_one::<String>("host").unwrap();
        let port_str = matches.get_one::<String>("port").unwrap();
        let timeout_str = matches.get_one::<String>("timeout").unwrap();
        
        let port = port_str.parse::<u16>().map_err(|e| {
            CommandError::ValidationError(format!("Invalid port number: {}", e))
        })?;
        
        let timeout = timeout_str.parse::<u64>().map_err(|e| {
            CommandError::ValidationError(format!("Invalid timeout value: {}", e))
        })?;
        
        let interactive = matches.get_flag("interactive");
        
        // Create a client instance
        let mut client = MCPClient::new(host.clone(), port);
        client = client.with_timeout(Duration::from_secs(timeout));
        
        // Attempt to connect with timeout
        match client.connect(None) {
            Ok(_) => {
                debug!("Connected to MCP server at {}:{}", host, port);
                
        if interactive {
                    // Run in interactive mode
                    match client.run_interactive() {
                        Ok(_) => {
                            Ok(serde_json::json!({
                                "status": "success",
                                "message": "Interactive session completed",
                            }))
                        }
                        Err(e) => {
                            Err(CommandError::ExecutionError(
                                format!("Interactive session error: {}", e)
                            ))
                        }
                    }
                } else if let Some(cmd) = matches.get_one::<String>("command") {
                    // Execute a single command
                    let args: Vec<String> = matches
                        .get_many::<String>("args")
                        .map(|args| args.cloned().collect())
                        .unwrap_or_default();
                    
                    let payload = Self::parse_args_to_json(&args);
                    
                    match client.send_command(cmd, payload) {
                        Ok(response) => {
                            // Return the response payload or a success message
                            if let Some(payload) = response.payload {
                                Ok(payload)
                            } else {
                                Ok(serde_json::json!({
                                    "status": "success",
                                    "message": format!("Command '{}' executed successfully", cmd),
                                }))
                            }
                        }
                        Err(e) => {
                            Err(CommandError::ExecutionError(
                                format!("Command execution failed: {}", e)
                            ))
                        }
                    }
        } else {
                    // No command specified, just show connection status
                    Ok(serde_json::json!({
                        "status": "connected",
                        "host": host,
                        "port": port,
                        "message": format!("Connected to MCP server at {}:{}", host, port),
                    }))
                }
            }
            Err(e) => {
                Err(CommandError::ExecutionError(
                    format!("Failed to connect to MCP server at {}:{}: {}", host, port, e)
                ))
            }
        }
    }
    
    /// Handle the subscribe command
    async fn handle_subscribe_command(&self, matches: &clap::ArgMatches) -> CommandResult<serde_json::Value> {
        let host = matches.get_one::<String>("host").unwrap();
        let port_str = matches.get_one::<String>("port").unwrap();
        let timeout_str = matches.get_one::<String>("timeout").unwrap();
        let topic = matches.get_one::<String>("topic").unwrap();
        let wait = matches.get_flag("wait");
        let count = matches.get_one::<u32>("count").unwrap();
        
        let port = port_str.parse::<u16>().map_err(|e| {
            CommandError::ValidationError(format!("Invalid port number: {}", e))
        })?;
        
        let timeout = timeout_str.parse::<u64>().map_err(|e| {
            CommandError::ValidationError(format!("Invalid timeout value: {}", e))
        })?;
        
        // Create a client instance and connect
        let mut client = MCPClient::new(host.clone(), port);
        client = client.with_timeout(Duration::from_secs(timeout));
        
        // Attempt to connect
        match client.connect(None) {
            Ok(_) => {
                debug!("Connected to MCP server at {}:{}", host, port);
                
                // Subscribe to the topic
                if wait {
                    // Set up a notification counter
                    let notification_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
                    let counter = notification_count.clone();
                    
                    // Subscribe and wait for notifications
                    info!("Subscribing to topic '{}' and waiting for notifications...", topic);
                    
                    // Create a channel to wait for notifications
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(10);
                    let tx = Arc::new(tokio::sync::Mutex::new(tx));
                    
                    // Subscribe to the topic with a callback that forwards notifications to the channel
                    let sub_id = match client.subscribe(topic, move |topic_param, msg| {
                        let _current = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let json = serde_json::to_string_pretty(msg).unwrap_or_else(|_| format!("{:?}", msg));
                        let topic_string = topic_param.to_string(); // Clone the topic to own it
                        
                        // Send the notification to the channel
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            if let Ok(_tx) = tx_clone.lock().await.try_send(json) {
                                debug!("Forwarded notification from topic {}", topic_string);
                            }
                        });
                        
                        Ok(())
                    }) {
                        Ok(id) => id,
                        Err(e) => {
                            return Err(CommandError::ExecutionError(
                                format!("Failed to subscribe to topic '{}': {}", topic, e)
                            ));
                        }
                    };
                    
                    // Track received notifications
                    let mut notifications = Vec::new();
                    
                    // Wait for the specified number of notifications or until canceled
                    loop {
                        tokio::select! {
                            Some(json) = rx.recv() => {
                                info!("Received notification: {}", json);
                                notifications.push(json);
                                
                                // If we've received the desired count, exit
                                if *count > 0 && notification_count.load(std::sync::atomic::Ordering::SeqCst) >= *count {
                                    break;
                                }
                            }
                            _ = tokio::signal::ctrl_c() => {
                                info!("Received interrupt signal, unsubscribing...");
                                break;
                            }
                        }
                    }
                    
                    // Unsubscribe and disconnect
                    if let Err(e) = client.unsubscribe(sub_id) {
                        warn!("Failed to unsubscribe from topic '{}': {}", topic, e);
                    }
                    
                    Ok(serde_json::json!({
                        "status": "success",
                        "topic": topic,
                        "notifications_received": notification_count.load(std::sync::atomic::Ordering::SeqCst),
                        "notifications": notifications,
                    }))
                } else {
                    // Just subscribe and immediately return
                    match client.send_command("subscribe", Some(serde_json::json!({
                        "topic": topic
                    }))) {
                        Ok(response) => {
                            info!("Subscribed to topic '{}'", topic);
                            
                            // Return the response
                            if let Some(payload) = response.payload {
                                Ok(payload)
                            } else {
                                Ok(serde_json::json!({
                                    "status": "success",
                                    "topic": topic,
                                    "message": format!("Subscribed to topic '{}'", topic),
                                }))
                            }
                        }
                        Err(e) => {
                            Err(CommandError::ExecutionError(
                                format!("Failed to subscribe to topic '{}': {}", topic, e)
                            ))
                        }
                    }
                }
            }
            Err(e) => {
                Err(CommandError::ExecutionError(
                    format!("Failed to connect to MCP server at {}:{}: {}", host, port, e)
                ))
            }
        }
    }
    
    /// Handle the publish command
    async fn handle_publish_command(&self, matches: &clap::ArgMatches) -> CommandResult<serde_json::Value> {
        let host = matches.get_one::<String>("host").unwrap();
        let port_str = matches.get_one::<String>("port").unwrap();
        let timeout_str = matches.get_one::<String>("timeout").unwrap();
        let topic = matches.get_one::<String>("topic").unwrap();
        
        let port = port_str.parse::<u16>().map_err(|e| {
            CommandError::ValidationError(format!("Invalid port number: {}", e))
        })?;
        
        let timeout = timeout_str.parse::<u64>().map_err(|e| {
            CommandError::ValidationError(format!("Invalid timeout value: {}", e))
        })?;
        
        // Parse payload if provided
        let payload = if let Some(payload_str) = matches.get_one::<String>("payload") {
            match serde_json::from_str(payload_str) {
                Ok(json) => Some(json),
                Err(e) => {
                    return Err(CommandError::ValidationError(
                        format!("Invalid JSON payload: {}", e)
                    ));
                }
            }
        } else {
            None
        };
        
        // Create a client instance and connect
        let mut client = MCPClient::new(host.clone(), port);
        client = client.with_timeout(Duration::from_secs(timeout));
        
        // Attempt to connect
        match client.connect(None) {
            Ok(_) => {
                debug!("Connected to MCP server at {}:{}", host, port);
                
                // Send the notification
                match client.send_notification(topic, payload.clone()) {
                    Ok(_) => {
                        info!("Notification sent to topic '{}'", topic);
                        
                        Ok(serde_json::json!({
                            "status": "success",
                            "topic": topic,
                            "message": format!("Notification sent to topic '{}'", topic),
                            "payload": payload,
                        }))
                    }
                    Err(e) => {
                        Err(CommandError::ExecutionError(
                            format!("Failed to send notification to topic '{}': {}", topic, e)
                        ))
                    }
                }
            }
            Err(e) => {
                Err(CommandError::ExecutionError(
                    format!("Failed to connect to MCP server at {}:{}: {}", host, port, e)
                ))
            }
        }
    }
    
    /// Handle the status subcommand
    async fn handle_status_command(&self) -> CommandResult<serde_json::Value> {
        // TODO: Implement status check logic
        // This would check if a local MCP server is running
        Ok(serde_json::json!({
            "message": "MCP status check not implemented yet"
        }))
    }
    
    /// Handle the protocol subcommand
    async fn handle_protocol_command(&self) -> CommandResult<serde_json::Value> {
        Ok(serde_json::json!({
            "protocol": "Machine Context Protocol (MCP)",
            "version": "1.0.0",
            "description": "A protocol for structured communication between machines and humans",
            "message_types": [
                {
                    "type": "request",
                    "description": "Request a command execution"
                },
                {
                    "type": "response",
                    "description": "Response to a request"
                },
                {
                    "type": "notification",
                    "description": "One-way notification (no response expected)"
                },
                {
                    "type": "error",
                    "description": "Error response"
                }
            ],
            "message_format": {
                "id": "Unique message identifier (UUID)",
                "type": "Message type (request, response, notification, error)",
                "command": "Command or topic name",
                "payload": "Optional JSON payload",
                "error": "Optional error message (for error responses)"
            },
            "subscription_model": {
                "description": "Pub/sub model for notifications",
                "commands": {
                    "subscribe": "Subscribe to a topic (payload should contain 'topic' field)",
                    "unsubscribe": "Unsubscribe from a topic (payload should contain 'topic' field)",
                    "publish": "Send a notification to a topic (command field is the topic)"
                }
            }
        }))
    }
} 
