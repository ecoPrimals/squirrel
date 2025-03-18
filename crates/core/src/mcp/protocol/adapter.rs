use std::sync::Arc;
use crate::error::Result;
use super::{MCPProtocol, MCPMessage, CommandHandler, ProtocolConfig};
use serde_json::Value;
use std::collections::HashMap;

/// Adapter for the MCP protocol to support dependency injection
#[derive(Debug)]
pub struct MCPProtocolAdapter {
    inner: Option<Arc<MCPProtocol>>,
}

impl MCPProtocolAdapter {
    /// Creates a new protocol adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing protocol
    #[must_use]
    pub fn with_protocol(protocol: Arc<MCPProtocol>) -> Self {
        Self {
            inner: Some(protocol),
        }
    }

    /// Handles a message using the protocol
    pub async fn handle_message(&self, message: &MCPMessage) -> Result<MCPMessage> {
        if let Some(protocol) = &self.inner {
            protocol.handle_message(message).await
        } else {
            // Try to initialize on-demand with default configuration
            let protocol = MCPProtocol::new(ProtocolConfig::default());
            Arc::new(protocol).handle_message(message).await
        }
    }

    /// Registers a command handler
    pub fn register_handler(&mut self, command: String, handler: Box<dyn CommandHandler>) -> Result<()> {
        if let Some(protocol) = &mut self.inner {
            protocol.register_handler(command, handler)
        } else {
            // Initialize on-demand with default configuration
            let mut protocol = MCPProtocol::new(ProtocolConfig::default());
            protocol.register_handler(command, handler)?;
            self.inner = Some(Arc::new(protocol));
            Ok(())
        }
    }

    /// Unregisters a command handler
    pub fn unregister_handler(&mut self, command: &str) -> Result<()> {
        if let Some(protocol) = &mut self.inner {
            protocol.unregister_handler(command)
        } else {
            // No protocol to unregister from
            Ok(())
        }
    }

    /// Gets the current protocol state
    pub fn get_state(&self) -> Value {
        if let Some(protocol) = &self.inner {
            protocol.get_state().clone()
        } else {
            Value::Null
        }
    }

    /// Sets the protocol state
    pub fn set_state(&mut self, state: Value) {
        if let Some(protocol) = &mut self.inner {
            protocol.set_state(state);
        }
    }

    /// Gets the protocol configuration
    pub fn get_config(&self) -> ProtocolConfig {
        if let Some(protocol) = &self.inner {
            protocol.get_config().clone()
        } else {
            ProtocolConfig::default()
        }
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
pub fn create_protocol_adapter_with_protocol(protocol: Arc<MCPProtocol>) -> Arc<MCPProtocolAdapter> {
    Arc::new(MCPProtocolAdapter::with_protocol(protocol))
} 