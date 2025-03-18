mod integration;
mod team_workflow;
mod git_operations;
mod validation;
mod adapter;
mod protocol_adapter_comprehensive;

// Import and re-export the adapter tests to make them discoverable
pub use adapter::*;
pub use protocol_adapter_comprehensive::*;

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;
use chrono::Utc;
use tokio;
use crate::mcp::{
    MCPMessage, MessageType, SecurityMetadata, SecurityLevel, EncryptionInfo,
    MCPProtocol, PortManager, PortConfig, PortStatus, SecurityManager,
    SecurityConfig, Tool, ToolManager, Capability, Parameter, ParameterType,
    ReturnType, ToolStatus, ErrorHandler,
};

#[tokio::test]
async fn test_protocol_message_handling() {
    let mut protocol = MCPProtocol::new();
    
    // Register message handler
    protocol.register_handler(MessageType::Command, Box::new(|msg| {
        assert_eq!(msg.type_, MessageType::Command);
        Ok(())
    })).await;

    // Create test message
    let message = MCPMessage {
        id: "test-1".to_string(),
        type_: MessageType::Command,
        payload: serde_json::json!({"action": "test"}),
        metadata: None,
        security: SecurityMetadata {
            security_level: SecurityLevel::Low,
            encryption_info: None,
            signature: None,
            auth_token: None,
        },
        timestamp: Utc::now(),
    };

    // Handle message
    assert!(protocol.handle_message(message).await.is_ok());
}

#[tokio::test]
async fn test_port_allocation() {
    let config = PortConfig {
        min_port: 8000,
        max_port: 9000,
        reserved_ports: HashSet::new(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };

    let port_manager = PortManager::new(config);

    // Allocate port
    let port = port_manager.allocate_port("test-owner".to_string()).await.unwrap();
    assert!(port >= 8000 && port <= 9000);

    // Validate connection
    let addr = IpAddr::from_str("127.0.0.1").unwrap();
    assert!(port_manager.validate_connection(port, addr).await.is_err()); // Should fail as no IPs are allowed

    // Release port
    assert!(port_manager.release_port(port).await.is_ok());
}

#[tokio::test]
async fn test_security_authentication() {
    let config = SecurityConfig {
        token_expiry: chrono::Duration::seconds(3600),
        key_rotation_interval: chrono::Duration::hours(24),
        min_key_size: 32,
    };

    let security_manager = SecurityManager::new(config);

    // Create roles
    let mut roles = HashSet::new();
    roles.insert("user".to_string());

    // Authenticate
    let token = security_manager.authenticate("test-user", roles).await.unwrap();
    assert_eq!(token.user_id, "test-user");
    assert!(token.roles.contains("user"));

    // Validate token
    let validated = security_manager.validate_token(&token.token).await.unwrap();
    assert_eq!(validated.user_id, token.user_id);
}

#[tokio::test]
async fn test_tool_management() {
    let tool_manager = ToolManager::new();

    // Create test tool
    let tool = Tool {
        id: "test-tool".to_string(),
        name: "Test Tool".to_string(),
        version: "1.0.0".to_string(),
        description: "A test tool".to_string(),
        capabilities: vec![Capability {
            name: "test".to_string(),
            description: "Test capability".to_string(),
            parameters: vec![Parameter {
                name: "param1".to_string(),
                type_: ParameterType::String,
                description: "Test parameter".to_string(),
                required: true,
                default_value: None,
            }],
            return_type: ReturnType {
                type_: ParameterType::String,
                description: "Test return".to_string(),
            },
            required_permissions: HashSet::new(),
        }],
        security_level: SecurityLevel::Low,
        metadata: HashMap::new(),
    };

    // Register tool
    assert!(tool_manager.register_tool(tool.clone()).await.is_ok());

    // Get tool
    let retrieved = tool_manager.get_tool("test-tool").await.unwrap().unwrap();
    assert_eq!(retrieved.id, tool.id);

    // Update tool state
    assert!(tool_manager.update_tool_state("test-tool", ToolStatus::Active).await.is_ok());

    // Validate capability
    assert!(tool_manager.validate_capability("test-tool", "test").await.is_ok());

    // Unregister tool
    assert!(tool_manager.unregister_tool("test-tool").await.is_ok());
}

#[tokio::test]
async fn test_error_handling() {
    let error_handler = ErrorHandler::new(100);

    // Register recovery handler
    error_handler.register_recovery_handler("test", Box::new(|_| {
        Ok(())
    })).await.unwrap();

    // Handle error
    assert!(error_handler.handle_error(
        crate::mcp::MCPError::Protocol(
            crate::mcp::error::ProtocolError::InvalidMessage("test error".to_string())
        ),
        "test-source".to_string()
    ).await.is_ok());

    // Get error history
    let history = error_handler.get_error_history().await.unwrap();
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_encryption() {
    let config = SecurityConfig {
        token_expiry: chrono::Duration::seconds(3600),
        key_rotation_interval: chrono::Duration::hours(24),
        min_key_size: 32,
    };

    let security_manager = SecurityManager::new(config);

    // Test data
    let data = b"test data";

    // Encrypt
    let (encrypted, key_id) = security_manager.encrypt(data).await.unwrap();
    assert!(!encrypted.is_empty());

    // Decrypt
    let decrypted = security_manager.decrypt(&encrypted, &key_id).await.unwrap();
    assert_eq!(decrypted, data);
}

#[tokio::test]
async fn test_port_monitoring() {
    let config = PortConfig {
        min_port: 8000,
        max_port: 9000,
        reserved_ports: HashSet::new(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };

    let port_manager = PortManager::new(config);

    // Allocate port
    let port = port_manager.allocate_port("test-owner".to_string()).await.unwrap();

    // Get metrics
    let metrics = port_manager.get_metrics(port).await.unwrap();
    assert_eq!(metrics.connections.load(std::sync::atomic::Ordering::SeqCst), 0);

    // Release port
    assert!(port_manager.release_port(port).await.is_ok());
} 