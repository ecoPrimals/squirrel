use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde_json::{Value, json};
use crate::protocol::{MCPProtocol, MCPProtocolBase, ProtocolConfig, ProtocolResult, ValidationResult, RoutingResult};
use crate::types::{MCPMessage, MessageType, ProtocolState};
use crate::error::{Result, MCPError, ProtocolError};
use thiserror::Error;

/// Errors specific to MCP protocol operations
#[derive(Debug, Error)]
pub enum ProtocolAdapterError {
    /// Protocol is not initialized
    #[error("Protocol not initialized")]
    NotInitialized,
    
    /// Protocol is already initialized
    #[error("Protocol already initialized")]
    AlreadyInitialized,
}

/// Protocol adapter that provides a clean interface for working with the MCP protocol
pub struct MCPProtocolAdapter {
    /// Inner protocol implementation
    inner: Arc<RwLock<Option<MCPProtocolBase>>>,
}

impl MCPProtocolAdapter {
    /// Creates a new empty protocol adapter
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Create a new protocol adapter with a given protocol implementation
    #[must_use]
    pub fn with_protocol(protocol: MCPProtocolBase) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Some(protocol))),
        }
    }
    
    /// Initialize the protocol adapter with a base protocol implementation
    ///
    /// # Errors
    /// Returns an error if the protocol cannot be initialized
    pub async fn initialize(&self) -> Result<()> {
        let mut inner = self.inner.write().await;
        
        if inner.is_some() {
            return Ok(());
        }
        
        *inner = Some(MCPProtocolBase::new(ProtocolConfig::default()));
        Ok(())
    }
    
    /// Initialize with a specific configuration
    ///
    /// # Errors
    /// Returns an error if the protocol cannot be initialized with the given config
    pub async fn initialize_with_config(&self, config: ProtocolConfig) -> Result<()> {
        let mut inner = self.inner.write().await;
        
        if inner.is_some() {
            return Ok(());
        }
        
        *inner = Some(MCPProtocolBase::with_config(config));
        Ok(())
    }
    
    /// Check if inner protocol is initialized
    pub async fn is_initialized(&self) -> bool {
        let inner = self.inner.read().await;
        inner.is_some()
    }

    /// Handle a message according to the protocol
    pub async fn handle_message(&self, msg: MCPMessage) -> ProtocolResult {
        let protocol_guard = self.inner.read().await;
        
        if let Some(protocol) = &*protocol_guard {
            // Special handling for setup message
            if msg.message_type == MessageType::Setup {
                // Setup messages should be processed even without payload
                return protocol.handle_protocol_message(&msg).await;
            }
            
            if !msg.payload.is_object() {
                return Err(MCPError::Protocol(ProtocolError::InvalidPayload("Empty or invalid payload".to_string())));
            }
            
            // Validate and route the message
            protocol.validate_message(&msg)?;
            
            protocol.handle_protocol_message(&msg).await
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized))
        }
    }

    /// Registers a command handler for a specific message type.
    ///
    /// Returns `ProtocolAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn register_handler(&self, message_type: crate::types::MessageType, handler: Box<dyn super::CommandHandler>) -> Result<()> {
        let mut inner = self.inner.write().await;
        if let Some(protocol) = &mut *inner {
            protocol.register_handler(message_type, handler)
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized))
        }
    }

    /// Unregisters a command handler for a specific message type.
    ///
    /// Returns `ProtocolAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn unregister_handler(&self, message_type: &crate::types::MessageType) -> Result<()> {
        let mut inner = self.inner.write().await;
        if let Some(protocol) = &mut *inner {
            protocol.unregister_handler(message_type)
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized))
        }
    }

    /// Get protocol state as JSON
    pub async fn get_state(&self) -> Value {
        let inner = self.inner.read().await;
        if let Some(ref protocol) = *inner {
            protocol.get_state().clone()
        } else {
            json!({})
        }
    }

    /// Set protocol state
    pub async fn set_state(&self, state: Value) -> Result<()> {
        let mut inner = self.inner.write().await;
        if let Some(ref mut protocol) = *inner {
            protocol.set_state(state);
            Ok(())
        } else {
            Err(MCPError::Protocol(ProtocolError::InvalidState("Protocol not initialized".to_string())))
        }
    }

    /// Get protocol configuration
    pub async fn get_config(&self) -> ProtocolConfig {
        let inner = self.inner.read().await;
        if let Some(ref protocol) = *inner {
            protocol.get_config().clone()
        } else {
            ProtocolConfig::default()
        }
    }

    /// Gets the current state
    pub fn get_version(&self) -> String {
        // The version is a static string, no need to lock
        "1.0".to_string()
    }
}

