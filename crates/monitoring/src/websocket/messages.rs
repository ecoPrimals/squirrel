use serde::{Deserialize, Serialize};
use serde_json::Value;

/// WebSocket client action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClientAction {
    /// Subscribe to a topic
    Subscribe,
    /// Unsubscribe from a topic
    Unsubscribe,
    /// Request component data
    GetData,
    /// Request available components
    GetComponents,
    /// Request health status
    GetHealth,
}

/// Client message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    /// Client action
    pub action: ClientAction,
    /// Topic to subscribe to/unsubscribe from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Request ID for correlating responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Server message types for responses to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Component data update
    ComponentUpdate {
        /// Component identifier
        component_id: String,
        /// Component data
        data: Value,
    },
    /// Available components list
    ComponentsList {
        /// List of component IDs
        components: Vec<String>,
        /// Request ID from the client request
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
    },
    /// Health status response
    HealthStatus {
        /// Health status data
        status: Value,
        /// Request ID from the client request
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
    },
    /// Error response
    Error {
        /// Error message
        message: String,
        /// Error code
        code: u16,
        /// Request ID from the client request
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
    },
}

/// Error response codes
pub mod error_codes {
    /// Invalid request format
    pub const INVALID_REQUEST: u16 = 400;
    /// Authentication required
    pub const AUTH_REQUIRED: u16 = 401;
    /// Component not found
    pub const NOT_FOUND: u16 = 404;
    /// Server error
    pub const SERVER_ERROR: u16 = 500;
}

/// Helper methods for ServerMessage
impl ServerMessage {
    /// Create a new component update message
    pub fn component_update(component_id: &str, data: Value) -> Self {
        Self::ComponentUpdate {
            component_id: component_id.to_string(),
            data,
        }
    }
    
    /// Create a new error message
    pub fn error(message: &str, code: u16, request_id: Option<String>) -> Self {
        Self::Error {
            message: message.to_string(),
            code,
            request_id,
        }
    }
} 