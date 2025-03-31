//! Types used within the integration module.

use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use crate::error::MCPResult;
use crate::protocol::types::MCPMessage;
use crate::types::MCPResponse;
use crate::error::MCPError; // Import MCPError
use std::collections::HashMap;
use std::sync::Arc;
use crate::security::{
    AuthCredentials, 
    Token, 
    Resource, 
    Action,
    UserId
};
use std::net::IpAddr;
use chrono::{DateTime, Utc};

/// Placeholder User struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub name: String,
    // Add other relevant user fields as needed
}

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

/// Security Context for MCP operations
#[derive(Debug, Clone, Default)]
pub struct SecurityContext {
    /// User context information
    pub user_context: Option<User>,
    /// Credentials if authentication is in progress
    pub credentials: Option<AuthCredentials>,
    /// Authentication token if user is authenticated
    pub token: Option<Token>,
    /// Source IP address of the request
    pub source_ip: Option<IpAddr>,
    /// Timestamp of the request
    pub timestamp: Option<DateTime<Utc>>,
}

/// Message handling interface for MCP communications
///
/// Defines the interface for components that need to handle MCP messages,
/// providing a uniform way to process incoming messages and generate responses.
#[async_trait]
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