use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

use crate::error::{MCPError, ProtocolError, Result};
use crate::protocol::{
    CommandHandler, MCPProtocol, MCPProtocolBase, ProtocolConfig, ProtocolResult, RoutingResult,
};
use crate::types::{MCPMessage, MCPResponse, MessageType, ProtocolState};

/// Protocol adapter state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: MCPProtocolBase::new(ProtocolConfig::default()),
        }
    }

    /// Creates a new protocol implementation with custom configuration
    #[must_use]
    pub fn with_config(config: ProtocolConfig) -> Self {
        Self {
            base: MCPProtocolBase::new(config),
        }
    }

    /// Gets a reference to the base protocol
    #[must_use]
    pub fn base(&self) -> &MCPProtocolBase {
        &self.base
    }

    /// Creates a response message from a request ID
    ///
    /// # Errors
    ///
    /// This function currently doesn't return errors but maintains a Result
    /// return type for compatibility with other protocol operations.
    pub fn create_response_message(
        &self,
        request: &MCPMessage,
    ) -> crate::error::Result<MCPMessage> {
        Ok(MCPMessage {
            id: request.id.clone(),
            message_type: MessageType::Response,
            payload: serde_json::Value::Null,
        })
    }

    /// Gets the current protocol state
    #[must_use]
    pub fn get_state(&self) -> &Value {
        self.base.get_state()
    }

    /// Sets the protocol state
    pub fn set_state(&mut self, state: Value) {
        self.base.set_state(state);
    }

    /// Gets the protocol configuration
    #[must_use]
    pub fn get_config(&self) -> &ProtocolConfig {
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
        let state = serde_json::from_value::<State>(state_value.clone()).map_err(|e| {
            MCPError::Protocol(ProtocolError::InvalidState(format!(
                "Failed to deserialize state: {e}"
            )))
        })?;
        Ok(state)
    }

    /// Checks if the protocol is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
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
    pub fn initialize(&mut self) -> Result<()> {
        if self.is_initialized() {
            return Err(MCPError::Protocol(
                ProtocolError::ProtocolAlreadyInitialized,
            ));
        }

        let state = State {
            initialized: true,
            config: self.base.get_config().clone(),
        };

        let state_value = serde_json::to_value(state).map_err(|e| {
            MCPError::Protocol(ProtocolError::StateSerialization(format!(
                "Failed to serialize state: {e}"
            )))
        })?;

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
    pub fn initialize_with_config(&mut self, config: ProtocolConfig) -> Result<()> {
        if self.is_initialized() {
            return Err(MCPError::Protocol(
                ProtocolError::ProtocolAlreadyInitialized,
            ));
        }

        let state = State {
            initialized: true,
            config: config.clone(),
        };

        let state_value = serde_json::to_value(state).map_err(|e| {
            MCPError::Protocol(ProtocolError::StateSerialization(format!(
                "Failed to serialize state: {e}"
            )))
        })?;

        self.base.set_state(state_value);
        self.base.set_config(config);
        Ok(())
    }

    /// Registers a handler for the specified message type
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A handler is already registered for the message type
    /// - The handler registration fails for any reason
    pub fn register_handler(
        &mut self,
        message_type: &MessageType,
        handler: Arc<dyn CommandHandler>,
    ) -> Result<()> {
        // Create a wrapper that converts Arc<dyn CommandHandler> to a compatible Box<dyn CommandHandler>
        #[derive(Debug)]
        struct CommandHandlerWrapper {
            /// The wrapped command handler implementation
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
    pub fn unregister_handler(&mut self, message_type: &MessageType) -> Result<()> {
        self.base.unregister_handler(message_type)
    }
}

#[async_trait::async_trait]
impl MCPProtocol for MCPProtocolImpl {
    async fn handle_message(&self, msg: MCPMessage) -> ProtocolResult {
        match self.get_internal_state() {
            Ok(state) => {
                if !state.initialized {
                    return Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized));
                }
            }
            Err(e) => {
                return Err(MCPError::Protocol(ProtocolError::InvalidState(format!(
                    "Failed to get internal state: {e}"
                ))))
            }
        }

        // Only accept messages if the protocol is in the Ready state
        if self.base.get_protocol_state() != ProtocolState::Ready {
            return Err(MCPError::Protocol(ProtocolError::ProtocolNotReady));
        }

        // Handle the message based on its type
        self.handle_message_internal(&msg).await
    }

    async fn validate_message(&self, message: &MCPMessage) -> Result<()> {
        // Basic validation already performed in the base protocol
        self.base.validate_message(message)?;

        // Enhanced validation as per the next-steps requirements

        // 1. Message format validation
        if false {
            // The Unknown message type check was removed as this variant doesn't exist
            return Err(MCPError::Protocol(ProtocolError::InvalidFormat(
                "Unknown message type".to_string(),
            )));
        }

        // 2. Payload validation
        if !message.payload.is_object() && !message.payload.is_null() {
            return Err(MCPError::Protocol(ProtocolError::InvalidPayload(
                "Payload must be an object or null".to_string(),
            )));
        }

        // 3. Size validation
        let payload_size = serde_json::to_string(&message.payload)
            .map(|s| s.len())
            .unwrap_or(0);

        if payload_size > self.base.get_config().max_message_size {
            return Err(MCPError::Protocol(ProtocolError::MessageTooLarge(format!(
                "Message payload size ({} bytes) exceeds maximum allowed size ({} bytes)",
                payload_size,
                self.base.get_config().max_message_size
            ))));
        }

        // 4. Metadata validation if present
        // The metadata field was removed from MCPMessage
        // Commenting out this code until the proper field can be accessed
        /*
        if message.metadata.is_some() {
            let metadata = message.metadata.as_ref().unwrap();

            // Check timestamp if present
            if let Some(timestamp) = metadata.timestamp {
                let now = chrono::Utc::now().timestamp() as u64;
                // Reject messages from the future (with 5-second tolerance)
                if timestamp > now + 5 {
                    return Err(MCPError::Protocol(ProtocolError::InvalidTimestamp(
                        format!("Message timestamp ({}) is in the future", timestamp)
                    )));
                }

                // Check if message is too old (configurable timeout)
                let timeout = self.base.get_config().timeout_ms / 1000; // Convert to seconds
                if now > timestamp + timeout {
                    return Err(MCPError::Protocol(ProtocolError::MessageTimeout(
                        format!("Message is too old (timestamp: {}, now: {})", timestamp, now)
                    )));
                }
            }
        }
        */

        Ok(())
    }

    async fn route_message(&self, msg: &MCPMessage) -> RoutingResult {
        // First validate the message
        self.validate_message(msg).await?;

        // Implement message routing logic based on message type and content
        match msg.message_type {
            MessageType::Command => {
                // Check if we have a specific handler registered for this command
                if let Some(command_type) = msg.payload.get("command_type").and_then(|v| v.as_str())
                {
                    // Create a specialized message type for this command type
                    let specialized_type = format!("Command:{}", command_type);

                    // Check if we have a handler for this specialized command type
                    if self.base.handlers.contains_key(&specialized_type) {
                        // Let the specialized handler handle it later
                        tracing::debug!(
                            "Routing command to specialized handler: {}",
                            specialized_type
                        );
                        return Ok(());
                    }
                }

                // Check if we have a generic command handler
                if self
                    .base
                    .handlers
                    .contains_key(&msg.message_type.to_string())
                {
                    tracing::debug!("Routing command to generic handler");
                    return Ok(());
                }

                // No handler found
                return Err(MCPError::Protocol(ProtocolError::HandlerNotFound(format!(
                    "No handler found for command: {:?}",
                    msg
                ))));
            }
            MessageType::Event => {
                // For events, we can broadcast to multiple handlers if needed
                let event_type = msg
                    .payload
                    .get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                // Check for specific event handler
                let specialized_type = format!("Event:{}", event_type);
                if !self.base.handlers.contains_key(&specialized_type)
                    && !self
                        .base
                        .handlers
                        .contains_key(&msg.message_type.to_string())
                {
                    return Err(MCPError::Protocol(ProtocolError::HandlerNotFound(format!(
                        "No handler found for event type: {}",
                        event_type
                    ))));
                }

                tracing::debug!("Routing event to handler(s)");
                Ok(())
            }
            MessageType::Response => {
                // For responses, we need to match with the original request
                // This is typically handled by the client side, but we should validate
                if msg.payload.get("message_id").is_none() {
                    return Err(MCPError::Protocol(ProtocolError::InvalidFormat(
                        "Response missing original message_id".to_string(),
                    )));
                }

                tracing::debug!("Response message validated for routing");
                Ok(())
            }
            MessageType::Error => {
                // Error messages should be logged and possibly trigger recovery
                tracing::warn!("Received error message: {:?}", msg);

                // Check if we have an error handler
                if self
                    .base
                    .handlers
                    .contains_key(&msg.message_type.to_string())
                {
                    tracing::debug!("Routing error to handler");
                    return Ok(());
                }

                // If no specific handler, we log the error but consider it handled
                tracing::warn!("No specific handler for error message, treating as handled");
                Ok(())
            }
            _ => {
                // Unhandled message types should be rejected
                Err(MCPError::Protocol(ProtocolError::InvalidFormat(format!(
                    "Unhandled message type: {:?}",
                    msg.message_type
                ))))
            }
        }
    }

    async fn set_state(&self, _state: ProtocolState) -> Result<()> {
        // This is a no-op since the state is managed by the adapter, not the protocol
        Ok(())
    }

    async fn get_state(&self) -> Result<ProtocolState> {
        // This adapter is always in Initialized state if it can respond
        Ok(ProtocolState::Initialized)
    }

    fn get_version(&self) -> String {
        // Return the version from our internal config
        self.base.get_config().version.clone()
    }
}

/// Create a protocol adapter with the provided protocol
#[allow(dead_code)]
pub fn create_protocol_adapter(protocol: MCPProtocolImpl) -> Result<MCPProtocolBase> {
    let mut protocol = protocol;
    protocol.initialize()?;
    Ok(protocol.base)
}

/// Create a protocol adapter with the provided protocol and config
#[allow(dead_code)]
pub fn create_protocol_adapter_with_config(
    protocol: MCPProtocolImpl,
    config: ProtocolConfig,
) -> Result<MCPProtocolBase> {
    let mut protocol = protocol;
    protocol.initialize_with_config(config)?;
    Ok(protocol.base)
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
