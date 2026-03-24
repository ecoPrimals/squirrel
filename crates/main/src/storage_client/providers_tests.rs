// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for storage provider management

use super::providers::*;
use crate::universal_primal_ecosystem::DiscoveredPrimal;
use chrono::Utc;
use universal_patterns::{PrimalContext, PrimalHealth, PrimalType};

#[test]
fn test_storage_provider_from_discovered_primal() {
    let primal = DiscoveredPrimal {
        id: "test-storage".to_string(),
        instance_id: "inst-storage-456".to_string(),
        primal_type: PrimalType::Storage,
        endpoint: "http://localhost:7070".to_string(),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        context: PrimalContext::default(),
        port_info: None,
    };

    let provider = StorageProvider::from_discovered_primal(&primal);

    assert_eq!(provider.provider_id, "inst-storage-456");
    assert_eq!(provider.metadata.name, "test-storage");
    assert_eq!(provider.metadata.version, "unknown");
    assert!(provider.metadata.protocols.contains(&"http".to_string()));
    assert_eq!(provider.metadata.regions, vec!["local"]);
    assert!(provider.metadata.compliance.is_empty());
    assert_eq!(provider.health.health_score, 1.0);
    assert_eq!(provider.health.current_latency_ms, 50.0);
    assert_eq!(provider.health.current_throughput_mbps, 100.0);
    assert_eq!(provider.health.availability_percent, 99.9);
    assert_eq!(provider.routing_score, 0.8);
}

#[test]
fn test_storage_provider_metadata_creation() {
    let metadata = StorageProviderMetadata {
        name: "Object Store".to_string(),
        version: "3.0.0".to_string(),
        protocols: vec!["s3".to_string(), "gcs".to_string()],
        regions: vec!["us-west-2".to_string()],
        compliance: vec!["hipaa".to_string()],
    };

    assert_eq!(metadata.name, "Object Store");
    assert_eq!(metadata.version, "3.0.0");
    assert_eq!(metadata.protocols.len(), 2);
    assert_eq!(metadata.regions.len(), 1);
    assert_eq!(metadata.compliance.len(), 1);
}

#[test]
fn test_storage_provider_health_creation() {
    let health = StorageProviderHealth {
        health_score: 0.98,
        current_latency_ms: 10.0,
        current_throughput_mbps: 500.0,
        availability_percent: 99.999,
        last_check: Utc::now(),
    };

    assert_eq!(health.health_score, 0.98);
    assert_eq!(health.current_latency_ms, 10.0);
    assert_eq!(health.current_throughput_mbps, 500.0);
    assert_eq!(health.availability_percent, 99.999);
}

#[test]
fn test_storage_provider_serialization() {
    let primal = DiscoveredPrimal {
        id: "serialize-storage".to_string(),
        instance_id: "inst-serialize-storage".to_string(),
        primal_type: PrimalType::Storage,
        endpoint: "http://test:7070".to_string(),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        context: PrimalContext::default(),
        port_info: None,
    };

    let provider = StorageProvider::from_discovered_primal(&primal);
    let json = serde_json::to_string(&provider).expect("should succeed");
    let deserialized: StorageProvider = serde_json::from_str(&json).expect("should succeed");

    assert_eq!(provider.provider_id, deserialized.provider_id);
    assert_eq!(provider.metadata.name, deserialized.metadata.name);
    assert_eq!(provider.routing_score, deserialized.routing_score);
}

#[test]
fn test_storage_provider_health_degraded() {
    let health = StorageProviderHealth {
        health_score: 0.30,
        current_latency_ms: 5000.0,
        current_throughput_mbps: 5.0,
        availability_percent: 90.0,
        last_check: Utc::now(),
    };

    assert!(health.health_score < 0.5);
    assert!(health.current_latency_ms > 1000.0);
    assert!(health.current_throughput_mbps < 10.0);
}
