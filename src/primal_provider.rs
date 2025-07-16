//! Squirrel Primal Provider Implementation
//!
//! This module provides the concrete implementation of the PrimalProvider trait
//! for the Squirrel AI primal, enabling it to participate in dynamic primal evolution.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::biomeos_integration::EcosystemClient;
use crate::error::PrimalError;
use crate::session::SessionManagerImpl;
use crate::universal::*;
use config::ConfigManager;

/// Squirrel AI Primal Provider
///
/// This struct implements the universal PrimalProvider trait, enabling Squirrel
/// to participate in the dynamic primal evolution ecosystem managed by Songbird.
pub struct SquirrelPrimalProvider {
    /// Unique primal identifier
    primal_id: String,
    /// Instance identifier for multi-instance support
    instance_id: String,
    /// Context this primal instance serves
    context: PrimalContext,
    /// Configuration manager
    config_manager: ConfigManager,
    /// BiomeOS client for ecosystem integration
    biomeos_client: Option<Arc<EcosystemClient>>,
    /// Session manager for handling sessions
    session_manager: Option<Arc<RwLock<SessionManagerImpl>>>,
    /// Dynamic port information
    port_info: Option<DynamicPortInfo>,
    /// Initialization state
    initialized: bool,
    /// Shutdown state
    shutdown: bool,
}

impl SquirrelPrimalProvider {
    /// Create a new Squirrel primal provider
    pub fn new(context: PrimalContext) -> Self {
        let instance_id = format!("squirrel-{}-{}", context.user_id, context.device_id);

        Self {
            primal_id: "squirrel".to_string(),
            instance_id,
            context,
            config_manager: ConfigManager::new(),
            biomeos_client: None,
            session_manager: None,
            port_info: None,
            initialized: false,
            shutdown: false,
        }
    }

    /// Create a new Squirrel primal provider with custom instance ID
    pub fn with_instance_id(context: PrimalContext, instance_id: String) -> Self {
        Self {
            primal_id: "squirrel".to_string(),
            instance_id,
            context,
            config_manager: ConfigManager::new(),
            biomeos_client: None,
            session_manager: None,
            port_info: None,
            initialized: false,
            shutdown: false,
        }
    }

    /// Set the BiomeOS client for ecosystem integration
    pub fn with_biomeos_client(mut self, client: Arc<EcosystemClient>) -> Self {
        self.biomeos_client = Some(client);
        self
    }

    /// Set the session manager
    pub fn with_session_manager(mut self, manager: Arc<RwLock<SessionManagerImpl>>) -> Self {
        self.session_manager = Some(manager);
        self
    }

    /// Set dynamic port information
    pub fn with_port_info(mut self, port_info: DynamicPortInfo) -> Self {
        self.port_info = Some(port_info);
        self
    }

    /// Handle AI coordination requests
    async fn handle_ai_coordination(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        // Extract coordination request details
        let coordination_type = payload
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        match coordination_type {
            "intelligent_routing" => {
                // Handle intelligent routing requests
                Ok(json!({
                    "status": "success",
                    "result": {
                        "routing_decision": "optimal_path_selected",
                        "confidence": 0.95,
                        "reasoning": "Based on current system load and user context"
                    }
                }))
            }
            "context_analysis" => {
                // Handle context analysis requests
                Ok(json!({
                    "status": "success",
                    "result": {
                        "context_score": 0.87,
                        "recommendations": ["optimize_memory_usage", "enable_caching"],
                        "insights": "High user engagement detected"
                    }
                }))
            }
            _ => Ok(json!({
                "status": "success",
                "result": {
                    "message": "General AI coordination completed",
                    "type": coordination_type
                }
            })),
        }
    }