impl Clone for MCPProtocolAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for MCPProtocolAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new protocol adapter
#[must_use]
pub fn create_protocol_adapter() -> Arc<MCPProtocolAdapter> {
    Arc::new(MCPProtocolAdapter::new())
}

/// Creates a new protocol adapter with an existing protocol
#[must_use]
pub fn create_protocol_adapter_with_protocol(protocol: MCPProtocolBase) -> Arc<MCPProtocolAdapter> {
    Arc::new(MCPProtocolAdapter::with_protocol(protocol))
}

/// Creates a new protocol adapter and initializes it with default configuration
/// 
/// # Errors
/// 
/// Returns an error if initialization fails.
pub async fn create_initialized_protocol_adapter() -> Result<Arc<MCPProtocolAdapter>> {
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize().await?;
    Ok(Arc::new(adapter))
}

/// Creates a new protocol adapter and initializes it with custom configuration
/// 
/// # Errors
/// 
/// Returns an error if initialization fails.
pub async fn create_protocol_adapter_with_config(config: ProtocolConfig) -> Result<Arc<MCPProtocolAdapter>> {
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize_with_config(config).await?;
    Ok(Arc::new(adapter))
}

#[async_trait]
impl MCPProtocol for MCPProtocolAdapter {
    async fn handle_message(&self, msg: MCPMessage) -> ProtocolResult {
        let protocol_guard = self.inner.read().await;
        
        if let Some(protocol) = &*protocol_guard {
            // Special handling for setup message
            if msg.message_type == MessageType::Setup {
                // Setup messages should be processed even without payload
                return protocol.handle_protocol_message(&msg).await;
            }
            
            if !msg.payload.is_object() {
                return Err(MCPError::Protocol(ProtocolError::InvalidPayload("Empty or invalid payload".to_string())));
            }
            
            // Validate and route the message
            protocol.validate_message(&msg)?;
            
            protocol.handle_protocol_message(&msg).await
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized))
        }
    }
    
    async fn validate_message(&self, msg: &MCPMessage) -> ValidationResult {
        let protocol_guard = self.inner.read().await;
        
        if let Some(protocol) = &*protocol_guard {
            protocol.validate_message(msg)
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized))
        }
    }
    
    async fn route_message(&self, _msg: &MCPMessage) -> RoutingResult {
        let protocol_guard = self.inner.read().await;
        
        if let Some(ref _protocol) = *protocol_guard {
            // Check if the protocol has registered a handler for this message type
            // For now, we're just implementing a basic placeholder
            // In the future, this would delegate to protocol's handlers
            Ok(())
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized))
        }
    }
    
    async fn set_state(&self, new_state: ProtocolState) -> Result<()> {
        let mut protocol_guard = self.inner.write().await;
        
        if let Some(protocol) = &mut *protocol_guard {
            protocol.set_protocol_state(new_state);
            Ok(())
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized))
        }
    }
    
    async fn get_state(&self) -> Result<ProtocolState> {
        let inner = self.inner.read().await;
        if let Some(ref _protocol) = *inner {
            // Here we need to convert from the internal state to the ProtocolState
            Ok(ProtocolState::Ready)
        } else {
            Err(MCPError::Protocol(ProtocolError::InvalidState("Protocol not initialized".to_string())))
        }
    }
    
    fn get_version(&self) -> String {
        // The version is a static string, so we can simply return it directly
        // without needing to await anything
        "1.0".to_string()
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::types::{MessageId, MessageType, ResponseStatus, MessageMetadata, MCPResponse};
    use serde_json::json;
    
    /// Test handler implementation
    #[derive(Debug)]
    struct TestCommandHandler;
    
    #[async_trait::async_trait]
    impl super::super::CommandHandler for TestCommandHandler {
        async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
            // Simple test implementation
            Ok(MCPResponse {
                protocol_version: "1.0".to_string(),
                message_id: message.id.0.clone(),
                status: ResponseStatus::Success,
                metadata: MessageMetadata::default(),
                payload: serde_json::to_vec(&json!({"response": "success"})).unwrap(),
                error_message: None,
            })
        }
    }
    
    #[tokio::test]
    async fn test_adapter_initialization() {
        // Create a new adapter
        let adapter = MCPProtocolAdapter::new();
        
        // Should not be initialized yet
        assert!(!adapter.is_initialized().await);
        
        // Initialize it
        adapter.initialize().await.unwrap();
        
        // Now it should be initialized
        assert!(adapter.is_initialized().await);
        
        // Second initialization should be ok (idempotent)
        let result = adapter.initialize().await;
        assert!(result.is_ok(), "Second initialization should be idempotent");
    }
    
    #[tokio::test]
    async fn test_adapter_with_config() {
        // Create a custom config
        let config = ProtocolConfig {
            version: "2.0".to_string(),
            max_message_size: 2048,
            timeout_ms: 10000,
        };
        
        // Create and initialize with config
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize_with_config(config.clone()).await.unwrap();
        
        // Check the config was set
        let adapter_config = adapter.get_config().await;
        assert_eq!(adapter_config.version, "2.0");
        assert_eq!(adapter_config.max_message_size, 2048);
        assert_eq!(adapter_config.timeout_ms, 10000);
    }
    
    #[tokio::test]
    async fn test_uninitialized_operations() {
        // Create a new adapter without initializing
        let adapter = MCPProtocolAdapter::new();
        
        // Create a test message
        let message = MCPMessage {
            id: MessageId("test-1".to_string()),
            message_type: MessageType::Command,
            payload: json!({"command": "test"}),
        };
        
        // Trying to handle a message should fail
        let err = adapter.handle_message(message).await.unwrap_err();
        assert!(err.to_string().contains("not initialized"));
        
        // Register handler should also fail
        let handler = Box::new(TestCommandHandler);
        let err = adapter.register_handler(MessageType::Command, handler).await.unwrap_err();
        assert!(err.to_string().contains("not initialized"));
    }
    
    #[tokio::test]
    async fn test_handler_registration() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();
        
        // Register a handler
        let handler = Box::new(TestCommandHandler);
        adapter.register_handler(MessageType::Command, handler).await.unwrap();
        
        // Create a test message
        let message = MCPMessage {
            id: MessageId("test-1".to_string()),
            message_type: MessageType::Command,
            payload: json!({"command": "test"}),
        };
        
        // Handle the message
        let response = adapter.handle_message(message).await.unwrap();
        
        // Verify response
        assert_eq!(response.status, crate::types::ResponseStatus::Success);
        assert_eq!(response.message_id, "test-1");
    }
    
    #[tokio::test]
    async fn test_factory_functions() {
        // Test simple creation
        let adapter1 = create_protocol_adapter();
        assert!(!adapter1.is_initialized().await);
        
        // Test creation with an existing protocol
        let config = ProtocolConfig::default();
        let protocol = MCPProtocolBase::new(config);
        let adapter2 = create_protocol_adapter_with_protocol(protocol);
        assert!(adapter2.is_initialized().await);
        
        // Test initialized creation
        let adapter3 = create_initialized_protocol_adapter().await.unwrap();
        assert!(adapter3.is_initialized().await);
        
        // Test creation with config
        let custom_config = ProtocolConfig {
            version: "3.0".to_string(),
            max_message_size: 4096,
            timeout_ms: 15000,
        };
        let adapter4 = create_protocol_adapter_with_config(custom_config).await.unwrap();
        assert!(adapter4.is_initialized().await);
        
        let config4 = adapter4.get_config().await;
        assert_eq!(config4.version, "3.0");
    }
    
    #[tokio::test]
    async fn test_state_management() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();
        
        // Default state should be null
        let state = adapter.get_state().await;
        assert!(state.is_null());
        
        // Set a new state
        let new_state = json!({"status": "connected", "client_id": "test-client"});
        adapter.set_state(new_state.clone()).await;
        
        // Get the state again
        let updated_state = adapter.get_state().await;
        assert_eq!(updated_state, new_state);
    }
    
    #[tokio::test]
    async fn test_adapter_cloning() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();
        
        // Register a handler
        let handler = Box::new(TestCommandHandler);
        adapter.register_handler(MessageType::Command, handler).await.unwrap();
        
        // Clone the adapter
        let adapter_clone = adapter.clone();
        
        // Both should be initialized
        assert!(adapter.is_initialized().await);
        assert!(adapter_clone.is_initialized().await);
        
        // Both should have the handler
        let message = MCPMessage {
            id: MessageId("test-1".to_string()),
            message_type: MessageType::Command,
            payload: json!({"command": "test"}),
        };
        
        let response1 = adapter.handle_message(message.clone()).await.unwrap();
        let response2 = adapter_clone.handle_message(message).await.unwrap();
        
        assert_eq!(response1.status, response2.status);
    }
} 