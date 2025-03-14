use std::collections::{HashMap, HashSet};
use std::time::Duration;
use chrono::Utc;
use tokio;

use crate::mcp::{
    MCPMessage, MessageType, SecurityMetadata, SecurityLevel, EncryptionInfo,
    MCPProtocol, PortManager, PortConfig, SecurityManager, SecurityConfig,
    Tool, ToolManager, Capability, Parameter, ParameterType, ReturnType,
    ToolStatus, ErrorHandler,
};

/// Test metrics for validation
const COMPLETENESS_THRESHOLD: f64 = 0.8;
const ACCURACY_THRESHOLD: f64 = 0.9;
const CLARITY_THRESHOLD: f64 = 0.85;

/// Code Understanding Test Suite
mod code_understanding_tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_validation() {
        let tool_manager = ToolManager::new();
        let mut metrics = ValidationMetrics::new();

        // Test invalid tool registration (empty ID)
        let invalid_tool = Tool {
            id: "".to_string(),
            name: "Invalid Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A tool with invalid configuration".to_string(),
            capabilities: vec![],
            security_level: SecurityLevel::Low,
            metadata: HashMap::new(),
        };

        let result = tool_manager.register_tool(invalid_tool.clone()).await;
        metrics.record_validation("empty_id_check", result.is_err());

        // Test invalid tool registration (no capabilities)
        let invalid_tool = Tool {
            id: "test-tool".to_string(),
            name: "Invalid Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A tool with no capabilities".to_string(),
            capabilities: vec![],
            security_level: SecurityLevel::Low,
            metadata: HashMap::new(),
        };

        let result = tool_manager.register_tool(invalid_tool.clone()).await;
        metrics.record_validation("no_capabilities_check", result.is_err());

        assert!(metrics.completeness() >= COMPLETENESS_THRESHOLD, 
            "Code understanding completeness below threshold");
    }
}

/// Project Analysis Test Suite
mod project_analysis_tests {
    use super::*;

    #[tokio::test]
    async fn test_protocol_version_validation() {
        let mut protocol = MCPProtocol::new();
        let mut metrics = ValidationMetrics::new();

        // Test incompatible version
        let invalid_message = MCPMessage {
            id: "test-version".to_string(),
            type_: MessageType::Command,
            version: "999.0.0".to_string(),
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

        let result = protocol.handle_message(invalid_message.clone()).await;
        metrics.record_validation("version_compatibility", result.is_err());

        assert!(metrics.accuracy() >= ACCURACY_THRESHOLD,
            "Project analysis accuracy below threshold");
    }

    #[tokio::test]
    async fn test_concurrent_validation() {
        let tool_manager = ToolManager::new();
        let mut metrics = ValidationMetrics::new();
        let tool_id = "concurrent-test-tool".to_string();

        // Create and register tool
        let tool = Tool {
            id: tool_id.clone(),
            name: "Concurrent Test Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "Tool for testing concurrent operations".to_string(),
            capabilities: vec![Capability {
                name: "test".to_string(),
                description: "Test capability".to_string(),
                parameters: vec![],
                return_type: ReturnType {
                    type_: ParameterType::String,
                    description: "Test return".to_string(),
                },
                required_permissions: HashSet::new(),
            }],
            security_level: SecurityLevel::Low,
            metadata: HashMap::new(),
        };

        metrics.record_validation("tool_registration", 
            tool_manager.register_tool(tool.clone()).await.is_ok());

        // Test concurrent state updates
        let mut handles = vec![];
        for status in [ToolStatus::Active, ToolStatus::Inactive, ToolStatus::Error] {
            let tool_manager = tool_manager.clone();
            let tool_id = tool_id.clone();
            handles.push(tokio::spawn(async move {
                tool_manager.update_tool_state(&tool_id, status).await
            }));
        }

        let mut concurrent_success = true;
        for handle in handles {
            if !handle.await.unwrap().is_ok() {
                concurrent_success = false;
            }
        }
        metrics.record_validation("concurrent_updates", concurrent_success);

        assert!(metrics.accuracy() >= ACCURACY_THRESHOLD,
            "Project analysis accuracy below threshold");
    }
}

/// Error Resolution Test Suite
mod error_resolution_tests {
    use super::*;

    #[tokio::test]
    async fn test_error_validation() {
        let error_handler = ErrorHandler::new(100);
        let mut metrics = ValidationMetrics::new();

        // Test invalid error recovery handler
        let result = error_handler.register_recovery_handler("", Box::new(|_| {
            Ok(())
        })).await;
        metrics.record_validation("empty_handler_check", result.is_err());

        // Test error history validation
        for i in 0..101 {
            let _ = error_handler.handle_error(
                crate::mcp::MCPError::Protocol(
                    crate::mcp::error::ProtocolError::InvalidMessage(
                        format!("test error {}", i)
                    )
                ),
                "test-source".to_string()
            ).await;
        }

        let history = error_handler.get_error_history().await.unwrap();
        metrics.record_validation("history_size_check", history.len() == 100);

        assert!(metrics.clarity() >= CLARITY_THRESHOLD,
            "Error resolution clarity below threshold");
    }

