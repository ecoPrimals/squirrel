//! Application state
//!
//! This module provides the central application state that is shared across
//! all handlers and services.

// This module will contain state management functionality and utilities. 

use std::sync::Arc;
use sqlx::Pool;
use crate::db::SqlitePool as DbPool;
use crate::websocket::ConnectionManager as WebSocketManager;
use crate::auth::AuthService;
use crate::api::commands::CommandService;
use crate::config::Config;
use crate::plugins_legacy::PluginManager;
use crate::plugins::WebPluginRegistry;
use crate::mcp::{McpClient, McpCommandClient, ContextManager};

/// Trait for machine context clients
#[async_trait::async_trait]
pub trait MachineContextClient: Send + Sync {
    /// Send a message to the MCP
    async fn send_message(&self, message: &str) -> Result<String, String>;
    
    /// Receive a message from the MCP
    async fn receive_message(&self) -> Result<String, String>;
}

/// Mock implementation for machine context client
#[derive(Clone)]
pub struct MockMcpClient {
    // Placeholder fields for mock implementation
}

impl MockMcpClient {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl MachineContextClient for MockMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, String> {
        // Return a mock message
        Ok(format!("Mock response to: {}", message))
    }
    
    async fn receive_message(&self) -> Result<String, String> {
        // Return a mock message
        Ok("Mock received message".to_string())
    }
}

/// Application state, shared across all handlers and services
pub struct AppState {
    /// Database connection pool
    pub db: DbPool,
    
    /// Application configuration
    pub config: Config,
    
    /// Machine context protocol client (legacy, deprecated)
    pub mcp: Option<Arc<dyn MachineContextClient>>,
    
    /// Modern MCP client - basic client interface
    pub mcp_client: Arc<dyn McpClient>,
    
    /// Modern MCP command client - command-specific interface
    pub mcp_command_client: Arc<dyn McpCommandClient>,
    
    /// WebSocket manager
    pub ws_manager: Arc<WebSocketManager>,
    
    /// Authentication service
    pub auth: AuthService,
    
    /// Command service
    pub command_service: Arc<dyn CommandService>,
    
    /// Legacy plugin manager
    pub plugin_manager: Arc<PluginManager>,
    
    /// Modern plugin registry
    pub plugin_registry: Option<Arc<WebPluginRegistry>>,
    
    /// MCP Context manager for enhanced context preservation
    pub context_manager: Arc<ContextManager>,
}

impl AppState {
    /// Get the database connection pool
    pub fn get_db(&self) -> &DbPool {
        &self.db
    }
    
    /// Get the legacy machine context client
    pub fn get_mcp(&self) -> Option<&Arc<dyn MachineContextClient>> {
        self.mcp.as_ref()
    }
    
    /// Get the modern machine context client
    pub fn get_mcp_client(&self) -> &Arc<dyn McpClient> {
        &self.mcp_client
    }
    
    /// Get the modern machine context command client
    pub fn get_mcp_command_client(&self) -> &Arc<dyn McpCommandClient> {
        &self.mcp_command_client
    }
    
    /// Get the websocket manager
    pub fn get_ws_manager(&self) -> &Arc<WebSocketManager> {
        &self.ws_manager
    }
    
    /// Get the auth service
    pub fn get_auth(&self) -> &AuthService {
        &self.auth
    }
    
    /// Get the command service
    pub fn get_command_service(&self) -> &Arc<dyn CommandService> {
        &self.command_service
    }
    
    /// Get the legacy plugin manager
    pub fn get_plugin_manager(&self) -> &Arc<PluginManager> {
        &self.plugin_manager
    }
    
    /// Get the modern plugin registry
    pub fn get_plugin_registry(&self) -> Option<&Arc<WebPluginRegistry>> {
        self.plugin_registry.as_ref()
    }
    
    /// Get the context manager
    pub fn get_context_manager(&self) -> &Arc<ContextManager> {
        &self.context_manager
    }
} 