// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

/// Port management module for MCP (Machine Context Protocol)
///
/// This module provides functionality for managing network ports used by MCP services.
/// It handles port configuration, connection tracking, and port lifecycle management.
/// The implementation is designed to be async-compatible and thread-safe.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio::time::timeout;
use tracing::{info, warn, error, debug};

use crate::error::Result;
use crate::error::production::ProductionError;

/// Port configuration for MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// Port number to listen on
    pub port: u16,
    /// IP address to bind to
    pub bind_address: String,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout: u64,
    /// Buffer size for connections
    pub buffer_size: usize,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            max_connections: 100,
            connection_timeout: 30,
            keep_alive_timeout: 300,
            buffer_size: 8192,
        }
    }
}

/// State of a port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortState {
    /// Whether the port is listening
    pub is_listening: bool,
    /// Number of active connections
    pub active_connections: usize,
    /// Total connections since start
    pub total_connections: u64,
    /// Last connection time
    pub last_connection: chrono::DateTime<chrono::Utc>,
    /// Errors encountered
    pub error_count: u64,
    /// Last error message
    pub last_error: Option<String>,
}

impl Default for PortState {
    fn default() -> Self {
        Self {
            is_listening: false,
            active_connections: 0,
            total_connections: 0,
            last_connection: chrono::Utc::now(),
            error_count: 0,
            last_error: None,
        }
    }
}

/// Connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Connection ID
    pub id: String,
    /// Remote address
    pub remote_addr: SocketAddr,
    /// Connection start time
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Last activity time
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
}

/// Port manager for MCP server
pub struct MCPPort {
    /// Port configuration, protected by a read-write lock for thread safety
    config: Arc<RwLock<PortConfig>>,
    
    /// Port state, protected by a read-write lock for thread safety
    state: Arc<RwLock<PortState>>,
    
    /// Active connections
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
    
    /// Connection handler
    connection_handler: Option<Arc<dyn ConnectionHandler>>,
}

/// Trait for handling incoming connections
#[async_trait::async_trait]
pub trait ConnectionHandler: Send + Sync {
    /// Handle a new connection
    async fn handle_connection(&self, stream: TcpStream, addr: SocketAddr) -> Result<()>;
    
    /// Handle connection close
    async fn handle_disconnect(&self, connection_id: &str) -> Result<()>;
}

/// Default connection handler
pub struct DefaultConnectionHandler {
    /// Connection timeout
    timeout: Duration,
}