    #[tokio::test]
    async fn test_cleanup_validation() {
        let tool_manager = ToolManager::new();
        let port_manager = PortManager::new(PortConfig {
            min_port: 8000,
            max_port: 9000,
            reserved_ports: HashSet::new(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
        });
        let mut metrics = ValidationMetrics::new();

        // Create and register tool
        let tool = Tool {
            id: "cleanup-test-tool".to_string(),
            name: "Cleanup Test Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "Tool for testing cleanup operations".to_string(),
            capabilities: vec![Capability {
                name: "test".to_string(),
                description: "Test capability".to_string(),
                parameters: vec![],
                return_type: ReturnType {
                    type_: ParameterType::String,
                    description: "Test return".to_string(),
                },
                required_permissions: HashSet::new(),
            }],
            security_level: SecurityLevel::Low,
            metadata: HashMap::new(),
        };

        metrics.record_validation("tool_registration",
            tool_manager.register_tool(tool.clone()).await.is_ok());

        // Allocate port and test cleanup
        if let Ok(port) = port_manager.allocate_port("cleanup-test-tool".to_string()).await {
            metrics.record_validation("port_allocation", true);
            
            // Test cleanup
            let cleanup_success = tool_manager.unregister_tool(&tool.id).await.is_ok();
            metrics.record_validation("cleanup_execution", cleanup_success);

            // Verify port release
            let port_released = port_manager.get_metrics(port).await.is_err();
            metrics.record_validation("port_release", port_released);
        }

        assert!(metrics.clarity() >= CLARITY_THRESHOLD,
            "Error resolution clarity below threshold");
    }
}

/// Validation metrics tracking
struct ValidationMetrics {
    results: Vec<bool>,
}

impl ValidationMetrics {
    fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    fn record_validation(&mut self, _name: &str, result: bool) {
        self.results.push(result);
    }

    fn completeness(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.results.iter().filter(|&&x| x).count() as f64 / self.results.len() as f64
    }

    fn accuracy(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.results.iter().filter(|&&x| x).count() as f64 / self.results.len() as f64
    }

    fn clarity(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.results.iter().filter(|&&x| x).count() as f64 / self.results.len() as f64
    }
}

#[tokio::test]
async fn test_port_validation() {
    let port_manager = PortManager::new(PortConfig {
        min_port: 8000,
        max_port: 9000,
        reserved_ports: HashSet::new(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
    });
    let mut metrics = ValidationMetrics::new();

    // Test port allocation
    let port = port_manager.allocate_port("test-tool".to_string()).await;
    metrics.record_validation("port_allocation", port.is_ok());

    // Test port reuse
    if let Ok(port) = port {
        let reuse_result = port_manager.allocate_port("test-tool".to_string()).await;
        metrics.record_validation("port_reuse", reuse_result.is_err());
    }

    assert!(metrics.accuracy() >= ACCURACY_THRESHOLD,
        "Port validation accuracy below threshold");
}

#[tokio::test]
async fn test_message_validation() {
    let mut protocol = MCPProtocol::new();
    let mut metrics = ValidationMetrics::new();

    // Test valid message
    let valid_message = MCPMessage {
        id: "test-valid".to_string(),
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
        timestamp: Utc::now(),
    };

    let result = protocol.handle_message(valid_message.clone()).await;
    metrics.record_validation("valid_message", result.is_ok());

    // Test invalid message (missing required fields)
    let invalid_message = MCPMessage {
        id: "".to_string(),
        type_: MessageType::Command,
        version: "1.0.0".to_string(),
        payload: serde_json::json!({}),
        metadata: None,
        security: SecurityMetadata {
            security_level: SecurityLevel::Low,
            encryption_info: None,
            signature: None,
            auth_token: None,
        },
        timestamp: Utc::now(),
    };

    let result = protocol.handle_message(invalid_message.clone()).await;
    metrics.record_validation("invalid_message", result.is_err());

    assert!(metrics.accuracy() >= ACCURACY_THRESHOLD,
        "Message validation accuracy below threshold");
}

#[tokio::test]
async fn test_security_validation() {
    let security_manager = SecurityManager::new(SecurityConfig {
        min_security_level: SecurityLevel::Medium,
        require_encryption: true,
        require_signature: true,
        require_auth: true,
    });
    let mut metrics = ValidationMetrics::new();

    // Test security level validation
    let result = security_manager.validate_security_level(SecurityLevel::Low);
    metrics.record_validation("security_level", result.is_err());

    // Test encryption validation
    let result = security_manager.validate_encryption(None);
    metrics.record_validation("encryption", result.is_err());

    // Test signature validation
    let result = security_manager.validate_signature(None);
    metrics.record_validation("signature", result.is_err());

    // Test auth validation
    let result = security_manager.validate_auth(None);
    metrics.record_validation("auth", result.is_err());

    assert!(metrics.accuracy() >= ACCURACY_THRESHOLD,
        "Security validation accuracy below threshold");
}

#[tokio::test]
async fn test_state_transition_validation() {
    let mut state_manager = StateManager::new();
    let mut metrics = ValidationMetrics::new();

    // Test valid state transition
    let result = state_manager.transition_to(State::Initializing).await;
    metrics.record_validation("valid_transition", result.is_ok());

    // Test invalid state transition
    let result = state_manager.transition_to(State::Error).await;
    metrics.record_validation("invalid_transition", result.is_err());

    assert!(metrics.accuracy() >= ACCURACY_THRESHOLD,
        "State transition validation accuracy below threshold");
} 