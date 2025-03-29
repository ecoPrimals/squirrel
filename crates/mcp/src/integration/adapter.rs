//! Core MCP Adapter implementation.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{Value, json};
use async_trait::async_trait;
use tracing::{info, error, warn, instrument, debug};

use crate::error::{MCPError, ProtocolError, Result as MCPResult, SecurityError, ConnectionError, ContextError, SessionError};
use crate::protocol::types::{MCPMessage, MessageType, ProtocolVersion as ProtocolTypesProtocolVersion};
use crate::security::types::{SecurityMetadata, SecurityContext};
use crate::types::{MCPResponse, ResponseStatus, MessageMetadata, ProtocolState, ProtocolVersion as TypesProtocolVersion};
use crate::config::CoreAdapterConfig;
use crate::integration::types::{CoreState, StateUpdate, User, MessageHandler};
use crate::metrics::{Metrics, MetricsCollector};
use crate::protocol::{self, MCPProtocol, ValidationResult, RoutingResult, MessageId};
use crate::security::{AuthToken, Permission, SessionToken, Credentials};
use crate::plugin::manager::PluginManager;
use super::auth::AuthManager;
use super::helpers::{create_error_response, create_success_response, extract_credentials};

/// Core-MCP adapter for integrating core system functionality with MCP
#[derive(Debug)]
pub struct CoreMCPAdapter {
    /// Core state manager
    core_state: Arc<RwLock<CoreState>>,
    /// MCP protocol interface
    protocol_handler: Arc<dyn MCPProtocol + Send + Sync>,
    /// Security manager for authentication and authorization
    auth_manager: Arc<AuthManager>, // Use the type from auth.rs
    /// Metrics collector for operational monitoring
    metrics: Arc<MetricsCollector>,
    /// Configuration for the adapter
    config: CoreAdapterConfig,
    /// Plugin manager for managing plugins
    plugin_manager: Arc<PluginManager>,
}

impl CoreMCPAdapter {
    /// Create a new Core-MCP adapter
    pub fn new(
        core_state: Arc<RwLock<CoreState>>,
        protocol_handler: Arc<dyn MCPProtocol + Send + Sync>,
        auth_manager: Arc<AuthManager>,
        metrics: Arc<MetricsCollector>,
        config: CoreAdapterConfig,
        plugin_manager: Arc<PluginManager>,
    ) -> Self {
        Self {
            core_state,
            protocol_handler,
            auth_manager,
            metrics,
            config,
            plugin_manager,
        }
    }

    /// Initialize the adapter by registering message handlers
    #[instrument(skip(self), name = "core_adapter_init")]
    pub async fn initialize(&self) -> crate::error::Result<()> {
        info!("Initializing Core-MCP adapter");
        info!("Core-MCP adapter will handle Event and Command message types");
        info!("Core-MCP adapter initialization complete");
        Ok(())
    }

    /// Send a state update notification
    #[instrument(skip(self, update), fields(update_type = %update.update_type))]
    pub async fn notify_state_update(&self, update: StateUpdate) -> crate::error::Result<()> {
        let message = crate::protocol::MCPMessage {
            id: MessageId::new(),
            type_: crate::protocol::MessageType::Event,
            payload: serde_json::to_value(update)?,
            metadata: None,
            security: SecurityMetadata::default(),
            timestamp: chrono::Utc::now(),
            version: crate::protocol::types::ProtocolVersion::default(),
            trace_id: None,
        };
        
        let timer = self.metrics.start_timer("state_notification_time");
        let result = self.protocol_handler.handle_message(message).await;
        let duration = timer.elapsed();
        
        match &result {
            Ok(_) => {
                self.metrics.increment_counter("state_notifications_success");
                info!("State update notification sent successfully");
            }
            Err(e) => {
                self.metrics.increment_counter("state_notifications_error");
                error!(error = %e, "Failed to send state update notification");
            }
        }
        
        self.metrics.record_histogram("state_notification_duration", duration);
        result.map(|_| ())
    }

    /// Execute a core operation
    #[instrument(skip(self, operation_name, params), fields(operation = %operation_name))]
    pub async fn execute_core_operation(
        &self, 
        operation_name: &str,
        params: serde_json::Value,
        user: Option<&User>,
    ) -> crate::error::Result<serde_json::Value> {
        if let Some(user) = user {
            if let Err(e) = self.authorize_operation(user, operation_name).await {
                error!(error = %e, "Authorization failed for core operation");
                return Err(e);
            }
        }
        self.perform_core_operation(operation_name, params).await
    }
    
