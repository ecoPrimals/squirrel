// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for compute provider management

use super::providers::*;
use crate::universal_primal_ecosystem::DiscoveredPrimal;
use chrono::Utc;
use universal_patterns::{PrimalContext, PrimalHealth, PrimalType, SecurityLevel};

#[test]
fn test_compute_provider_from_discovered_primal() {
    let primal = DiscoveredPrimal {
        id: "test-compute".to_string(),
        instance_id: "inst-compute-123".to_string(),
        primal_type: PrimalType::Compute,
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        context: PrimalContext::default(),
        port_info: None,
    };

    let provider = ComputeProvider::from_discovered_primal(&primal);

    assert_eq!(provider.provider_id, "inst-compute-123");
    assert_eq!(provider.metadata.name, "test-compute");
    assert_eq!(provider.metadata.version, "unknown");
    assert_eq!(provider.metadata.architectures, vec!["x86_64"]);
    assert_eq!(provider.metadata.regions, vec!["local"]);
    assert_eq!(provider.health.health_score, 1.0);
    assert_eq!(provider.routing_score, 0.8);
}

#[test]
fn test_compute_provider_metadata() {
    let metadata = ComputeProviderMetadata {
        name: "GPU Compute".to_string(),
        version: "2.0.0".to_string(),
        architectures: vec!["x86_64".to_string(), "arm64".to_string()],
        frameworks: vec![
            "pytorch".to_string(),
            "tensorflow".to_string(),
            "jax".to_string(),
        ],
        regions: vec!["us-west-2".to_string(), "eu-central-1".to_string()],
        compliance: vec!["SOC2".to_string(), "HIPAA".to_string()],
    };

    assert_eq!(metadata.name, "GPU Compute");
    assert_eq!(metadata.architectures.len(), 2);
    assert_eq!(metadata.frameworks.len(), 3);
    assert_eq!(metadata.regions.len(), 2);
    assert_eq!(metadata.compliance.len(), 2);
}

#[test]
fn test_compute_provider_health() {
    let health = ComputeProviderHealth {
        health_score: 0.92,
        cpu_load: 0.65,
        memory_usage: 0.70,
        queue_length: 5,
        avg_execution_time_ms: 1500.0,
        last_check: Utc::now(),
    };

    assert_eq!(health.health_score, 0.92);
    assert_eq!(health.cpu_load, 0.65);
    assert_eq!(health.memory_usage, 0.70);
    assert_eq!(health.queue_length, 5);
    assert_eq!(health.avg_execution_time_ms, 1500.0);
}

#[test]
fn test_compute_provider_serialization() {
    let primal = DiscoveredPrimal {
        id: "serialize-test".to_string(),
        instance_id: "inst-serialize".to_string(),
        primal_type: PrimalType::Compute,
        endpoint: "http://test:8080".to_string(),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        context: PrimalContext::default(),
        port_info: None,
    };

    let provider = ComputeProvider::from_discovered_primal(&primal);
    let json = serde_json::to_string(&provider).unwrap();
    let deserialized: ComputeProvider = serde_json::from_str(&json).unwrap();

    assert_eq!(provider.provider_id, deserialized.provider_id);
    assert_eq!(provider.metadata.name, deserialized.metadata.name);
}

#[test]
fn test_compute_provider_with_gpu() {
    let metadata = ComputeProviderMetadata {
        name: "GPU Cluster".to_string(),
        version: "1.0.0".to_string(),
        architectures: vec!["x86_64".to_string()],
        frameworks: vec!["cuda".to_string(), "pytorch".to_string()],
        regions: vec!["us-east-1".to_string()],
        compliance: vec![],
    };

    assert!(metadata.frameworks.contains(&"cuda".to_string()));
    assert!(metadata.frameworks.contains(&"pytorch".to_string()));
}

#[test]
fn test_compute_provider_health_metrics() {
    let health = ComputeProviderHealth {
        health_score: 0.99,
        cpu_load: 0.20,
        memory_usage: 0.30,
        queue_length: 0,
        avg_execution_time_ms: 500.0,
        last_check: Utc::now(),
    };

    // High health score with low resource usage
    assert!(health.health_score > 0.9);
    assert!(health.cpu_load < 0.5);
    assert!(health.memory_usage < 0.5);
    assert_eq!(health.queue_length, 0);
}

#[test]
fn test_compute_provider_loaded() {
    let health = ComputeProviderHealth {
        health_score: 0.60,
        cpu_load: 0.85,
        memory_usage: 0.90,
        queue_length: 100,
        avg_execution_time_ms: 5000.0,
        last_check: Utc::now(),
    };

    // Provider under heavy load
    assert!(health.cpu_load > 0.8);
    assert!(health.memory_usage > 0.8);
    assert!(health.queue_length > 50);
}

#[test]
fn test_compute_provider_routing_score() {
    let primal = DiscoveredPrimal {
        id: "routing-test".to_string(),
        instance_id: "inst-routing".to_string(),
        primal_type: PrimalType::Compute,
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        context: PrimalContext::default(),
        port_info: None,
    };

    let provider = ComputeProvider::from_discovered_primal(&primal);

    // Default routing score should be 0.8
    assert_eq!(provider.routing_score, 0.8);
}
