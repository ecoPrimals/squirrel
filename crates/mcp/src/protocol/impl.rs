use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use crate::error::{MCPError, Result};
use crate::mcp::types::{MCPMessage, ProtocolVersion, ProtocolState, MCPCommand, MCPResponse};
use serde_json::Value;

use super::{MCPProtocol, CommandHandler};

impl MCPProtocol {
    pub fn new() -> Self {
        Self {
            state: Value::Null,
            handlers: HashMap::new(),
        }
    }

    pub async fn handle_message(&self, message: &MCPMessage) -> Result<MCPMessage> {
        let handler = self.handlers.get(&message.command)
            .ok_or_else(|| MCPError::Protocol(format!("No handler for command: {}", message.command)))?;

        handler.handle(message).await
    }

    pub fn register_handler(&mut self, command: String, handler: Box<dyn CommandHandler>) -> Result<()> {
        if self.handlers.contains_key(&command) {
            return Err(MCPError::Protocol(format!("Handler already exists for command: {}", command)));
        }
        self.handlers.insert(command, handler);
        Ok(())
    }

    pub fn unregister_handler(&mut self, command: &str) -> Result<()> {
        self.handlers.remove(command)
            .ok_or_else(|| MCPError::Protocol(format!("No handler found for command: {}", command)))?;
        Ok(())
    }

    pub fn get_state(&self) -> &Value {
        &self.state
    }

    pub fn set_state(&mut self, state: Value) {
        self.state = state;
    }
}

impl Default for MCPProtocol {
    fn default() -> Self {
        Self::new()
    }
} 