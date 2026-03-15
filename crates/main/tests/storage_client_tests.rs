// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for Universal Storage Client types and configurations
//!
//! Tests storage types, configurations, capability preferences, and data classification.

use squirrel::storage_client::{
    DataClassification, PerformanceRequirements, StorageCapabilityPreference,
    StorageCapabilityType, StorageClientConfig,
};
use std::time::Duration;

#[test]
fn test_storage_client_config_default() {
    let config = StorageClientConfig::default();

    assert!(config.operation_timeout > Duration::ZERO);
    assert!(config.max_retries > 0);
}

#[test]
fn test_data_classification_public() {
    let class = DataClassification::Public;

    assert!(matches!(class, DataClassification::Public));
}

#[test]
fn test_data_classification_internal() {
    let class = DataClassification::Internal;

    assert!(matches!(class, DataClassification::Internal));
}

#[test]
fn test_data_classification_confidential() {
    let class = DataClassification::Confidential;

    assert!(matches!(class, DataClassification::Confidential));
}

#[test]
fn test_data_classification_restricted() {
    let class = DataClassification::Restricted;

    assert!(matches!(class, DataClassification::Restricted));
}

#[test]
fn test_storage_capability_object_storage() {
    let cap = StorageCapabilityType::ObjectStorage {
        compression: true,
        encryption: true,
        replication: true,
    };

    assert!(matches!(cap, StorageCapabilityType::ObjectStorage { .. }));
}

#[test]
fn test_storage_capability_file_system() {
    let cap = StorageCapabilityType::FileSystem {
        posix_compliance: true,
        atomic_operations: true,
    };

    assert!(matches!(cap, StorageCapabilityType::FileSystem { .. }));
}

#[test]
fn test_storage_capability_database() {
    let cap = StorageCapabilityType::Database {
        acid_compliance: true,
        query_capabilities: vec!["SQL".to_string(), "NoSQL".to_string()],
    };

    assert!(matches!(cap, StorageCapabilityType::Database { .. }));
}

#[test]
fn test_storage_capability_cache() {
    let cap = StorageCapabilityType::Cache {
        ttl_support: true,
        eviction_policies: vec!["LRU".to_string(), "LFU".to_string()],
    };

    assert!(matches!(cap, StorageCapabilityType::Cache { .. }));
}

#[test]
fn test_storage_capability_archive() {
    let cap = StorageCapabilityType::Archive {
        retrieval_time: Duration::from_secs(3600),
        cost_optimization: true,
    };

    assert!(matches!(cap, StorageCapabilityType::Archive { .. }));
}

#[test]
fn test_performance_requirements_default() {
    let perf = PerformanceRequirements {
        max_latency_ms: 1000,
        min_throughput_mbps: 10.0,
        availability_sla: 0.99,
        durability_nines: 9,
    };

    assert!(perf.max_latency_ms > 0);
    assert!(perf.availability_sla >= 0.0 && perf.availability_sla <= 1.0);
    assert!(perf.durability_nines <= 11);
}

#[test]
fn test_performance_requirements_custom() {
    let perf = PerformanceRequirements {
        max_latency_ms: 100,
        min_throughput_mbps: 10.0,
        availability_sla: 0.999,
        durability_nines: 11,
    };

    assert_eq!(perf.max_latency_ms, 100);
    assert_eq!(perf.min_throughput_mbps, 10.0);
    assert_eq!(perf.availability_sla, 0.999);
    assert_eq!(perf.durability_nines, 11);
}

