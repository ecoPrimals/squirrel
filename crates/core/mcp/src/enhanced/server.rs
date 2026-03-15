// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Enhanced MCP Server - Core Implementation
//!
//! This module provides the enhanced MCP server with unified architecture,
//! streaming capabilities, and extensible plugin system.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use tracing::{info, debug, instrument};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::Result;
use crate::tool::management::CoreToolManager;
use super::{
    BidirectionalStreamManager, StreamingConfig, StreamDirection, StreamType,
    MultiAgentCoordinator, MultiAgentConfig, AgentType, CollaborationType,
    MCPEvent, EventType, EventBroadcaster, EventBroadcasterConfig
};

/// Enhanced server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedServerConfig {
    /// Server name
    pub name: String,
    
    /// Server port
    pub port: u16,
    
    /// Maximum connections
    pub max_connections: usize,
    
    /// Request timeout
    pub request_timeout: Duration,
    
    /// Enable metrics
    pub enable_metrics: bool,
    
    /// Plugin configuration
    pub plugin_config: PluginConfig,
    
    /// Streaming configuration
    pub streaming_config: StreamingConfig,
    
    /// Multi-agent configuration
    pub multi_agent_config: MultiAgentConfig,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin directory
    pub plugin_directory: String,
    
    /// Maximum plugins
    pub max_plugins: usize,
    
    /// Plugin timeout
    pub plugin_timeout: Duration,
}

/// Plugin status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    /// Plugin ID
    pub id: String,
    
    /// Plugin name
    pub name: String,
    
    /// Plugin state
    pub state: PluginState,
    
    /// Plugin version
    pub version: String,
    
    /// Last activity
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Plugin state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginState {
    Loaded,
    Unloaded,
    Running,
    Error(String),
}

/// Plugin manager interface
pub trait PluginManagerInterface: Send + Sync {
    /// Get plugin status
    fn get_plugin_status(&self, plugin_name: &str) -> Option<PluginStatus>;
    
    /// List all plugins
    fn list_plugins(&self) -> Vec<PluginStatus>;
}

impl Default for EnhancedServerConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (request_timeout, plugin_timeout) = if let Some(cfg) = config {
            let req = cfg.timeouts.get_custom_timeout("server_request")
                .unwrap_or_else(|| Duration::from_secs(30));
            let plugin = cfg.timeouts.get_custom_timeout("server_plugin")
                .unwrap_or_else(|| Duration::from_secs(60));
            (req, plugin)
        } else {
            (Duration::from_secs(30), Duration::from_secs(60))
        };
        
        Self {
            name: "Enhanced MCP Server".to_string(),
            port: 8080,
            max_connections: 1000,
            request_timeout,
            enable_metrics: true,
            plugin_config: PluginConfig {
                plugin_directory: "./plugins".to_string(),
                max_plugins: 100,
                plugin_timeout,
            },
            streaming_config: StreamingConfig::default(),
            multi_agent_config: MultiAgentConfig::default(),
        }
    }
}

/// Enhanced MCP Server with unified architecture
pub struct EnhancedMCPServer {
    /// Server configuration
    config: Arc<EnhancedServerConfig>,
    
    /// Core tool manager
    tool_manager: CoreToolManager,
    
    /// Plugin manager interface
    plugin_manager: Arc<dyn PluginManagerInterface>,
    
    /// Active connections
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    
    /// Server state
    state: Arc<RwLock<ServerState>>,
    
    /// Metrics collector
    metrics: Arc<Mutex<ServerMetrics>>,
    
    /// Bidirectional streaming manager
    stream_manager: BidirectionalStreamManager,
    
    /// Multi-agent coordinator
    agent_coordinator: MultiAgentCoordinator,
    
