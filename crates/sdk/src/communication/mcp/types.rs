// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! MCP protocol types and data structures
//!
//! This module contains all the core data structures used in the MCP protocol,
//! including messages, capabilities, tools, resources, and connection states.

use serde::{Deserialize, Serialize};

/// MCP message structure
///
/// Core message format for all MCP protocol communication. Each message contains
/// an ID for correlation, a type to identify the operation, payload data, and
/// timestamp for tracking.
///
/// # Examples
///
/// ```
/// use squirrel_sdk::communication::mcp::McpMessage;
/// use serde_json::json;
///
/// let message = McpMessage {
///     id: "msg-001".to_string(),
///     message_type: "ping".to_string(),
///     payload: json!({"timestamp": "2024-01-01T00:00:00Z"}),
///     timestamp: "2024-01-01T00:00:00Z".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    /// Unique message identifier for correlation
    pub id: String,
    /// Message type indicating the operation
    pub message_type: String,
    /// Message payload containing operation-specific data
    pub payload: serde_json::Value,
    /// ISO 8601 timestamp when the message was created
    pub timestamp: String,
}

/// Connection state for MCP client
///
/// Represents the current connection status of the MCP client, used for
/// connection lifecycle management and user feedback.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Client is disconnected from the server
    Disconnected,
    /// Client is in the process of connecting
    Connecting,
    /// Client is connected and ready for operations
    Connected,
    /// Client is attempting to reconnect after connection loss
    Reconnecting,
    /// Connection has failed and cannot be established
    Failed,
}

impl ConnectionState {
    /// Get string representation of connection state
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::ConnectionState;
    ///
    /// let state = ConnectionState::Connected;
    /// assert_eq!(state.as_str(), "Connected");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionState::Disconnected => "Disconnected",
            ConnectionState::Connecting => "Connecting",
            ConnectionState::Connected => "Connected",
            ConnectionState::Reconnecting => "Reconnecting",
            ConnectionState::Failed => "Failed",
        }
    }
}

/// MCP tool definition
///
/// Represents a tool that can be executed through the MCP protocol.
/// Tools are callable functions with defined input and output schemas.
///
/// # Examples
///
/// ```
/// use squirrel_sdk::communication::mcp::McpTool;
/// use serde_json::json;
///
/// let tool = McpTool {
///     name: "calculator".to_string(),
///     description: "Performs basic arithmetic operations".to_string(),
///     input_schema: json!({
///         "type": "object",
///         "properties": {
///             "operation": {"type": "string"},
///             "operands": {"type": "array", "items": {"type": "number"}}
///         }
///     }),
///     output_schema: Some(json!({
///         "type": "object",
///         "properties": {
///             "result": {"type": "number"}
///         }
///     })),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name (unique identifier)
    pub name: String,
    /// Human-readable description of the tool's purpose
    pub description: String,
    /// JSON schema defining the expected input structure
    pub input_schema: serde_json::Value,
    /// Optional JSON schema defining the output structure
    pub output_schema: Option<serde_json::Value>,
}

/// MCP resource definition
///
/// Represents a resource that can be accessed through the MCP protocol.
/// Resources are data sources with metadata and content.
///
/// # Examples
///
/// ```
/// use squirrel_sdk::communication::mcp::McpResource;
/// use serde_json::json;
///
/// let resource = McpResource {
///     uri: "file:///path/to/data.json".to_string(),
///     name: "Configuration Data".to_string(),
///     description: "Application configuration settings".to_string(),
///     metadata: json!({
///         "size": 2048,
///         "format": "json",
///         "last_modified": "2024-01-01T00:00:00Z"
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// Resource URI (unique identifier)
    pub uri: String,
    /// Human-readable resource name
    pub name: String,
    /// Description of the resource content
    pub description: String,
    /// Additional metadata about the resource
    pub metadata: serde_json::Value,
}

/// MCP prompt definition
///
/// Represents a prompt template that can be used through the MCP protocol.
/// Prompts are reusable templates with parameters for generating text.
///
/// # Examples
///
/// ```
/// use squirrel_sdk::communication::mcp::McpPrompt;
/// use serde_json::json;
///
/// let prompt = McpPrompt {
///     name: "summarize".to_string(),
///     description: "Summarizes text content".to_string(),
///     template: "Please summarize the following text: {text}".to_string(),
///     parameters: json!({
///         "text": {
///             "type": "string",
///             "description": "The text to summarize"
///         }
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    /// Prompt name (unique identifier)
    pub name: String,
    /// Human-readable description of the prompt's purpose
    pub description: String,
    /// Template string with parameter placeholders
    pub template: String,
    /// JSON schema defining available parameters
    pub parameters: serde_json::Value,
}

