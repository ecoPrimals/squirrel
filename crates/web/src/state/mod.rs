//! Application state management for the web interface.

// This module will contain state management functionality and utilities. 

use std::sync::Arc;
use sqlx::SqlitePool;
use async_trait::async_trait;
use crate::config::Config;
use crate::websocket::ConnectionManager;
use crate::auth::AuthService;
use crate::mcp::McpCommandClient;
use crate::handlers::commands::CommandService;
use crate::api::error::AppError;

/// Machine Context Protocol client trait (legacy)
#[async_trait]
pub trait MachineContextClient: Send + Sync + 'static {
    /// Send a message to the MCP
    async fn send_message(&self, message: &str) -> Result<String, String>;
    
    /// Receive a message from the MCP
    async fn receive_message(&self) -> Result<String, String>;
}

/// Default mock MCP client for testing
pub struct DefaultMockMCPClient;

#[async_trait]
impl MachineContextClient for DefaultMockMCPClient {
    async fn send_message(&self, message: &str) -> Result<String, String> {
        Ok(format!("Sent: {}", message))
    }
    
    async fn receive_message(&self) -> Result<String, String> {
        Ok("Mock response".to_string())
    }
}

/// Application state shared across all requests
pub struct AppState {
    /// Database connection pool
    pub db: SqlitePool,
    /// Application configuration
    pub config: Config,
    /// Machine Context Protocol client (legacy)
    pub mcp: Option<Arc<dyn MachineContextClient>>,
    /// MCP Command client for advanced command execution
    pub mcp_command: Option<Arc<dyn McpCommandClient>>,
    /// WebSocket connection manager
    pub ws_manager: ConnectionManager,
    /// Authentication service
    pub auth: AuthService,
    /// Command service
    pub command_service: Option<Arc<dyn CommandService>>,
}

impl AppState {
    /// Get the command service
    pub fn get_command_service(&self) -> Result<&Arc<dyn CommandService>, AppError> {
        self.command_service.as_ref()
            .ok_or_else(|| AppError::Internal("Command service not configured".to_string()))
    }
} 