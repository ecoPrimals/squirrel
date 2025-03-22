//! Application state management for the web interface.

// This module will contain state management functionality and utilities. 

use std::sync::Arc;
use sqlx::SqlitePool;
use crate::config::Config;
use crate::websocket::ConnectionManager;
use crate::auth::AuthService;

/// Application state shared across all handlers
#[derive(Debug)]
pub struct AppState {
    /// Database connection pool
    pub db: SqlitePool,
    /// Application configuration
    pub config: Config,
    /// Machine Context Protocol client
    pub mcp: Option<Arc<dyn MachineContextClient>>,
    /// WebSocket connection manager
    pub ws_manager: ConnectionManager,
    /// Authentication service
    pub auth: AuthService,
}

/// Trait for MCP client
pub trait MachineContextClient: Send + Sync + std::fmt::Debug {
    /// Send a message to the MCP
    fn send_message(&self, message: &str) -> Result<(), String>;
    
    /// Receive a message from the MCP
    fn receive_message(&self) -> Result<String, String>;
}

/// Mock MCP client for testing
#[derive(Debug)]
pub struct DefaultMockMCPClient;

impl MachineContextClient for DefaultMockMCPClient {
    fn send_message(&self, _message: &str) -> Result<(), String> {
        // Mock implementation that always succeeds
        Ok(())
    }
    
    fn receive_message(&self) -> Result<String, String> {
        // Mock implementation that always returns a success message
        Ok("Mock response".to_string())
    }
} 