/// MCP-specific capabilities for plugins
///
/// Defines the capabilities and configuration of an MCP client,
/// including supported protocol version, methods, and limits.
///
/// # Examples
///
/// ```
/// use squirrel_sdk::communication::mcp::McpCapabilities;
///
/// let capabilities = McpCapabilities::new()
///     .add_method("custom_tool".to_string());
///
/// assert!(capabilities.supports_mcp);
/// assert!(capabilities.supported_methods.contains(&"custom_tool".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpCapabilities {
    /// Whether the plugin supports MCP protocol
    pub supports_mcp: bool,
    /// Protocol version supported (e.g., "1.0")
    pub protocol_version: String,
    /// List of supported MCP methods
    pub supported_methods: Vec<String>,
    /// Maximum payload size in bytes
    pub max_payload_size: Option<usize>,
}

impl McpCapabilities {
    /// Create new MCP capabilities with default configuration
    ///
    /// Initializes capabilities with commonly supported methods and
    /// configuration from environment variables.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpCapabilities;
    ///
    /// let capabilities = McpCapabilities::new();
    /// assert!(capabilities.supports_mcp);
    /// assert!(!capabilities.supported_methods.is_empty());
    /// ```
    pub fn new() -> Self {
        use crate::config::McpClientConfig;

        let config = McpClientConfig::from_env();
        Self {
            supports_mcp: true,
            protocol_version: config.protocol_version,
            supported_methods: vec![
                "initialize".to_string(),
                "ping".to_string(),
                "list_tools".to_string(),
                "call_tool".to_string(),
                "list_resources".to_string(),
                "read_resource".to_string(),
                "list_prompts".to_string(),
                "get_prompt".to_string(),
            ],
            max_payload_size: Some(config.max_message_size),
        }
    }

    /// Enable MCP support
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpCapabilities;
    ///
    /// let capabilities = McpCapabilities::default().enable_mcp();
    /// assert!(capabilities.supports_mcp);
    /// ```
    pub fn enable_mcp(mut self) -> Self {
        self.supports_mcp = true;
        self
    }

