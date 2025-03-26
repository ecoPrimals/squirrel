//! API client for the Squirrel Web UI.
//!
//! This module provides a typed client for communicating with the Squirrel Web API.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

mod auth;
mod commands;
mod jobs;
mod websocket;

pub use auth::{AuthClient, AuthConfig, LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse};
pub use commands::{CommandClient, CommandDefinition, CommandExecution, CommandStatus, CommandSummary};
pub use jobs::{JobClient, JobDefinition, JobStatus, JobSummary};
pub use websocket::{WebSocketClient, WebSocketConfig, WebSocketEvent, WebSocketSubscription};

/// API client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClientConfig {
    /// Base URL for the API
    pub base_url: String,
    /// Timeout for API requests in seconds
    pub request_timeout_secs: u64,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// WebSocket configuration
    pub websocket: WebSocketConfig,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            request_timeout_secs: 30,
            auth: AuthConfig::default(),
            websocket: WebSocketConfig::default(),
        }
    }
}

/// Main API client for the Squirrel Web UI
#[derive(Debug, Clone)]
pub struct ApiClient {
    /// Configuration
    config: ApiClientConfig,
    /// Authentication client
    auth: Arc<AuthClient>,
    /// Command client
    commands: Arc<CommandClient>,
    /// Job client
    jobs: Arc<JobClient>,
    /// WebSocket client
    websocket: Arc<WebSocketClient>,
}

impl ApiClient {
    /// Create a new API client with the given configuration
    pub fn new(config: ApiClientConfig) -> Self {
        let auth = Arc::new(AuthClient::new(config.auth.clone(), config.base_url.clone(), Duration::from_secs(config.request_timeout_secs)));
        let commands = Arc::new(CommandClient::new(config.base_url.clone(), Duration::from_secs(config.request_timeout_secs)));
        let jobs = Arc::new(JobClient::new(config.base_url.clone(), Duration::from_secs(config.request_timeout_secs)));
        let websocket = Arc::new(WebSocketClient::new(config.websocket.clone()));
        
        Self {
            config,
            auth,
            commands,
            jobs,
            websocket,
        }
    }
    
    /// Get the authentication client
    pub fn auth(&self) -> &AuthClient {
        &self.auth
    }
    
    /// Get the command client
    pub fn commands(&self) -> &CommandClient {
        &self.commands
    }
    
    /// Get the job client
    pub fn jobs(&self) -> &JobClient {
        &self.jobs
    }
    
    /// Get the WebSocket client
    pub fn websocket(&self) -> &WebSocketClient {
        &self.websocket
    }
    
    /// Initialize the API client
    pub async fn init(&self) -> Result<()> {
        // Initialize WebSocket connection
        self.websocket.connect().await?;
        
        Ok(())
    }
} 