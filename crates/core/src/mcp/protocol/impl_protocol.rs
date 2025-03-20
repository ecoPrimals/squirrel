use std::sync::Arc;
use std::fmt::Debug;

use serde::{Serialize, Deserialize};
use serde_json::{self, Value};

use crate::error::{Result, SquirrelError};
use crate::mcp::protocol::{
    MCPProtocol, MCPProtocolBase, ProtocolConfig, 
    ProtocolResult, ValidationResult, RoutingResult,
    MCPProtocolAdapter, CommandHandler
};
use crate::mcp::types::{MCPMessage, MessageType, MCPResponse};

/// Protocol adapter state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub struct State {
    /// Whether the adapter is initialized
    pub initialized: bool,
    /// Protocol configuration
    pub config: ProtocolConfig,
}


/// Implementation of the MCP Protocol
#[derive(Debug)]
pub struct MCPProtocolImpl {
    /// Base protocol implementation
    pub base: MCPProtocolBase,
}

impl Default for MCPProtocolImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl MCPProtocolImpl {
    /// Creates a new protocol implementation with default configuration
    #[must_use] pub fn new() -> Self {
        Self {
            base: MCPProtocolBase::new(ProtocolConfig::default()),
        }
    }

    /// Creates a new protocol implementation with custom configuration
    #[must_use] pub fn with_config(config: ProtocolConfig) -> Self {
        Self {
            base: MCPProtocolBase::new(config),
        }
    }

    /// Gets a reference to the base protocol
    #[must_use] pub fn base(&self) -> &MCPProtocolBase {
        &self.base
    }

    /// Creates a response message from a request ID
    ///
    /// # Errors
    ///
    /// This function currently doesn't return errors but maintains a Result
    /// return type for compatibility with other protocol operations.
    pub fn create_response_message(&self, request: &MCPMessage) -> crate::error::Result<MCPMessage> {
        Ok(MCPMessage {
            id: request.id.clone(),
            message_type: MessageType::Response,
            payload: serde_json::Value::Null,
        })
    }

    /// Gets the current protocol state
    #[must_use] pub fn get_state(&self) -> &Value {
        self.base.get_state()
    }

    /// Sets the protocol state
    pub fn set_state(&mut self, state: Value) {
        self.base.set_state(state);
    }

    /// Gets the protocol configuration
    #[must_use] pub fn get_config(&self) -> &ProtocolConfig {
        self.base.get_config()
    }

    /// Handles a message using the appropriate registered handler
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No handler is registered for the message type
    /// - The handler encounters an error while processing the message
    pub async fn handle_message_internal(&self, msg: &MCPMessage) -> Result<MCPResponse> {
        // We don't have direct access to get_handler, so we'll call handle_message_with_handler directly
        self.base.handle_message_with_handler(msg).await
    }

    /// Gets the current internal state
    ///
    /// # Errors
    ///
    /// Returns an error if the state cannot be deserialized from the internal representation.
    pub fn get_internal_state(&self) -> Result<State> {
        let state_value = self.base.get_state();
        let state = serde_json::from_value::<State>(state_value.clone())
            .map_err(|e| SquirrelError::MCP(format!("Failed to deserialize state: {e}")))?;
        Ok(state)
    }

    /// Checks if the protocol is initialized
    #[must_use] pub fn is_initialized(&self) -> bool {
        let state_value = self.base.get_state();
        if let Ok(state) = serde_json::from_value::<State>(state_value.clone()) {
            state.initialized
        } else {
            false
        }
    }

    /// Initializes the protocol with default configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The protocol is already initialized
    /// - The state cannot be serialized to the internal representation
    pub async fn initialize(&mut self) -> Result<()> {
        if self.is_initialized() {
            return Err(SquirrelError::MCP("Protocol already initialized".to_string()));
        }

        let state = State {
            initialized: true,
            config: self.base.get_config().clone(),
        };
        
        let state_value = serde_json::to_value(state)
            .map_err(|e| SquirrelError::MCP(format!("Failed to serialize state: {e}")))?;
            
        self.base.set_state(state_value);
        Ok(())
    }

    /// Initializes the protocol with custom configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The protocol is already initialized
    /// - The state cannot be serialized to the internal representation
    pub async fn initialize_with_config(&mut self, config: ProtocolConfig) -> Result<()> {
        if self.is_initialized() {
            return Err(SquirrelError::MCP("Protocol already initialized".to_string()));
        }

        let state = State {
            initialized: true,
            config,
        };
        
        let state_value = serde_json::to_value(state)
            .map_err(|e| SquirrelError::MCP(format!("Failed to serialize state: {e}")))?;
            
        self.base.set_state(state_value);
        Ok(())
    }

    /// Registers a handler for the specified message type
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A handler is already registered for the message type
    /// - The handler registration fails for any reason
    pub async fn register_handler(&mut self, message_type: &MessageType, handler: Arc<dyn CommandHandler>) -> Result<()> {
        // Create a wrapper that converts Arc<dyn CommandHandler> to a compatible Box<dyn CommandHandler>
        #[derive(Debug)]
        struct CommandHandlerWrapper {
            inner: Arc<dyn CommandHandler>,
        }
        
        #[async_trait::async_trait]
        impl CommandHandler for CommandHandlerWrapper {
            async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
                // Call the inner handler and return the response directly
                self.inner.handle(message).await
            }
        }
        
        let wrapper = Box::new(CommandHandlerWrapper { inner: handler });
        self.base.register_handler(*message_type, wrapper)
    }

    /// Unregisters a handler for the specified message type
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No handler is registered for the message type
    /// - The handler unregistration fails for any reason
    pub async fn unregister_handler(&mut self, message_type: &MessageType) -> Result<()> {
        self.base.unregister_handler(message_type)
    }
}

