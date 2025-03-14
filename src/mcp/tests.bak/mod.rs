mod team_workflow;
mod validation;
mod git_operations;
mod integration;

pub use team_workflow::*;
pub use validation::*;
pub use git_operations::*;
pub use integration::*;

/// Test utilities for MCP module
pub mod utils {
    use std::time::Duration;
    use tokio;

    use crate::mcp::{
        MCPProtocol, MCPMessage, MessageType, SecurityMetadata, SecurityLevel,
        PortManager, PortConfig, SecurityManager, SecurityConfig, ToolManager,
        Tool, Capability, Parameter, ParameterType, ReturnType, ToolStatus,
    };

    /// Creates a test protocol instance with default configuration
    pub fn create_test_protocol() -> MCPProtocol {
        MCPProtocol::new()
    }

    /// Creates a test port manager with default configuration
    pub fn create_test_port_manager() -> PortManager {
        PortManager::new(PortConfig {
            min_port: 8000,
            max_port: 9000,
            reserved_ports: std::collections::HashSet::new(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
        })
    }

    /// Creates a test security manager with default configuration
    pub fn create_test_security_manager() -> SecurityManager {
        SecurityManager::new(SecurityConfig {
            encryption_enabled: true,
            signature_required: true,
            auth_required: true,
            allowed_origins: vec!["localhost".to_string()],
        })
    }

    /// Creates a test tool manager
    pub fn create_test_tool_manager() -> ToolManager {
        ToolManager::new()
    }

    /// Creates a test tool with default configuration
    pub fn create_test_tool(id: &str) -> Tool {
        Tool {
            id: id.to_string(),
            name: format!("Test Tool {}", id),
            version: "1.0.0".to_string(),
            description: "A test tool".to_string(),
            capabilities: vec![Capability {
                name: "test_operation".to_string(),
                description: "Test operation".to_string(),
                parameters: vec![],
                return_type: ReturnType {
                    type_: ParameterType::String,
                    description: "Test result".to_string(),
                },
                required_permissions: std::collections::HashSet::new(),
            }],
            security_level: SecurityLevel::Low,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Creates a test message with default configuration
    pub fn create_test_message(id: &str) -> MCPMessage {
        MCPMessage {
            id: id.to_string(),
            type_: MessageType::Command,
            version: "1.0.0".to_string(),
            payload: serde_json::json!({"action": "test"}),
            metadata: None,
            security: SecurityMetadata {
                security_level: SecurityLevel::Low,
                encryption_info: None,
                signature: None,
                auth_token: None,
            },
            timestamp: chrono::Utc::now(),
        }
    }

    /// Waits for a specified duration
    pub async fn wait(duration: Duration) {
        tokio::time::sleep(duration).await;
    }
} 