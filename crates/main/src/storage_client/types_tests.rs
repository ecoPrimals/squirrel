//! Tests for storage client types

#[cfg(test)]
mod tests {
    use crate::storage_client::types::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn test_storage_client_config_default() {
        let config = StorageClientConfig::default();

        assert_eq!(config.max_retries, 3);
        assert_eq!(config.operation_timeout.as_secs(), 300);
        assert!(!config.preferred_capabilities.is_empty());
        assert_eq!(config.performance_requirements.durability_nines, 11);
    }

    #[test]
    fn test_data_classification_variants() {
        let public = DataClassification::Public;
        let internal = DataClassification::Internal;
        let confidential = DataClassification::Confidential;
        let restricted = DataClassification::Restricted;

        assert!(matches!(public, DataClassification::Public));
        assert!(matches!(internal, DataClassification::Internal));
        assert!(matches!(confidential, DataClassification::Confidential));
        assert!(matches!(restricted, DataClassification::Restricted));
    }

    #[test]
    fn test_access_frequency_variants() {
        let hot = AccessFrequency::Hot;
        let warm = AccessFrequency::Warm;
        let cold = AccessFrequency::Cold;
        let archive = AccessFrequency::Archive;

        assert!(matches!(hot, AccessFrequency::Hot));
        assert!(matches!(warm, AccessFrequency::Warm));
        assert!(matches!(cold, AccessFrequency::Cold));
        assert!(matches!(archive, AccessFrequency::Archive));
    }

    #[test]
    fn test_sharing_scope_variants() {
        let private = SharingScope::Private;
        let team = SharingScope::Team;
        let org = SharingScope::Organization;
        let public = SharingScope::Public;

        assert!(matches!(private, SharingScope::Private));
        assert!(matches!(team, SharingScope::Team));
        assert!(matches!(org, SharingScope::Organization));
        assert!(matches!(public, SharingScope::Public));
    }