    /// Authorizes a specific operation for a given user.
    async fn authorize_operation(&self, user: &User, operation: &str) -> MCPResult<()> {
        info!("Authorizing operation '{}' for user {}", operation, user.id);
        let resource = format!("command:{}", operation);
        let required_permission = crate::security::Permission::new(&resource, crate::security::Action::Execute);
        debug!("Checking permission: {:?}", required_permission);
        // Use the AuthManager from self.auth_manager
        self.auth_manager.authorize(user, &[required_permission])
            .await
            .map_err(|e| MCPError::Authorization(e))
    }
    
    /// Perform the actual core operation
    async fn perform_core_operation(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        info!(operation = %operation, "Executing core operation");
        let timer = self.metrics.start_timer("core_operation_time");
        let result = match operation {
            "get_state" => {
                let state = self.core_state.read().await.clone();
                serde_json::to_value(state)?
            },
            "update_state" => {
                let update: StateUpdate = serde_json::from_value(params)?;
                {
                    let mut state = self.core_state.write().await;
                    state.apply_update(&update)?;
                }
                self.notify_state_update(update).await?;
                json!({ "status": "updated" })
            },
            "reset_state" => {
                {
                    let mut state = self.core_state.write().await;
                    *state = CoreState::default();
                }
                json!({ "status": "reset" })
            },
            _ => {
                warn!(operation = %operation, "Unknown core operation requested");
                return Err(MCPError::UnsupportedOperation(operation.to_string()));
            }
        };
        let duration = timer.elapsed();
        self.metrics.record_histogram(&format!("core_operation_{operation}_time"), duration);
        self.metrics.increment_counter("core_operations_success");
        info!(operation = %operation, duration_ms = %duration.as_millis(), "Core operation completed successfully");
        Ok(result)
    }

    /// Placeholder for extracting security context from a message
    async fn extract_security_context(&self, _message: &crate::protocol::MCPMessage) -> MCPResult<SecurityContext> {
        // TODO: Implement actual security context extraction
        Ok(SecurityContext::default())
    }
}

// Allow cloning of the adapter for use in message handlers
impl Clone for CoreMCPAdapter {
    fn clone(&self) -> Self {
        Self {
            core_state: self.core_state.clone(),
            protocol_handler: self.protocol_handler.clone(),
            auth_manager: self.auth_manager.clone(),
            metrics: self.metrics.clone(),
            config: self.config.clone(),
            plugin_manager: self.plugin_manager.clone(),
        }
    }
}

// Implement MessageHandler trait to handle MCP messages
#[async_trait::async_trait]
impl MessageHandler for CoreMCPAdapter {
    async fn handle_message(&self, message: crate::protocol::MCPMessage) -> MCPResult<MCPResponse> {
        info!("Core adapter processing message: ID={}", message.id.0);
        let security_context = self.extract_security_context(&message).await?; 

        let result = match message.type_ {
            crate::protocol::MessageType::Sync => {
                info!("Handling Sync message (state query)");
                match self.execute_core_operation("get_state", Value::Null, security_context.user_context.as_ref()).await {
                    Ok(result) => Ok(create_success_response(&message, &result)),
                    Err(e) => Ok(create_error_response(&message, e)),
                }
            }
            crate::protocol::MessageType::Command => {
                info!("Handling Command message");
                let command_payload: Value = serde_json::from_value(message.payload.clone())
                    .map_err(|e| MCPError::Protocol(ProtocolError::InvalidFormat(format!("Invalid command payload format: {}", e))))?;

                let command = command_payload.get("command")
                    .and_then(Value::as_str)
                    .ok_or_else(|| MCPError::Protocol(ProtocolError::InvalidFormat("Missing command field in payload".to_string())))?;
                
                let params = command_payload.get("params").cloned().unwrap_or(Value::Null);

                let user: Option<User> = if let Some(creds) = extract_credentials(&message) { 
                    match self.auth_manager.authenticate(&creds).await {
                        Ok(user) => Some(user),
                        Err(e) => {
                            error!("Authentication failed: {}", e);
                            return Ok(create_error_response(&message, MCPError::Security(SecurityError::AuthenticationFailed(e))));
                        }
                    }
                } else {
                    security_context.user_context
                };

                if let Some(ref user) = user { 
                    match self.authorize_operation(user, command).await {
                        Ok(_) => {
                            info!("Operation authorized for user {:?}", user.id);
                        }
                        Err(e) => {
                            warn!("Authorization failed for user {:?}: {}", user.id, e);
                            return Ok(create_error_response(&message, e)); 
                        }
                    }
                } else {
                    warn!("Authorization skipped: No authenticated user context for message {}", message.id.0);
                    return Ok(create_error_response(&message, MCPError::NotAuthorized("Authentication required".to_string())));
                }

                match self.execute_core_operation(command, params, user.as_ref()).await {
                    Ok(result) => Ok(create_success_response(&message, &result)),
                    Err(e) => {
                        error!(error = %e, "Core command execution failed");
                        Ok(create_error_response(&message, e))
                    }
                }
            },
            _ => {
                warn!("Received unhandled message type: {:?}", message.type_);
                Ok(create_error_response(
                    &message,
                    MCPError::UnsupportedOperation(format!("Unsupported message type: {:?}", message.type_))
                ))
            }
        };

        // self.metrics.record_message_handled(&message.type_);
        result
    }
}

