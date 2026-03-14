// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for universal adapters core types

#[cfg(test)]
mod tests {
    use super::super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn test_universal_response_success() {
        let request_id = "test-123".to_string();
        let data = serde_json::json!({"result": "ok"});

        let response = UniversalResponse::success(request_id.clone(), data.clone());

        assert_eq!(response.request_id, request_id);
        assert!(matches!(response.status, ResponseStatus::Success));
        assert_eq!(response.data, data);
    }

    #[test]
    fn test_universal_response_error() {
        let request_id = "test-456".to_string();
        let code = "ERR_001";
        let message = "Test error";

        let response = UniversalResponse::error(request_id.clone(), code, message);

        assert_eq!(response.request_id, request_id);
        match response.status {
            ResponseStatus::Error {
                code: c,
                message: m,
            } => {
                assert_eq!(c, code);
                assert_eq!(m, message);
            }
            _ => panic!("Expected Error status"),
        }
        assert_eq!(response.data, serde_json::Value::Null);
    }

    #[test]
    fn test_universal_request_creation() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), serde_json::json!("value"));

        let request = UniversalRequest {
            request_id: "req-123".to_string(),
            operation: "test_op".to_string(),
            parameters: params,
            context: HashMap::new(),
            requester: "test_service".to_string(),
            timestamp: Utc::now(),
        };

        assert_eq!(request.request_id, "req-123");
        assert_eq!(request.operation, "test_op");
        assert_eq!(request.requester, "test_service");
        assert!(request.parameters.contains_key("key"));
    }

    #[test]
    fn test_service_category_compute() {
        let category = ServiceCategory::Compute {
            specialties: vec!["gpu".to_string(), "tensor".to_string()],
        };

        match category {
            ServiceCategory::Compute { specialties } => {
                assert_eq!(specialties.len(), 2);
                assert!(specialties.contains(&"gpu".to_string()));
            }
            _ => panic!("Wrong category type"),
        }
    }

    #[test]
    fn test_service_category_storage() {
        let category = ServiceCategory::Storage {
            types: vec!["object".to_string(), "block".to_string()],
        };

        match category {
            ServiceCategory::Storage { types } => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Wrong category type"),
        }
    }

    #[test]
    fn test_service_category_security() {
        let category = ServiceCategory::Security {
            domains: vec!["auth".to_string(), "encryption".to_string()],
        };

        match category {
            ServiceCategory::Security { domains } => {
                assert_eq!(domains.len(), 2);
            }
            _ => panic!("Wrong category type"),
        }
    }

    #[test]
    fn test_service_category_orchestration() {
        let category = ServiceCategory::Orchestration {
            scopes: vec!["workflow".to_string(), "coordination".to_string()],
        };

        match category {
            ServiceCategory::Orchestration { scopes } => {
                assert_eq!(scopes.len(), 2);
            }
            _ => panic!("Wrong category type"),
        }
    }

    #[test]
    fn test_service_category_intelligence() {
        let category = ServiceCategory::Intelligence {
            modalities: vec!["text".to_string(), "vision".to_string()],
        };

        match category {
            ServiceCategory::Intelligence { modalities } => {
                assert_eq!(modalities.len(), 2);
            }
            _ => panic!("Wrong category type"),
        }
    }

    #[test]
    fn test_service_category_custom() {
        let category = ServiceCategory::Custom {
            category: "ml_ops".to_string(),
            subcategories: vec!["monitoring".to_string(), "deployment".to_string()],
        };

        match category {
            ServiceCategory::Custom {
                category: c,
                subcategories,
            } => {
                assert_eq!(c, "ml_ops");
                assert_eq!(subcategories.len(), 2);
            }
            _ => panic!("Wrong category type"),
        }
    }

    #[test]
    fn test_service_capability_security() {
        let capability = ServiceCapability::Security {
            functions: vec!["encryption".to_string(), "signing".to_string()],
            compliance: vec!["gdpr".to_string(), "hipaa".to_string()],
            trust_levels: vec!["high".to_string()],
        };

        match capability {
            ServiceCapability::Security {
                functions,
                compliance,
                trust_levels,
            } => {
                assert_eq!(functions.len(), 2);
                assert_eq!(compliance.len(), 2);
                assert_eq!(trust_levels.len(), 1);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_service_capability_coordination() {
        let capability = ServiceCapability::Coordination {
            patterns: vec!["saga".to_string(), "choreography".to_string()],
            consistency: "eventual".to_string(),
            fault_tolerance: "high".to_string(),
        };

        match capability {
            ServiceCapability::Coordination {
                patterns,
                consistency,
                fault_tolerance,
            } => {
                assert_eq!(patterns.len(), 2);
                assert_eq!(consistency, "eventual");
                assert_eq!(fault_tolerance, "high");
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_service_capability_data_management() {
        let capability = ServiceCapability::DataManagement {
            operations: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
            ],
            consistency: "strong".to_string(),
            durability: "persistent".to_string(),
        };

        match capability {
            ServiceCapability::DataManagement {
                operations,
                consistency,
                durability,
            } => {
                assert_eq!(operations.len(), 3);
                assert_eq!(consistency, "strong");
                assert_eq!(durability, "persistent");
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_service_capability_computation() {
        let mut resources = HashMap::new();
        resources.insert("cpu".to_string(), serde_json::json!(16));

        let capability = ServiceCapability::Computation {
            types: vec!["batch".to_string(), "streaming".to_string()],
            resources,
            constraints: vec!["memory<32GB".to_string()],
        };

        match capability {
            ServiceCapability::Computation {
                types,
                resources,
                constraints,
            } => {
                assert_eq!(types.len(), 2);
                assert!(resources.contains_key("cpu"));
                assert_eq!(constraints.len(), 1);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_service_capability_ai() {
        let capability = ServiceCapability::ArtificialIntelligence {
            models: vec!["gpt-4".to_string(), "claude".to_string()],
            tasks: vec!["completion".to_string(), "embedding".to_string()],
            interfaces: vec!["rest".to_string(), "tarpc".to_string()],
        };

        match capability {
            ServiceCapability::ArtificialIntelligence {
                models,
                tasks,
                interfaces,
            } => {
                assert_eq!(models.len(), 2);
                assert_eq!(tasks.len(), 2);
                assert_eq!(interfaces.len(), 2);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_service_capability_custom() {
        let mut parameters = HashMap::new();
        parameters.insert("param1".to_string(), serde_json::json!("value1"));

        let capability = ServiceCapability::Custom {
            domain: "iot".to_string(),
            capability: "device_management".to_string(),
            parameters,
        };

        match capability {
            ServiceCapability::Custom {
                domain,
                capability: cap,
                parameters,
            } => {
                assert_eq!(domain, "iot");
                assert_eq!(cap, "device_management");
                assert!(parameters.contains_key("param1"));
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_service_endpoint_creation() {
        let endpoint = ServiceEndpoint {
            name: "api".to_string(),
            url: "https://example.com".to_string(),
            protocol: "https".to_string(),
            port: Some(443),
            path: Some("/v1".to_string()),
        };

        assert_eq!(endpoint.name, "api");
        assert_eq!(endpoint.url, "https://example.com");
        assert_eq!(endpoint.protocol, "https");
        assert_eq!(endpoint.port, Some(443));
        assert_eq!(endpoint.path, Some("/v1".to_string()));
    }

    #[test]
    fn test_resource_spec_creation() {
        let mut custom = HashMap::new();
        custom.insert("gpu_count".to_string(), serde_json::json!(4));

        let spec = ResourceSpec {
            cpu_cores: Some(16),
            memory_gb: Some(64),
            storage_gb: Some(1000),
            network_bandwidth: Some(10_000),
            custom_resources: custom,
        };

        assert_eq!(spec.cpu_cores, Some(16));
        assert_eq!(spec.memory_gb, Some(64));
        assert_eq!(spec.storage_gb, Some(1000));
        assert_eq!(spec.network_bandwidth, Some(10_000));
        assert!(spec.custom_resources.contains_key("gpu_count"));
    }

    #[test]
    fn test_integration_preferences_creation() {
        let prefs = IntegrationPreferences {
            preferred_protocols: vec!["tarpc".to_string(), "http2".to_string()],
            retry_policy: "exponential_backoff".to_string(),
            timeout_seconds: 30,
            load_balancing_weight: 10,
        };

        assert_eq!(prefs.preferred_protocols.len(), 2);
        assert_eq!(prefs.retry_policy, "exponential_backoff");
        assert_eq!(prefs.timeout_seconds, 30);
        assert_eq!(prefs.load_balancing_weight, 10);
    }

    #[test]
    fn test_service_metadata_creation() {
        let metadata = ServiceMetadata {
            name: "test-service".to_string(),
            category: ServiceCategory::Compute {
                specialties: vec!["ml".to_string()],
            },
            version: "1.0.0".to_string(),
            description: "Test service".to_string(),
            maintainer: "test@example.com".to_string(),
            protocols: vec!["json-rpc".to_string(), "tarpc".to_string()],
        };

        assert_eq!(metadata.name, "test-service");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, "Test service");
        assert_eq!(metadata.maintainer, "test@example.com");
        assert_eq!(metadata.protocols.len(), 2);
    }

    #[test]
    fn test_universal_service_registration_creation() {
        let registration = UniversalServiceRegistration {
            service_id: Uuid::new_v4(),
            metadata: ServiceMetadata {
                name: "test".to_string(),
                category: ServiceCategory::Compute {
                    specialties: vec!["ml".to_string()],
                },
                version: "1.0".to_string(),
                description: "Test".to_string(),
                maintainer: "test".to_string(),
                protocols: vec!["http".to_string()],
            },
            capabilities: vec![],
            endpoints: vec![],
            resources: ResourceSpec {
                cpu_cores: Some(8),
                memory_gb: Some(32),
                storage_gb: Some(500),
                network_bandwidth: Some(1000),
                custom_resources: HashMap::new(),
            },
            integration: IntegrationPreferences {
                preferred_protocols: vec!["http".to_string()],
                retry_policy: "simple".to_string(),
                timeout_seconds: 30,
                load_balancing_weight: 5,
            },
            extensions: HashMap::new(),
            registration_timestamp: Utc::now(),
            service_version: "1.0.0".to_string(),
            instance_id: "instance-1".to_string(),
            priority: 5,
        };

        assert_eq!(registration.service_version, "1.0.0");
        assert_eq!(registration.instance_id, "instance-1");
        assert_eq!(registration.priority, 5);
        assert_eq!(registration.resources.cpu_cores, Some(8));
    }

    #[test]
    fn test_response_status_success() {
        let status = ResponseStatus::Success;
        assert!(matches!(status, ResponseStatus::Success));
    }

    #[test]
    fn test_response_status_error() {
        let status = ResponseStatus::Error {
            code: "E001".to_string(),
            message: "Error occurred".to_string(),
        };

        match status {
            ResponseStatus::Error { code, message } => {
                assert_eq!(code, "E001");
                assert_eq!(message, "Error occurred");
            }
            _ => panic!("Wrong status type"),
        }
    }

    #[test]
    fn test_response_status_partial() {
        let status = ResponseStatus::Partial {
            completed: 50,
            total: 100,
        };

        match status {
            ResponseStatus::Partial { completed, total } => {
                assert_eq!(completed, 50);
                assert_eq!(total, 100);
            }
            _ => panic!("Wrong status type"),
        }
    }

    #[test]
    fn test_service_health_healthy() {
        let health = ServiceHealth {
            healthy: true,
            message: Some("All systems operational".to_string()),
            metrics: HashMap::new(),
        };

        assert!(health.healthy);
        assert_eq!(health.message, Some("All systems operational".to_string()));
    }

    #[test]
    fn test_service_health_unhealthy() {
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), serde_json::json!(95.5));

        let health = ServiceHealth {
            healthy: false,
            message: Some("High CPU usage".to_string()),
            metrics,
        };

        assert!(!health.healthy);
        assert!(health.message.is_some());
        assert!(health.metrics.contains_key("cpu_usage"));
    }

    #[test]
    fn test_universal_response_serialization() {
        let response =
            UniversalResponse::success("test-123".to_string(), serde_json::json!({"data": "test"}));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test-123"));
        assert!(json.contains("Success"));
    }

    #[test]
    fn test_universal_request_serialization() {
        let request = UniversalRequest {
            request_id: "req-123".to_string(),
            operation: "test".to_string(),
            parameters: HashMap::new(),
            context: HashMap::new(),
            requester: "service".to_string(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("req-123"));
        assert!(json.contains("test"));
    }
}