impl DefaultConnectionHandler {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

#[async_trait::async_trait]
impl ConnectionHandler for DefaultConnectionHandler {
    async fn handle_connection(&self, mut stream: TcpStream, addr: SocketAddr) -> Result<()> {
        debug!("Handling connection from {}", addr);
        
        // Set up connection timeout
        let connection_future = async {
            // Basic echo server for demonstration
            let mut buffer = [0; 1024];
            loop {
                match stream.readable().await {
                    Ok(()) => {
                        match stream.try_read(&mut buffer) {
                            Ok(0) => {
                                debug!("Connection closed by client: {}", addr);
                                break;
                            }
                            Ok(n) => {
                                debug!("Received {} bytes from {}", n, addr);
                                // Echo back the data
                                if let Err(e) = stream.try_write(&buffer[..n]) {
                                    if e.kind() != std::io::ErrorKind::WouldBlock {
                                        warn!("Failed to write to {}: {}", addr, e);
                                        break;
                                    }
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // No data available right now, continue
                                tokio::task::yield_now().await;
                            }
                            Err(e) => {
                                warn!("Error reading from {}: {}", addr, e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Error waiting for readable on {}: {}", addr, e);
                        break;
                    }
                }
            }
            Ok::<(), ProductionError>(())
        };
        
        // Apply timeout
        match timeout(self.timeout, connection_future).await {
            Ok(Ok(())) => {
                debug!("Connection {} completed successfully", addr);
            }
            Ok(Err(e)) => {
                warn!("Connection {} failed: {}", addr, e);
            }
            Err(_) => {
                warn!("Connection {} timed out", addr);
            }
        }
        
        Ok(())
    }
    
    async fn handle_disconnect(&self, connection_id: &str) -> Result<()> {
        debug!("Connection {} disconnected", connection_id);
        Ok(())
    }
}

impl MCPPort {
    /// Creates a new port manager with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the port
    ///
    /// # Returns
    ///
    /// A new `MCPPort` instance initialized with the given configuration
    #[must_use] pub fn new(config: PortConfig) -> Self {
        let state = PortState::default();

        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(state)),
            connections: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
            connection_handler: None,
        }
    }

    /// Set a custom connection handler
    pub fn with_connection_handler(mut self, handler: Arc<dyn ConnectionHandler>) -> Self {
        self.connection_handler = Some(handler);
        self
    }

    /// Start the port
    ///
    /// Begins listening for incoming connections on the configured port.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the port cannot be started or if the internal
    /// state lock cannot be acquired
    pub async fn start(&mut self) -> Result<()> {
        let config = self.config.read().await.clone();
        let bind_addr = format!("{}:{}", config.bind_address, config.port);
        
        info!("Starting MCP port listener on {}", bind_addr);
        
        // Create TCP listener
        let listener = match TcpListener::bind(&bind_addr).await {
            Ok(listener) => listener,
            Err(e) => {
                let error_msg = format!("Failed to bind to {}: {}", bind_addr, e);
                error!("{}", error_msg);
                
                // Update state with error
                {
                    let mut state = self.state.write().await;
                    state.error_count += 1;
                    state.last_error = Some(error_msg.clone());
                }
                
                return Err(ProductionError::network_endpoint(
                    error_msg,
                    bind_addr,
                    None,
                    Some(5000),
                ).into());
            }
        };
        
        // Update state to listening
        {
            let mut state = self.state.write().await;
            state.is_listening = true;
            state.last_error = None;
        }
        
        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        // Create connection handler
        let handler = self.connection_handler.clone().unwrap_or_else(|| {
            Arc::new(DefaultConnectionHandler::new(Duration::from_secs(config.connection_timeout)))
        });
        
        // Clone necessary data for the listener task
        let state = Arc::clone(&self.state);
        let connections = Arc::clone(&self.connections);
        
        // Start listener task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle new connections
                    connection_result = listener.accept() => {
                        match connection_result {
                            Ok((stream, addr)) => {
                                debug!("Accepted connection from {}", addr);
                                
                                // Update state
                                {
                                    let mut state = state.write().await;
                                    state.active_connections += 1;
                                    state.total_connections += 1;
                                    state.last_connection = chrono::Utc::now();
                                }
                                
                                // Create connection info
                                let connection_id = format!("{}:{}", addr.ip(), addr.port());
                                let connection_info = ConnectionInfo {
                                    id: connection_id.clone(),
                                    remote_addr: addr,
                                    connected_at: chrono::Utc::now(),
                                    last_activity: chrono::Utc::now(),
                                    bytes_sent: 0,
                                    bytes_received: 0,
                                };
                                
                                // Store connection
                                {
                                    let mut connections = connections.write().await;
                                    connections.insert(connection_id.clone(), connection_info);
                                }
                                
                                // Handle connection in separate task
                                let handler = Arc::clone(&handler);
                                let state = Arc::clone(&state);
                                let connections = Arc::clone(&connections);
                                
                                tokio::spawn(async move {
                                    // Handle the connection
                                    if let Err(e) = handler.handle_connection(stream, addr).await {
                                        warn!("Connection handler failed for {}: {}", addr, e);
                                    }
                                    
                                    // Clean up connection
                                    {
                                        let mut connections = connections.write().await;
                                        connections.remove(&connection_id);
                                    }
                                    
                                    // Update state
                                    {
                                        let mut state = state.write().await;
                                        state.active_connections = state.active_connections.saturating_sub(1);
                                    }
                                    
                                    // Notify handler of disconnect
                                    if let Err(e) = handler.handle_disconnect(&connection_id).await {
                                        warn!("Disconnect handler failed for {}: {}", connection_id, e);
                                    }
                                });
                            }
                            Err(e) => {
                                warn!("Failed to accept connection: {}", e);
                                
                                // Update error state
                                {
                                    let mut state = state.write().await;
                                    state.error_count += 1;
                                    state.last_error = Some(e.to_string());
                                }
                            }
                        }
                    }
                    
                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        info!("Received shutdown signal, stopping listener");
                        break;
                    }
                }
            }
            
            // Update state to not listening
            {
                let mut state = state.write().await;
                state.is_listening = false;
            }
            
