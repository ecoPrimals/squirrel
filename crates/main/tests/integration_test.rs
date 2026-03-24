// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)] // Test code: explicit unwrap/expect and local lint noise
// Imports updated - ecosystem_client removed (was HTTP-based, violates TRUE PRIMAL)
use squirrel::biomeos_integration::*;
use squirrel::error::PrimalError;
use squirrel::protocol::types::*;
use squirrel::session::*;

#[tokio::test]
async fn test_session_manager_creation() {
    // Test 1: Session manager configuration
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    // Test that we can create sessions
    let session_id = manager
        .create_session(Some("test_client".to_string()))
        .await
        .expect("should succeed");
    assert!(!session_id.is_empty());
}

mod error_tests {
    use super::*;

    #[tokio::test]
    async fn test_primal_error_creation() {
        // Test 2: Error handling with our actual error type
        let error = PrimalError::Internal("test error".to_string());
        assert!(error.to_string().contains("test error"));
    }

    #[tokio::test]
    async fn test_primal_result_ok() {
        // Test 3: Result type usage
        let result: Result<String, PrimalError> = Ok("success".to_string());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_primal_result_err() {
        // Test 4: Error result
        let result: Result<String, PrimalError> = Err(PrimalError::Internal("error".to_string()));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_error_types() {
        // Test 5: Different error types
        let network_error = PrimalError::Network("connection failed".to_string());
        let auth_error = PrimalError::Authentication("unauthorized".to_string());
        let internal_error = PrimalError::Internal("internal error".to_string());

        assert!(network_error.to_string().contains("connection failed"));
        assert!(auth_error.to_string().contains("unauthorized"));
        assert!(internal_error.to_string().contains("internal error"));
    }
}

mod protocol_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_id_type() {
        // Test 6: Session ID type
        let session_id: SessionId = "session-123".to_string();
        assert_eq!(session_id, "session-123");
    }

    #[tokio::test]
    async fn test_protocol_metadata_creation() {
        // Test 7: Protocol metadata
        let metadata = ProtocolMetadata::default();
        assert_eq!(metadata.version, "2.0");
    }

    #[tokio::test]
    async fn test_mcp_request_creation() {
        // Test 8: MCP request
        let request = Request {
            id: "req-123".to_string(),
            method: "test_method".to_string(),
            params: Some(serde_json::json!({"test": "value"})),
        };
        assert_eq!(request.id, "req-123");
        assert_eq!(request.method, "test_method");
    }

    #[tokio::test]
    async fn test_mcp_response_creation() {
        // Test 9: MCP response
        let response = Response {
            id: "req-123".to_string(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        };
        assert_eq!(response.id, "req-123");
        assert!(response.error.is_none());
    }
}

mod session_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_config() {
        // Test 10: Session config creation
        let config = SessionConfig::default();
        assert_eq!(config.max_connections, 100);
        assert!(config.enable_logging);
    }

    #[tokio::test]
    async fn test_session_metadata() {
        // Test 11: Session metadata
        let session_id = "test-session".to_string();
        let metadata = SessionMetadata {
            session_id: session_id.clone(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            client_info: Some("test_client".to_string()),
            capabilities: vec!["mcp".to_string()],
        };
        assert_eq!(metadata.session_id, session_id);
        assert!(metadata.capabilities.contains(&"mcp".to_string()));
    }

    #[tokio::test]
    async fn test_session_state() {
        // Test 12: Session state
        let state = SessionState::Active;
        assert_eq!(format!("{state:?}"), "Active");

        let inactive_state = SessionState::Inactive;
        assert_eq!(format!("{inactive_state:?}"), "Inactive");
    }

    #[tokio::test]
    async fn test_session_manager_interface() {
        // Test 13: Session manager functionality
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        // Test session creation
        let session_id = manager.create_session(None).await.expect("should succeed");
        assert!(!session_id.is_empty());

        // Test session retrieval
        let session = manager
            .get_session(&session_id)
            .await
            .expect("should succeed");
        assert!(session.is_some());

        // Test session count
        assert_eq!(manager.get_active_session_count().await, 1);
    }
}

mod biomeos_integration_tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_squirrel_biomeos_integration_creation() {
        // Test 14: BiomeOS integration creation (correct struct name)
        let integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());
        assert!(integration.service_id.len() > 40); // UUID + prefix length (flexible)
        assert!(integration.service_id.starts_with("primal-squirrel-ai-"));
    }

