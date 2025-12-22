//! Comprehensive tests for the traits module

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    // ==================== Display Tests ====================

    #[test]
    fn test_primal_state_display() {
        assert_eq!(PrimalState::Initializing.to_string(), "Initializing");
        assert_eq!(PrimalState::Starting.to_string(), "Starting");
        assert_eq!(PrimalState::Running.to_string(), "Running");
        assert_eq!(PrimalState::Stopping.to_string(), "Stopping");
        assert_eq!(PrimalState::Stopped.to_string(), "Stopped");
        assert_eq!(PrimalState::Restarting.to_string(), "Restarting");
        assert_eq!(PrimalState::Maintenance.to_string(), "Maintenance");
        assert_eq!(
            PrimalState::Error("test error".to_string()).to_string(),
            "Error: test error"
        );
    }

    #[test]
    fn test_primal_type_display() {
        assert_eq!(PrimalType::Coordinator.to_string(), "Coordinator");
        assert_eq!(PrimalType::Security.to_string(), "Security");
        assert_eq!(PrimalType::Orchestration.to_string(), "Orchestration");
        assert_eq!(PrimalType::Storage.to_string(), "Storage");
        assert_eq!(PrimalType::Compute.to_string(), "Compute");
        assert_eq!(PrimalType::AI.to_string(), "AI");
        assert_eq!(PrimalType::Network.to_string(), "Network");
        assert_eq!(
            PrimalType::Custom("MyType".to_string()).to_string(),
            "Custom(MyType)"
        );
    }

    // ==================== Default Tests ====================

    #[test]
    fn test_primal_endpoints_default() {
        let endpoints = PrimalEndpoints::default();

        assert_eq!(endpoints.primary, "http://localhost:8080");
        assert_eq!(endpoints.health, "http://localhost:8080/health");
        assert_eq!(endpoints.metrics, None);
        assert_eq!(endpoints.admin, None);
        assert_eq!(endpoints.websocket, None);
        assert_eq!(endpoints.custom, "");
    }

    #[test]
    fn test_primal_context_default() {
        let context = PrimalContext::default();

        assert_eq!(context.user_id, "default");
        assert_eq!(context.device_id, "default");
        assert!(!context.session_id.is_empty());
        assert_eq!(context.network_location.ip_address, "127.0.0.1");
        assert_eq!(context.network_location.subnet, None);
        assert_eq!(context.security_level, SecurityLevel::Standard);
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn test_health_state_default() {
        let state = HealthState::default();
        assert_eq!(state, HealthState::Unknown);
    }

    // ==================== Hash Tests ====================

    #[test]
    fn test_primal_capability_hash_authentication() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let cap1 = PrimalCapability::Authentication {
            methods: vec!["password".to_string(), "oauth".to_string()],
        };
        let cap2 = PrimalCapability::Authentication {
            methods: vec!["password".to_string(), "oauth".to_string()],
        };

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        cap1.hash(&mut hasher1);
        cap2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_primal_capability_hash_all_variants() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let capabilities = vec![
            PrimalCapability::Authentication {
                methods: vec!["password".to_string()],
            },
            PrimalCapability::Encryption {
                algorithms: vec!["aes256".to_string()],
            },
            PrimalCapability::KeyManagement { hsm_support: true },
            PrimalCapability::ThreatDetection { ml_enabled: true },
            PrimalCapability::AuditLogging {
                compliance: vec!["hipaa".to_string()],
            },
            PrimalCapability::Authorization { rbac_support: true },
            PrimalCapability::FileSystem { supports_zfs: true },
            PrimalCapability::ObjectStorage {
                backends: vec!["s3".to_string()],
            },
            PrimalCapability::DataReplication {
                consistency: "strong".to_string(),
            },
            PrimalCapability::Backup { incremental: true },
            PrimalCapability::DataArchiving {
                compression: vec!["gzip".to_string()],
            },
            PrimalCapability::ContainerRuntime {
                orchestrators: vec!["k8s".to_string()],
            },
            PrimalCapability::ServerlessExecution {
                languages: vec!["python".to_string()],
            },
            PrimalCapability::GpuAcceleration { cuda_support: true },
            PrimalCapability::LoadBalancing {
                algorithms: vec!["round-robin".to_string()],
            },
            PrimalCapability::AutoScaling {
                metrics: vec!["cpu".to_string()],
            },
            PrimalCapability::ModelInference {
                models: vec!["llm".to_string()],
            },
            PrimalCapability::AgentFramework { mcp_support: true },
            PrimalCapability::MachineLearning {
                training_support: true,
            },
            PrimalCapability::Custom {
                name: "test".to_string(),
                attributes: "{}".to_string(),
            },
        ];

        // Verify all can be hashed without panicking
        for cap in capabilities {
            let mut hasher = DefaultHasher::new();
            cap.hash(&mut hasher);
            assert!(hasher.finish() > 0 || hasher.finish() == 0); // Just verify it completes
        }
    }

    // ==================== Error Tests ====================

    #[test]
    fn test_primal_error_display() {
        let errors = vec![
            (
                PrimalError::Configuration("test".to_string()),
                "Configuration error",
            ),
            (PrimalError::Network("test".to_string()), "Network error"),
            (PrimalError::Security("test".to_string()), "Security error"),
            (
                PrimalError::Orchestration("test".to_string()),
                "Orchestration error",
            ),
            (PrimalError::State("test".to_string()), "State error"),
            (
                PrimalError::HealthCheck("test".to_string()),
                "Health check error",
            ),
            (PrimalError::Metrics("test".to_string()), "Metrics error"),
            (PrimalError::Shutdown("test".to_string()), "Shutdown error"),
            (PrimalError::Internal("test".to_string()), "Internal error"),
            (PrimalError::Timeout("test".to_string()), "Timeout error"),
            (
                PrimalError::Permission("test".to_string()),
                "Permission error",
            ),
            (PrimalError::Resource("test".to_string()), "Resource error"),
            (
                PrimalError::Communication("test".to_string()),
                "Communication error",
            ),
            (
                PrimalError::Validation("test".to_string()),
                "Validation error",
            ),
            (
                PrimalError::NotImplemented("test".to_string()),
                "Not implemented",
            ),
            (
                PrimalError::ServiceUnavailable("test".to_string()),
                "Service unavailable",
            ),
            (
                PrimalError::AlreadyExists("test".to_string()),
                "Already exists",
            ),
            (PrimalError::NotFound("test".to_string()), "Not found"),
        ];

        for (error, expected_prefix) in errors {
            let display = error.to_string();
            assert!(
                display.contains(expected_prefix),
                "Error '{}' should contain '{}'",
                display,
                expected_prefix
            );
            assert!(display.contains("test"));
        }
    }

    // ==================== Credentials Tests ====================

    #[test]
    fn test_credentials_all_variants() {
        let creds = vec![
            Credentials::Password {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            Credentials::ApiKey {
                key: "key123".to_string(),
                service_id: "svc".to_string(),
            },
            Credentials::Bearer {
                token: "token123".to_string(),
            },
            Credentials::Token {
                token: "jwt123".to_string(),
            },
            Credentials::Certificate {
                cert: vec![1, 2, 3],
            },
            Credentials::ServiceAccount {
                service_id: "svc".to_string(),
                api_key: "key".to_string(),
            },
            Credentials::Bootstrap {
                service_id: "svc".to_string(),
            },
            Credentials::Test {
                service_id: "test".to_string(),
            },
            Credentials::Custom(HashMap::new()),
        ];

        assert_eq!(creds.len(), 9);

        // Verify they can all be cloned and debugged
        for cred in creds {
            let _cloned = cred.clone();
            let _debug = format!("{:?}", cred);
        }
    }

    // ==================== Type Creation Tests ====================

    #[test]
    fn test_network_location_creation() {
        let loc = NetworkLocation {
            ip_address: "192.168.1.1".to_string(),
            subnet: Some("192.168.1.0/24".to_string()),
            network_id: Some("net-123".to_string()),
            geo_location: Some("US-CA".to_string()),
        };

        assert_eq!(loc.ip_address, "192.168.1.1");
        assert_eq!(loc.subnet.unwrap(), "192.168.1.0/24");
    }

    #[test]
    fn test_auth_result_creation() {
        use chrono::Utc;

        let result = AuthResult {
            principal: Principal {
                id: "user-123".to_string(),
                name: "Test User".to_string(),
                principal_type: PrincipalType::User,
                roles: vec!["admin".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
                metadata: HashMap::new(),
            },
            token: "token123".to_string(),
            expires_at: Utc::now(),
            permissions: vec!["read".to_string()],
            metadata: HashMap::new(),
        };

        assert_eq!(result.principal.id, "user-123");
        assert_eq!(result.token, "token123");
        assert_eq!(result.permissions.len(), 1);
    }

    #[test]
    fn test_principal_type_variants() {
        let types = [
            PrincipalType::User,
            PrincipalType::Service,
            PrincipalType::Client,
            PrincipalType::System,
        ];

        assert_eq!(types.len(), 4);
    }

    #[test]
    fn test_security_level_variants() {
        let levels = [
            SecurityLevel::Basic,
            SecurityLevel::Standard,
            SecurityLevel::High,
            SecurityLevel::Maximum,
        ];

        assert_eq!(levels.len(), 4);
    }

    #[test]
    fn test_metric_value_variants() {
        use chrono::{Duration, Utc};

        let metrics = vec![
            MetricValue::Counter(42),
            MetricValue::Gauge(3.14),
            MetricValue::Histogram {
                count: 100,
                sum: 500.0,
                buckets: vec![(1.0, 10), (5.0, 50)],
            },
            MetricValue::String("test".to_string()),
            MetricValue::Boolean(true),
            MetricValue::Duration(Duration::seconds(60)),
            MetricValue::Timestamp(Utc::now()),
        ];

        assert_eq!(metrics.len(), 7);
    }

    #[test]
    fn test_primal_request_creation() {
        use chrono::Utc;
        use uuid::Uuid;

        let request = PrimalRequest {
            id: Uuid::new_v4(),
            request_type: PrimalRequestType::Authenticate,
            payload: HashMap::new(),
            timestamp: Utc::now(),
            context: Some("test-context".to_string()),
            priority: Some(5),
            security_level: Some("high".to_string()),
        };

        assert_eq!(request.priority, Some(5));
        assert_eq!(request.context, Some("test-context".to_string()));
    }

    #[test]
    fn test_primal_response_creation() {
        use chrono::Utc;
        use uuid::Uuid;

        let request_id = Uuid::new_v4();
        let response = PrimalResponse {
            request_id,
            response_type: PrimalResponseType::Authentication,
            payload: HashMap::new(),
            timestamp: Utc::now(),
            success: true,
            error_message: None,
            metadata: Some(HashMap::new()),
        };

        assert!(response.success);
        assert_eq!(response.request_id, request_id);
    }

    #[test]
    fn test_primal_request_type_variants() {
        let types = vec![
            PrimalRequestType::Authenticate,
            PrimalRequestType::Encrypt,
            PrimalRequestType::Decrypt,
            PrimalRequestType::Authorize,
            PrimalRequestType::AuditLog,
            PrimalRequestType::ThreatDetection,
            PrimalRequestType::HealthCheck,
            PrimalRequestType::Store,
            PrimalRequestType::Retrieve,
            PrimalRequestType::Compute,
            PrimalRequestType::Infer,
            PrimalRequestType::Custom("test".to_string()),
        ];

        assert_eq!(types.len(), 12);
    }

    #[test]
    fn test_primal_response_type_variants() {
        let types = vec![
            PrimalResponseType::Authentication,
            PrimalResponseType::Encryption,
            PrimalResponseType::Decryption,
            PrimalResponseType::Authorization,
            PrimalResponseType::Audit,
            PrimalResponseType::ThreatDetection,
            PrimalResponseType::HealthCheck,
            PrimalResponseType::Storage,
            PrimalResponseType::Retrieval,
            PrimalResponseType::Compute,
            PrimalResponseType::Inference,
            PrimalResponseType::Custom("test".to_string()),
        ];

        assert_eq!(types.len(), 12);
    }

    #[test]
    fn test_health_detail_creation() {
        let detail = HealthDetail {
            status: HealthState::Healthy,
            message: "All systems operational".to_string(),
            data: HashMap::new(),
        };

        assert_eq!(detail.status, HealthState::Healthy);
        assert!(!detail.message.is_empty());
    }

    #[test]
    fn test_primal_state_variants() {
        let states = [
            PrimalState::Initializing,
            PrimalState::Starting,
            PrimalState::Running,
            PrimalState::Stopping,
            PrimalState::Stopped,
            PrimalState::Error("test".to_string()),
            PrimalState::Restarting,
            PrimalState::Maintenance,
        ];

        assert_eq!(states.len(), 8);
    }

    #[test]
    fn test_health_state_variants() {
        let states = [
            HealthState::Healthy,
            HealthState::Degraded,
            HealthState::Unhealthy,
            HealthState::Unknown,
        ];

        assert_eq!(states.len(), 4);
    }

    #[test]
    fn test_primal_type_variants() {
        let types = [
            PrimalType::Coordinator,
            PrimalType::Security,
            PrimalType::Orchestration,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::AI,
            PrimalType::Network,
            PrimalType::Custom("test".to_string()),
        ];

        assert_eq!(types.len(), 8);
    }
}
