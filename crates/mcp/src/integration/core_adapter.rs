use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::json;
use crate::error::{MCPError, Result as MCPResult};
use crate::types::{MCPMessage, MCPResponse, MessageType, MessageId, ProtocolVersion, SecurityMetadata, ResponseStatus, MessageMetadata};
use crate::protocol::MCPProtocol;
use tracing::{info, error, warn, instrument};
use serde::{Serialize, Deserialize};
use crate::logging::Logger;
use crate::metrics::MetricsCollector;

/// State update information for the core system
///
/// Represents an update to the core system state, containing the type of update
/// and associated data that needs to be applied to the state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    /// The type of update being applied (e.g., "`component_added`", "`feature_enabled`")
    pub update_type: String,
    
    /// The data payload associated with the update
    pub data: serde_json::Value,
}

/// Core system state representation
///
/// Contains the state information for the core system, including version,
/// operational status, available features, and component information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreState {
    /// Core system version identifier
    pub version: String,
    
    /// Current operational status (e.g., "running", "maintenance")
    pub status: String,
    
    /// List of enabled features
    pub features: Vec<String>,
    
    /// Detailed component information
    pub components: serde_json::Value,
}

impl CoreState {
    /// Applies a state update to this `CoreState` instance
    ///
    /// Updates the core state based on the provided state update information.
    /// Currently this is a placeholder implementation.
    ///
    /// # Arguments
    ///
    /// * `_update` - The state update to apply
    ///
    /// # Returns
    ///
    /// Result indicating success or an `MCPError`
    /// 
    /// # Errors
    /// 
    /// This implementation currently does not return errors, but in a full implementation
    /// it could return an `MCPError` if the update is invalid or cannot be applied
    pub fn apply_update(&mut self, _update: &StateUpdate) -> Result<(), MCPError> {
        // TODO: Implement actual state update logic
        Ok(())
    }
}

impl Default for CoreState {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            status: "initializing".to_string(),
            features: vec!["basic".to_string()],
            components: serde_json::json!({}),
        }
    }
}

/// Message handling interface for MCP communications
///
/// Defines the interface for components that need to handle MCP messages,
/// providing a uniform way to process incoming messages and generate responses.
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handles an incoming MCP message and produces a response
    ///
    /// # Arguments
    ///
    /// * `message` - The incoming message to handle
    ///
    /// # Returns
    ///
    /// A result containing the response message or an error
    async fn handle_message(&self, message: MCPMessage) -> MCPResult<MCPResponse>;
}

/// User representation for authentication and authorization
///
/// Contains user identity information including ID, name, and assigned roles
/// for use in security operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for the user
    pub id: String,
    
    /// Display name of the user
    pub name: String,
    
    /// List of roles assigned to the user
    pub roles: Vec<String>,
}

/// Authentication and authorization manager
///
/// Handles user authentication and authorization checks against
/// required permissions.
pub struct AuthManager {
    // Implementation details omitted
}

impl AuthManager {
    /// Authorizes a user against a set of required permissions
    ///
    /// Checks if the user has the necessary permissions to perform
    /// a specific operation.
    ///
    /// # Arguments
    ///
    /// * `_user` - The user to authorize
    /// * `_permissions` - The permissions required for the operation
    ///
    /// # Returns
    ///
    /// Result indicating success if authorized or an error string if not
    /// 
    /// # Errors
    /// 
    /// Returns an error string if the user lacks the required permissions or
    /// if the authorization check cannot be completed
    pub async fn authorize(&self, _user: &User, _permissions: &[Permission]) -> Result<(), String> {
        // TODO: Implement authorization logic
        Ok(())
    }