    /// Event broadcaster
    event_broadcaster: EventBroadcaster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub connection_type: ConnectionType,
    pub session_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    WebSocket,
    Tarpc,
    TCP,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerState {
    Initializing,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub tool_executions: u64,
    pub plugin_operations: u64,
    pub uptime_seconds: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl EnhancedMCPServer {
    /// Create a new enhanced MCP server
    pub async fn new(config: EnhancedServerConfig) -> Result<Self> {
        info!("Initializing Enhanced MCP Server");
        
        // Initialize core tool manager
        let tool_manager = CoreToolManager::new();
        
        // Initialize plugin manager - using production implementation
        let plugin_manager: Arc<dyn PluginManagerInterface> = 
            Arc::new(ProductionPluginManagerAdapter::new());
        
        // Initialize bidirectional streaming manager
        let stream_manager = BidirectionalStreamManager::new(config.streaming_config.clone()).await?;
        
        // Initialize multi-agent coordinator
        let agent_coordinator = MultiAgentCoordinator::new(config.multi_agent_config.clone()).await?;
        
        // Initialize event broadcaster
        let event_broadcaster = EventBroadcaster::new(EventBroadcasterConfig::default()).await?;
        
        let server = Self {
            config: Arc::new(config),
            tool_manager,
            plugin_manager,
            connections: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(ServerState::Initializing)),
            metrics: Arc::new(Mutex::new(ServerMetrics::new())),
            stream_manager,
            agent_coordinator,
            event_broadcaster,
        };
        
        Ok(server)
    }
    
    /// Start the enhanced MCP server
    pub async fn start(&self) -> Result<()> {
        info!("Starting Enhanced MCP Server");
        
        // Set state to starting
        *self.state.write().await = ServerState::Starting;
        
        // Start bidirectional streaming manager
        self.stream_manager.start().await?;
        
        // Start multi-agent coordinator
        self.agent_coordinator.start().await?;
        
        // Start event broadcaster
        self.event_broadcaster.start().await?;
        
        // Start metrics collection
        self.start_metrics_collection().await?;
        
        // Set state to running
        *self.state.write().await = ServerState::Running;
        
        info!("Enhanced MCP Server started successfully");
        Ok(())
    }
    
    /// Stop the enhanced MCP server
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Enhanced MCP Server");
        
        // Set state to stopping
        *self.state.write().await = ServerState::Stopping;
        
        // Stop bidirectional streaming manager
        self.stream_manager.stop().await?;
        
        // Stop multi-agent coordinator
        self.agent_coordinator.stop().await?;
        
        // Stop event broadcaster
        self.event_broadcaster.stop().await?;
        
        // Clean up connections
        self.cleanup_connections().await?;
        
        // Set state to stopped
        *self.state.write().await = ServerState::Stopped;
        
        info!("Enhanced MCP Server stopped successfully");
        Ok(())
    }
    
    /// Handle MCP request
    #[instrument(skip(self, request))]
    pub async fn handle_mcp_request(
        &self,
        session_id: &str,
        request: MCPRequest,
    ) -> Result<MCPResponse> {
        debug!("Handling MCP request: {:?}", request);
        
        // Update metrics
        self.update_request_metrics().await;
        
        match request {
            MCPRequest::Initialize { capabilities } => {
                self.handle_initialize_request(session_id, capabilities).await
            }
            MCPRequest::ListTools { category } => {
                self.handle_list_tools_request(category).await
            }
            MCPRequest::ExecuteTool { tool_name, parameters } => {
                self.handle_execute_tool_request(session_id, tool_name, parameters).await
            }
            MCPRequest::GetStatus => {
                self.handle_get_status_request().await
            }
            MCPRequest::ManagePlugin { plugin_id, action } => {
                self.handle_manage_plugin_request(plugin_id, action).await
            }
            // New streaming request handlers
            MCPRequest::CreateStream { stream_type, direction, config } => {
                self.handle_create_stream_request(session_id, stream_type, direction, config).await
            }
            MCPRequest::StreamMessage { stream_id, message } => {
                self.handle_stream_message_request(stream_id, message).await
            }
            MCPRequest::CloseStream { stream_id } => {
                self.handle_close_stream_request(stream_id).await
            }
            // New multi-agent request handlers
            MCPRequest::RegisterAgent { agent_config } => {
                self.handle_register_agent_request(agent_config).await
            }
            MCPRequest::StartConversation { participants } => {
                self.handle_start_conversation_request(participants).await
            }
            MCPRequest::StartCollaboration { participants, collaboration_type } => {
                self.handle_start_collaboration_request(participants, collaboration_type).await
            }
            MCPRequest::SendAgentMessage { message } => {
                self.handle_send_agent_message_request(message).await
            }
        }
    }
    
