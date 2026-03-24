// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_primal_type_as_str() {
        assert_eq!(PrimalType::ToadStool.as_str(), "toadstool");
        assert_eq!(PrimalType::Songbird.as_str(), "songbird");
        assert_eq!(PrimalType::BearDog.as_str(), "beardog");
        assert_eq!(PrimalType::NestGate.as_str(), "nestgate");
        assert_eq!(PrimalType::Squirrel.as_str(), "squirrel");
        assert_eq!(PrimalType::BiomeOS.as_str(), "biomeos");
        assert_eq!(PrimalType::Any.as_str(), "any");
    }

    #[test]
    fn test_primal_type_serde_roundtrip() {
        for pt in &[
            PrimalType::ToadStool,
            PrimalType::Songbird,
            PrimalType::BearDog,
            PrimalType::NestGate,
            PrimalType::Squirrel,
            PrimalType::BiomeOS,
            PrimalType::Any,
        ] {
            let json = serde_json::to_string(pt).expect("should succeed");
            let deser: PrimalType = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(*pt, deser);
        }
    }

    #[test]
    fn test_security_level_serde_roundtrip() {
        for level in &[
            SecurityLevel::Public,
            SecurityLevel::Internal,
            SecurityLevel::Restricted,
            SecurityLevel::Confidential,
        ] {
            let json = serde_json::to_string(level).expect("should succeed");
            let deser: SecurityLevel = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(*level, deser);
        }
    }

    #[test]
    fn test_response_status_serde() {
        use std::sync::Arc;

        let success = ResponseStatus::Success;
        let json = serde_json::to_string(&success).expect("should succeed");
        let deser: ResponseStatus = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(success, deser);

        let error = ResponseStatus::Error {
            code: Arc::from("E001"),
            message: "Something went wrong".to_string(),
        };
        let json = serde_json::to_string(&error).expect("should succeed");
        let deser: ResponseStatus = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(error, deser);

        let timeout = ResponseStatus::Timeout;
        let json = serde_json::to_string(&timeout).expect("should succeed");
        let deser: ResponseStatus = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(timeout, deser);

        let unavailable = ResponseStatus::ServiceUnavailable;
        let json = serde_json::to_string(&unavailable).expect("should succeed");
        let deser: ResponseStatus = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(unavailable, deser);
    }

    #[test]
    fn test_health_status_serde() {
        for status in &[
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unhealthy,
            HealthStatus::Unknown,
        ] {
            let json = serde_json::to_string(status).expect("should succeed");
            let deser: HealthStatus = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(*status, deser);
        }
    }

    #[test]
    fn test_primal_response_default() {
        let resp = PrimalResponse::default();
        assert_eq!(resp.status, ResponseStatus::Success);
        assert_eq!(resp.payload, serde_json::Value::Null);
        assert!(resp.metadata.is_empty());
    }

    #[test]
    fn test_ecosystem_request_default() {
        let req = EcosystemRequest::default();
        assert_eq!(req.source_service.as_ref(), "unknown");
        assert_eq!(req.target_service.as_ref(), "unknown");
        assert_eq!(req.operation.as_ref(), "unknown");
        assert_eq!(req.payload, serde_json::Value::Null);
        assert!(req.metadata.is_empty());
    }

    #[test]
    fn test_security_context_default() {
        let ctx = SecurityContext::default();
        assert!(ctx.auth_token.is_none());
        assert_eq!(ctx.identity.as_ref(), "anonymous");
        assert!(ctx.permissions.is_empty());
        assert_eq!(ctx.security_level, SecurityLevel::Public);
    }

    #[test]
    fn test_primal_context_default() {
        let ctx = PrimalContext::default();
        assert_eq!(ctx.user_id.as_ref(), "system");
        assert_eq!(ctx.device_id.as_ref(), "unknown");
        assert!(!ctx.session_id.is_empty());
        assert_eq!(ctx.security_level, SecurityLevel::Internal);
        assert!(ctx.biome_id.is_none());
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_network_location_default() {
        let loc = NetworkLocation::default();
        assert!(loc.ip_address.is_none());
        assert!(loc.region.is_none());
        assert!(loc.zone.is_none());
        assert!(loc.segment.is_none());
    }

    #[test]
    fn test_service_mesh_status_default() {
        let status = ServiceMeshStatus::default();
        assert!(!status.connected);
        assert!(status.service_mesh_endpoint.is_none());
        assert!(status.registration_time.is_none());
        assert!(status.last_heartbeat.is_none());
        assert!(status.metadata.is_empty());
    }

    #[test]
    fn test_ecosystem_request_serde() {
        use std::sync::Arc;

        let req = EcosystemRequest {
            request_id: uuid::Uuid::new_v4(),
            source_service: Arc::from("squirrel"),
            target_service: Arc::from("songbird"),
            operation: Arc::from("discover"),
            payload: serde_json::json!({"key": "value"}),
            security_context: SecurityContext::default(),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&req).expect("should succeed");
        let deser: EcosystemRequest = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deser.source_service.as_ref(), "squirrel");
        assert_eq!(deser.target_service.as_ref(), "songbird");
        assert_eq!(deser.operation.as_ref(), "discover");
    }

    #[test]
    fn test_primal_capability_serde() {
        let cap = PrimalCapability::ModelInference {
            models: vec!["gpt-4".to_string(), "claude".to_string()],
        };
        let json = serde_json::to_string(&cap).expect("should succeed");
        let deser: PrimalCapability = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(cap, deser);
    }

    #[test]
    fn test_primal_dependency_serde() {
        let dep = PrimalDependency {
            primal_type: PrimalType::Any,
            name: "security-provider".to_string(),
            capabilities: vec!["authentication".to_string(), "encryption".to_string()],
            required: true,
            min_version: Some("1.0.0".to_string()),
        };
        let json = serde_json::to_string(&dep).expect("should succeed");
        let deser: PrimalDependency = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(dep, deser);
    }

    #[test]
    fn test_all_primal_capabilities_serde() {
        let caps: Vec<PrimalCapability> = vec![
            PrimalCapability::ContainerRuntime {
                orchestrators: vec!["kubernetes".to_string()],
            },
            PrimalCapability::ServerlessExecution {
                languages: vec!["rust".to_string()],
            },
            PrimalCapability::GpuAcceleration { cuda_support: true },
            PrimalCapability::NativeExecution {
                architectures: vec!["x86_64".to_string()],
            },
            PrimalCapability::WasmExecution { wasi_support: true },
            PrimalCapability::Authentication {
                methods: vec!["oauth2".to_string()],
            },
            PrimalCapability::Encryption {
                algorithms: vec!["AES-256".to_string()],
            },
            PrimalCapability::KeyManagement { hsm_support: false },
            PrimalCapability::ThreatDetection { ml_enabled: true },
            PrimalCapability::Compliance {
                frameworks: vec!["GDPR".to_string()],
            },
            PrimalCapability::FileSystem { supports_zfs: true },
            PrimalCapability::ObjectStorage {
                backends: vec!["s3".to_string()],
            },
            PrimalCapability::DataReplication {
                consistency: "strong".to_string(),
            },
            PrimalCapability::VolumeManagement {
                protocols: vec!["nfs".to_string()],
            },
            PrimalCapability::BackupRestore { incremental: true },
            PrimalCapability::ServiceDiscovery {
                protocols: vec!["dns".to_string()],
            },
            PrimalCapability::NetworkRouting {
                protocols: vec!["http".to_string()],
            },
            PrimalCapability::LoadBalancing {
                algorithms: vec!["round-robin".to_string()],
            },
            PrimalCapability::CircuitBreaking { enabled: true },
            PrimalCapability::ModelInference {
                models: vec!["gpt-4".to_string()],
            },
            PrimalCapability::AgentFramework { mcp_support: true },
            PrimalCapability::MachineLearning {
                training_support: false,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".to_string()],
            },
            PrimalCapability::Orchestration {
                primals: vec!["squirrel".to_string()],
            },
            PrimalCapability::Manifests {
                formats: vec!["yaml".to_string()],
            },
            PrimalCapability::Deployment {
                strategies: vec!["rolling".to_string()],
            },
            PrimalCapability::Monitoring {
                metrics: vec!["prometheus".to_string()],
            },
        ];

        for cap in &caps {
            let json = serde_json::to_string(cap).expect("should succeed");
            let deser: PrimalCapability = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(*cap, deser, "Failed roundtrip for: {cap:?}");
        }
    }

    #[test]
    fn test_primal_type_hash_works_in_collections() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PrimalType::Squirrel);
        set.insert(PrimalType::Songbird);
        set.insert(PrimalType::Squirrel);
        assert_eq!(set.len(), 2);
        assert!(set.contains(&PrimalType::Squirrel));
        assert!(set.contains(&PrimalType::Songbird));
    }
}
