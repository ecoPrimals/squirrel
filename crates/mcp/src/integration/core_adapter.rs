use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use serde_json::json;
use crate::error::{MCPError, Result};
use crate::protocol::{types::MCPMessage, types::MCPResponse, types::MessageType};
use crate::security::{Credentials, Permission, Action};
use crate::integration::prelude::MessageHandler;
use crate::monitoring::metrics;
use tracing::{info, error, warn, instrument, Instrument};
use async_trait::async_trait;
use uuid::Uuid;
use chrono::Utc;

/// Core-MCP adapter for integrating core system functionality with MCP
pub struct CoreMCPAdapter {
    /// Core state manager
    core_state: Arc<RwLock<CoreState>>,
    /// MCP protocol interface
    mcp: Arc<dyn crate::protocol::MCPProtocol>,
    /// Security manager for authentication and authorization
    auth_manager: Arc<AuthManager>,
    /// Metrics collector for operational monitoring
    metrics: Arc<metrics::MetricsCollector>,
    /// Logger for structured logging
    logger: tracing::Logger,
}

impl CoreMCPAdapter {
    /// Create a new Core-MCP adapter
    pub fn new(
        core_state: Arc<RwLock<CoreState>>,
        mcp: Arc<dyn crate::protocol::MCPProtocol>,
        auth_manager: Arc<AuthManager>,
        metrics: Arc<metrics::MetricsCollector>,
        logger: tracing::Logger,
    ) -> Self {
        Self {
            core_state,
            mcp,
            auth_manager,
            metrics,
            logger,
        }
    }

    /// Initialize the adapter by registering message handlers
    #[instrument(skip(self), name = "core_adapter_init")]
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing Core-MCP adapter");
        
        // Register handlers for core-related message types
        self.mcp.register_handler(
            MessageType::StateQuery,
            Box::new(self.clone()),
        ).await?;
        
        self.mcp.register_handler(
            MessageType::StateUpdate,
            Box::new(self.clone()),
        ).await?;
        
        self.mcp.register_handler(
            MessageType::CoreCommand,
            Box::new(self.clone()),
        ).await?;
        