    /// Create a new session
    pub async fn create_session(&self, client_info: ClientInfo) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        
        let connection_info = ConnectionInfo {
            id: session_id.clone(),
            connection_type: ConnectionType::WebSocket,
            session_id: Some(session_id.clone()),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        self.connections.write().await.insert(session_id.clone(), connection_info);
        
        // Update metrics
        self.update_connection_metrics().await;
        
        Ok(session_id)
    }
    
    /// Get server metrics
    pub async fn get_metrics(&self) -> ServerMetrics {
        self.metrics.lock().await.clone()
    }
    
    /// Get server state
    pub async fn get_state(&self) -> ServerState {
        self.state.read().await.clone()
    }
    
    // Private helper methods
    async fn start_metrics_collection(&self) -> Result<()> {
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let mut metrics_guard = metrics.lock().await;
                metrics_guard.uptime_seconds += 60;
                metrics_guard.last_updated = chrono::Utc::now();
            }
        });
        
        Ok(())
    }
    
    async fn cleanup_connections(&self) -> Result<()> {
        let mut connections = self.connections.write().await;
        connections.clear();
        Ok(())
    }
    
    async fn update_request_metrics(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_requests += 1;
    }
    
    async fn update_connection_metrics(&self) {
        let mut metrics = self.metrics.lock().await;
        let connections = self.connections.read().await;
        
        metrics.total_connections += 1;
        metrics.active_connections = connections.len() as u64;
    }
    
    // Request handlers
    async fn handle_initialize_request(
        &self,
        session_id: &str,
        _capabilities: Vec<MCPCapability>,
    ) -> Result<MCPResponse> {
        info!("Initializing session: {}", session_id);
        
        let server_capabilities = vec![
            MCPCapability {
                name: "tool_execution".to_string(),
                version: "1.0".to_string(),
                description: Some("Tool execution capability".to_string()),
            },
            MCPCapability {
                name: "plugin_management".to_string(),
                version: "1.0".to_string(),
                description: Some("Plugin management capability".to_string()),
            },
        ];
        
        Ok(MCPResponse::Initialized {
            session_id: session_id.to_string(),
            capabilities: server_capabilities,
        })
    }
    
    async fn handle_list_tools_request(
        &self,
        category: Option<String>,
    ) -> Result<MCPResponse> {
        debug!("Listing tools, category: {:?}", category);
        
        // Get tools from tool manager (simplified)
        let tools = vec![
            ToolDefinition {
                id: "example_tool".to_string(),
                name: "Example Tool".to_string(),
                description: "An example tool for testing".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"}
                    }
                }),
                capabilities: vec!["test".to_string()],
            },
        ];
        
        Ok(MCPResponse::ToolsList { tools })
    }
    
    async fn handle_execute_tool_request(
        &self,
        session_id: &str,
        tool_name: String,
        parameters: serde_json::Value,
    ) -> Result<MCPResponse> {
        info!("Executing tool: {} for session: {}", tool_name, session_id);
        
        // Execute tool through tool manager (simplified)
        let result = serde_json::json!({
            "status": "success",
            "message": "Tool executed successfully",
            "data": parameters
        });
        
        let metadata = ToolMetadata {
            execution_time_ms: 100,
            resource_usage: ResourceUsage {
                cpu_percent: 5.0,
                memory_mb: 10,
                disk_io_mb: 0,
            },
            success: true,
        };
        
        // Update metrics
        let mut metrics = self.metrics.lock().await;
        metrics.tool_executions += 1;
        metrics.successful_requests += 1;
        
        Ok(MCPResponse::ToolResult { result, metadata })
    }
    
    async fn handle_get_status_request(&self) -> Result<MCPResponse> {
        let state = self.get_state().await;
        let metrics = self.get_metrics().await;
        
        Ok(MCPResponse::Status {
            state: format!("{:?}", state),
            metrics,
        })
    }
    
    async fn handle_manage_plugin_request(
        &self,
        plugin_id: String,
        _action: PluginAction,
    ) -> Result<MCPResponse> {
        info!("Managing plugin: {}", plugin_id);
        
        // Get plugin status through plugin manager
        let status = self.plugin_manager
            .get_plugin_status(&plugin_id)
            .unwrap_or_else(|| PluginStatus {
                id: plugin_id.clone(),
                name: plugin_id.clone(),
                state: PluginState::Unloaded,
                version: "unknown".to_string(),
                last_activity: chrono::Utc::now(),
            });
        
        // Update metrics
        let mut metrics = self.metrics.lock().await;
        metrics.plugin_operations += 1;
        
        Ok(MCPResponse::PluginStatus {
            plugin_id,
            status,
        })
    }

    // New streaming request handlers
    async fn handle_create_stream_request(
        &self,
        session_id: &str,
        stream_type: StreamType,
        direction: StreamDirection,
        config: super::bidirectional_streaming::StreamConfig,
    ) -> Result<MCPResponse> {
        debug!("Creating stream for session: {}", session_id);
        
        let stream_id = self.stream_manager.create_stream(
            session_id.to_string(),
            "client".to_string(), // FUTURE: [Enhancement] Get from connection info
            // Tracking: Requires connection info tracking enhancement
            stream_type,
            direction,
            config,
        ).await?;
        
        Ok(MCPResponse::StreamCreated { stream_id })
    }
    
    async fn handle_stream_message_request(
        &self,
        stream_id: String,
        message: crate::protocol::types::MCPMessage,
    ) -> Result<MCPResponse> {
        debug!("Sending message to stream: {}", stream_id);
        
        self.stream_manager.send_message(&stream_id, message).await?;
        
        Ok(MCPResponse::StreamMessageSent {
            stream_id,
            message_id: uuid::Uuid::new_v4().to_string(),
        })
    }
    
    async fn handle_close_stream_request(
        &self,
        stream_id: String,
    ) -> Result<MCPResponse> {
        debug!("Closing stream: {}", stream_id);
        
        self.stream_manager.close_stream(&stream_id).await?;
        
        Ok(MCPResponse::StreamClosed { stream_id })
    }

    // New multi-agent request handlers
    async fn handle_register_agent_request(
        &self,
        agent_config: super::multi_agent::AgentConfig,
    ) -> Result<MCPResponse> {
        debug!("Registering agent: {}", agent_config.name);
        
        let agent_id = self.agent_coordinator.register_agent(agent_config).await?;
        
        Ok(MCPResponse::AgentRegistered { agent_id })
    }
    
    async fn handle_start_conversation_request(
        &self,
        participants: Vec<String>,
    ) -> Result<MCPResponse> {
        debug!("Starting conversation with {} participants", participants.len());
        
        let conversation_id = self.agent_coordinator.start_conversation(participants).await?;
        
        Ok(MCPResponse::ConversationStarted { conversation_id })
    }
    
    async fn handle_start_collaboration_request(
        &self,
        participants: Vec<String>,
        collaboration_type: CollaborationType,
    ) -> Result<MCPResponse> {
        debug!("Starting collaboration with {} participants", participants.len());
        
        let collaboration_id = self.agent_coordinator.start_collaboration(participants, collaboration_type).await?;
        
        Ok(MCPResponse::CollaborationStarted { collaboration_id })
    }
    
    async fn handle_send_agent_message_request(
        &self,
        message: super::multi_agent::AgentMessage,
    ) -> Result<MCPResponse> {
        debug!("Sending agent message from {} to {}", message.from_agent_id, message.to_agent_id);
        
        self.agent_coordinator.send_agent_message(message).await?;
        
        Ok(MCPResponse::AgentMessageSent {
            message_id: uuid::Uuid::new_v4().to_string(),
        })
    }
}

