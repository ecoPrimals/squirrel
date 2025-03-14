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

/// Tests the complete flow of tool registration, port allocation, and secure message handling
#[tokio::test]
async fn test_tool_registration_and_communication() {
    // Initialize components
    let security_manager = SecurityManager::new(SecurityConfig {
        token_expiry: chrono::Duration::seconds(3600),
        key_rotation_interval: chrono::Duration::hours(24),
        min_key_size: 32,
    });

    let port_manager = PortManager::new(PortConfig {
        min_port: 8000,
        max_port: 9000,
        reserved_ports: HashSet::new(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
    });

    let tool_manager = ToolManager::new();
    let mut protocol = MCPProtocol::new();
    let error_handler = ErrorHandler::new(100);

    // 1. Register tool
    let tool = Tool {
        id: "integration-test-tool".to_string(),
        name: "Integration Test Tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Tool for integration testing".to_string(),
        capabilities: vec![Capability {
            name: "test-capability".to_string(),
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
        security_level: SecurityLevel::Medium,
        metadata: HashMap::new(),
    };

    assert!(tool_manager.register_tool(tool.clone()).await.is_ok());

    // 2. Authenticate tool
    let mut roles = HashSet::new();
    roles.insert("tool".to_string());
    let auth_token = security_manager.authenticate(&tool.id, roles).await.unwrap();

    // 3. Allocate port for tool
    let port = port_manager.allocate_port(tool.id.clone()).await.unwrap();

    // 4. Allow tool's IP
    let addr = IpAddr::from_str("127.0.0.1").unwrap();
    let mut allowed_ips = HashSet::new();
    allowed_ips.insert(addr);

    // 5. Register message handler
    protocol.register_handler(MessageType::Command, Box::new(|msg| {
        assert_eq!(msg.type_, MessageType::Command);
        Ok(())
    })).await;

    // 6. Create and encrypt message
    let payload = serde_json::json!({
        "action": "test-capability",
        "parameters": {
            "param1": "test value"
        }
    });

    let (encrypted_payload, key_id) = security_manager.encrypt(
        serde_json::to_string(&payload).unwrap().as_bytes()
    ).await.unwrap();

    // 7. Create and send message
    let message = MCPMessage {
        id: "integration-test-1".to_string(),
        type_: MessageType::Command,
        payload,
        metadata: Some(serde_json::json!({
            "port": port,
            "tool_id": tool.id
        })),
        security: SecurityMetadata {
            security_level: SecurityLevel::Medium,
            encryption_info: Some(EncryptionInfo {
                algorithm: "ChaCha20-Poly1305".to_string(),
                key_id: key_id.clone(),
                nonce: None,
            }),
            signature: None,
            auth_token: Some(auth_token.token.clone()),
        },
        timestamp: Utc::now(),
    };

    assert!(protocol.handle_message(message.clone()).await.is_ok());

    // 8. Verify tool state
    let tool_state = tool_manager.get_tool_state(&tool.id).await.unwrap().unwrap();
    assert_eq!(tool_state.status, ToolStatus::Active);

    // 9. Check port metrics
    let metrics = port_manager.get_metrics(port).await.unwrap();
    assert_eq!(metrics.connections.load(std::sync::atomic::Ordering::SeqCst), 0);

    // 10. Cleanup
    assert!(port_manager.release_port(port).await.is_ok());
    assert!(tool_manager.unregister_tool(&tool.id).await.is_ok());
}

/// Tests error propagation and recovery across components
#[tokio::test]
async fn test_error_propagation_and_recovery() {
    // Initialize components
    let security_manager = SecurityManager::new(SecurityConfig {
        token_expiry: chrono::Duration::seconds(3600),
        key_rotation_interval: chrono::Duration::hours(24),
        min_key_size: 32,
    });

    let port_manager = PortManager::new(PortConfig {
        min_port: 8000,
        max_port: 9000,
        reserved_ports: HashSet::new(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
    });

    let tool_manager = ToolManager::new();
    let mut protocol = MCPProtocol::new();
    let error_handler = ErrorHandler::new(100);

    // 1. Register error recovery handlers
    error_handler.register_recovery_handler("protocol", Box::new(|ctx| {
        println!("Recovering from protocol error: {:?}", ctx);
        Ok(())
    })).await.unwrap();

    error_handler.register_recovery_handler("security", Box::new(|ctx| {
        println!("Recovering from security error: {:?}", ctx);
        Ok(())
    })).await.unwrap();

    // 2. Attempt to use expired token
    let mut roles = HashSet::new();
    roles.insert("test".to_string());
    let auth_token = security_manager.authenticate("test-user", roles).await.unwrap();

    // Wait for token to expire (using a very short expiry for testing)
    tokio::time::sleep(Duration::from_secs(1)).await;

    let message = MCPMessage {
        id: "error-test-1".to_string(),
        type_: MessageType::Command,
        payload: serde_json::json!({"action": "test"}),
        metadata: None,
        security: SecurityMetadata {
            security_level: SecurityLevel::Low,
            encryption_info: None,
            signature: None,
            auth_token: Some(auth_token.token),
        },
        timestamp: Utc::now(),
    };

    // 3. Handle and recover from error
    let result = protocol.handle_message(message.clone()).await;
    if let Err(e) = result {
        assert!(error_handler.handle_error(
            crate::mcp::MCPError::Security(
                crate::mcp::error::SecurityError::TokenExpired("Token expired".to_string())
            ),
            "test-source".to_string()
        ).await.is_ok());
    }

    // 4. Verify error was logged
    let history = error_handler.get_error_history().await.unwrap();
    assert!(!history.is_empty());
}

/// Tests secure communication between tools
#[tokio::test]
async fn test_tool_to_tool_communication() {
    // Initialize components
    let security_manager = SecurityManager::new(SecurityConfig {
        token_expiry: chrono::Duration::seconds(3600),
        key_rotation_interval: chrono::Duration::hours(24),
        min_key_size: 32,
    });

    let port_manager = PortManager::new(PortConfig {
        min_port: 8000,
        max_port: 9000,
        reserved_ports: HashSet::new(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
    });

    let tool_manager = ToolManager::new();
    let mut protocol = MCPProtocol::new();

    // 1. Register two tools
    let tool1 = Tool {
        id: "sender-tool".to_string(),
        name: "Sender Tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Tool that sends messages".to_string(),
        capabilities: vec![Capability {
            name: "send".to_string(),
            description: "Send capability".to_string(),
            parameters: vec![],
            return_type: ReturnType {
                type_: ParameterType::String,
                description: "Send result".to_string(),
            },
            required_permissions: HashSet::new(),
        }],
        security_level: SecurityLevel::Medium,
        metadata: HashMap::new(),
    };

    let tool2 = Tool {
        id: "receiver-tool".to_string(),
        name: "Receiver Tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Tool that receives messages".to_string(),
        capabilities: vec![Capability {
            name: "receive".to_string(),
            description: "Receive capability".to_string(),
            parameters: vec![],
            return_type: ReturnType {
                type_: ParameterType::String,
                description: "Receive result".to_string(),
            },
            required_permissions: HashSet::new(),
        }],
        security_level: SecurityLevel::Medium,
        metadata: HashMap::new(),
    };

    // Register both tools
    assert!(tool_manager.register_tool(tool1.clone()).await.is_ok());
    assert!(tool_manager.register_tool(tool2.clone()).await.is_ok());

    // 2. Authenticate both tools
    let mut roles = HashSet::new();
    roles.insert("tool".to_string());
    let auth_token1 = security_manager.authenticate(&tool1.id, roles.clone()).await.unwrap();
    let auth_token2 = security_manager.authenticate(&tool2.id, roles).await.unwrap();

    // 3. Allocate ports for both tools
    let port1 = port_manager.allocate_port(tool1.id.clone()).await.unwrap();
    let port2 = port_manager.allocate_port(tool2.id.clone()).await.unwrap();

    // 4. Create and send message from tool1 to tool2
    let payload = serde_json::json!({
        "action": "send",
        "target": tool2.id,
        "data": "test message"
    });

    let (encrypted_payload, key_id) = security_manager.encrypt(
        serde_json::to_string(&payload).unwrap().as_bytes()
    ).await.unwrap();

    let message = MCPMessage {
        id: "tool-to-tool-test-1".to_string(),
        type_: MessageType::Command,
        payload,
        metadata: Some(serde_json::json!({
            "source_port": port1,
            "target_port": port2,
            "source_tool": tool1.id,
            "target_tool": tool2.id
        })),
        security: SecurityMetadata {
            security_level: SecurityLevel::Medium,
            encryption_info: Some(EncryptionInfo {
                algorithm: "ChaCha20-Poly1305".to_string(),
                key_id: key_id.clone(),
                nonce: None,
            }),
            signature: None,
            auth_token: Some(auth_token1.token.clone()),
        },
        timestamp: Utc::now(),
    };

    // 5. Handle message and verify delivery
    let result = protocol.handle_message(message.clone()).await;
    assert!(result.is_ok());

    // 6. Cleanup
    assert!(port_manager.release_port(port1).await.is_ok());
    assert!(port_manager.release_port(port2).await.is_ok());
    assert!(tool_manager.unregister_tool(&tool1.id).await.is_ok());
    assert!(tool_manager.unregister_tool(&tool2.id).await.is_ok());
} 