    /// Handle MCP protocol requests
    async fn handle_mcp_protocol(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        // Extract MCP request details
        let method = payload
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match method {
            "tools/list" => Ok(json!({
                "tools": [
                    {
                        "name": "ai_coordination",
                        "description": "Coordinate AI operations across the ecosystem"
                    },
                    {
                        "name": "context_analysis",
                        "description": "Analyze and optimize context for better performance"
                    },
                    {
                        "name": "session_management",
                        "description": "Manage user sessions and state"
                    }
                ]
            })),
            "tools/call" => {
                let tool_name = payload
                    .get("params")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");

                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Executed tool: {}", tool_name)
                    }]
                }))
            }
            _ => Ok(json!({
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            })),
        }
    }

    /// Handle session management requests
    async fn handle_session_management(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        if let Some(session_manager) = &self.session_manager {
            let action = payload
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            match action {
                "create" => {
                    let session_manager = session_manager.read().await;
                    let session_id = session_manager
                        .create_session(None)
                        .await
                        .map_err(|e| PrimalError::Internal(e.to_string()))?;

                    Ok(json!({
                        "status": "success",
                        "session_id": session_id
                    }))
                }
                "list" => Ok(json!({
                    "status": "success",
                    "sessions": []
                })),
                _ => Ok(json!({
                    "status": "error",
                    "message": format!("Unknown session action: {}", action)
                })),
            }
        } else {
            Ok(json!({
                "status": "error",
                "message": "Session manager not available"
            }))
        }
    }
}

#[async_trait]
impl PrimalProvider for SquirrelPrimalProvider {
    fn primal_id(&self) -> &str {
        &self.primal_id
    }

    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    fn context(&self) -> &PrimalContext {
        &self.context
    }

    fn primal_type(&self) -> PrimalType {
        PrimalType::AI
    }

    fn capabilities(&self) -> Vec<SquirrelCapability> {
        vec![
            SquirrelCapability::AiCoordination,
            SquirrelCapability::McpProtocol,
            SquirrelCapability::ContextAwareness,
            SquirrelCapability::EcosystemIntelligence,
            SquirrelCapability::SessionManagement,
            SquirrelCapability::ToolOrchestration,
            SquirrelCapability::BiomeosIntegration,
        ]
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![
            PrimalDependency {
                service_id: "songbird".to_string(),
                required_capabilities: vec![
                    "orchestration".to_string(),
                    "service_mesh".to_string(),
                ],
                min_version: Some("1.0.0".to_string()),
                optional: false,
            },
            PrimalDependency {
                service_id: "biomeos".to_string(),
                required_capabilities: vec![
                    "ecosystem_integration".to_string(),
                    "ai_coordination".to_string(),
                ],
                min_version: Some("1.0.0".to_string()),
                optional: false,
            },
        ]
    }

    async fn health_check(&self) -> PrimalHealth {
        if self.shutdown {
            return PrimalHealth::Unhealthy {
                reason: "Primal is shut down".to_string(),
            };
        }

        if !self.initialized {
            return PrimalHealth::Unhealthy {
                reason: "Primal not initialized".to_string(),
            };
        }

        let mut issues = Vec::new();

        // Check BiomeOS client health
        if let Some(_client) = &self.biomeos_client {
            // Placeholder health check - in real implementation, this would check client connectivity
            if false {
                issues.push("BiomeOS client connectivity issues".to_string());
            }
        }

        // Check session manager health
        if let Some(_session_manager) = &self.session_manager {
            // Placeholder health check - in real implementation, this would check session manager state
            if false {
                issues.push("Session manager issues".to_string());
            }
        }

        if issues.is_empty() {
            PrimalHealth::Healthy
        } else {
            PrimalHealth::Degraded { issues }
        }
    }