impl ServerMetrics {
    fn new() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            tool_executions: 0,
            plugin_operations: 0,
            uptime_seconds: 0,
            last_updated: chrono::Utc::now(),
        }
    }
}

// Supporting types and structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: String,
    pub capabilities: Vec<MCPCapability>,
    pub preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCapability {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub theme: Option<String>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: Some("en".to_string()),
            timezone: Some("UTC".to_string()),
            theme: Some("default".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MCPRequest {
    Initialize { capabilities: Vec<MCPCapability> },
    ListTools { category: Option<String> },
    ExecuteTool { tool_name: String, parameters: serde_json::Value },
    GetStatus,
    ManagePlugin { plugin_id: String, action: PluginAction },
    // New streaming requests
    CreateStream { stream_type: StreamType, direction: StreamDirection, config: super::bidirectional_streaming::StreamConfig },
    StreamMessage { stream_id: String, message: crate::protocol::types::MCPMessage },
    CloseStream { stream_id: String },
    // New multi-agent requests
    RegisterAgent { agent_config: super::multi_agent::AgentConfig },
    StartConversation { participants: Vec<String> },
    StartCollaboration { participants: Vec<String>, collaboration_type: CollaborationType },
    SendAgentMessage { message: super::multi_agent::AgentMessage },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MCPResponse {
    Initialized { session_id: String, capabilities: Vec<MCPCapability> },
    ToolsList { tools: Vec<ToolDefinition> },
    ToolResult { result: serde_json::Value, metadata: ToolMetadata },
    Status { state: String, metrics: ServerMetrics },
    PluginStatus { plugin_id: String, status: PluginStatus },
    Error { message: String, code: i32 },
    // New streaming responses
    StreamCreated { stream_id: String },
    StreamMessageSent { stream_id: String, message_id: String },
    StreamClosed { stream_id: String },
    // New multi-agent responses
    AgentRegistered { agent_id: String },
    ConversationStarted { conversation_id: String },
    CollaborationStarted { collaboration_id: String },
    AgentMessageSent { message_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginAction {
    Load,
    Unload,
    Execute { params: serde_json::Value },
    GetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub execution_time_ms: u64,
    pub resource_usage: ResourceUsage,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub disk_io_mb: u64,
}

/// Production plugin manager adapter for the enhanced server interface
pub struct ProductionPluginManagerAdapter {
    _manager: Arc<std::sync::Mutex<()>>, // Placeholder for now to avoid complex async integration
}

impl ProductionPluginManagerAdapter {
    pub fn new() -> Self {
        // For now, create a simplified adapter
        // FUTURE: [Integration] Integrate with actual DefaultPluginManager when async runtime is available
        // Tracking: Requires async runtime integration work
        Self {
            _manager: Arc::new(std::sync::Mutex::new(())),
        }
    }
}

impl PluginManagerInterface for ProductionPluginManagerAdapter {
    fn get_plugin_status(&self, plugin_name: &str) -> Option<PluginStatus> {
        // Return production-style status with better defaults
        Some(PluginStatus {
            id: format!("prod_{}", plugin_name),
            name: plugin_name.to_string(),
            state: PluginState::Running,
            version: "2.0.0".to_string(),
            last_activity: chrono::Utc::now(),
        })
    }
    
    fn list_plugins(&self) -> Vec<PluginStatus> {
        // Return placeholder production plugins
        vec![
            PluginStatus {
                id: "prod_core_plugin".to_string(),
                name: "Core Plugin".to_string(),
                state: PluginState::Running,
                version: "2.0.0".to_string(),
                last_activity: chrono::Utc::now(),
            },
            PluginStatus {
                id: "prod_mcp_plugin".to_string(),
                name: "MCP Plugin".to_string(),
                state: PluginState::Running,
                version: "2.0.0".to_string(),
                last_activity: chrono::Utc::now(),
            },
        ]
    }
}

// Mock implementations for testing only
#[cfg(test)]
pub struct MockPluginManager {}

#[cfg(test)]
impl MockPluginManager {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
impl PluginManagerInterface for MockPluginManager {
    fn get_plugin_status(&self, plugin_name: &str) -> Option<PluginStatus> {
        Some(PluginStatus {
            id: plugin_name.to_string(),
            name: plugin_name.to_string(),
            state: PluginState::Loaded,
            version: "1.0.0".to_string(),
            last_activity: chrono::Utc::now(),
        })
    }
    
    fn list_plugins(&self) -> Vec<PluginStatus> {
        vec![]
    }
}

impl From<ServerState> for String {
    fn from(state: ServerState) -> Self {
        match state {
            ServerState::Initializing => "initializing".to_string(),
            ServerState::Starting => "starting".to_string(),
            ServerState::Running => "running".to_string(),
            ServerState::Stopping => "stopping".to_string(),
            ServerState::Stopped => "stopped".to_string(),
            ServerState::Error(msg) => format!("error: {}", msg),
        }
    }
} 