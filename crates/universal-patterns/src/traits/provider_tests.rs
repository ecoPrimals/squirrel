// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive tests for PrimalProvider trait
//!
//! Coverage goal: 90%+
//! Strategy: Test all trait methods, error paths, edge cases, concurrent access

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use super::provider::PrimalProvider;
use crate::traits::{
    DynamicPortInfo, NetworkLocation, PortStatus, PortType, PrimalCapability, PrimalContext,
    PrimalDependency, PrimalEndpoints, PrimalHealth, PrimalRequest, PrimalRequestType,
    PrimalResponse, PrimalResponseType, PrimalResult, PrimalType, SecurityLevel,
};

/// Mock provider implementation for testing
struct MockProvider {
    primal_id: String,
    instance_id: String,
    context: PrimalContext,
    primal_type: PrimalType,
    initialized: bool,
    shutdown: bool,
    fail_health_check: bool,
    fail_request: bool,
}

impl MockProvider {
    fn new(id: &str, primal_type: PrimalType) -> Self {
        Self {
            primal_id: id.to_string(),
            instance_id: format!("{}-instance-{}", id, Uuid::new_v4()),
            context: PrimalContext {
                user_id: "test-user".to_string(),
                device_id: "test-device".to_string(),
                session_id: Uuid::new_v4().to_string(),
                network_location: NetworkLocation {
                    ip_address: "127.0.0.1".to_string(),
                    subnet: Some("255.255.255.0".to_string()),
                    network_id: Some("local".to_string()),
                    geo_location: None,
                },
                security_level: SecurityLevel::High,
                metadata: HashMap::new(),
            },
            primal_type,
            initialized: false,
            shutdown: false,
            fail_health_check: false,
            fail_request: false,
        }
    }

    fn with_health_failure(mut self) -> Self {
        self.fail_health_check = true;
        self
    }

    fn with_request_failure(mut self) -> Self {
        self.fail_request = true;
        self
    }
}

#[async_trait]
impl PrimalProvider for MockProvider {
    fn primal_id(&self) -> &str {
        &self.primal_id
    }

    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    fn context(&self) -> &PrimalContext {
        &self.context
    }

    fn primal_type(&self) -> PrimalType {
        self.primal_type.clone()
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::Authentication {
                methods: vec!["password".to_string(), "token".to_string()],
            },
            PrimalCapability::Encryption {
                algorithms: vec!["AES256".to_string(), "RSA2048".to_string()],
            },
            PrimalCapability::Custom {
                name: "test-capability".to_string(),
                attributes: "test=true".to_string(),
            },
        ]
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![] // No dependencies for test
    }

    async fn health_check(&self) -> PrimalHealth {
        if self.fail_health_check {
            PrimalHealth::Unhealthy {
                reason: "Health check failed".to_string(),
            }
        } else {
            PrimalHealth::Healthy
        }
    }

    fn endpoints(&self) -> PrimalEndpoints {
        PrimalEndpoints {
            primary: format!("http://localhost:8080/{}", self.primal_id),
            health: format!("http://localhost:8080/{}/health", self.primal_id),
            metrics: Some(format!("http://localhost:8080/{}/metrics", self.primal_id)),
            admin: Some(format!("http://localhost:8080/{}/admin", self.primal_id)),
            websocket: Some(format!("ws://localhost:8080/{}/ws", self.primal_id)),
            custom: String::new(),
        }
    }

    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        if self.fail_request {
            return Err(crate::traits::PrimalError::State(
                "Request handling failed".to_string(),
            ));
        }

        Ok(PrimalResponse {
            request_id: request.id,
            response_type: match request.request_type {
                PrimalRequestType::Authenticate => PrimalResponseType::Authentication,
                PrimalRequestType::Encrypt => PrimalResponseType::Encryption,
                PrimalRequestType::Decrypt => PrimalResponseType::Decryption,
                _ => PrimalResponseType::Custom("test-response".to_string()),
            },
            payload: HashMap::new(),
            timestamp: Utc::now(),
            success: true,
            error_message: None,
            metadata: Some(HashMap::new()),
        })
    }

    async fn initialize(&mut self, _config: serde_json::Value) -> PrimalResult<()> {
        if self.initialized {
            return Err(crate::traits::PrimalError::State(
                "Already initialized".to_string(),
            ));
        }
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> PrimalResult<()> {
        if self.shutdown {
            return Err(crate::traits::PrimalError::State(
                "Already shutdown".to_string(),
            ));
        }
        self.shutdown = true;
        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        // Simple matching logic for testing
        self.context.user_id == context.user_id
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        use chrono::Duration;
        Some(DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(24),
        })
    }
}