        info!("Core-MCP adapter initialization complete");
        Ok(())
    }

    /// Send a state update notification
    #[instrument(skip(self, update), fields(update_type = %update.update_type))]
    pub async fn notify_state_update(&self, update: StateUpdate) -> Result<()> {
        let message = MCPMessage {
            id: Uuid::new_v4().to_string(),
            message_type: MessageType::StateNotification,
            payload: serde_json::to_value(update)?,
            metadata: Default::default(),
            timestamp: Utc::now(),
            version: "1.0".to_string(),
        };
        
        let timer = self.metrics.start_timer("state_notification_time");
        let result = self.mcp.send_message(message).await;
        let duration = timer.stop();
        
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

    /// Execute a core operation with circuit breaker pattern
    #[instrument(skip(self, operation_name, params), fields(operation = %operation_name))]
    pub async fn execute_core_operation(
        &self, 
        operation_name: &str,
        params: serde_json::Value,
        user: Option<&User>,
    ) -> Result<serde_json::Value> {
        // Check authorization if user is provided
        if let Some(user) = user {
            if let Err(e) = self.authorize_operation(user, operation_name).await {
                error!(error = %e, "Authorization failed for core operation");
                return Err(e);
            }
        }
        
        // Create circuit breaker for the operation
        let circuit_breaker = CircuitBreaker::new(
            5,  // Failure threshold
            10000, // Recovery timeout (ms)
        );
        
        // Execute with circuit breaker
        circuit_breaker.execute(|| async {
            self.perform_core_operation(operation_name, params).await
        }).await
    }
    
    /// Authorize a user for a specific operation
    async fn authorize_operation(&self, user: &User, operation: &str) -> Result<()> {
        let resource = format!("core:{}", operation);
        let permission = Permission::new(&resource, Action::Execute);
        
        self.auth_manager.authorize(user, &[permission])
            .await
            .map_err(|e| MCPError::Authorization(e.to_string()))
    }
    
    /// Perform the actual core operation
    async fn perform_core_operation(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Log operation attempt
        info!(operation = %operation, "Executing core operation");
        
        // Start timer for performance tracking
        let timer = self.metrics.start_timer("core_operation_time");
        
        // Handle different operation types
        let result = match operation {
            "get_state" => {
                // Use scoped lock to prevent holding across await points
                let state = {
                    let state = self.core_state.read().await;
                    state.clone()
                };
                
                serde_json::to_value(state)?
            },
            "update_state" => {
                // Extract update parameters
                let update: StateUpdate = serde_json::from_value(params)?;
                
                // Apply update with proper lock handling
                {
                    let mut state = self.core_state.write().await;
                    state.apply_update(&update)?;
                }
                
                // Notify of state change
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
        
        // Record metrics
        let duration = timer.stop();
        self.metrics.record_histogram(
            &format!("core_operation_{}_time", operation),
            duration,
        );
        self.metrics.increment_counter("core_operations_success");
        
        // Log success
        info!(
            operation = %operation,
            duration_ms = %duration,
            "Core operation completed successfully"
        );
        
        Ok(result)
    }
}

// Allow cloning of the adapter for use in message handlers
impl Clone for CoreMCPAdapter {
    fn clone(&self) -> Self {
        Self {
            core_state: self.core_state.clone(),
            mcp: self.mcp.clone(),
            auth_manager: self.auth_manager.clone(),
            metrics: self.metrics.clone(),
            logger: self.logger.clone(),
        }
    }
}

// Implement MessageHandler trait to handle MCP messages
#[async_trait]
impl MessageHandler for CoreMCPAdapter {
    #[instrument(skip(self, message), fields(message_id = %message.id, message_type = ?message.message_type))]
    async fn handle_message(&self, message: MCPMessage) -> Result<MCPResponse> {
        // Start performance timer
        let timer = self.metrics.start_timer("core_message_handling_time");
        
        // Log message receipt
        info!("Core adapter processing message");
        
        // Extract credentials if present
        let user = if let Some(credentials) = extract_credentials(&message) {
            match self.auth_manager.authenticate(&credentials).await {
                Ok(user) => Some(user),
                Err(e) => {
                    error!(error = %e, "Authentication failed");
                    return Ok(create_error_response(&message, "Authentication failed"));
                }
            }
        } else {
            None
        };
        
        // Process message based on type
        let result = match message.message_type {
            MessageType::StateQuery => {
                let operation = "get_state";
                let params = message.payload.clone();
                
                match self.execute_core_operation(operation, params, user.as_ref()).await {
                    Ok(result) => create_success_response(&message, result),
                    Err(e) => {
                        error!(error = %e, "State query failed");
                        create_error_response(&message, e.to_string())
                    }
                }
            },
            MessageType::StateUpdate => {
                let operation = "update_state";
                let params = message.payload.clone();
                
                match self.execute_core_operation(operation, params, user.as_ref()).await {
                    Ok(result) => create_success_response(&message, result),
                    Err(e) => {
                        error!(error = %e, "State update failed");
                        create_error_response(&message, e.to_string())
                    }
                }
            },
            MessageType::CoreCommand => {
                // Extract command from payload
                let command = match message.payload.get("command") {
                    Some(cmd) => match cmd.as_str() {
                        Some(cmd_str) => cmd_str,
                        None => {
                            return Ok(create_error_response(
                                &message,
                                "Invalid command format: expected string",
                            ));
                        }
                    },
                    None => {
                        return Ok(create_error_response(
                            &message,
                            "Missing command field in payload",
                        ));
                    }
                };
                
                let params = message.payload
                    .get("parameters")
                    .unwrap_or(&json!({}))
                    .clone();
                
                match self.execute_core_operation(command, params, user.as_ref()).await {
                    Ok(result) => create_success_response(&message, result),
                    Err(e) => {
                        error!(error = %e, "Core command execution failed");
                        create_error_response(&message, e.to_string())
                    }
                }
            },
            _ => {
                warn!(
                    message_type = ?message.message_type,
                    "Unsupported message type for core adapter"
                );
                
                create_error_response(
                    &message,
                    format!("Unsupported message type: {:?}", message.message_type),
                )
            }
        };
        
        // Record metrics
        let duration = timer.stop();
        self.metrics.record_histogram("core_message_handling_time", duration);
        
        if result.status == Status::Success {
            self.metrics.increment_counter("core_messages_success");
        } else {
            self.metrics.increment_counter("core_messages_error");
        }
        
        Ok(result)
    }
}

// Simple circuit breaker implementation
struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout_ms: u64,
    state: Mutex<CircuitState>,
}

enum CircuitState {
    Closed,
    Open(std::time::Instant),
    HalfOpen,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, recovery_timeout_ms: u64) -> Self {
        Self {
            failure_threshold,
            recovery_timeout_ms,
            state: Mutex::new(CircuitState::Closed),
        }
    }
    
    async fn execute<F, T, E>(&self, operation: F) -> Result<T, MCPError>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, MCPError>> + Send>>,
        T: Send + 'static,
    {
        // Check circuit state
        {
            let mut state = self.state.lock().unwrap();
            
            match *state {
                CircuitState::Open(instant) => {
                    let elapsed = instant.elapsed();
                    if elapsed.as_millis() as u64 >= self.recovery_timeout_ms {
                        // Recovery timeout has elapsed, move to half-open
                        *state = CircuitState::HalfOpen;
                    } else {
                        // Circuit is open, fast fail
                        return Err(MCPError::CircuitBreaker(
                            format!("Circuit is open, retry after {} ms", 
                                    self.recovery_timeout_ms - elapsed.as_millis() as u64)
                        ));
                    }
                },
                _ => {} // Continue with execution
            }
        }
        
        // Execute the operation
        match operation().await {
            Ok(result) => {
                // On success, close the circuit if it was half-open
                let mut state = self.state.lock().unwrap();
                if matches!(*state, CircuitState::HalfOpen) {
                    *state = CircuitState::Closed;
                }
                Ok(result)
            },
            Err(error) => {
                // Handle failure - potentially open the circuit
                let mut state = self.state.lock().unwrap();
                match *state {
                    CircuitState::Closed => {
                        // If this is a transient error that might trigger circuit breaking
                        if is_circuit_breaking_error(&error) {
                            // For simplicity, just open the circuit immediately
                            // A real implementation would track consecutive failures
                            *state = CircuitState::Open(std::time::Instant::now());
                        }
                    },
                    CircuitState::HalfOpen => {
                        // Failed during half-open state, reopen the circuit
                        *state = CircuitState::Open(std::time::Instant::now());
                    },
                    _ => {}
                }
                Err(error)
            }
        }
    }
}

