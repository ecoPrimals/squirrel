use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{Result, SquirrelError};
use super::{MCPProtocolBase, ProtocolConfig};
use serde_json::Value;
use thiserror::Error;
use crate::mcp::types::{MCPMessage, MCPResponse};

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
#[derive(Debug)]
pub struct MCPProtocolAdapter {
    /// Inner protocol implementation wrapped in a thread-safe container
    inner: Arc<RwLock<Option<MCPProtocolBase>>>,
}

impl MCPProtocolAdapter {
    /// Creates a new protocol adapter
    #[must_use]
    pub fn new() -> Self {
        Self { 
            inner: Arc::new(RwLock::new(None)),
        }
    }

    /// Creates a new adapter with an existing protocol
    #[must_use]
    pub fn with_protocol(protocol: MCPProtocolBase) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Some(protocol))),
        }
    }
    
    /// Initializes the adapter with default configuration
    /// 
    /// # Errors
    /// 
    /// Returns `ProtocolAdapterError::AlreadyInitialized` if the adapter
    /// is already initialized.
    pub async fn initialize(&self) -> Result<()> {
        let mut inner = self.inner.write().await;
        if inner.is_some() {
            return Err(SquirrelError::MCP(ProtocolAdapterError::AlreadyInitialized.to_string()));
        }
        
        let protocol = MCPProtocolBase::new(ProtocolConfig::default());
        *inner = Some(protocol);
        Ok(())
    }
    
    /// Initializes the adapter with a specific configuration
    /// 
    /// # Errors
    /// 
    /// Returns `ProtocolAdapterError::AlreadyInitialized` if the adapter
    /// is already initialized.
    pub async fn initialize_with_config(&self, config: ProtocolConfig) -> Result<()> {
        let mut inner = self.inner.write().await;
        if inner.is_some() {
            return Err(SquirrelError::MCP(ProtocolAdapterError::AlreadyInitialized.to_string()));
        }
        
        let protocol = MCPProtocolBase::new(config);
        *inner = Some(protocol);
        Ok(())
    }

    /// Handles a message using the protocol
    /// 
    /// # Errors
    /// 
    /// Returns `ProtocolAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn handle_message(&self, message: &MCPMessage) -> Result<MCPResponse> {
        let inner = self.inner.read().await;
        if let Some(protocol) = &*inner {
            protocol.handle_message_with_handler(message).await
        } else {
            Err(SquirrelError::MCP(ProtocolAdapterError::NotInitialized.to_string()))
        }
    }

    /// Registers a command handler
    /// 
    /// # Errors
    /// 
    /// Returns `ProtocolAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn register_handler(&self, message_type: crate::mcp::types::MessageType, handler: Box<dyn super::CommandHandler>) -> Result<()> {
        let mut inner = self.inner.write().await;
        if let Some(protocol) = &mut *inner {
            protocol.register_handler(message_type, handler)
        } else {
            Err(SquirrelError::MCP(ProtocolAdapterError::NotInitialized.to_string()))
        }
    }

    /// Unregisters a command handler
    /// 
    /// # Errors
    /// 
    /// Returns `ProtocolAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub async fn unregister_handler(&self, message_type: &crate::mcp::types::MessageType) -> Result<()> {
        let mut inner = self.inner.write().await;
        if let Some(protocol) = &mut *inner {
            protocol.unregister_handler(message_type)
        } else {
            Err(SquirrelError::MCP(ProtocolAdapterError::NotInitialized.to_string()))
        }
    }

    /// Gets the current protocol state
    /// 
    /// Returns `Value::Null` if the adapter is not initialized.
    pub async fn get_state(&self) -> Value {
        let inner = self.inner.read().await;
        if let Some(protocol) = &*inner {
            protocol.get_state().clone()
        } else {
            Value::Null
        }
    }

    /// Sets the protocol state
    /// 
    /// Has no effect if the adapter is not initialized.
    pub async fn set_state(&self, state: Value) {
        let mut inner = self.inner.write().await;
        if let Some(protocol) = &mut *inner {
            protocol.set_state(state);
        }
    }

    /// Gets the protocol configuration
    /// 
    /// Returns default configuration if the adapter is not initialized.
    pub async fn get_config(&self) -> ProtocolConfig {
        let inner = self.inner.read().await;
        if let Some(protocol) = &*inner {
            protocol.get_config().clone()
        } else {
            ProtocolConfig::default()
        }
    }
    
    /// Checks if the adapter is initialized
    pub async fn is_initialized(&self) -> bool {
        let inner = self.inner.read().await;
        inner.is_some()
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

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::mcp::types::{MessageId, MessageType, ResponseStatus, MessageMetadata};
    use serde_json::json;
    
    /// Test handler implementation
    #[derive(Debug)]
    struct TestCommandHandler;
    
    #[async_trait::async_trait]
    impl super::super::CommandHandler for TestCommandHandler {
        async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
            // Create a proper response
            Ok(MCPResponse {
                protocol_version: "1.0".to_string(),
                message_id: message.id.0.clone(),
                status: ResponseStatus::Success,
                payload: serde_json::to_vec(&json!({"response": "success", "original": message.payload})).unwrap(),
                error_message: None,
                metadata: MessageMetadata::default(),
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
        
        // Second initialization should fail
        let err = adapter.initialize().await.unwrap_err();
        assert!(err.to_string().contains("already initialized"));
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
        let err = adapter.handle_message(&message).await.unwrap_err();
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
        let response = adapter.handle_message(&message).await.unwrap();
        
        // Verify response
        assert_eq!(response.status, crate::mcp::types::ResponseStatus::Success);
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
        
        let response1 = adapter.handle_message(&message).await.unwrap();
        let response2 = adapter_clone.handle_message(&message).await.unwrap();
        
        assert_eq!(response1.status, response2.status);
    }
} 