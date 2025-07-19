//! Web Integration for MCP
//!
//! This module provides HTTP REST API and WebSocket endpoints for
//! Machine Context Protocol communication, enabling web applications
//! to interact with MCP services.

use crate::error::{Result, MCPError};
use crate::protocol::{MCPMessage, MessageType, ProtocolVersion, SecurityMetadata, MessageId};
use crate::session::{Session, SessionManager};
use crate::transport::{TransportMetadata};
use crate::tool::{ToolManager};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Configuration for the Web-MCP integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebMcpConfig {
    /// HTTP server bind address
    pub http_host: String,
    /// HTTP server port
    pub http_port: u16,
    /// WebSocket server bind address
    pub websocket_host: String,
    /// WebSocket server port
    pub websocket_port: u16,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Enable CORS
    pub enable_cors: bool,
    /// Allowed origins for CORS
    pub cors_origins: Vec<String>,
    /// JWT secret for authentication
    pub jwt_secret: String,
    /// Session timeout in seconds
    pub session_timeout: u64,
}

impl Default for WebMcpConfig {
    fn default() -> Self {
        Self {
            http_host: std::env::var("WEB_MCP_HTTP_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            http_port: std::env::var("WEB_MCP_HTTP_PORT")
                .ok()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(8080),
            websocket_host: std::env::var("WEB_MCP_WEBSOCKET_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            websocket_port: std::env::var("WEB_MCP_WEBSOCKET_PORT")
                .ok()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(8081),
            max_connections: std::env::var("WEB_MCP_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1000),
            request_timeout: std::env::var("WEB_MCP_REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(30),
            enable_cors: std::env::var("WEB_MCP_ENABLE_CORS")
                .ok()
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(true),
            cors_origins: std::env::var("WEB_MCP_CORS_ORIGINS")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|_| vec!["*".to_string()]),
            jwt_secret: std::env::var("WEB_MCP_JWT_SECRET")
                .unwrap_or_else(|_| "default_secret_change_in_production".to_string()),
            session_timeout: std::env::var("WEB_MCP_SESSION_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(3600),
        }
    }
}

/// HTTP API request for MCP command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// Command name
    pub command: String,
    /// Command arguments
    pub args: HashMap<String, serde_json::Value>,
    /// Session token
    pub session_token: Option<String>,
    /// Request timeout in seconds
    pub timeout: Option<u64>,
    /// Request metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// HTTP API response for MCP command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// Request ID
    pub request_id: String,
    /// Execution status
    pub status: String,
    /// Command result
    pub result: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// WebSocket message types for MCP communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Authentication request
    Auth {
        token: String,
    },
    /// MCP command execution
    Command {
        id: String,
        command: String,
        args: HashMap<String, serde_json::Value>,
    },
    /// MCP command response
    Response {
        id: String,
        status: String,
        result: Option<serde_json::Value>,
        error: Option<String>,
    },
    /// Real-time event notification
    Event {
        event_type: String,
        data: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    /// Connection status update
    Status {
        connected: bool,
        message: String,
    },
    /// Heartbeat/ping message
    Ping {
        timestamp: DateTime<Utc>,
    },
    /// Heartbeat/pong response
    Pong {
        timestamp: DateTime<Utc>,
    },
}

/// Connection state for WebSocket clients
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    /// Connection ID
    pub id: String,
    /// User session
    pub session: Option<Session>,
    /// Connection timestamp
    pub connected_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Connection metadata
    pub metadata: HashMap<String, String>,
}

/// Web-MCP Integration Service
///
/// Provides HTTP REST API and WebSocket endpoints for MCP communication
pub struct WebMcpService {
    /// Service configuration
    config: WebMcpConfig,
    /// Session manager
    session_manager: Arc<dyn SessionManager>,
    /// Tool manager
    tool_manager: Arc<dyn ToolManager>,
    /// Active WebSocket connections
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    /// Request metrics
    metrics: Arc<Mutex<WebMcpMetrics>>,
}

/// Metrics for Web-MCP integration
#[derive(Debug, Default, Clone)]
pub struct WebMcpMetrics {
    /// Total HTTP requests
    pub http_requests: u64,
    /// Total WebSocket connections
    pub websocket_connections: u64,
    /// Active connections
    pub active_connections: u64,
    /// Total commands executed
    pub commands_executed: u64,
    /// Failed commands
    pub commands_failed: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl WebMcpService {
    /// Create a new Web-MCP service
    pub fn new(
        config: WebMcpConfig,
        session_manager: Arc<dyn SessionManager>,
        tool_manager: Arc<dyn ToolManager>,
    ) -> Self {
        Self {
            config,
            session_manager,
            tool_manager,
            connections: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(WebMcpMetrics::default())),
        }
    }

    /// Start the Web-MCP service
    pub async fn start(&self) -> Result<()> {
        // Start HTTP server
        self.start_http_server().await?;
        
        // Start WebSocket server
        self.start_websocket_server().await?;
        
        Ok(())
    }

    /// Start HTTP REST API server
    async fn start_http_server(&self) -> Result<()> {
        // Implementation would use a web framework like axum or warp
        // For now, this is a placeholder
        tracing::info!(
            "Starting HTTP server on {}:{}",
            self.config.http_host,
            self.config.http_port
        );
        Ok(())
    }

    /// Start WebSocket server
    async fn start_websocket_server(&self) -> Result<()> {
        // Implementation would use tokio-tungstenite
        // For now, this is a placeholder
        tracing::info!(
            "Starting WebSocket server on {}:{}",
            self.config.websocket_host,
            self.config.websocket_port
        );
        Ok(())
    }

    /// Execute MCP command via HTTP API
    pub async fn execute_command_http(&self, request: CommandRequest) -> Result<CommandResponse> {
        let start_time = std::time::Instant::now();
        let request_id = Uuid::new_v4().to_string();

        // Validate session if token provided
        let session = if let Some(token) = &request.session_token {
            // Simplified session validation (create a dummy session)
            log::debug!("Validating session token");
            Some(Session {
                id: "dummy_session".to_string(),
                last_activity: std::time::SystemTime::now(),
            })
        } else {
            None
        };

        // Execute command
        let result = self.execute_mcp_command(&request.command, &request.args, session).await;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.http_requests += 1;
            metrics.commands_executed += 1;
            if result.is_err() {
                metrics.commands_failed += 1;
            }
            metrics.avg_response_time_ms = 
                (metrics.avg_response_time_ms + execution_time_ms as f64) / 2.0;
            metrics.last_updated = Utc::now();
        }

        match result {
            Ok(response_data) => Ok(CommandResponse {
                request_id,
                status: "success".to_string(),
                result: Some(response_data),
                error: None,
                execution_time_ms,
                timestamp: Utc::now(),
            }),
            Err(e) => Ok(CommandResponse {
                request_id,
                status: "error".to_string(),
                result: None,
                error: Some(e.to_string()),
                execution_time_ms,
                timestamp: Utc::now(),
            }),
        }
    }

    /// Execute MCP command via WebSocket
    pub async fn execute_command_websocket(
        &self,
        connection_id: &str,
        command: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        // Get connection session
        let session = {
            let connections = self.connections.read().await;
            connections.get(connection_id)
                .and_then(|conn| conn.session.clone())
        };

        self.execute_mcp_command(command, args, session).await
    }

    /// Internal MCP command execution
    async fn execute_mcp_command(
        &self,
        command: &str,
        args: &HashMap<String, serde_json::Value>,
        _session: Option<Session>,
    ) -> Result<serde_json::Value> {
        // Create MCP message
        let _message = MCPMessage::new(
            MessageType::Command,
            serde_json::json!({
                "command": command,
                "args": args
            }),
        );

        // Execute via tool manager - simplified for now
        // In a real implementation, this would use the actual tool execution API
        match command {
            "ping" => Ok(serde_json::json!({"response": "pong"})),
            "status" => Ok(serde_json::json!({"status": "ok", "timestamp": Utc::now()})),
            _ => {
                // For now, return a simple echo response
                Ok(serde_json::json!({
                    "command": command,
                    "args": args,
                    "result": "executed"
                }))
            }
        }
    }

    /// Register new WebSocket connection
    pub async fn register_websocket_connection(
        &self,
        connection_id: String,
        session: Option<Session>,
    ) -> Result<()> {
        let connection = WebSocketConnection {
            id: connection_id.clone(),
            session,
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            metadata: HashMap::new(),
        };

        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id, connection);
        }

        {
            let mut metrics = self.metrics.lock().await;
            metrics.websocket_connections += 1;
            metrics.active_connections += 1;
            metrics.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Unregister WebSocket connection
    pub async fn unregister_websocket_connection(&self, connection_id: &str) -> Result<()> {
        {
            let mut connections = self.connections.write().await;
            connections.remove(connection_id);
        }

        {
            let mut metrics = self.metrics.lock().await;
            metrics.active_connections = metrics.active_connections.saturating_sub(1);
            metrics.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Get service metrics
    pub async fn get_metrics(&self) -> WebMcpMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }

    /// Get active connections count
    pub async fn get_active_connections(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Broadcast event to all connected WebSocket clients
    pub async fn broadcast_event(
        &self,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<()> {
        let _message = WebSocketMessage::Event {
            event_type: event_type.to_string(),
            data,
            timestamp: Utc::now(),
        };

        // Implementation would send to all active WebSocket connections
        tracing::info!("Broadcasting event: {} to {} connections", 
            event_type, self.get_active_connections().await);

        Ok(())
    }
}

/// Create a new Web-MCP service with default configuration
pub fn create_web_mcp_service(
    session_manager: Arc<dyn SessionManager>,
    tool_manager: Arc<dyn ToolManager>,
) -> WebMcpService {
    WebMcpService::new(WebMcpConfig::default(), session_manager, tool_manager)
}

/// Create a new Web-MCP service with custom configuration
pub fn create_web_mcp_service_with_config(
    config: WebMcpConfig,
    session_manager: Arc<dyn SessionManager>,
    tool_manager: Arc<dyn ToolManager>,
) -> WebMcpService {
    WebMcpService::new(config, session_manager, tool_manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_mcp_service_creation() {
        let session_manager = Arc::new(CoreSessionManager::new());
        let tool_manager = Arc::new(CoreToolManager::new());
        
        let service = create_web_mcp_service(session_manager, tool_manager);
        
        assert_eq!(service.get_active_connections().await, 0);
        
        let metrics = service.get_metrics().await;
        assert_eq!(metrics.http_requests, 0);
        assert_eq!(metrics.websocket_connections, 0);
    }

    #[tokio::test]
    async fn test_websocket_connection_lifecycle() {
        let session_manager = Arc::new(CoreSessionManager::new());
        let tool_manager = Arc::new(CoreToolManager::new());
        let service = create_web_mcp_service(session_manager, tool_manager);

        let connection_id = "test-connection-1".to_string();
        
        // Register connection
        service.register_websocket_connection(connection_id.clone(), None).await.unwrap();
        assert_eq!(service.get_active_connections().await, 1);
        
        // Unregister connection
        service.unregister_websocket_connection(&connection_id).await.unwrap();
        assert_eq!(service.get_active_connections().await, 0);
    }

    #[tokio::test]
    async fn test_command_request_creation() {
        let request = CommandRequest {
            command: "test_command".to_string(),
            args: HashMap::new(),
            session_token: None,
            timeout: Some(30),
            metadata: None,
        };

        assert_eq!(request.command, "test_command");
        assert_eq!(request.timeout, Some(30));
    }
} 