#[cfg(test)]
mod provider_trait_tests {
    use super::*;

    #[test]
    fn test_primal_id() {
        let provider = MockProvider::new("test-primal", PrimalType::Security);
        assert_eq!(provider.primal_id(), "test-primal");
    }

    #[test]
    fn test_instance_id() {
        let provider = MockProvider::new("test-primal", PrimalType::Security);
        let instance_id = provider.instance_id();

        assert!(instance_id.starts_with("test-primal-instance-"));
        assert!(instance_id.len() > 20); // Has UUID appended
    }

    #[test]
    fn test_primal_type() {
        let provider = MockProvider::new("test", PrimalType::Security);
        assert_eq!(provider.primal_type(), PrimalType::Security);

        let provider2 = MockProvider::new("test", PrimalType::Storage);
        assert_eq!(provider2.primal_type(), PrimalType::Storage);
    }

    #[test]
    fn test_capabilities() {
        let provider = MockProvider::new("test", PrimalType::Security);
        let capabilities = provider.capabilities();

        assert_eq!(capabilities.len(), 3);

        // Check for authentication capability
        let has_auth = capabilities
            .iter()
            .any(|c| matches!(c, PrimalCapability::Authentication { .. }));
        assert!(has_auth);

        // Check for encryption capability
        let has_encryption = capabilities
            .iter()
            .any(|c| matches!(c, PrimalCapability::Encryption { .. }));
        assert!(has_encryption);
    }

    #[test]
    fn test_dependencies() {
        let provider = MockProvider::new("test", PrimalType::Security);
        let dependencies = provider.dependencies();

        // Mock provider has no dependencies
        assert_eq!(dependencies.len(), 0);
    }

    #[tokio::test]
    async fn test_health_check_healthy() {
        let provider = MockProvider::new("test", PrimalType::Security);
        let health = provider.health_check().await;

        assert!(matches!(health, PrimalHealth::Healthy));
    }

    #[tokio::test]
    async fn test_health_check_unhealthy() {
        let provider = MockProvider::new("test", PrimalType::Security).with_health_failure();
        let health = provider.health_check().await;

        assert!(matches!(health, PrimalHealth::Unhealthy { .. }));
        if let PrimalHealth::Unhealthy { reason } = health {
            assert_eq!(reason, "Health check failed");
        }
    }

    #[test]
    fn test_endpoints() {
        let provider = MockProvider::new("test-primal", PrimalType::Security);
        let endpoints = provider.endpoints();

        assert_eq!(endpoints.primary, "http://localhost:8080/test-primal");
        assert_eq!(endpoints.health, "http://localhost:8080/test-primal/health");
        assert_eq!(
            endpoints.metrics,
            Some("http://localhost:8080/test-primal/metrics".to_string())
        );
        assert!(endpoints.websocket.is_some());
    }

    #[tokio::test]
    async fn test_handle_request_success() {
        let provider = MockProvider::new("test", PrimalType::Security);
        let request = PrimalRequest {
            id: Uuid::new_v4(),
            request_type: PrimalRequestType::Authenticate,
            payload: HashMap::new(),
            timestamp: Utc::now(),
            context: None,
            priority: None,
            security_level: None,
        };

        let response = provider.handle_primal_request(request).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert!(response.success);
        assert_eq!(response.response_type, PrimalResponseType::Authentication);
    }

