// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Core MCP Adapter implementation.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{json};
use tracing::{info, error, warn, instrument, debug};
use std::time::Duration;

// Import from error module
use crate::error::{Result as MCPResult, MCPError};

// Import from security module
// BearDog handles security: // use crate::security::Token;

// Import from security::types
// BearDog handles security: // use crate::security::types::{SecurityMetadata, Resource, Action};

// Import from security::manager
// BearDog handles security: // use crate::security::manager::SecurityManager;

// Import from types module
use crate::types::{MCPResponse, ResponseStatus, MessageMetadata, ProtocolState};

// Import from integration types
use crate::integration::types::{CoreState, StateUpdate};

// Import context manager for Context

// Import from protocol module
use crate::protocol::{MCPProtocol, ValidationResult, RoutingResult, ProtocolResult};
use crate::protocol::types::{MCPMessage, MessageType, MessageId, ProtocolVersion, SecurityMetadata, Token, Resource, Action};

// Import helpers

/// Simple metrics collection for operational monitoring
#[derive(Clone, Debug)]
pub struct Metrics {
    // Fields would be added here for a real implementation
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {}
    }
    
    /// Start a timer and return an object that tracks elapsed time
    pub fn start_timer(&self, name: &str) -> MetricsTimer {
        debug!("Starting timer: {}", name);
        MetricsTimer::new()
    }
    
    /// Record a value in a histogram
    pub fn record_histogram(&self, name: &str, duration: Duration) {
        debug!("Recording histogram {}: {}ms", name, duration.as_millis());
    }
    
    /// Increment a counter
    pub fn increment_counter(&self, name: &str) {
        debug!("Incrementing counter: {}", name);
    }
}

/// Timer for tracking elapsed time
pub struct MetricsTimer {
    start_time: std::time::Instant,
}