            info!("Port listener stopped");
        });
        
        info!("MCP port started successfully on {}", bind_addr);
        Ok(())
    }

    /// Stop the port
    ///
    /// Stops listening for incoming connections on the configured port.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the port cannot be stopped or if the internal
    /// state lock cannot be acquired
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping MCP port");
        
        // Send shutdown signal
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            if let Err(e) = shutdown_tx.send(()).await {
                warn!("Failed to send shutdown signal: {}", e);
            }
        }
        
        // Wait for connections to close (with timeout)
        let timeout_duration = Duration::from_secs(
            std::env::var("PORT_SHUTDOWN_TIMEOUT_SECS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10)
        );
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout_duration {
            let active_connections = {
                let state = self.state.read().await;
                state.active_connections
            };
            
            if active_connections == 0 {
                break;
            }
            
            debug!("Waiting for {} connections to close", active_connections);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Force close remaining connections
        {
            let active_connections = {
                let state = self.state.read().await;
                state.active_connections
            };
            
            if active_connections > 0 {
                warn!("Forcefully closing {} remaining connections", active_connections);
                
                // Clear connections
                {
                    let mut connections = self.connections.write().await;
                    connections.clear();
                }
                
                // Update state
                {
                    let mut state = self.state.write().await;
                    state.active_connections = 0;
                }
            }
        }
        
        // Update state to not listening
        {
            let mut state = self.state.write().await;
            state.is_listening = false;
        }
        
        info!("MCP port stopped successfully");
        Ok(())
    }

    /// Update port configuration
    ///
    /// Updates the configuration of the port.
    ///
    /// # Arguments
    ///
    /// * `new_config` - The new configuration to apply
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be updated or if the
    /// internal config lock cannot be acquired
    pub async fn update_config(&self, new_config: PortConfig) -> Result<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        info!("Port configuration updated");
        Ok(())
    }

    /// Get the current port configuration
    ///
    /// # Returns
    ///
    /// The current port configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be read or if the
    /// internal config lock cannot be acquired
    pub async fn get_config(&self) -> Result<PortConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }

    /// Get the current port state
    ///
    /// # Returns
    ///
    /// The current port state
    ///
    /// # Errors
    ///
    /// Returns an error if the state cannot be read or if the internal
    /// state lock cannot be acquired
    pub async fn get_state(&self) -> Result<PortState> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    /// Get information about active connections
    ///
    /// # Returns
    ///
    /// A vector of connection information
    ///
    /// # Errors
    ///
    /// Returns an error if the connections cannot be read or if the
    /// internal connections lock cannot be acquired
    pub async fn get_connections(&self) -> Result<Vec<ConnectionInfo>> {
        let connections = self.connections.read().await;
        Ok(connections.values().cloned().collect())
    }

    /// Get connection by ID
    ///
    /// # Arguments
    ///
    /// * `connection_id` - The ID of the connection to retrieve
    ///
    /// # Returns
    ///
    /// The connection information if found
    ///
    /// # Errors
    ///
    /// Returns an error if the connections cannot be read or if the
    /// internal connections lock cannot be acquired
    pub async fn get_connection(&self, connection_id: &str) -> Result<Option<ConnectionInfo>> {
        let connections = self.connections.read().await;
        Ok(connections.get(connection_id).cloned())
    }

    /// Check if the port is listening
    ///
    /// # Returns
    ///
    /// True if the port is listening, false otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if the state cannot be read or if the internal
    /// state lock cannot be acquired
    pub async fn is_listening(&self) -> Result<bool> {
        let state = self.state.read().await;
        Ok(state.is_listening)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_port_lifecycle() {
        let config = PortConfig {
            port: 0, // Use port 0 to get any available port
            bind_address: "127.0.0.1".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            keep_alive_timeout: 30,
            buffer_size: 1024,
        };
        
        let mut port = MCPPort::new(config);
        
        // Port should not be listening initially
        assert!(!port.is_listening().await.expect("should succeed"));
        
        // Start the port
        port.start().await.expect("should succeed");
        
        // Port should be listening now
        assert!(port.is_listening().await.expect("should succeed"));
        
        // Stop the port
        port.stop().await.expect("should succeed");
        
        // Port should not be listening anymore
        assert!(!port.is_listening().await.expect("should succeed"));
    }
    
    #[tokio::test]
    async fn test_port_config_update() {
        let config = PortConfig::default();
        let port = MCPPort::new(config);
        
        let new_config = PortConfig {
            port: 9999,
            bind_address: "0.0.0.0".to_string(),
            max_connections: 200,
            connection_timeout: 60,
            keep_alive_timeout: 600,
            buffer_size: 16384,
        };
        
        port.update_config(new_config.clone()).await.expect("should succeed");
        
        let current_config = port.get_config().await.expect("should succeed");
        assert_eq!(current_config.port, 9999);
        assert_eq!(current_config.bind_address, "0.0.0.0");
        assert_eq!(current_config.max_connections, 200);
    }
} 