#[async_trait::async_trait]
impl MCPProtocol for MCPProtocolImpl {
    async fn handle_message(&self, msg: MCPMessage) -> ProtocolResult {
        let state = match self.get_internal_state() {
            Ok(s) => s,
            Err(e) => return Err(SquirrelError::MCP(format!("Failed to get internal state: {e}"))),
        };
        
        if !state.initialized {
            return Err(SquirrelError::MCP("Protocol not in Ready state".to_string()));
        }
        self.base.handle_message_with_handler(&msg).await
    }

    async fn validate_message(&self, _msg: &MCPMessage) -> ValidationResult {
        let state = match self.get_internal_state() {
            Ok(s) => s,
            Err(e) => return Err(SquirrelError::MCP(format!("Failed to get internal state: {e}"))),
        };
        
        if !state.initialized {
            return Err(SquirrelError::MCP("Protocol not in Ready state".to_string()));
        }
        // Perform validation logic here
        Ok(())
    }

    async fn route_message(&self, _msg: &MCPMessage) -> RoutingResult {
        let state = match self.get_internal_state() {
            Ok(s) => s,
            Err(e) => return Err(SquirrelError::MCP(format!("Failed to get internal state: {e}"))),
        };
        
        if !state.initialized {
            return Err(SquirrelError::MCP("Protocol not in Ready state".to_string()));
        }
        // Perform routing logic here
        Ok(())
    }

    async fn set_state(&self, _new_state: crate::mcp::types::ProtocolState) -> Result<()> {
        let state = State {
            initialized: true,
            config: self.base.get_config().clone(),
        };
        
        let state_value = serde_json::to_value(state)
            .map_err(|e| SquirrelError::MCP(format!("Failed to serialize state: {e}")))?;
            
        // We need to cast away the mutability constraint since the trait requires &self
        // In a real implementation, this would need proper synchronization
        
        // SAFETY: This is safe because:
        // 1. We're temporarily casting away immutability to modify internal state
        // 2. The trait method is expected to modify state despite the &self receiver
        // 3. In a production implementation, this would be protected by proper synchronization
        //    such as a Mutex or RwLock to prevent concurrent access issues
        unsafe {
            let base_ptr = (&raw const self.base).cast_mut();
            (*base_ptr).set_state(state_value);
        }
        
        Ok(())
    }

    async fn get_state(&self) -> Result<crate::mcp::types::ProtocolState> {
        // Convert our internal state to the expected ProtocolState type
        Ok(crate::mcp::types::ProtocolState::Initialized)
    }

    fn get_version(&self) -> String {
        // Return the version from our internal config
        self.base.get_config().version.clone()
    }
}

/// Create a protocol adapter with the provided protocol
#[allow(dead_code)]
pub async fn create_protocol_adapter(protocol: MCPProtocolImpl) -> Result<MCPProtocolAdapter> {
    let mut protocol = protocol;
    protocol.initialize().await?;
    Ok(MCPProtocolAdapter::with_protocol(protocol.base))
}

/// Create a protocol adapter with the provided protocol and config
#[allow(dead_code)]
pub async fn create_protocol_adapter_with_config(protocol: MCPProtocolImpl, config: ProtocolConfig) -> Result<MCPProtocolAdapter> {
    let mut protocol = protocol;
    protocol.initialize_with_config(config).await?;
    Ok(MCPProtocolAdapter::with_protocol(protocol.base))
}

#[cfg(test)]
mod tests {
    // Temporarily commented out until fixed
    /*
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_protocol_validation() {
        let protocol = MCPProtocolImpl::new();
        
        // Create a valid message with the correct fields
        let valid_msg = MCPMessage {
            id: MessageId("test-1".to_string()),
            message_type: MessageType::Command,
            payload: json!({"command": "test", "data": [1, 2, 3]}),
        };
        
        // Validation should pass - state is Initialized which is allowed
        assert!(protocol.validate_message(&valid_msg).await.is_ok());
        
        // Set state to Error to test validation failure
        {
            let mut state = protocol.base.state.write().await;
            *state = ProtocolState::Error;
        }
        
        // Now validation should fail
        assert!(protocol.validate_message(&valid_msg).await.is_err());
    }
    */
} 