//! Core MCPServer implementation
//!
//! This module contains the main implementation of the MCPServer including:
//! - Connection handling
//! - Message processing
//! - Listener task management

use crate::error::{MCPError, Result};
use crate::message::{Message, MessageBuilder};
use crate::message_router::MessageRouter;
use crate::protocol::adapter_wire::{WireFormatAdapter, DomainObject as WireDomainObject};
use crate::transport::Transport;

use super::{ClientConnection, ConnectionHandler};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, watch};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Start the listener task to accept client connections
pub async fn start_listener_task(
    listener: TcpListener,
    clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
    wire_format_adapter: Arc<WireFormatAdapter>,
    message_router: Arc<MessageRouter>,
    shutdown_rx: watch::Receiver<bool>,
    connection_handlers: Arc<RwLock<Vec<Box<dyn ConnectionHandler>>>>,
    state: Arc<RwLock<super::ServerState>>,
) -> Result<()> {
    info!("Starting MCP server listener task on {}", listener.local_addr()?);
    let mut shutdown_rx = shutdown_rx.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                // Accept new connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            info!(client_addr = %addr, "Accepted new client connection");
                            
                            // Generate unique client ID
                            let client_id = Uuid::new_v4().to_string();
                            
                            // Create transport for the connection
                            use crate::transport::tcp::{TcpTransport, TcpTransportConfig};
                            let transport_config = TcpTransportConfig {
                                remote_address: addr,
                                buffer_size: 8192,
                                timeout: Duration::from_secs(30),
                            };
                            
                            let transport = Arc::new(TcpTransport::new_with_stream(
                                stream, 
                                addr, 
                                transport_config
                            ));
                            
                            // Create session for the client
                            use crate::client::session::Session;
                            let session_config = crate::client::session::SessionConfig {
                                timeout: Duration::from_secs(300), // 5 minutes
                                max_pending_requests: 100,
                                enable_compression: false,
                            };
                            let session = Arc::new(Session::new(&client_id, session_config));
                            
                            // Create client connection
                            use super::types::ClientConnection;
                            let client = ClientConnection {
                                client_id: client_id.clone(),
                                address: addr,
                                transport: transport.clone(),
                                session,
                            };
                            
                            // Store client connection
                            {
                                let mut clients_guard = clients.write().await;
                                clients_guard.insert(client_id.clone(), client.clone());
                            }
                            
                            // Notify connection handlers
                            {
                                let handlers_guard = connection_handlers.read().await;
                                for handler in handlers_guard.iter() {
                                    if let Err(e) = handler.handle_connection(&client_id).await {
                                        error!("Error in connection handler: {}", e);
                                    }
                                }
                            }
                            
                            // Spawn task to handle client messages
                            let clients_for_task = clients.clone();
                            let wire_format_adapter_for_task = wire_format_adapter.clone();
                            let message_router_for_task = message_router.clone();
                            
                            tokio::spawn(async move {
                                if let Err(e) = handle_client_connection(
                                    client,
                                    clients_for_task,
                                    wire_format_adapter_for_task,
                                    message_router_for_task,
                                ).await {
                                    error!("Error handling client connection: {}", e);
                                }
                            });
                        },
                        Err(e) => {
                            error!(error = %e, "Failed to accept client connection");
                        }
                    }
                },
                // Shutdown signal
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("Listener task received shutdown signal");
                        break;
                    }
                }
            }
        }
        
        // Update state
        let mut state_guard = state.write().await;
        *state_guard = super::ServerState::Stopped;
        
        info!("Listener task completed");
    });
    
    Ok(())
}

/// Handle a client connection
pub async fn handle_client_connection(
    client: ClientConnection,
    clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
    wire_format_adapter: Arc<WireFormatAdapter>,
    message_router: Arc<MessageRouter>,
) -> Result<()> {
    info!(client_id = %client.client_id, "Starting message handling for client");
    
    // Main message loop
    loop {
        // Receive message from client
        let wire_message = match client.transport.receive_message().await {
            Ok(msg) => msg,
            Err(e) => {
                // Check if connection was closed
                let error_msg = format!("{}", e);
                if error_msg.contains("connection closed") || error_msg.contains("EOF") {
                    info!(client_id = %client.client_id, "Client connection closed");
                    break;
                }
                error!(client_id = %client.client_id, error = %e, "Failed to receive message from client");
                break;
            }
        };
        
        // Convert wire format to internal message
        let message = match wire_format_adapter.wire_to_internal(&wire_message) {
            Ok(msg) => msg,
            Err(e) => {
                error!(client_id = %client.client_id, error = %e, "Failed to convert wire message to internal format");
                continue;
            }
        };
        
        // Route the message
        match message_router.route_message(&message).await {
            Ok(responses) => {
                // Send responses back to client
                for response in responses {
                    // Convert to wire format
                    match wire_format_adapter.internal_to_wire(&response) {
                        Ok(wire_response) => {
                            if let Err(e) = client.transport.send_message(&wire_response).await {
                                error!(client_id = %client.client_id, error = %e, "Failed to send response to client");
                                break;
                            }
                        }
                        Err(e) => {
                            error!(client_id = %client.client_id, error = %e, "Failed to convert response to wire format");
                        }
                    }
                }
                
                // Send acknowledgment
                if message.requires_ack {
                    let ack = MessageBuilder::new()
                        .with_message_type("ack")
                        .with_correlation_id(message.id.clone())
                        .build();
                        
                    if let Ok(wire_ack) = wire_format_adapter.internal_to_wire(&ack) {
                        if let Err(e) = client.transport.send_message(&wire_ack).await {
                            error!(client_id = %client.client_id, error = %e, "Failed to send acknowledgment to client");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!(client_id = %client.client_id, error = %e, "Message routing failed");
                
                // Send error response
                let error_response = MessageBuilder::new()
                    .with_message_type("error")
                    .with_correlation_id(message.id.clone())
                    .with_content(json!({
                        "error": "routing_error",
                        "message": e.to_string()
                    }))
                    .build();
                    
                if let Ok(wire_error) = wire_format_adapter.internal_to_wire(&error_response) {
                    let _ = client.transport.send_message(&wire_error).await;
                }
            }
        }
    }
    
    // Clean up: remove client from active connections
    {
        let mut clients_guard = clients.write().await;
        clients_guard.remove(&client.client_id);
    }
    
    info!(client_id = %client.client_id, "Client message handling loop ended");
    Ok(())
}

