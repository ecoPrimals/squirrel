//! MCP server implementation
//!
//! Provides a server for the Machine Context Protocol, enabling
//! structured machine-to-machine communication.

use std::sync::{Arc, Mutex};
use std::io::{Write, BufRead, BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use log::{debug, info, error};
use serde_json::json;

use squirrel_commands::CommandRegistry;
use crate::mcp::protocol::{MCPMessage, MCPMessageType, MCPResult, MCPError};

/// An MCP command handler
type CommandHandler = Box<dyn Fn(&MCPMessage) -> MCPResult<MCPMessage> + Send + Sync>;

/// An MCP server that listens for and processes MCP protocol messages
pub struct MCPServer {
    /// Host to bind to
    host: String,
    
    /// Port to listen on
    port: u16,
    
    /// Command registry for executing commands
    registry: Arc<Mutex<CommandRegistry>>,
    
    /// Custom command handlers
    handlers: Arc<Mutex<Vec<(String, CommandHandler)>>>,
    
    /// Flag to control server shutdown
    running: Arc<Mutex<bool>>,
}

impl MCPServer {
    /// Create a new MCP server
    ///
    /// # Arguments
    ///
    /// * `host` - Host to bind to
    /// * `port` - Port to listen on
    /// * `registry` - Command registry for executing commands
    ///
    /// # Returns
    ///
    /// A new MCP server
    pub fn new(
        host: String,
        port: u16,
        registry: CommandRegistry,
    ) -> Self {
        Self {
            host,
            port,
            registry: Arc::new(Mutex::new(registry)),
            handlers: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Register a custom command handler
    ///
    /// # Arguments
    ///
    /// * `command` - Command name
    /// * `handler` - Handler function
    pub fn register_handler<F>(&mut self, command: &str, handler: F)
    where
        F: Fn(&MCPMessage) -> MCPResult<MCPMessage> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push((command.to_string(), Box::new(handler)));
    }
    
    /// Start the MCP server
    ///
    /// This method starts the server in a background thread.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn start(&self) -> MCPResult<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port))
            .map_err(|e| MCPError::ConnectionError(format!("Failed to bind to {}:{}: {}", self.host, self.port, e)))?;
        
        // Set non-blocking mode
        listener.set_nonblocking(true)?;
        
        // Set running flag
        *self.running.lock().unwrap() = true;
        
        // Clone Arc values for the thread
        let registry = Arc::clone(&self.registry);
        let handlers = Arc::clone(&self.handlers);
        let running = Arc::clone(&self.running);
        
        // Start server in a background thread
        thread::spawn(move || {
            info!("MCP server listening on {}:{}", listener.local_addr().unwrap().ip(), listener.local_addr().unwrap().port());
            
            while *running.lock().unwrap() {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        info!("New MCP connection from {}", addr);
                        
                        // Clone Arc values for the client thread
                        let registry_clone = Arc::clone(&registry);
                        let handlers_clone = Arc::clone(&handlers);
                        
                        // Handle client in a separate thread
                        thread::spawn(move || {
                            if let Err(e) = Self::handle_client(stream, registry_clone, handlers_clone) {
                                error!("Error handling MCP client: {}", e);
                            }
                        });
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No incoming connections, sleep and try again
                        thread::sleep(Duration::from_millis(100));
                    },
                    Err(e) => {
                        error!("Error accepting connection: {}", e);
                        break;
                    }
                }
            }
            
            info!("MCP server stopped");
        });
        
        Ok(())
    }
    
    /// Stop the MCP server
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
        info!("MCP server shutdown initiated");
    }
    
    /// Handle an MCP client connection
    ///
    /// # Arguments
    ///
    /// * `stream` - TCP stream for the client
    /// * `registry` - Command registry
    /// * `handlers` - Custom command handlers
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    fn handle_client(
        stream: TcpStream,
        registry: Arc<Mutex<CommandRegistry>>,
        handlers: Arc<Mutex<Vec<(String, CommandHandler)>>>,
    ) -> MCPResult<()> {
        let peer_addr = stream.peer_addr()?;
        info!("Handling MCP client: {}", peer_addr);
        
        // Set up buffered reader and writer
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = BufWriter::new(stream);
        
        // Create a buffer to read messages
        let mut buffer = String::new();
        
        // Process messages
        loop {
            // Reset buffer
            buffer.clear();
            
            // Read a line
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    // EOF - client disconnected
                    info!("MCP client disconnected: {}", peer_addr);
                    break;
                },
                Ok(_) => {
                    // Process the message
                    match Self::process_message(&buffer, &registry, &handlers) {
                        Ok(response) => {
                            // Send the response
                            let response_json = response.to_json()?;
                            writeln!(writer, "{}", response_json)?;
                            writer.flush()?;
                        },
                        Err(e) => {
                            error!("Error processing MCP message: {}", e);
                            
                            // Try to extract the message ID and command from the original message
                            let original = MCPMessage::from_json(&buffer).ok();
                            let id = original.as_ref().map(|msg| msg.id.clone()).unwrap_or_else(|| "unknown".to_string());
                            let command = original.as_ref().map(|msg| msg.command.clone()).unwrap_or_else(|| "unknown".to_string());
                            
                            // Create an error response
                            let error_response = MCPMessage::new_error(id, command, e.to_string());
                            let error_json = error_response.to_json()?;
                            
                            // Send the error response
                            writeln!(writer, "{}", error_json)?;
                            writer.flush()?;
                        }
                    }
                },
                Err(e) => {
                    return Err(MCPError::IoError(e));
                }
            }
        }
        
        Ok(())
    }
    
    /// Process an MCP message
    ///
    /// # Arguments
    ///
    /// * `message_json` - JSON message
    /// * `registry` - Command registry
    /// * `handlers` - Custom command handlers
    ///
    /// # Returns
    ///
    /// A Result containing the response message or an error
    fn process_message(
        message_json: &str,
        registry: &Arc<Mutex<CommandRegistry>>,
        handlers: &Arc<Mutex<Vec<(String, CommandHandler)>>>,
    ) -> MCPResult<MCPMessage> {
        // Parse the message
        let message = MCPMessage::from_json(message_json)?;
        debug!("Processing MCP message: {}", message);
        
        // Check message type
        match message.message_type {
            MCPMessageType::Request => {
                // Check for a custom handler first
                let handlers_lock = handlers.lock().unwrap();
                for (command, handler) in handlers_lock.iter() {
                    if command == &message.command {
                        return handler(&message);
                    }
                }
                
                // No custom handler, try to execute as a command
                let registry_lock = registry.lock().unwrap();
                
                // Get the command name and args from the payload
                let args = match &message.payload {
                    Some(payload) => {
                        if let Some(args_array) = payload.get("args").and_then(|a| a.as_array()) {
                            args_array.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect::<Vec<String>>()
                        } else {
                            Vec::new() // No args provided
                        }
                    },
                    None => Vec::new(),
                };
                
                // Execute the command
                match registry_lock.execute(&message.command, &args) {
                    Ok(result) => {
                        Ok(MCPMessage::new_response(
                            message.id,
                            message.command,
                            Some(json!({
                                "success": true,
                                "output": result,
                            })),
                        ))
                    },
                    Err(e) => {
                        Err(MCPError::CommandError(format!("Command execution failed: {}", e)))
                    }
                }
            },
            MCPMessageType::Notification => {
                // Just acknowledge notifications
                Ok(MCPMessage::new_response(
                    message.id,
                    message.command,
                    Some(json!({
                        "received": true,
                    })),
                ))
            },
            _ => {
                // We don't handle responses or errors
                Err(MCPError::ProtocolError(format!("Unexpected message type: {:?}", message.message_type)))
            }
        }
    }
} 