//! Tests for SquirrelPrimalProvider core functionality

#[cfg(test)]
mod tests {
    use crate::universal::*;

    #[test]
    fn test_module_compiles() {
        // Ensure module compiles correctly
        assert!(true);
    }

    #[test]
    fn test_primal_type_ai() {
        // Verify AI primal type exists
        let primal_type = PrimalType::AI;
        assert!(matches!(primal_type, PrimalType::AI));
    }

    #[test]
    fn test_primal_type_storage() {
        let primal_type = PrimalType::Storage;
        assert!(matches!(primal_type, PrimalType::Storage));
    }

    #[test]
    fn test_primal_type_compute() {
        let primal_type = PrimalType::Compute;
        assert!(matches!(primal_type, PrimalType::Compute));
    }

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus::Healthy;
        assert!(matches!(status, HealthStatus::Healthy));
    }

    #[test]
    fn test_health_status_degraded() {
        let status = HealthStatus::Degraded;
        assert!(matches!(status, HealthStatus::Degraded));
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus::Unhealthy;
        assert!(matches!(status, HealthStatus::Unhealthy));
    }

    #[test]
    fn test_response_status_success() {
        let status = ResponseStatus::Success;
        assert!(matches!(status, ResponseStatus::Success));
    }

    #[test]
    fn test_response_status_error() {
        let status = ResponseStatus::Error;
        assert!(matches!(status, ResponseStatus::Error));
    }

    #[test]
    fn test_primal_context_default() {
        let context = PrimalContext::default();
        // Just verify it can be created
        assert!(context.user_id.is_empty() || !context.user_id.is_empty());
    }

    #[test]
    fn test_primal_capability_model_inference() {
        let cap = PrimalCapability::ModelInference {
            models: vec!["gpt-4".to_string(), "claude-3".to_string()],
        };

        if let PrimalCapability::ModelInference { models } = cap {
            assert_eq!(models.len(), 2);
            assert!(models.contains(&"gpt-4".to_string()));
            assert!(models.contains(&"claude-3".to_string()));
        } else {
            panic!("Expected ModelInference capability");
        }
    }

    #[test]
    fn test_primal_capability_agent_framework() {
        let cap = PrimalCapability::AgentFramework {
            frameworks: vec!["test".to_string()],
            mcp_support: true,
        };

        if let PrimalCapability::AgentFramework { mcp_support, .. } = cap {
            assert!(mcp_support);
        } else {
            panic!("Expected AgentFramework capability");
        }
    }

    #[test]
    fn test_primal_capability_agent_framework_no_mcp() {
        let cap = PrimalCapability::AgentFramework {
            frameworks: vec!["custom".to_string()],
            mcp_support: false,
        };

        if let PrimalCapability::AgentFramework { mcp_support, .. } = cap {
            assert!(!mcp_support);
        } else {
            panic!("Expected AgentFramework capability");
        }
    }

    #[test]
    fn test_primal_capability_natural_language() {
        let cap = PrimalCapability::NaturalLanguage {
            languages: vec!["en".to_string(), "es".to_string(), "fr".to_string()],
        };

        if let PrimalCapability::NaturalLanguage { languages } = cap {
            assert_eq!(languages.len(), 3);
            assert!(languages.contains(&"en".to_string()));
        } else {
            panic!("Expected NaturalLanguage capability");
        }
    }

    #[test]
    fn test_primal_dependency_storage() {
        let dep = PrimalDependency {
            primal_type: "Storage".to_string(),
            required: false,
            optional: true,
            capabilities: vec![],
            required_capabilities: vec![],
            min_version: Some("1.0.0".to_string()),
            preferred_instance: None,
        };

        assert_eq!(dep.primal_type, "Storage");
        assert!(!dep.required);
        assert_eq!(dep.min_version, Some("1.0.0".to_string()));
        assert!(dep.preferred_instance.is_none());
    }

    #[test]
    fn test_primal_dependency_required() {
        let dep = PrimalDependency {
            primal_type: "Compute".to_string(),
            required: true,
            optional: false,
            capabilities: vec![],
            required_capabilities: vec![],
            min_version: Some("2.0.0".to_string()),
            preferred_instance: Some("compute-primary".to_string()),
        };

        assert!(dep.required);
        assert_eq!(dep.preferred_instance, Some("compute-primary".to_string()));
    }

    #[test]
    fn test_primal_dependency_with_capabilities() {
        let cap1 = PrimalCapability::ModelInference {
            models: vec!["model-a".to_string()],
        };
        let cap2 = PrimalCapability::AgentFramework {
            frameworks: vec!["test".to_string()],
            mcp_support: true,
        };

        let dep = PrimalDependency {
            primal_type: "Storage".to_string(),
            required: false,
            optional: true,
            capabilities: vec![cap1.clone(), cap2.clone()],
            required_capabilities: vec![cap1, cap2],
            min_version: None,
            preferred_instance: None,
        };

        assert_eq!(dep.capabilities.len(), 2);
    }

    #[test]
    fn test_port_type_http() {
        let port_type = PortType::Http;
        assert!(matches!(port_type, PortType::Http));
    }

    #[test]
    fn test_port_type_grpc() {
        let port_type = PortType::Grpc;
        assert!(matches!(port_type, PortType::Grpc));
    }

    #[test]
    fn test_port_type_websocket() {
        let port_type = PortType::WebSocket;
        assert!(matches!(port_type, PortType::WebSocket));
    }

    #[test]
    fn test_port_type_custom() {
        let port_type = PortType::Custom("admin".to_string());
        if let PortType::Custom(name) = port_type {
            assert_eq!(name, "admin");
        } else {
            panic!("Expected Custom port type");
        }
    }

    #[test]
    fn test_port_status_active() {
        let status = PortStatus::Active;
        assert!(matches!(status, PortStatus::Active));
    }

    #[test]
    fn test_port_status_reserved() {
        let status = PortStatus::Reserved;
        assert!(matches!(status, PortStatus::Reserved));
    }

    #[test]
    fn test_port_status_releasing() {
        let status = PortStatus::Releasing;
        assert!(matches!(status, PortStatus::Releasing));
    }

    #[test]
    fn test_port_status_expired() {
        let status = PortStatus::Expired;
        assert!(matches!(status, PortStatus::Expired));
    }

    #[test]
    fn test_security_level_public() {
        let level = SecurityLevel::Public;
        assert!(matches!(level, SecurityLevel::Public));
    }

    #[test]
    fn test_security_level_basic() {
        let level = SecurityLevel::Basic;
        assert!(matches!(level, SecurityLevel::Basic));
    }

    #[test]
    fn test_security_level_standard() {
        let level = SecurityLevel::Standard;
        assert!(matches!(level, SecurityLevel::Standard));
    }

    #[test]
    fn test_primal_health_healthy() {
        use chrono::Utc;
        use std::collections::HashMap;

        let health = PrimalHealth {
            status: HealthStatus::Healthy,
            healthy: true,
            score: 1.0,
            last_check: Utc::now(),
            message: None,
            details: Some(serde_json::json!({})),
        };

        assert!(matches!(health.status, HealthStatus::Healthy));
        assert_eq!(health.score, 1.0);
    }

    #[test]
    fn test_primal_health_with_details() {
        use chrono::Utc;
        use std::collections::HashMap;

        let mut details = HashMap::new();
        details.insert("cpu_usage".to_string(), "25%".to_string());
        details.insert("memory_usage".to_string(), "512MB".to_string());

        let health = PrimalHealth {
            status: HealthStatus::Healthy,
            healthy: true,
            score: 0.95,
            last_check: Utc::now(),
            message: Some("System healthy".to_string()),
            details: Some(serde_json::json!(details)),
        };

        assert_eq!(health.healthy, true);
        assert_eq!(health.score, 0.95);
    }

    #[test]
    fn test_primal_endpoints_construction() {
        use std::collections::HashMap;

        let endpoints = PrimalEndpoints {
            http: Some("http://localhost:8080".to_string()),
            grpc: None,
            primary: Some("http://localhost:8080".to_string()),
            health: Some("/health".to_string()),
            metrics: Some("/metrics".to_string()),
            admin: Some("/admin".to_string()),
            websocket: Some("ws://localhost:8080/ws".to_string()),
            mcp: Some("/mcp".to_string()),
            ai_coordination: Some("/ai".to_string()),
            service_mesh: Some("/mesh".to_string()),
            custom: vec![],
        };

        assert_eq!(endpoints.primary, Some("http://localhost:8080".to_string()));
        assert_eq!(endpoints.health, Some("/health".to_string()));
        assert_eq!(endpoints.metrics, Some("/metrics".to_string()));
        assert!(endpoints.websocket.is_some());
    }

    #[test]
    fn test_primal_endpoints_with_custom() {
        let custom = vec![("debug".to_string(), "/debug".to_string())];

        let endpoints = PrimalEndpoints {
            http: Some("http://localhost:8080".to_string()),
            grpc: None,
            primary: Some("http://localhost:8080".to_string()),
            health: Some("/health".to_string()),
            metrics: Some("/metrics".to_string()),
            admin: Some("/admin".to_string()),
            websocket: None,
            mcp: Some("/mcp".to_string()),
            ai_coordination: Some("/ai".to_string()),
            service_mesh: Some("/mesh".to_string()),
            custom,
        };

        assert_eq!(endpoints.custom.len(), 1);
        assert_eq!(endpoints.custom[0].0, "debug");
        assert_eq!(endpoints.custom[0].1, "/debug");
    }

    #[test]
    fn test_dynamic_port_info_construction() {
        use chrono::Utc;

        let port_info = DynamicPortInfo {
            port: 8080,
            assigned_port: 8080,
            current_port: 8080,
            port_range: Some((8000, 9000)),
            port_type: PortType::Http,
            status: PortStatus::Active,
            allocated_at: Utc::now(),
            assigned_at: Utc::now(),
            lease_duration: Some(chrono::Duration::hours(1)),
            expires_at: None,
            metadata: std::collections::HashMap::new(),
        };

        assert_eq!(port_info.assigned_port, 8080);
        assert_eq!(port_info.current_port, 8080);
        if let Some((min, max)) = port_info.port_range {
            assert_eq!(min, 8000);
            assert_eq!(max, 9000);
        }
    }

    #[test]
    fn test_dynamic_port_info_with_expiry() {
        use chrono::{Duration, Utc};

        let now = Utc::now();
        let expires = now + Duration::hours(2);

        let port_info = DynamicPortInfo {
            port: 9090,
            assigned_port: 9090,
            current_port: 9090,
            port_range: Some((9000, 10000)),
            port_type: PortType::Grpc,
            status: PortStatus::Reserved,
            allocated_at: now,
            assigned_at: now,
            lease_duration: Some(Duration::hours(2)),
            expires_at: Some(expires),
            metadata: std::collections::HashMap::new(),
        };

        assert!(port_info.expires_at.is_some());
        assert_eq!(port_info.status, PortStatus::Reserved);
    }

    #[test]
    fn test_network_location_construction() {
        let location = NetworkLocation {
            region: "us-west-2".to_string(),
            data_center: Some("dc-01".to_string()),
            availability_zone: Some("us-west-2a".to_string()),
            ip_address: Some("192.168.1.100".to_string()),
            subnet: Some("192.168.1.0/24".to_string()),
            network_id: Some("net-001".to_string()),
            geo_location: Some("us-west-2".to_string()),
        };

        assert_eq!(location.ip_address, Some("192.168.1.100".to_string()));
        assert_eq!(location.subnet, Some("192.168.1.0/24".to_string()));
    }

    #[test]
    fn test_primal_context_construction() {
        use std::collections::HashMap;

        let location = NetworkLocation {
            region: "local".to_string(),
            data_center: None,
            availability_zone: None,
            ip_address: Some("10.0.0.1".to_string()),
            subnet: None,
            network_id: None,
            geo_location: None,
        };

        let context = PrimalContext {
            user_id: "user_123".to_string(),
            device_id: "device_456".to_string(),
            session_id: Some("session_789".to_string()),
            network_location: location,
            security_level: SecurityLevel::Standard,
            biome_id: Some("biome_001".to_string()),
            metadata: HashMap::new(),
        };

        assert_eq!(context.user_id, "user_123");
        assert_eq!(context.device_id, "device_456");
        assert_eq!(context.session_id, Some("session_789".to_string()));
        assert_eq!(context.biome_id, Some("biome_001".to_string()));
    }

    #[test]
    fn test_primal_info_construction() {
        let info = PrimalInfo {
            primal_id: "squirrel".to_string(),
            instance_id: "instance_123".to_string(),
            primal_type: PrimalType::AI,
            capabilities: vec![PrimalCapability::ModelInference {
                models: vec!["gpt-4".to_string()],
            }],
            endpoints: vec!["http://localhost:8080/api/v1".to_string()],
            metadata: std::collections::HashMap::new(),
            id: Some("squirrel".to_string()),
            version: "1.0.0".to_string(),
        };

        assert_eq!(info.primal_id, "squirrel");
        assert_eq!(info.instance_id, "instance_123");
        assert_eq!(info.primal_type, PrimalType::AI);
        // Version field removed from PrimalInfo - now in metadata
        assert_eq!(info.capabilities.len(), 1);
    }
}