// Implementation of MCPProtocol trait for CoreMCPAdapter
#[async_trait::async_trait]
impl crate::protocol::MCPProtocol for CoreMCPAdapter {
    async fn handle_message(&self, msg: crate::protocol::MCPMessage) -> crate::protocol::ProtocolResult {
        debug!(message_id = %msg.id.0, message_type = ?msg.type_, "Handling message");
        // self.metrics.record_message_received(&msg);

        if msg.version.major != 0 { // Assuming ProtocolVersion has major
            return Err(MCPError::UnsupportedVersion(msg.version.to_string())); // Use to_string()
        }

        match msg.type_ {
            crate::protocol::MessageType::Command => self.handle_command_message(msg).await,
            crate::protocol::MessageType::Sync => self.handle_sync_message(msg).await,
            crate::protocol::MessageType::Error => {
                error!(message_id = %msg.id.0, payload = ?String::from_utf8_lossy(&msg.payload), "Received Error message");
                Ok(None)
            }
            crate::protocol::MessageType::Response => {
                warn!(message_id = %msg.id.0, "Received unexpected Response message");
                Ok(None)
            }
            _ => {
                 warn!(message_id = %msg.id.0, message_type = ?msg.type_, "Received unhandled message type");
                 Err(MCPError::UnsupportedOperation(format!("Core adapter cannot handle {:?} directly", msg.type_)))
            }
        }
    }

    async fn validate_message(&self, _msg: &crate::protocol::MCPMessage) -> ValidationResult {
        Ok(())
    }

    async fn route_message(&self, _msg: &crate::protocol::MCPMessage) -> RoutingResult {
        Ok(None)
    }
    
    async fn set_state(&self, new_state: crate::types::ProtocolState) -> crate::error::Result<()> {
        warn!("CoreMCPAdapter::set_state called with {:?}, but state mapping is not fully implemented.", new_state);
        // TODO: Implement mapping from ProtocolState enum to CoreState fields if needed
        Ok(())
    }
    
    async fn get_state(&self) -> crate::error::Result<crate::types::ProtocolState> {
        // let state = self.core_state.read().await;
        // Ok(crate::types::ProtocolState {
        //     version: state.version.clone(),
        //     status: state.status.clone(),
        //     features: state.features.clone(),
        //     components: state.components.clone(),
        // }) // <-- This is struct-like construction

        // TODO: Map CoreState status to ProtocolState enum variant
        let core_state_status = self.core_state.read().await.status.clone();
        let protocol_state = match core_state_status.as_str() {
            "running" | "active" => crate::types::ProtocolState::Ready,
            "initializing" => crate::types::ProtocolState::Initializing,
            "disconnected" => crate::types::ProtocolState::Disconnected,
            _ => crate::types::ProtocolState::Error, // Default to Error for unknown statuses
        };
        Ok(protocol_state)
    }

    fn get_version(&self) -> String {
        // TODO: Implement version retrieval
        "1.0".to_string()
    }

    #[instrument(skip(self, msg))]
    async fn handle_command_message(&self, msg: crate::protocol::MCPMessage) -> crate::protocol::ProtocolResult {
        // Simplified: Call the main handle_message which uses the MessageHandler trait
        match self.handle_message(msg).await {
            Ok(response_option) => Ok(Some(response_option)), // Assume handle_message returns ProtocolResult<Option<MCPResponse>>
            Err(e) => Err(e), // Propagate error
        }
    }

    #[instrument(skip(self, msg))]
    async fn handle_sync_message(&self, msg: crate::protocol::MCPMessage) -> crate::protocol::ProtocolResult {
        // Simplified: Call the main handle_message which uses the MessageHandler trait
        match self.handle_message(msg).await {
            Ok(response_option) => Ok(Some(response_option)), // Assume handle_message returns ProtocolResult<Option<MCPResponse>>
            Err(e) => Err(e), // Propagate error
        }
    }

    // The create_*_response methods are now in helpers.rs
    // The extract_security_context method is part of the CoreMCPAdapter impl block
} 