#[test]
fn test_storage_capability_preference() {
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
fn test_storage_capability_preference_optional() {
    let pref = StorageCapabilityPreference {
        capability: StorageCapabilityType::Cache {
            ttl_support: true,
            eviction_policies: vec!["LRU".to_string()],
        },
        weight: 0.3,
        required: false,
    };

    assert_eq!(pref.weight, 0.3);
    assert!(!pref.required);
}

#[test]
fn test_data_classification_clone() {
    let class1 = DataClassification::Confidential;
    let class2 = class1.clone();

    assert!(matches!(class2, DataClassification::Confidential));
}

#[test]
fn test_data_classification_debug() {
    let class = DataClassification::Public;
    let debug_str = format!("{:?}", class);

    assert!(!debug_str.is_empty());
}

#[test]
fn test_storage_capability_debug() {
    let cap = StorageCapabilityType::ObjectStorage {
        compression: true,
        encryption: false,
        replication: true,
    };
    let debug_str = format!("{:?}", cap);

    assert!(!debug_str.is_empty());
}

#[test]
fn test_performance_requirements_debug() {
    let perf = PerformanceRequirements {
        max_latency_ms: 1000,
        min_throughput_mbps: 10.0,
        availability_sla: 0.99,
        durability_nines: 9,
    };
    let debug_str = format!("{:?}", perf);

    assert!(!debug_str.is_empty());
}

#[test]
fn test_storage_config_debug() {
    let config = StorageClientConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(!debug_str.is_empty());
}

#[test]
fn test_storage_config_clone() {
    let config1 = StorageClientConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.operation_timeout, config2.operation_timeout);
    assert_eq!(config1.max_retries, config2.max_retries);
}

#[test]
fn test_performance_requirements_clone() {
    let perf1 = PerformanceRequirements {
        max_latency_ms: 1000,
        min_throughput_mbps: 10.0,
        availability_sla: 0.99,
        durability_nines: 9,
    };
    let perf2 = perf1.clone();

    assert_eq!(perf1.max_latency_ms, perf2.max_latency_ms);
    assert_eq!(perf1.availability_sla, perf2.availability_sla);
}

#[test]
fn test_storage_capability_clone() {
    let cap1 = StorageCapabilityType::ObjectStorage {
        compression: true,
        encryption: true,
        replication: false,
    };
    let cap2 = cap1.clone();

    assert!(matches!(cap2, StorageCapabilityType::ObjectStorage { .. }));
}

#[test]
fn test_storage_capability_preference_clone() {
    let pref1 = StorageCapabilityPreference {
        capability: StorageCapabilityType::FileSystem {
            posix_compliance: true,
            atomic_operations: true,
        },
        weight: 0.5,
        required: true,
    };
    let pref2 = pref1.clone();

    assert_eq!(pref1.weight, pref2.weight);
    assert_eq!(pref1.required, pref2.required);
}

#[test]
fn test_high_performance_requirements() {
    let perf = PerformanceRequirements {
        max_latency_ms: 10,
        min_throughput_mbps: 1000.0,
        availability_sla: 0.9999,
        durability_nines: 11,
    };

    assert!(perf.max_latency_ms < 100);
    assert!(perf.min_throughput_mbps > 100.0);
    assert!(perf.availability_sla > 0.999);
}

#[test]
fn test_low_performance_requirements() {
    let perf = PerformanceRequirements {
        max_latency_ms: 10000,
        min_throughput_mbps: 0.1,
        availability_sla: 0.95,
        durability_nines: 9,
    };

    assert!(perf.max_latency_ms > 1000);
    assert!(perf.min_throughput_mbps < 1.0);
}

#[test]
fn test_object_storage_all_features() {
    let cap = StorageCapabilityType::ObjectStorage {
        compression: true,
        encryption: true,
        replication: true,
    };

    if let StorageCapabilityType::ObjectStorage {
        compression,
        encryption,
        replication,
    } = cap
    {
        assert!(compression);
        assert!(encryption);
        assert!(replication);
    }
}

#[test]
fn test_object_storage_no_features() {
    let cap = StorageCapabilityType::ObjectStorage {
        compression: false,
        encryption: false,
        replication: false,
    };

    if let StorageCapabilityType::ObjectStorage {
        compression,
        encryption,
        replication,
    } = cap
    {
        assert!(!compression);
        assert!(!encryption);
        assert!(!replication);
    }
}

#[test]
fn test_file_system_posix() {
    let cap = StorageCapabilityType::FileSystem {
        posix_compliance: true,
        atomic_operations: true,
    };

    if let StorageCapabilityType::FileSystem {
        posix_compliance,
        atomic_operations,
    } = cap
    {
        assert!(posix_compliance);
        assert!(atomic_operations);
    }
}

