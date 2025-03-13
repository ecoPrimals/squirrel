use std::time::Duration;
use tokio;
use tempfile::tempdir;

use crate::mcp::{
    MCPProtocol, MCPMessage, MessageType, SecurityMetadata, SecurityLevel,
    PortManager, PortConfig, SecurityManager, SecurityConfig, ToolManager,
    Tool, Capability, Parameter, ParameterType, ReturnType, ToolStatus,
    ErrorHandler, GitManager, GitConfig,
};

#[tokio::test]
async fn test_protocol_integration() {
    let mut protocol = MCPProtocol::new();
    let port_manager = PortManager::new(PortConfig {
        min_port: 8000,
        max_port: 9000,
        reserved_ports: std::collections::HashSet::new(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
    });

    let security_manager = SecurityManager::new(SecurityConfig {
        encryption_enabled: true,
        signature_required: true,
        auth_required: true,
        allowed_origins: vec!["localhost".to_string()],
    });

    let tool_manager = ToolManager::new();
    let error_handler = ErrorHandler::new(100);

    // Test message handling
    let message = MCPMessage {
        id: "test-message".to_string(),
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
    };

    let result = protocol.handle_message(message).await;
    assert!(result.is_ok(), "Failed to handle message");
}

#[tokio::test]
async fn test_tool_security_integration() {
    let tool_manager = ToolManager::new();
    let security_manager = SecurityManager::new(SecurityConfig {
        encryption_enabled: true,
        signature_required: true,
        auth_required: true,
        allowed_origins: vec!["localhost".to_string()],
    });

    // Create and register tool with security requirements
    let tool = Tool {
        id: "secure-tool".to_string(),
        name: "Secure Test Tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Tool with security requirements".to_string(),
        capabilities: vec![Capability {
            name: "secure_operation".to_string(),
            description: "Secure test operation".to_string(),
            parameters: vec![],
            return_type: ReturnType {
                type_: ParameterType::String,
                description: "Secure result".to_string(),
            },
            required_permissions: std::collections::HashSet::from(["admin".to_string()]),
        }],
        security_level: SecurityLevel::High,
        metadata: std::collections::HashMap::new(),
    };

    let result = tool_manager.register_tool(tool.clone()).await;
    assert!(result.is_ok(), "Failed to register secure tool");

    // Test security validation
    let result = security_manager.validate_tool_access(&tool.id, &["user".to_string()]).await;
    assert!(result.is_err(), "Unauthorized access should be rejected");

    let result = security_manager.validate_tool_access(&tool.id, &["admin".to_string()]).await;
    assert!(result.is_ok(), "Authorized access should be allowed");
}

#[tokio::test]
async fn test_git_tool_integration() {
    let temp_dir = tempdir().unwrap();
    let git_manager = GitManager::new(GitConfig {
        repo_path: temp_dir.path().to_path_buf(),
        branch: "main".to_string(),
        remote: None,
        credentials: None,
    });

    let tool_manager = ToolManager::new();

    // Initialize git repository
    git_manager.initialize().await.unwrap();

    // Create and register git tool
    let tool = Tool {
        id: "git-tool".to_string(),
        name: "Git Test Tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Tool for git operations".to_string(),
        capabilities: vec![Capability {
            name: "git_operation".to_string(),
            description: "Git test operation".to_string(),
            parameters: vec![
                Parameter {
                    name: "operation".to_string(),
                    type_: ParameterType::String,
                    description: "Git operation to perform".to_string(),
                    required: true,
                    default: None,
                },
            ],
            return_type: ReturnType {
                type_: ParameterType::String,
                description: "Git operation result".to_string(),
            },
            required_permissions: std::collections::HashSet::new(),
        }],
        security_level: SecurityLevel::Low,
        metadata: std::collections::HashMap::new(),
    };

    let result = tool_manager.register_tool(tool.clone()).await;
    assert!(result.is_ok(), "Failed to register git tool");

    // Test git operation through tool
    let result = tool_manager.execute_tool_capability(
        &tool.id,
        "git_operation",
        &[("operation".to_string(), "status".to_string())],
    ).await;
    assert!(result.is_ok(), "Failed to execute git operation");
}

#[tokio::test]
async fn test_error_handling_integration() {
    let error_handler = ErrorHandler::new(100);
    let tool_manager = ToolManager::new();

    // Register error recovery handler
    let result = error_handler.register_recovery_handler(
        "test_error",
        Box::new(|_| Ok(())),
    ).await;
    assert!(result.is_ok(), "Failed to register error handler");

    // Test error handling with tool execution
    let tool = Tool {
        id: "error-tool".to_string(),
        name: "Error Test Tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Tool for testing error handling".to_string(),
        capabilities: vec![Capability {
            name: "error_operation".to_string(),
            description: "Operation that may fail".to_string(),
            parameters: vec![],
            return_type: ReturnType {
                type_: ParameterType::String,
                description: "Operation result".to_string(),
            },
            required_permissions: std::collections::HashSet::new(),
        }],
        security_level: SecurityLevel::Low,
        metadata: std::collections::HashMap::new(),
    };

    let result = tool_manager.register_tool(tool.clone()).await;
    assert!(result.is_ok(), "Failed to register error tool");

    // Test error recovery
    let result = error_handler.handle_error(
        crate::mcp::MCPError::Protocol(
            crate::mcp::error::ProtocolError::InvalidMessage("test error".to_string())
        ),
        "test-source".to_string(),
    ).await;
    assert!(result.is_ok(), "Failed to handle error");

    // Verify error history
    let history = error_handler.get_error_history().await;
    assert!(history.is_ok(), "Failed to get error history");
    let history = history.unwrap();
    assert!(!history.is_empty(), "Error history should not be empty");
} 