    #[test]
    fn test_storage_capability_type_object_storage() {
        let obj_storage = StorageCapabilityType::ObjectStorage {
            compression: true,
            encryption: true,
            replication: false,
        };

        match obj_storage {
            StorageCapabilityType::ObjectStorage {
                compression,
                encryption,
                replication,
            } => {
                assert!(compression);
                assert!(encryption);
                assert!(!replication);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_storage_capability_type_file_system() {
        let fs = StorageCapabilityType::FileSystem {
            posix_compliance: true,
            atomic_operations: false,
        };

        match fs {
            StorageCapabilityType::FileSystem {
                posix_compliance,
                atomic_operations,
            } => {
                assert!(posix_compliance);
                assert!(!atomic_operations);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_storage_capability_type_database() {
        let db = StorageCapabilityType::Database {
            acid_compliance: true,
            query_capabilities: vec!["SQL".to_string(), "NoSQL".to_string()],
        };

        match db {
            StorageCapabilityType::Database {
                acid_compliance,
                query_capabilities,
            } => {
                assert!(acid_compliance);
                assert_eq!(query_capabilities.len(), 2);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_storage_capability_type_cache() {
        let cache = StorageCapabilityType::Cache {
            ttl_support: true,
            eviction_policies: vec!["LRU".to_string(), "LFU".to_string()],
        };

        match cache {
            StorageCapabilityType::Cache {
                ttl_support,
                eviction_policies,
            } => {
                assert!(ttl_support);
                assert_eq!(eviction_policies.len(), 2);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_storage_capability_type_archive() {
        let archive = StorageCapabilityType::Archive {
            retrieval_time: std::time::Duration::from_secs(3600),
            cost_optimization: true,
        };

        match archive {
            StorageCapabilityType::Archive {
                retrieval_time,
                cost_optimization,
            } => {
                assert_eq!(retrieval_time.as_secs(), 3600);
                assert!(cost_optimization);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_storage_operation_store() {
        let op = StorageOperation::Store;
        assert!(matches!(op, StorageOperation::Store));
    }

    #[test]
    fn test_storage_operation_retrieve() {
        let op = StorageOperation::Retrieve;
        assert!(matches!(op, StorageOperation::Retrieve));
    }

    #[test]
    fn test_storage_operation_list() {
        let op = StorageOperation::List;
        assert!(matches!(op, StorageOperation::List));
    }

    #[test]
    fn test_storage_operation_delete() {
        let op = StorageOperation::Delete;
        assert!(matches!(op, StorageOperation::Delete));
    }

    #[test]
    fn test_storage_operation_copy() {
        let op = StorageOperation::Copy {
            destination: "/backup/file.txt".to_string(),
        };

        match op {
            StorageOperation::Copy { destination } => {
                assert_eq!(destination, "/backup/file.txt");
            }
            _ => panic!("Wrong operation type"),
        }
    }

    #[test]
    fn test_storage_operation_move() {
        let op = StorageOperation::Move {
            destination: "/new/location.txt".to_string(),
        };

        match op {
            StorageOperation::Move { destination } => {
                assert_eq!(destination, "/new/location.txt");
            }
            _ => panic!("Wrong operation type"),
        }
    }

    #[test]
    fn test_storage_operation_snapshot() {
        let op = StorageOperation::Snapshot;
        assert!(matches!(op, StorageOperation::Snapshot));
    }

    #[test]
    fn test_storage_operation_restore() {
        let op = StorageOperation::Restore {
            snapshot_id: "snap-123".to_string(),
        };

        match op {
            StorageOperation::Restore { snapshot_id } => {
                assert_eq!(snapshot_id, "snap-123");
            }
            _ => panic!("Wrong operation type"),
        }
    }

    #[test]
    fn test_performance_requirements_creation() {
        let perf = PerformanceRequirements {
            max_latency_ms: 100,
            min_throughput_mbps: 50.0,
            availability_sla: 0.999,
            durability_nines: 11,
        };

        assert_eq!(perf.max_latency_ms, 100);
        assert_eq!(perf.min_throughput_mbps, 50.0);
        assert_eq!(perf.availability_sla, 0.999);
        assert_eq!(perf.durability_nines, 11);
    }

    #[test]
    fn test_storage_capability_preference_creation() {
        let pref = StorageCapabilityPreference {
            capability: StorageCapabilityType::ObjectStorage {
                compression: true,
                encryption: true,
                replication: true,
            },
            weight: 0.8,
            required: true,
        };

        assert_eq!(pref.weight, 0.8);
        assert!(pref.required);
    }

    #[test]
    fn test_ai_request_context_creation() {
        let context = AIRequestContext {
            access_frequency: AccessFrequency::Hot,
            data_lifetime: std::time::Duration::from_secs(86400),
            sharing_scope: SharingScope::Team,
            processing_hints: vec!["analytics".to_string(), "ml".to_string()],
        };

        assert!(matches!(context.access_frequency, AccessFrequency::Hot));
        assert_eq!(context.data_lifetime.as_secs(), 86400);
        assert!(matches!(context.sharing_scope, SharingScope::Team));
        assert_eq!(context.processing_hints.len(), 2);
    }

    #[test]
    fn test_universal_storage_request_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("content_type".to_string(), "application/json".to_string());

        let request = UniversalStorageRequest {
            request_id: Uuid::new_v4(),
            operation: StorageOperation::Store,
            object_key: "/data/file.json".to_string(),
            data: Some(vec![1, 2, 3, 4]),
            metadata,
            classification: DataClassification::Internal,
            requirements: PerformanceRequirements {
                max_latency_ms: 100,
                min_throughput_mbps: 10.0,
                availability_sla: 0.99,
                durability_nines: 9,
            },
            ai_context: AIRequestContext {
                access_frequency: AccessFrequency::Warm,
                data_lifetime: std::time::Duration::from_secs(3600),
                sharing_scope: SharingScope::Private,
                processing_hints: vec![],
            },
        };

        assert_eq!(request.object_key, "/data/file.json");
        assert_eq!(request.data, Some(vec![1, 2, 3, 4]));
        assert!(matches!(
            request.classification,
            DataClassification::Internal
        ));
    }

    #[test]
    fn test_universal_storage_request_serialization() {
        let request = UniversalStorageRequest {
            request_id: Uuid::new_v4(),
            operation: StorageOperation::Retrieve,
            object_key: "/test.txt".to_string(),
            data: None,
            metadata: HashMap::new(),
            classification: DataClassification::Public,
            requirements: PerformanceRequirements {
                max_latency_ms: 50,
                min_throughput_mbps: 5.0,
                availability_sla: 0.95,
                durability_nines: 9,
            },
            ai_context: AIRequestContext {
                access_frequency: AccessFrequency::Cold,
                data_lifetime: std::time::Duration::from_secs(7200),
                sharing_scope: SharingScope::Public,
                processing_hints: vec![],
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("/test.txt"));

        let deserialized: UniversalStorageRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.object_key, "/test.txt");
        assert!(matches!(deserialized.operation, StorageOperation::Retrieve));
    }

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics {
            latency_ms: 45.5,
            throughput_mbps: 125.3,
            provider_health: 0.98,
            estimated_cost: 0.05,
        };

        assert_eq!(metrics.latency_ms, 45.5);
        assert_eq!(metrics.throughput_mbps, 125.3);
        assert_eq!(metrics.provider_health, 0.98);
        assert_eq!(metrics.estimated_cost, 0.05);
    }

    #[test]
    fn test_access_pattern_creation() {
        let pattern = AccessPattern {
            pattern_type: "sequential".to_string(),
            confidence: 0.87,
            optimizations: vec!["prefetch".to_string(), "cache".to_string()],
        };

        assert_eq!(pattern.pattern_type, "sequential");
        assert_eq!(pattern.confidence, 0.87);
        assert_eq!(pattern.optimizations.len(), 2);
    }

    #[test]
    fn test_ai_storage_insights_creation() {
        let insights = AIStorageInsights {
            confidence_score: 0.95,
            optimizations: vec!["use_compression".to_string()],
            alternative_providers: vec!["s3".to_string(), "gcs".to_string()],
            access_predictions: vec![AccessPattern {
                pattern_type: "random".to_string(),
                confidence: 0.75,
                optimizations: vec!["indexing".to_string()],
            }],
            cost_recommendations: vec!["move_to_cold_storage".to_string()],
        };

        assert_eq!(insights.confidence_score, 0.95);
        assert_eq!(insights.optimizations.len(), 1);
        assert_eq!(insights.alternative_providers.len(), 2);
        assert_eq!(insights.access_predictions.len(), 1);
        assert_eq!(insights.cost_recommendations.len(), 1);
    }

    #[test]
    fn test_universal_storage_response_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), "1024".to_string());

        let response = UniversalStorageResponse {
            request_id: Uuid::new_v4(),
            success: true,
            data: Some(vec![5, 6, 7, 8]),
            metadata,
            provider_id: "aws-s3".to_string(),
            performance: PerformanceMetrics {
                latency_ms: 35.2,
                throughput_mbps: 150.0,
                provider_health: 0.99,
                estimated_cost: 0.03,
            },
            ai_insights: AIStorageInsights {
                confidence_score: 0.92,
                optimizations: vec![],
                alternative_providers: vec![],
                access_predictions: vec![],
                cost_recommendations: vec![],
            },
            error: None,
        };

        assert!(response.success);
        assert_eq!(response.provider_id, "aws-s3");
        assert_eq!(response.data, Some(vec![5, 6, 7, 8]));
        assert_eq!(response.performance.latency_ms, 35.2);
    }

    #[test]
    fn test_universal_storage_response_with_error() {
        let response = UniversalStorageResponse {
            request_id: Uuid::new_v4(),
            success: false,
            data: None,
            metadata: HashMap::new(),
            provider_id: "provider-1".to_string(),
            performance: PerformanceMetrics {
                latency_ms: 1000.0,
                throughput_mbps: 0.0,
                provider_health: 0.5,
                estimated_cost: 0.0,
            },
            ai_insights: AIStorageInsights {
                confidence_score: 0.2,
                optimizations: vec!["retry_with_different_provider".to_string()],
                alternative_providers: vec!["provider-2".to_string()],
                access_predictions: vec![],
                cost_recommendations: vec![],
            },
            error: Some("Connection timeout".to_string()),
        };

        assert!(!response.success);
        assert_eq!(response.error, Some("Connection timeout".to_string()));
        assert_eq!(response.performance.provider_health, 0.5);
    }

    #[test]
    fn test_storage_config_serialization() {
        let config = StorageClientConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        let deserialized: StorageClientConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_retries, 3);
        assert!(matches!(
            deserialized.data_classification,
            DataClassification::Internal
        ));
    }
}