    /// Creates a new test authentication manager
    ///
    /// Creates an instance configured for testing purposes that
    /// allows all authorization requests.
    ///
    /// # Returns
    ///
    /// A new `AuthManager` instance configured for testing
    #[must_use] pub const fn new_test() -> Self {
        Self {}
    }

    /// Authenticates a user with the provided credentials
    ///
    /// Verifies user credentials and returns the authenticated user
    /// if successful.
    ///
    /// # Arguments
    ///
    /// * `_credentials` - The credentials to verify
    ///
    /// # Returns
    ///
    /// Result containing the authenticated User or an error string
    /// 
    /// # Errors
    /// 
    /// Returns an error string if authentication fails due to invalid credentials,
    /// account lockout, or other authentication infrastructure issues
    pub async fn authenticate(&self, _credentials: &Credentials) -> Result<User, String> {
        // TODO: Implement authentication logic
        Ok(User {
            id: "test".to_string(),
            name: "Test User".to_string(),
            roles: vec!["user".to_string()],
        })
    }
}

/// Permission descriptor for authorization checks
///
/// Represents a single permission that can be checked during authorization,
/// consisting of a resource identifier and an action.
pub struct Permission {
    resource: String,
    action: Action,
}

impl Permission {
    /// Creates a new permission
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource identifier the permission applies to
    /// * `action` - The action being permitted on the resource
    ///
    /// # Returns
    ///
    /// A new Permission instance
    #[must_use] pub fn new(resource: &str, action: Action) -> Self {
        Self {
            resource: resource.to_string(),
            action,
        }
    }
}

/// Action types for permissions
///
/// Represents the different types of actions that can be performed
/// on resources during permission checks.
pub enum Action {
    /// Permission to execute or run an operation
    Execute,
    
    /// Permission to read or view a resource
    Read,
    
    /// Permission to modify or update a resource
    Write,
}

/// User credentials for authentication
///
/// Contains authentication information provided by a user
/// during login or other authentication operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Username for authentication
    pub username: String,
    
    /// Optional password for authentication
    pub password: Option<String>,
}

/// Core-MCP adapter for integrating core system functionality with MCP
pub struct CoreMCPAdapter {
    /// Core state manager
    core_state: Arc<RwLock<CoreState>>,
    /// MCP protocol interface
    mcp: Arc<dyn MCPProtocol>,
    /// Security manager for authentication and authorization
    auth_manager: Arc<AuthManager>,
    /// Metrics collector for operational monitoring
    metrics: Arc<MetricsCollector>,
    /// Logger for structured logging
    logger: Logger,
}