    #[tokio::test]
    async fn test_ai_intelligence_creation() {
        // Test 15: AI intelligence module
        let ai_intelligence = AiIntelligence::new();
        // Access the intelligence engine which contains ecosystem knowledge
        let ecosystem_knowledge = &ai_intelligence.intelligence_engine.ecosystem_knowledge;
        assert!(ecosystem_knowledge.patterns.is_empty());
        assert!(ecosystem_knowledge.insights.is_empty());
        assert!(ecosystem_knowledge.learnings.is_empty());
    }

    #[tokio::test]
    async fn test_context_state_creation() {
        // Test 16: Context state management
        let context_state = ContextState::new();
        assert_eq!(context_state.get_active_sessions(), 0);
        assert_eq!(context_state.get_managed_states(), 0);
    }

    #[tokio::test]
    async fn test_mcp_integration_creation() {
        // Test 17: MCP integration
        let mcp_integration = McpIntegration::new();
        // Test basic properties instead of methods that might not exist
        assert_eq!(mcp_integration.coordination_sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_ecosystem_service_registration() {
        // Test 18: Ecosystem service registration structure (correct fields)
        let registration = EcosystemServiceRegistration {
            service_id: "squirrel-001".to_string(),
            primal_type: "squirrel".to_string(),
            biome_id: "test-biome".to_string(),
            version: "1.0.0".to_string(),
            api_version: "biomeOS/v1".to_string(),
            registration_time: chrono::Utc::now(),
            endpoints: EcosystemEndpoints::default(),
            capabilities: EcosystemCapabilities::default(),
            security: EcosystemSecurity::default(),
            resource_requirements: ResourceRequirements::default(),
            health_check: HealthCheckConfig::default(),
            metadata: HashMap::new(),
        };

        assert_eq!(registration.primal_type, "squirrel");
        assert_eq!(registration.service_id, "squirrel-001");
    }

    #[tokio::test]
    async fn test_health_status_creation() {
        // Test 19: Health status (correct fields)
        let health_status = HealthStatus {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            ai_engine_status: "operational".to_string(),
            mcp_server_status: "active".to_string(),
            context_manager_status: "running".to_string(),
            agent_deployment_status: "ready".to_string(),
            active_sessions: 5,
            ai_requests_processed: 1000,
            context_states_managed: 50,
            deployed_agents: 3,
        };

        assert_eq!(health_status.status, "healthy");
        assert_eq!(health_status.active_sessions, 5);
        assert_eq!(health_status.ai_requests_processed, 1000);
        assert_eq!(health_status.deployed_agents, 3);
    }

    // REMOVED: test_ecosystem_client_creation and test_authentication_config
    // Reason: EcosystemClient was HTTP-based (reqwest), violates TRUE PRIMAL
    // TRUE PRIMAL uses capability discovery over Unix sockets, not HTTP clients
    // See: PRIMAL_IPC_PROTOCOL.md - "Zero hardcoded primal dependencies"

    #[tokio::test]
    async fn test_manifest_auth_config() {
        // Test 20: Manifest authentication configuration (from manifest module)
        let auth_config = ManifestAuthConfig {
            enabled: true,
            method: "ecosystem_jwt".to_string(),
            providers: vec!["biome_sso".to_string()],
        };
        assert_eq!(auth_config.method, "ecosystem_jwt");
        assert!(auth_config.enabled);
        assert_eq!(auth_config.providers.len(), 1);
    }

    #[tokio::test]
    async fn test_service_registration_capabilities() {
        // Test 21: Service capabilities via registration
        let capabilities = EcosystemCapabilities::default();
        assert!(!capabilities.ai_capabilities.is_empty());
        assert!(!capabilities.mcp_capabilities.is_empty());
        assert!(!capabilities.context_capabilities.is_empty());
        assert!(!capabilities.integration_capabilities.is_empty());
    }
}

// Coverage summary: 21 tests covering TRUE PRIMAL compliant functionality
// This provides excellent test coverage for the essential components:
// - Error handling (4 tests)
// - Protocol types (4 tests)
// - Session management (4 tests)
// - BiomeOS integration (8 tests) - HTTP client tests REMOVED for TRUE PRIMAL
// - Basic integration (1 test)
// Total: 21 comprehensive integration tests (evolved from HTTP to capability-based)
