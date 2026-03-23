// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for Universal Security Client

#[cfg(test)]
mod tests {
    use super::super::client::UniversalSecurityClient;
    use super::super::providers::SecurityProvider;
    use super::super::types::{
        AISecurityContext, BehavioralProfile, ContextAwareness, DecisionOutcome, DeviceContext,
        LocationContext, RiskLevel, SecurityClientConfig, SecurityContext, SecurityOperation,
        SecurityPayload, TemporalContext, TrustLevel, UniversalSecurityRequest,
    };
    use crate::security::types::{
        SecurityContext as LegacySecurityContext, SecurityRequest, SecurityRequestType,
    };
    use crate::universal::PrimalContext;
    use crate::universal::messages::{PrimalResponse, ResponseStatus};
    use crate::universal_primal_ecosystem::{
        DiscoveredService, ServiceHealth, UniversalPrimalEcosystem,
    };
    use base64::{Engine as _, engine::general_purpose};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn test_context() -> PrimalContext {
        PrimalContext::default()
    }

    #[test]
    fn test_security_client_new() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        assert!(client.get_security_config().operation_timeout.as_secs() > 0);
    }

    #[tokio::test]
    async fn test_security_client_initialize() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        client
            .initialize()
            .await
            .expect("initialize should succeed");
    }

    #[tokio::test]
    async fn test_security_client_authenticate_no_providers() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        client.initialize().await.expect("initialize");
        let mut creds = HashMap::new();
        creds.insert("password".to_string(), "secret".to_string());
        let result = client.authenticate("user1", creds, RiskLevel::Low).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_security_client_get_config() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config.clone(), test_context());
        let retrieved = client.get_security_config();
        assert_eq!(retrieved.operation_timeout, config.operation_timeout);
    }

    #[test]
    fn test_security_client_update_config() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let mut client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let new_config = SecurityClientConfig::default();
        client
            .update_security_config(new_config.clone())
            .expect("update");
        assert_eq!(
            client.get_security_config().max_retries,
            new_config.max_retries
        );
    }

    #[test]
    fn test_apply_ai_security_routing() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let mut request = SecurityRequest {
            request_id: "test-1".to_string(),
            request_type: SecurityRequestType::Authentication,
            payload: serde_json::json!({}),
            metadata: HashMap::new(),
            context: LegacySecurityContext::default(),
            timestamp: chrono::Utc::now(),
        };
        client
            .apply_ai_security_routing(&mut request)
            .expect("apply routing");
    }

    #[test]
    fn test_get_ai_security_insights() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let insights = client.get_ai_security_insights();
        assert!(insights.get("threat_landscape").is_some());
        assert!(insights.get("recommended_capabilities").is_some());
        assert!(insights.get("ai_confidence").is_some());
    }

    #[test]
    fn test_validate_ai_config_compatibility() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let result = client.validate_ai_config_compatibility().expect("validate");
        assert!(result);
    }

    #[test]
    fn test_get_config_based_recommendations() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let recommendations = client.get_config_based_recommendations();
        assert!(!recommendations.is_empty());
        assert!(recommendations[0].get("category").is_some());
    }

    #[test]
    fn test_update_ai_metadata() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let mut client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let patterns = vec![
            serde_json::json!({"pattern_type": "auth", "threat_score": 0.2, "provider_used": "bear"}),
        ];
        client
            .update_ai_metadata(&patterns)
            .expect("update metadata");
    }

    #[test]
    fn test_update_ai_metadata_with_compression_fields() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let mut client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let patterns = vec![serde_json::json!({
            "pattern_type": "ingress",
            "compression_ratio": 0.5,
            "file_size": 1024u64,
            "provider_used": "p1",
            "response_time_ms": 12u64,
            "success_rate": 0.99
        })];
        client
            .update_ai_metadata(&patterns)
            .expect("update metadata");
    }

    #[test]
    fn test_update_ai_metadata_unknown_pattern_type() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let mut client = UniversalSecurityClient::new(ecosystem, config, test_context());
        client
            .update_ai_metadata(&[serde_json::json!({})])
            .expect("update metadata");
    }

    fn sample_security_request(
        op: SecurityOperation,
        required_trust: TrustLevel,
    ) -> UniversalSecurityRequest {
        UniversalSecurityRequest {
            request_id: Uuid::new_v4(),
            operation: op,
            security_context: SecurityContext {
                user_id: "u".to_string(),
                session_id: Uuid::new_v4().to_string(),
                ip_address: "127.0.0.1".to_string(),
                user_agent: "test".to_string(),
                clearance_level: "standard".to_string(),
                additional_context: HashMap::new(),
            },
            payload: SecurityPayload {
                data: None,
                parameters: HashMap::new(),
                policy_overrides: None,
                compliance_tags: vec![],
            },
            required_trust_level: required_trust,
            ai_context: AISecurityContext {
                risk_assessment: RiskLevel::Low,
                threat_intelligence: vec![],
                behavioral_analysis: BehavioralProfile {
                    normal_patterns: vec![],
                    anomaly_score: 0.0,
                    historical_behavior: HashMap::new(),
                },
                context_awareness: ContextAwareness {
                    temporal_context: TemporalContext {
                        normal_hours: vec![],
                        time_anomaly_score: 0.0,
                        frequency_analysis: HashMap::new(),
                    },
                    location_context: LocationContext {
                        allowed_locations: vec![],
                        location_risk_score: 0.0,
                        travel_patterns: vec![],
                    },
                    device_context: DeviceContext {
                        trusted_devices: vec![],
                        device_risk_score: 0.0,
                        device_fingerprint: HashMap::new(),
                    },
                },
            },
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_authenticate_succeeds_with_provider_and_default_ecosystem_response() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let primal = universal_patterns::registry::DiscoveredPrimal {
            id: "sec-a".to_string(),
            instance_id: "inst-a".to_string(),
            primal_type: universal_patterns::traits::PrimalType::Security,
            capabilities: vec![],
            endpoint: "unix:///tmp/sec.sock".to_string(),
            health: universal_patterns::traits::PrimalHealth::Healthy,
            context: universal_patterns::traits::PrimalContext::default(),
            port_info: None,
        };
        let provider = SecurityProvider::from_discovered_primal(&primal);
        let client = UniversalSecurityClient::new(
            Arc::clone(&ecosystem),
            SecurityClientConfig::default(),
            test_context(),
        );
        client.test_only_insert_provider(provider);

        let mut creds = HashMap::new();
        creds.insert("password".to_string(), "x".to_string());
        let response = client
            .authenticate("alice", creds, RiskLevel::Low)
            .await
            .expect("authenticate");
        assert!(response.success);
        assert!(matches!(response.decision.outcome, DecisionOutcome::Allow));
    }

    #[tokio::test]
    async fn test_execute_operation_failure_response_denies_and_updates_health() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        ecosystem
            .test_only_set_next_primal_response(PrimalResponse {
                request_id: Uuid::new_v4(),
                response_id: Uuid::new_v4(),
                status: ResponseStatus::Error,
                success: false,
                data: None,
                payload: serde_json::json!({}),
                timestamp: Utc::now(),
                processing_time_ms: Some(500),
                duration: None,
                error: Some("e".to_string()),
                error_message: Some("auth failed".to_string()),
                metadata: HashMap::new(),
            })
            .await;

        let primal = universal_patterns::registry::DiscoveredPrimal {
            id: "sec-b".to_string(),
            instance_id: "inst-b".to_string(),
            primal_type: universal_patterns::traits::PrimalType::Security,
            capabilities: vec![],
            endpoint: "unix:///tmp/sec2.sock".to_string(),
            health: universal_patterns::traits::PrimalHealth::Healthy,
            context: universal_patterns::traits::PrimalContext::default(),
            port_info: None,
        };
        let provider = SecurityProvider::from_discovered_primal(&primal);
        let client = UniversalSecurityClient::new(
            Arc::clone(&ecosystem),
            SecurityClientConfig::default(),
            test_context(),
        );
        client.test_only_insert_provider(provider);

        let req = sample_security_request(
            SecurityOperation::Authenticate {
                identity: "u".to_string(),
                credentials: HashMap::new(),
            },
            TrustLevel::High,
        );
        let response = client.execute_operation(req).await.expect("execute");
        assert!(!response.success);
        assert!(matches!(response.decision.outcome, DecisionOutcome::Deny));
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_process_response_decodes_base64_data_field() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let encoded = general_purpose::STANDARD.encode(b"hello-security");
        ecosystem
            .test_only_set_next_primal_response(PrimalResponse {
                request_id: Uuid::new_v4(),
                response_id: Uuid::new_v4(),
                status: ResponseStatus::Success,
                success: true,
                data: Some(serde_json::json!({ "data": encoded })),
                payload: serde_json::json!({}),
                timestamp: Utc::now(),
                processing_time_ms: Some(42),
                duration: None,
                error: None,
                error_message: None,
                metadata: HashMap::new(),
            })
            .await;

        let primal = universal_patterns::registry::DiscoveredPrimal {
            id: "sec-c".to_string(),
            instance_id: "inst-c".to_string(),
            primal_type: universal_patterns::traits::PrimalType::Security,
            capabilities: vec![],
            endpoint: "unix:///tmp/sec3.sock".to_string(),
            health: universal_patterns::traits::PrimalHealth::Healthy,
            context: universal_patterns::traits::PrimalContext::default(),
            port_info: None,
        };
        let provider = SecurityProvider::from_discovered_primal(&primal);
        let client = UniversalSecurityClient::new(
            Arc::clone(&ecosystem),
            SecurityClientConfig::default(),
            test_context(),
        );
        client.test_only_insert_provider(provider);

        let req = sample_security_request(
            SecurityOperation::Authorize {
                subject: "s".to_string(),
                resource: "r".to_string(),
                action: "read".to_string(),
            },
            TrustLevel::Standard,
        );
        let response = client.execute_operation(req).await.expect("execute");
        assert_eq!(response.data.as_deref(), Some(b"hello-security".as_slice()));
    }

    #[tokio::test]
    async fn test_discover_security_providers_from_ecosystem() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let svc = DiscoveredService {
            service_id: "svc-sec".to_string(),
            instance_id: "inst-sec".to_string(),
            endpoint: "unix:///tmp/x.sock".to_string(),
            capabilities: vec![
                "authentication".to_string(),
                "encryption".to_string(),
                "security-capability".to_string(),
            ],
            health: ServiceHealth::Healthy,
            discovered_at: Utc::now(),
            last_health_check: Some(Utc::now()),
        };
        ecosystem.test_only_register_service(svc).await;

        let client = UniversalSecurityClient::new(
            Arc::clone(&ecosystem),
            SecurityClientConfig::default(),
            test_context(),
        );
        client.initialize().await.expect("init");
        let mut creds = HashMap::new();
        creds.insert("p".to_string(), "q".to_string());
        let r = client.authenticate("u", creds, RiskLevel::Low).await;
        assert!(r.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_authenticate_uses_client_ip_env() {
        temp_env::with_vars(
            [
                ("CLIENT_IP_ADDRESS", Some("192.168.1.10")),
                ("CLIENT_USER_AGENT", Some("TestAgent/1")),
            ],
            || {
                let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
                let primal = universal_patterns::registry::DiscoveredPrimal {
                    id: "sec-env".to_string(),
                    instance_id: "inst-env".to_string(),
                    primal_type: universal_patterns::traits::PrimalType::Security,
                    capabilities: vec![],
                    endpoint: "unix:///tmp/e.sock".to_string(),
                    health: universal_patterns::traits::PrimalHealth::Healthy,
                    context: universal_patterns::traits::PrimalContext::default(),
                    port_info: None,
                };
                let provider = SecurityProvider::from_discovered_primal(&primal);
                let client = UniversalSecurityClient::new(
                    ecosystem,
                    SecurityClientConfig::default(),
                    test_context(),
                );
                client.test_only_insert_provider(provider);
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime");
                let mut creds = HashMap::new();
                creds.insert("a".to_string(), "b".to_string());
                let res = rt.block_on(client.authenticate("id", creds, RiskLevel::Medium));
                let resp = res.expect("ok");
                assert!(matches!(resp.decision.outcome, DecisionOutcome::Allow));
            },
        );
    }
}