impl CoreMCPAdapter {
    /// Create a new Core-MCP adapter
    pub fn new(
        core_state: Arc<RwLock<CoreState>>,
        mcp: Arc<dyn MCPProtocol>,
        auth_manager: Arc<AuthManager>,
        metrics: Arc<MetricsCollector>,
        logger: Logger,
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
    pub async fn initialize(&self) -> MCPResult<()> {
        info!("Initializing Core-MCP adapter");
        
        // Note: In the real implementation, we would register handlers
        // but since the route_message method of MCPProtocol doesn't accept handlers,
        // we'll just log that initialization is complete
        info!("Core-MCP adapter will handle Event and Command message types");
        info!("Core-MCP adapter initialization complete");
        Ok(())
    }

    /// Send a state update notification
    #[instrument(skip(self, update), fields(update_type = %update.update_type))]
    pub async fn notify_state_update(&self, update: StateUpdate) -> MCPResult<()> {
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
        // Use send_message instead of handle_message to match the MCPProtocol trait
        let result = self.mcp.handle_message(message).await;
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
    ) -> MCPResult<serde_json::Value> {
        // Check authorization if user is provided
        if let Some(user) = user {
            if let Err(e) = self.authorize_operation(user, operation_name).await {
                error!(error = %e, "Authorization failed for core operation");
                return Err(e);
            }
        }
        
        // Perform the operation directly
        self.perform_core_operation(operation_name, params).await
    }
    
    /// Authorize a user for a specific operation
    async fn authorize_operation(&self, user: &User, operation: &str) -> MCPResult<()> {
        let resource = format!("core:{operation}");
        let permission = Permission::new(&resource, Action::Execute);
        
        self.auth_manager.authorize(user, &[permission])
            .await
            .map_err(|e| MCPError::Authorization(e))
    }
    
    /// Perform the actual core operation
    async fn perform_core_operation(
        &self,
        operation: &str,
        params: serde_json::Value,
    ) -> MCPResult<serde_json::Value> {
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
        let duration = timer.elapsed();
        self.metrics.record_histogram(
            &format!("core_operation_{operation}_time"),
            duration,
        );
        self.metrics.increment_counter("core_operations_success");
        
        // Log success
        info!(
            operation = %operation,
            duration_ms = %duration.as_millis(),
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
#[async_trait::async_trait]
impl MessageHandler for CoreMCPAdapter {
    #[instrument(skip(self, message), fields(message_id = %message.id.0, message_type = ?message.type_))]
    async fn handle_message(&self, message: MCPMessage) -> MCPResult<MCPResponse> {
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
        let result = match message.type_ {
            MessageType::Sync => {
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
            MessageType::Event => {
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
            MessageType::Command => {
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
                    message_type = ?message.type_,
                    "Unsupported message type for core adapter"
                );
                
                create_error_response(
                    &message,
                    format!("Unsupported message type: {:?}", message.type_),
                )
            }
        };
        
        // Record metrics
        let duration = timer.elapsed();
        self.metrics.record_histogram("core_message_handling_time", duration);
        
        if result.status == ResponseStatus::Success {
            self.metrics.increment_counter("core_messages_success");
        } else {
            self.metrics.increment_counter("core_messages_error");
        }
        
        Ok(result)
    }
}

// Helper function to determine if an error should trigger circuit breaking
const fn is_circuit_breaking_error(error: &MCPError) -> bool {
    matches!(error, 
        MCPError::Connection(_) | 
        MCPError::Network(_) |
        MCPError::Transport(_)
    )
}

// Helper function to extract credentials from message metadata
fn extract_credentials(message: &MCPMessage) -> Option<Credentials> {
    if let Some(metadata) = &message.metadata {
        if let Some(auth) = metadata.get("auth") {
            if let Ok(credentials) = serde_json::from_value(auth.clone()) {
                return Some(credentials);
            }
        }
    }
    None
}

// Helper function to create a success response
fn create_success_response(message: &MCPMessage, payload: serde_json::Value) -> MCPResponse {
    MCPResponse {
        protocol_version: message.version.version_string(),
        message_id: message.id.0.clone(),
        status: ResponseStatus::Success,
        payload: serde_json::to_vec(&payload).unwrap_or_default(),
        error_message: None,
        metadata: MessageMetadata::default(),
    }
}

// Helper function to create an error response
fn create_error_response(message: &MCPMessage, error: impl Into<String>) -> MCPResponse {
    MCPResponse {
        protocol_version: message.version.version_string(),
        message_id: message.id.0.clone(),
        status: ResponseStatus::Error,
        payload: Vec::new(),
        error_message: Some(error.into()),
        metadata: MessageMetadata::default(),
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
            async fn handle_message(&self, msg: MCPMessage) -> crate::protocol::ProtocolResult;
            async fn validate_message(&self, msg: &MCPMessage) -> crate::protocol::ValidationResult;
            async fn route_message(&self, msg: &MCPMessage) -> crate::protocol::RoutingResult;
            async fn set_state(&self, new_state: crate::types::ProtocolState) -> MCPResult<()>;
            async fn get_state(&self) -> MCPResult<crate::types::ProtocolState>;
            fn get_version(&self) -> String;
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
        
        // No need to set up expectations for register_handler since it's not used anymore
        
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
            id: MessageId("test-id".to_string()),
            type_: MessageType::Sync,
            payload: json!({}),
            metadata: Some(serde_json::Value::Null),
            security: SecurityMetadata::default(),
            timestamp: Utc::now(),
            version: ProtocolVersion::new(1, 0),
            trace_id: None,
        };
        
        // Handle message
        let response = adapter.handle_message(message).await.unwrap();
        
        // Verify response
        assert_eq!(response.status, ResponseStatus::Success);
        
        // Parse payload back to JSON for verification
        let payload_json: serde_json::Value = serde_json::from_slice(&response.payload).unwrap();
        assert_eq!(payload_json.get("version").unwrap(), "1.0.0");
        assert_eq!(payload_json.get("status").unwrap(), "active");
    }
} 