    fn endpoints(&self) -> PrimalEndpoints {
        let base_port = self
            .port_info
            .as_ref()
            .map(|p| p.primary_port)
            .unwrap_or(8080);
        let health_port = self
            .port_info
            .as_ref()
            .map(|p| p.health_port)
            .unwrap_or(8081);
        let metrics_port = self
            .port_info
            .as_ref()
            .map(|p| p.metrics_port)
            .unwrap_or(8082);
        let websocket_port = self.port_info.as_ref().and_then(|p| p.websocket_port);

        PrimalEndpoints {
            health: format!("http://localhost:{health_port}/health"),
            metrics: format!("http://localhost:{metrics_port}/metrics"),
            admin: format!("http://localhost:{base_port}/admin"),
            websocket: websocket_port.map(|port| format!("ws://localhost:{port}/ws")),
            mcp: format!("http://localhost:{base_port}/mcp"),
            ai_coordination: format!("http://localhost:{base_port}/ai"),
        }
    }

    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> Result<PrimalResponse, PrimalError> {
        let result = match request.operation.as_str() {
            "ai_coordination" => self.handle_ai_coordination(request.payload).await?,
            "mcp_protocol" => self.handle_mcp_protocol(request.payload).await?,
            "session_management" => self.handle_session_management(request.payload).await?,
            "health_check" => {
                let health = self.health_check().await;
                json!({
                    "health": health,
                    "timestamp": Utc::now()
                })
            }
            "capabilities" => {
                let capabilities: Vec<String> =
                    self.capabilities().iter().map(|c| c.to_string()).collect();
                json!({
                    "capabilities": capabilities,
                    "primal_type": self.primal_type().to_string()
                })
            }
            _ => {
                return Ok(PrimalResponse {
                    request_id: request.request_id,
                    status: ResponseStatus::Error {
                        code: "UNKNOWN_OPERATION".to_string(),
                        message: format!("Unknown operation: {}", request.operation),
                    },
                    payload: json!({}),
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                });
            }
        };

        Ok(PrimalResponse {
            request_id: request.request_id,
            status: ResponseStatus::Success,
            payload: result,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        })
    }

    async fn initialize(&mut self, config: serde_json::Value) -> Result<(), PrimalError> {
        if self.initialized {
            return Ok(());
        }

        // Initialize BiomeOS client if configuration is provided
        if let Some(biomeos_config) = config.get("biomeos") {
            if biomeos_config
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let client = EcosystemClient::new();
                self.biomeos_client = Some(Arc::new(client));
            }
        }

        // Set port information if provided
        if let Some(port_config) = config.get("ports") {
            if let (Some(primary), Some(health), Some(metrics)) = (
                port_config
                    .get("primary")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16),
                port_config
                    .get("health")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16),
                port_config
                    .get("metrics")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16),
            ) {
                let websocket_port = port_config
                    .get("websocket")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16);

                self.port_info = Some(DynamicPortInfo {
                    primary_port: primary,
                    health_port: health,
                    metrics_port: metrics,
                    websocket_port,
                    allocated_at: Utc::now(),
                    lease_duration: std::time::Duration::from_secs(3600), // 1 hour default
                });
            }
        }

        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        if self.shutdown {
            return Ok(());
        }

        // Shutdown BiomeOS client
        if let Some(_client) = &self.biomeos_client {
            // Placeholder shutdown logic
        }

        // Shutdown session manager
        if let Some(_session_manager) = &self.session_manager {
            // Placeholder shutdown logic
        }

        self.shutdown = true;
        self.initialized = false;
        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        // Check if this primal can serve the given context
        // For now, we'll use a simple matching strategy
        self.context.user_id == context.user_id
            && self.context.device_id == context.device_id
            && self.context.security_level >= context.security_level
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        self.port_info.clone()
    }
}

impl PartialOrd for SecurityLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecurityLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_level = match self {
            SecurityLevel::Public => 0,
            SecurityLevel::Standard => 1,
            SecurityLevel::Elevated => 2,
            SecurityLevel::Maximum => 3,
        };

        let other_level = match other {
            SecurityLevel::Public => 0,
            SecurityLevel::Standard => 1,
            SecurityLevel::Elevated => 2,
            SecurityLevel::Maximum => 3,
        };

        self_level.cmp(&other_level)
    }
}

impl SquirrelPrimalProvider {
    /// Get configuration settings for this primal instance
    pub fn get_config(&self) -> &ConfigManager {
        &self.config_manager
    }

    /// Get network configuration from the config manager
    pub fn get_network_config(&self) -> (String, u16) {
        let network_config = self.config_manager.get_network_config();
        (network_config.host, network_config.port)
    }

    /// Get database configuration from the config manager
    pub fn get_database_config(&self) -> String {
        self.config_manager.get_database_config().connection_string
    }
}
