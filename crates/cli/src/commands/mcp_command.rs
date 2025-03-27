//! MCP command for the Squirrel CLI
//!
//! This module implements a command to work with the Machine Context Protocol.

use clap::{Command as ClapCommand, Arg};
use log::{debug, info};
use serde_json::Value;
use commands::{Command, CommandResult};
use crate::commands::error::CommandError;

/// MCP command implementation
#[derive(Debug, Clone)]
pub struct McpCommand {
    // No state needed for now
}

impl Default for McpCommand {
    fn default() -> Self {
        Self {}
    }
}

impl McpCommand {
    /// Create a new MCP command
    pub fn new() -> Self {
        Self {}
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

impl Command for McpCommand {
    /// Get the name of the command
    fn name(&self) -> &str {
        "mcp"
    }
    
    /// Get the description of the command
    fn description(&self) -> &str {
        "Machine Context Protocol client"
    }
    
    /// Create the command parser
    fn parser(&self) -> ClapCommand {
        // Using hardcoded strings instead of self references to avoid lifetime issues
        ClapCommand::new("mcp")
            .about("Machine Context Protocol client")
            .arg(Arg::new("action")
                .help("MCP action (connect, disconnect, send, status)")
                .required(true)
                .index(1))
            .arg(Arg::new("host")
                .help("MCP server host")
                .required(false)
                .index(2))
            .arg(Arg::new("port")
                .help("MCP server port")
                .required(false)
                .index(3))
            .arg(Arg::new("topic")
                .help("Topic to send message to")
                .required(false)
                .index(2))
            .arg(Arg::new("message")
                .help("Message to send")
                .required(false)
                .index(3))
    }
    
    /// Execute the command
    fn execute(&self, args: &[String]) -> CommandResult<String> {
        debug!("Executing MCP command with args: {:?}", args);

        if args.is_empty() {
            return Ok("MCP commands: connect, disconnect, send, status".to_string());
        }

        let action = &args[0];
        
        match action.as_str() {
            "connect" => self.handle_connect(&args[1..]),
            "disconnect" => self.handle_disconnect(),
            "send" => self.handle_send(&args[1..]),
            "status" => self.handle_status(),
            _ => Err(CommandError::InvalidArguments(format!("Unknown MCP action: {}", action)).into()),
        }
    }
    
    /// Clone the command
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
    
    /// Return help text for the command
    fn help(&self) -> String {
        "Machine Context Protocol (MCP) client\n\nActions:\n  connect <host:port> - Connect to an MCP server\n  disconnect - Disconnect from the current server\n  send <topic> <message> - Send a message to a topic\n  status - Show the current connection status".to_string()
    }
}

impl McpCommand {
    fn handle_connect(&self, args: &[String]) -> CommandResult<String> {
        let endpoint = if args.is_empty() {
            "localhost:7000"
        } else {
            &args[0]
        };

        debug!("Connecting to MCP server: {}", endpoint);
        
        // For now just simulate connecting to MCP
        Ok(format!("Connected to MCP server at {}", endpoint))
    }

    fn handle_disconnect(&self) -> CommandResult<String> {
        debug!("Disconnecting from MCP");
        
        // Simulate disconnecting
        Ok("Disconnected from MCP server".to_string())
    }
    
    fn handle_send(&self, args: &[String]) -> CommandResult<String> {
        if args.len() < 2 {
            return Err(CommandError::InvalidArguments("Send requires topic and message arguments".to_string()).into());
        }

        let topic = &args[0];
        let message = &args[1];

        info!("Sending message: {} to topic: {}", message, topic);
        Ok(format!("Message sent to topic {}: {}", topic, message))
    }

    fn handle_status(&self) -> CommandResult<String> {
        debug!("Checking MCP status");
        
        // Simulate connection status
        Ok("MCP client: Connected to localhost:7000".to_string())
    }
} 