    /// Add a supported method
    ///
    /// # Arguments
    ///
    /// * `method` - The method name to add to supported methods
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpCapabilities;
    ///
    /// let capabilities = McpCapabilities::new()
    ///     .add_method("custom_operation".to_string());
    ///
    /// assert!(capabilities.supported_methods.contains(&"custom_operation".to_string()));
    /// ```
    pub fn add_method(mut self, method: String) -> Self {
        if !self.supported_methods.contains(&method) {
            self.supported_methods.push(method);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_connection_state_as_str() {
        assert_eq!(ConnectionState::Disconnected.as_str(), "Disconnected");
        assert_eq!(ConnectionState::Connecting.as_str(), "Connecting");
        assert_eq!(ConnectionState::Connected.as_str(), "Connected");
        assert_eq!(ConnectionState::Reconnecting.as_str(), "Reconnecting");
        assert_eq!(ConnectionState::Failed.as_str(), "Failed");
    }

    #[test]
    fn test_mcp_message_serialization() {
        let message = McpMessage {
            id: "test-id".to_string(),
            message_type: "test_type".to_string(),
            payload: json!({"key": "value"}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: McpMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, message.id);
        assert_eq!(deserialized.message_type, message.message_type);
        assert_eq!(deserialized.payload, message.payload);
        assert_eq!(deserialized.timestamp, message.timestamp);
    }

    #[test]
    fn test_mcp_capabilities_new() {
        let capabilities = McpCapabilities::new();
        assert!(capabilities.supports_mcp);
        assert!(!capabilities.protocol_version.is_empty());
        assert!(!capabilities.supported_methods.is_empty());
        assert!(capabilities.max_payload_size.is_some());
    }

    #[test]
    fn test_mcp_capabilities_enable_mcp() {
        let mut capabilities = McpCapabilities::default();
        capabilities.supports_mcp = false;

        let enabled_capabilities = capabilities.enable_mcp();
        assert!(enabled_capabilities.supports_mcp);
    }

    #[test]
    fn test_mcp_capabilities_add_method() {
        let capabilities = McpCapabilities::new().add_method("custom_method".to_string());

        assert!(capabilities
            .supported_methods
            .contains(&"custom_method".to_string()));
    }

    #[test]
    fn test_mcp_capabilities_add_duplicate_method() {
        let capabilities = McpCapabilities::new().add_method("ping".to_string()); // "ping" already exists in default

        // Should not add duplicate
        let ping_count = capabilities
            .supported_methods
            .iter()
            .filter(|m| *m == "ping")
            .count();
        assert_eq!(ping_count, 1);
    }

    #[test]
    fn test_mcp_capabilities_default() {
        let capabilities = McpCapabilities::default();
        assert!(!capabilities.supports_mcp);
        assert!(capabilities.protocol_version.is_empty());
        assert!(capabilities.supported_methods.is_empty());
        assert!(capabilities.max_payload_size.is_none());
    }

    #[test]
    fn test_mcp_capabilities_chaining() {
        let capabilities = McpCapabilities::default()
            .enable_mcp()
            .add_method("method1".to_string())
            .add_method("method2".to_string());

        assert!(capabilities.supports_mcp);
        assert!(capabilities
            .supported_methods
            .contains(&"method1".to_string()));
        assert!(capabilities
            .supported_methods
            .contains(&"method2".to_string()));
    }

    #[test]
    fn test_mcp_capabilities_serde() {
        let capabilities = McpCapabilities::new();
        let json = serde_json::to_string(&capabilities).unwrap();
        let deserialized: McpCapabilities = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.supports_mcp, capabilities.supports_mcp);
        assert_eq!(deserialized.protocol_version, capabilities.protocol_version);
        assert_eq!(
            deserialized.supported_methods.len(),
            capabilities.supported_methods.len()
        );
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connecting, ConnectionState::Reconnecting);
    }

    #[test]
    fn test_mcp_tool_serialization() {
        let tool = McpTool {
            name: "calculator".to_string(),
            description: "Performs arithmetic".to_string(),
            input_schema: json!({"type": "object"}),
            output_schema: Some(json!({"type": "number"})),
        };

        let json_str = serde_json::to_string(&tool).unwrap();
        let deserialized: McpTool = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.name, "calculator");
        assert_eq!(deserialized.description, "Performs arithmetic");
        assert!(deserialized.output_schema.is_some());
    }

    #[test]
    fn test_mcp_tool_without_output_schema() {
        let tool = McpTool {
            name: "notify".to_string(),
            description: "Sends notification".to_string(),
            input_schema: json!({"message": "string"}),
            output_schema: None,
        };

        let json_str = serde_json::to_string(&tool).unwrap();
        let deserialized: McpTool = serde_json::from_str(&json_str).unwrap();

        assert!(deserialized.output_schema.is_none());
    }

    #[test]
    fn test_mcp_resource_serialization() {
        let resource = McpResource {
            uri: "file:///data.json".to_string(),
            name: "Data File".to_string(),
            description: "Test data".to_string(),
            metadata: json!({"size": 1024}),
        };

        let json_str = serde_json::to_string(&resource).unwrap();
        let deserialized: McpResource = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.uri, "file:///data.json");
        assert_eq!(deserialized.name, "Data File");
        assert_eq!(deserialized.metadata["size"], 1024);
    }

    #[test]
    fn test_mcp_prompt_serialization() {
        let prompt = McpPrompt {
            name: "summarize".to_string(),
            description: "Summarizes text".to_string(),
            template: "Summarize: {text}".to_string(),
            parameters: json!({"text": {"type": "string"}}),
        };

        let json_str = serde_json::to_string(&prompt).unwrap();
        let deserialized: McpPrompt = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.name, "summarize");
        assert_eq!(deserialized.template, "Summarize: {text}");
    }

    #[test]
    fn test_mcp_message_clone() {
        let message = McpMessage {
            id: "msg-1".to_string(),
            message_type: "ping".to_string(),
            payload: json!({"ts": 12345}),
            timestamp: "2024-01-01".to_string(),
        };

        let cloned = message.clone();
        assert_eq!(cloned.id, message.id);
        assert_eq!(cloned.message_type, message.message_type);
        assert_eq!(cloned.payload, message.payload);
    }
}