#[test]
fn test_database_with_queries() {
    let cap = StorageCapabilityType::Database {
        acid_compliance: true,
        query_capabilities: vec!["SQL".to_string(), "JOIN".to_string(), "INDEX".to_string()],
    };

    if let StorageCapabilityType::Database {
        acid_compliance,
        query_capabilities,
    } = cap
    {
        assert!(acid_compliance);
        assert_eq!(query_capabilities.len(), 3);
    }
}

#[test]
fn test_cache_with_eviction() {
    let cap = StorageCapabilityType::Cache {
        ttl_support: true,
        eviction_policies: vec!["LRU".to_string(), "LFU".to_string(), "FIFO".to_string()],
    };

    if let StorageCapabilityType::Cache {
        ttl_support,
        eviction_policies,
    } = cap
    {
        assert!(ttl_support);
        assert_eq!(eviction_policies.len(), 3);
    }
}

#[test]
fn test_archive_long_retrieval() {
    let cap = StorageCapabilityType::Archive {
        retrieval_time: Duration::from_secs(86400), // 24 hours
        cost_optimization: true,
    };

    if let StorageCapabilityType::Archive {
        retrieval_time,
        cost_optimization,
    } = cap
    {
        assert!(retrieval_time > Duration::from_secs(3600));
        assert!(cost_optimization);
    }
}

#[test]
fn test_weight_bounds() {
    let pref_min = StorageCapabilityPreference {
        capability: StorageCapabilityType::ObjectStorage {
            compression: true,
            encryption: true,
            replication: true,
        },
        weight: 0.0,
        required: false,
    };

    let pref_max = StorageCapabilityPreference {
        capability: StorageCapabilityType::ObjectStorage {
            compression: true,
            encryption: true,
            replication: true,
        },
        weight: 1.0,
        required: true,
    };

    assert_eq!(pref_min.weight, 0.0);
    assert_eq!(pref_max.weight, 1.0);
}

#[test]
fn test_storage_client_config_custom() {
    let config = StorageClientConfig {
        operation_timeout: Duration::from_secs(30),
        max_retries: 5,
        preferred_capabilities: vec![],
        data_classification: DataClassification::Confidential,
        performance_requirements: PerformanceRequirements {
            max_latency_ms: 1000,
            min_throughput_mbps: 10.0,
            availability_sla: 0.99,
            durability_nines: 9,
        },
    };

    assert_eq!(config.operation_timeout, Duration::from_secs(30));
    assert_eq!(config.max_retries, 5);
}

#[test]
fn test_storage_client_config_with_preferences() {
    let config = StorageClientConfig {
        operation_timeout: Duration::from_secs(60),
        max_retries: 3,
        preferred_capabilities: vec![
            StorageCapabilityPreference {
                capability: StorageCapabilityType::ObjectStorage {
                    compression: true,
                    encryption: true,
                    replication: true,
                },
                weight: 1.0,
                required: true,
            },
            StorageCapabilityPreference {
                capability: StorageCapabilityType::Cache {
                    ttl_support: true,
                    eviction_policies: vec!["LRU".to_string()],
                },
                weight: 0.5,
                required: false,
            },
        ],
        data_classification: DataClassification::Public,
        performance_requirements: PerformanceRequirements {
            max_latency_ms: 1000,
            min_throughput_mbps: 10.0,
            availability_sla: 0.99,
            durability_nines: 9,
        },
    };

    assert_eq!(config.preferred_capabilities.len(), 2);
}

#[test]
fn test_all_data_classifications() {
    let classifications = vec![
        DataClassification::Public,
        DataClassification::Internal,
        DataClassification::Confidential,
        DataClassification::Restricted,
    ];

    assert_eq!(classifications.len(), 4);
}

#[test]
fn test_durability_nines() {
    let perf_low = PerformanceRequirements {
        max_latency_ms: 1000,
        min_throughput_mbps: 1.0,
        availability_sla: 0.99,
        durability_nines: 9,
    };

    let perf_high = PerformanceRequirements {
        max_latency_ms: 100,
        min_throughput_mbps: 100.0,
        availability_sla: 0.9999,
        durability_nines: 11,
    };

    assert!(perf_low.durability_nines < perf_high.durability_nines);
}