impl MetricsTimer {
    /// Create a new timer
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Get the elapsed time since the timer was started
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Core MCP adapter implementation
/// 
/// This struct provides a bridge between the core system state and the MCP protocol,
/// handling communication, security, and state management.
#[derive(Clone)]
pub struct CoreMCPAdapter {
    /// Core state manager
    core_state: Arc<RwLock<CoreState>>,
    /// MCP protocol interface
    protocol_handler: Arc<dyn MCPProtocol>,
    /// Metrics collector for operational monitoring
    metrics: Arc<Metrics>,
}

impl std::fmt::Debug for CoreMCPAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreMCPAdapter")
            .field("core_state", &"Arc<RwLock<CoreState>>")
            .field("protocol_handler", &"Arc<dyn MCPProtocol>")
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl CoreMCPAdapter {
    /// Create a new Core-MCP adapter
    pub fn new(
        core_state: Arc<RwLock<CoreState>>,
        protocol_handler: Arc<dyn MCPProtocol>,
        metrics: Arc<Metrics>,
    ) -> Self {
        Self {
            core_state,
            protocol_handler,
            metrics,
        }
    }

    /// Initialize the adapter by registering message handlers
    #[instrument(skip(self), name = "core_adapter_init")]
    pub async fn initialize(&self) -> crate::error::Result<()> {
        info!("Initializing Core-MCP adapter");
        // FUTURE: [Integration] Register handlers with a message router
        // Tracking: Planned for v0.2.0 - message routing integration
        info!("Core-MCP adapter initialization complete");
        Ok(())
    }

    /// Send a state update notification
    #[instrument(skip(self, update), fields(update_type = %update.update_type))]
    pub async fn notify_state_update(&self, update: StateUpdate) -> crate::error::Result<()> {
        let message = MCPMessage {
            id: MessageId::new(),
            type_: MessageType::Event,
            payload: serde_json::to_value(update)?,
            metadata: None,
            security: SecurityMetadata::default(),
            timestamp: chrono::Utc::now(),
            version: ProtocolVersion::default(),
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
    #[instrument(skip(self, params), fields(operation = %operation_name))]
    pub async fn execute_core_operation(
        &self, 
        operation_name: &str,
        params: serde_json::Value,
        token: Option<&Token>,
    ) -> crate::error::Result<serde_json::Value> {
        // Check authorization if token is provided
        if let Some(token) = token {
            let resource = Resource {
                id: "integration_operation".to_string(),
                name: operation_name.to_string(),
                resource_type: "operation".to_string(),
                attributes: std::collections::HashMap::new(),
            };

            let action = Action {
                name: "authorize".to_string(),
                action_type: "authorization".to_string(),
                parameters: std::collections::HashMap::new(),
            };

            // Authorization moved to BearDog framework
            // For now, allow all operations (development mode)
            Ok(())
        }
        
        self.perform_core_operation(operation_name, params).await
    }
    
    /// Perform the actual core operation
    async fn perform_core_operation(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> MCPResult<serde_json::Value> {
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
                return Err(MCPError::UnsupportedOperation(operation.to_string()).into());
            }
        };
        
        let duration = timer.elapsed();
        self.metrics.record_histogram(&format!("core_operation_{}_time", operation), duration);
        self.metrics.increment_counter("core_operations_success");
        
        info!(
            operation = %operation, 
            duration_ms = %duration.as_millis(),
            "Core operation completed successfully"
        );
        
        Ok(result)
    }

    /// Handle a command message
    #[instrument(skip(self, msg), fields(message_id = %msg.id.0))]
    async fn handle_command_message(&self, msg: &MCPMessage) -> MCPResult<MCPResponse> {
        debug!("Handling command message: {}", msg.id.0);
        
        // Extract operation name and params from the message payload
        let operation = msg.payload.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::InvalidOperation("Missing operation field".to_string()))?;
        
        let params = msg.payload.get("params")
            .unwrap_or(&json!({}))
            .clone();
        
        // Get user token from security metadata if available
        let user_token = if let Some(ref auth_token) = msg.security.auth_token {
            // Authorization moved to BearDog framework
            // For now, create a placeholder token (development mode)
            Some(Token {
                value: auth_token.clone(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                permissions: vec!["*".to_string()], // Allow all in dev mode
            })
        } else {
            None
        };
        
        // Execute the operation
        match self.execute_core_operation(operation, params, user_token.as_ref()).await {
            Ok(result) => Ok(self.create_success_response(result, Some(msg.id.clone()))),
            Err(e) => {
                error!(error = %e, "Failed to execute operation: {}", operation);
                Ok(self.create_error_response(e, Some(msg.id.clone())))
            }
        }
    }

    /// Handle a sync message
    #[instrument(skip(self, msg), fields(message_id = %msg.id.0))]
    async fn handle_sync_message(&self, msg: &MCPMessage) -> MCPResult<MCPResponse> {
        debug!("Handling sync message: {}", msg.id.0);
        
        // For sync messages, simply return the current state
        match serde_json::to_value(self.core_state.read().await.clone()) {
            Ok(state) => Ok(self.create_success_response(state, Some(msg.id.clone()))),
            Err(e) => {
                error!(error = %e, "Failed to serialize core state");
                Ok(self.create_error_response(MCPError::InternalError(e.to_string()).into(), Some(msg.id.clone())))
            }
        }
    }

    /// Create an error response for a message
    pub fn create_error_response(&self, error: MCPError, message_id: Option<MessageId>) -> MCPResponse {
        let error_details = json!({
            "error": error.to_string(),
        });
        
        let response = MCPResponse {
            protocol_version: "1.0".to_string(),
            message_id: message_id.unwrap_or_else(MessageId::new),
            status: ResponseStatus::Error,
            payload: vec![error_details],
            error_message: Some(error.to_string()),
            metadata: MessageMetadata::default(),
        };
        
        response
    }
    
    /// Create a success response for a message
    pub fn create_success_response(&self, data: serde_json::Value, message_id: Option<MessageId>) -> MCPResponse {
        let response = MCPResponse {
            protocol_version: "1.0".to_string(),
            message_id: message_id.unwrap_or_else(MessageId::new),
            status: ResponseStatus::Success,
            payload: vec![data],
            error_message: None,
            metadata: MessageMetadata::default(),
        };
        
        response
    }

    /// Authorize an operation
    async fn authorize_operation(
        &self,
        operation_name: &str,
        token: Option<&Token>,
    ) -> MCPResult<()> {
        // BearDog handles authorization - this is a placeholder
        // In production, this would call the BearDog security framework
        
        // For now, allow all operations (development mode)
        // self.auth_manager.authorize(operation_name, token).await // Moved to BearDog
        Ok(())
    }
}

// Implementation of the message handling trait
impl crate::integration::types::MessageHandler for CoreMCPAdapter {
    fn handle_message(&self, message: MCPMessage) -> impl std::future::Future<Output = MCPResult<MCPResponse>> + Send {
        let adapter = self.clone();
        async move {
            info!("CoreMCPAdapter: Handling message: {}", message.id.0);
            
            match message.type_ {
                MessageType::Command => adapter.handle_command_message(&message).await,
                MessageType::Sync => adapter.handle_sync_message(&message).await,
                MessageType::Event => {
                    // Just acknowledge events
                    Ok(adapter.create_success_response(json!({ "status": "acknowledged" }), Some(message.id.clone())))
                },
                MessageType::Error => {
                    // Log errors but don't do much else
                    let error_details = message.payload.get("error")
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "Unknown error".to_string());
                    
                    warn!("Received error message: {}", error_details);
                    Ok(MCPResponse {
                        protocol_version: message.version.version_string(),
                        message_id: message.id.clone(),
                        status: ResponseStatus::Error,
                        payload: vec![json!({"error": error_details})],
                        error_message: Some(error_details),
                        metadata: MessageMetadata::default(),
                    })
                },
                _ => {
                    warn!("Unsupported message type: {:?}", message.type_);
                    Ok(adapter.create_error_response(
                        MCPError::UnsupportedOperation(format!("Unsupported message type: {:?}", message.type_)).into(),
                        Some(message.id.clone())
                    ))
                }
            }
        }
    }
}

// Implement MCPProtocol for CoreMCPAdapter by delegating to the protocol handler
// (native async - Phase 4 migration Session 30)
impl MCPProtocol for CoreMCPAdapter {
    fn handle_message(&self, msg: MCPMessage) -> impl std::future::Future<Output = ProtocolResult> + Send {
        async move {
            // Delegate to the message handler implementation
            let result = crate::integration::types::MessageHandler::handle_message(self, msg).await;
            // Convert MCPResult<MCPResponse> to ProtocolResult
            match result {
                Ok(response) => Ok(response),
                Err(err) => Err(err.into()),
            }
        }
    }

    fn validate_message(&self, msg: &MCPMessage) -> impl std::future::Future<Output = ValidationResult> + Send {
        async move {
            self.protocol_handler.validate_message(msg).await
        }
    }

    fn get_version(&self) -> impl std::future::Future<Output = ProtocolVersion> + Send {
        async move {
            ProtocolVersion::default()
        }
    }
} 