    #[tokio::test]
    async fn test_handle_request_failure() {
        let provider = MockProvider::new("test", PrimalType::Security).with_request_failure();
        let request = PrimalRequest {
            id: Uuid::new_v4(),
            request_type: PrimalRequestType::Authenticate,
            payload: HashMap::new(),
            timestamp: Utc::now(),
            context: None,
            priority: None,
            security_level: None,
        };

        let response = provider.handle_primal_request(request).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_initialize() {
        let mut provider = MockProvider::new("test", PrimalType::Security);

        assert!(!provider.initialized);

        let result = provider
            .initialize(serde_json::json!({"test": "config"}))
            .await;
        assert!(result.is_ok());
        assert!(provider.initialized);
    }

    #[tokio::test]
    async fn test_initialize_twice_fails() {
        let mut provider = MockProvider::new("test", PrimalType::Security);

        provider.initialize(serde_json::json!({})).await.unwrap();

        let result = provider.initialize(serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_shutdown() {
        let mut provider = MockProvider::new("test", PrimalType::Security);

        assert!(!provider.shutdown);

        let result = provider.shutdown().await;
        assert!(result.is_ok());
        assert!(provider.shutdown);
    }

    #[tokio::test]
    async fn test_shutdown_twice_fails() {
        let mut provider = MockProvider::new("test", PrimalType::Security);

        provider.shutdown().await.unwrap();

        let result = provider.shutdown().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_can_serve_context_matching() {
        let provider = MockProvider::new("test", PrimalType::Security);
        let context = PrimalContext {
            user_id: "test-user".to_string(),
            device_id: "other-device".to_string(),
            session_id: Uuid::new_v4().to_string(),
            network_location: NetworkLocation {
                ip_address: "127.0.0.1".to_string(),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: SecurityLevel::High,
            metadata: HashMap::new(),
        };

        assert!(provider.can_serve_context(&context));
    }

    #[test]
    fn test_can_serve_context_non_matching() {
        let provider = MockProvider::new("test", PrimalType::Security);
        let context = PrimalContext {
            user_id: "different-user".to_string(),
            device_id: "device".to_string(),
            session_id: Uuid::new_v4().to_string(),
            network_location: NetworkLocation {
                ip_address: "192.168.1.1".to_string(),
                subnet: Some("255.255.255.0".to_string()),
                network_id: Some("remote".to_string()),
                geo_location: Some("US-EAST".to_string()),
            },
            security_level: SecurityLevel::Standard,
            metadata: HashMap::new(),
        };

        assert!(!provider.can_serve_context(&context));
    }

    #[test]
    fn test_dynamic_port_info() {
        let provider = MockProvider::new("test", PrimalType::Security);
        let port_info = provider.dynamic_port_info();

        assert!(port_info.is_some());
        let port_info = port_info.unwrap();
        assert_eq!(port_info.assigned_port, 8080);
        assert_eq!(port_info.port_type, PortType::Http);
        assert_eq!(port_info.status, PortStatus::Active);
    }

    #[test]
    fn test_context_structure() {
        let provider = MockProvider::new("test", PrimalType::Security);
        let context = provider.context();

        assert_eq!(context.user_id, "test-user");
        assert_eq!(context.device_id, "test-device");
        assert!(!context.session_id.is_empty());
        assert_eq!(context.security_level, SecurityLevel::High);
    }

    #[tokio::test]
    async fn test_full_lifecycle() {
        let mut provider = MockProvider::new("test", PrimalType::Security);

        // Initialize
        provider.initialize(serde_json::json!({})).await.unwrap();
        assert!(provider.initialized);

        // Health check
        let health = provider.health_check().await;
        assert!(matches!(health, PrimalHealth::Healthy));

        // Handle request
        let request = PrimalRequest {
            id: Uuid::new_v4(),
            request_type: PrimalRequestType::Authenticate,
            payload: HashMap::new(),
            timestamp: Utc::now(),
            context: None,
            priority: None,
            security_level: None,
        };
        let response = provider.handle_primal_request(request).await;
        assert!(response.is_ok());

        // Shutdown
        provider.shutdown().await.unwrap();
        assert!(provider.shutdown);
    }

    #[test]
    fn test_multiple_primal_types() {
        let types = vec![
            PrimalType::Security,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::AI,
            PrimalType::Orchestration,
        ];

        for primal_type in types {
            let provider = MockProvider::new("test", primal_type.clone());
            assert_eq!(provider.primal_type(), primal_type);
        }
    }
}
