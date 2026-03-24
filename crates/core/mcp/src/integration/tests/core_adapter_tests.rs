// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the Core MCP Adapter.

use super::*;
use crate::integration::types::{CoreState, User}; // Use types from the new module
use crate::integration::auth::AuthManager;      // Use AuthManager from the new module
use crate::integration::adapter::CoreMCPAdapter; // Use CoreMCPAdapter from the new module
use crate::types::{MCPMessage, MCPResponse, MessageType, ResponseStatus, SecurityMetadata, ProtocolState, MessageMetadata}; // Keep necessary types
use crate::protocol::{MessageId, ProtocolVersion as ProtocolTypesProtocolVersion}; // Keep necessary protocol types
use crate::config::CoreAdapterConfig;
use crate::plugin::manager::PluginManager;
use crate::metrics::MetricsCollector;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::json;
use mockall::predicate::*;
use mockall::mock;
use async_trait::async_trait;

// Mock MCP Protocol
mock! {
    pub MCPProtocol {}
    
    #[async_trait]
    impl crate::protocol::MCPProtocol for MCPProtocol {
        async fn handle_message(&self, msg: crate::types::MCPMessage) -> crate::protocol::ProtocolResult;
        async fn validate_message(&self, msg: &crate::types::MCPMessage) -> crate::protocol::ValidationResult;
        async fn route_message(&self, msg: &crate::types::MCPMessage) -> crate::protocol::RoutingResult;
        async fn set_state(&self, new_state: crate::types::ProtocolState) -> crate::error::Result<()>;
        async fn get_state(&self) -> crate::error::Result<crate::types::ProtocolState>;
        fn get_version(&self) -> String;
    }
}

#[tokio::test]
async fn test_core_adapter_initialization() {
    // Create mocks
    let mut mock_mcp = MockMCPProtocol::new();
    let core_state = Arc::new(RwLock::new(CoreState::default()));
    let auth_manager = Arc::new(AuthManager::new_test());
    let metrics = Arc::new(crate::metrics::MetricsCollector::new_test());
    
    // Create adapter
    let adapter = CoreMCPAdapter::new(
        core_state,
        Arc::new(mock_mcp),
        auth_manager,
        metrics,
        CoreAdapterConfig::default(),
        Arc::new(PluginManager::new()),
    );
    
    // Test initialization
    let result = adapter.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_state_query() {
    // Create test state
    let mut initial_state = CoreState::default();
    initial_state.version = "1.0.0".to_string();
    initial_state.status = "active".to_string();
    
    // Create components
    let core_state = Arc::new(RwLock::new(initial_state));
    let mock_mcp = Arc::new(MockMCPProtocol::new());
    let auth_manager = Arc::new(AuthManager::new_test());
    let metrics = Arc::new(crate::metrics::MetricsCollector::new_test());
    
    // Create adapter
    let adapter = CoreMCPAdapter::new(
        core_state,
        mock_mcp,
        auth_manager,
        metrics,
        CoreAdapterConfig::default(),
        Arc::new(PluginManager::new()),
    );
    
    // Create test message
    let message = crate::types::MCPMessage {
        id: MessageId("test-id".to_string()),
        type_: MessageType::Sync,
        payload: json!({}),
        metadata: Some(serde_json::Value::Null),
        security: SecurityMetadata::default(),
        timestamp: chrono::Utc::now(),
        version: ProtocolTypesProtocolVersion::new(1, 0),
        trace_id: None,
    };
    
    // Handle message
    let response = adapter.handle_message(message).await.expect("should succeed");
    
    // Verify response
    assert_eq!(response.status, ResponseStatus::Success);
    
    // Parse payload back to JSON for verification
    let payload_json: serde_json::Value = serde_json::from_slice(&response.payload).expect("should succeed");
    assert_eq!(payload_json.get("version").expect("should succeed"), "1.0.0");
    assert_eq!(payload_json.get("status").expect("should succeed"), "active");
} 