// Helper function to determine if an error should trigger circuit breaking
fn is_circuit_breaking_error(error: &MCPError) -> bool {
    matches!(error, 
        MCPError::Connection(_) | 
        MCPError::Timeout(_) | 
        MCPError::Server(_)
    )
}

// Helper function to extract credentials from message metadata
fn extract_credentials(message: &MCPMessage) -> Option<Credentials> {
    if let Some(auth) = message.metadata.get("auth") {
        if let Ok(credentials) = serde_json::from_value(auth.clone()) {
            return Some(credentials);
        }
    }
    None
}

// Helper function to create a success response
fn create_success_response(message: &MCPMessage, payload: serde_json::Value) -> MCPResponse {
    MCPResponse {
        id: message.id.clone(),
        status: Status::Success,
        payload,
        error: None,
        timestamp: Utc::now(),
    }
}

// Helper function to create an error response
fn create_error_response(message: &MCPMessage, error: impl Into<String>) -> MCPResponse {
    MCPResponse {
        id: message.id.clone(),
        status: Status::Error,
        payload: serde_json::Value::Null,
        error: Some(error.into()),
        timestamp: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    
    // Mock MCP Protocol
    mock! {
        pub MCPProtocol {}
        
        #[async_trait]
        impl crate::protocol::MCPProtocol for MCPProtocol {
            async fn send_message(&self, message: MCPMessage) -> Result<MCPResponse>;
            async fn register_handler(&self, message_type: MessageType, handler: Box<dyn MessageHandler>) -> Result<()>;
            async fn subscribe(&self, message_type: MessageType) -> Result<crate::protocol::MCPSubscription>;
        }
    }
    
    #[tokio::test]
    async fn test_core_adapter_initialization() {
        // Create mocks
        let mut mock_mcp = MockMCPProtocol::new();
        let core_state = Arc::new(RwLock::new(CoreState::default()));
        let auth_manager = Arc::new(AuthManager::new_test());
        let metrics = Arc::new(crate::metrics::MetricsCollector::new_test());
        let logger = crate::logging::Logger::new_test();
        
        // Set up expectations
        mock_mcp.expect_register_handler()
            .times(3)
            .returning(|_, _| Ok(()));
        
        // Create adapter
        let adapter = CoreMCPAdapter::new(
            core_state,
            Arc::new(mock_mcp),
            auth_manager,
            metrics,
            logger,
        );
        
        // Test initialization
        let result = adapter.initialize().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_handle_state_query() {
        // Create test state
        let mut initial_state = CoreState::default();
        initial_state.version = "1.0.0".to_string();
        initial_state.status = "active".to_string();
        
        // Create components
        let core_state = Arc::new(RwLock::new(initial_state));
        let mock_mcp = Arc::new(MockMCPProtocol::new());
        let auth_manager = Arc::new(AuthManager::new_test());
        let metrics = Arc::new(crate::metrics::MetricsCollector::new_test());
        let logger = crate::logging::Logger::new_test();
        
        // Create adapter
        let adapter = CoreMCPAdapter::new(
            core_state,
            mock_mcp,
            auth_manager,
            metrics,
            logger,
        );
        
        // Create test message
        let message = MCPMessage {
            id: "test-id".to_string(),
            message_type: MessageType::StateQuery,
            payload: json!({}),
            metadata: Default::default(),
            timestamp: Utc::now(),
            version: "1.0".to_string(),
        };
        
        // Handle message
        let response = adapter.handle_message(message).await.unwrap();
        
        // Verify response
        assert_eq!(response.status, Status::Success);
        assert_eq!(response.payload.get("version").unwrap(), "1.0.0");
        assert_eq!(response.payload.get("status").unwrap(), "active");
    }
} 