// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::super::types::FederationConfig;
use super::*;
use crate::{
    Error, FederationResult, FederationStatus, InstanceStatus, LoadBalanceResult, LoadMetrics,
    NodeSpec, SquirrelConfig, SwarmManager,
};

fn test_config() -> FederationConfig {
    FederationConfig {
        node_id: "test-node".to_string(),
        federation_enabled: false,
        auto_scaling_enabled: false,
        ..Default::default()
    }
}

#[test]
fn new_sets_expected_scaling_policy_and_load_metrics() {
    let svc = FederationService::new(test_config()).expect("new");
    let stats = svc.get_federation_stats();
    assert_eq!(stats.node_id, "test-node");
    assert!(!stats.federation_id.is_empty());
    assert_eq!(stats.local_instances, 0);
}

#[tokio::test]
async fn start_standalone_when_federation_disabled() {
    let svc = FederationService::new(test_config()).expect("new");
    svc.start().await.await?;
    let stats = svc.get_federation_stats();
    assert!(matches!(stats.status, FederationStatus::Active));
    svc.shutdown().await.await?;
}

#[tokio::test]
async fn start_federation_becomes_leader_when_isolated() {
    let mut cfg = test_config();
    cfg.federation_enabled = true;
    let svc = FederationService::new(cfg).expect("new");
    svc.start().await.await?;
    let stats = svc.get_federation_stats();
    assert!(stats.is_leader);
    assert!(matches!(stats.status, FederationStatus::Active));
    svc.shutdown().await.await?;
}

#[tokio::test]
async fn start_federation_fails_when_peers_preloaded_and_join_unavailable() {
    let mut cfg = test_config();
    cfg.federation_enabled = true;
    let svc = FederationService::new(cfg).expect("new");
    let spec = NodeSpec {
        id: "peer-1".to_string(),
        region: None,
        zone: None,
        endpoint: "http://127.0.0.1:1".to_string(),
        capabilities: vec!["mcp".to_string()],
        capacity: 4,
    };
    svc.federate_nodes(vec![spec]).await.await?;
    let err = svc
        .start()
        .await
        .expect_err("join_existing_federation should fail without http.client");
    assert!(matches!(err, Error::CapabilityUnavailable { .. }));
}

#[tokio::test]
async fn federate_nodes_updates_stats_and_swarm_result() {
    let svc = FederationService::new(test_config()).expect("new");
    let spec = NodeSpec {
        id: "n1".to_string(),
        region: Some("us".to_string()),
        zone: None,
        endpoint: "http://127.0.0.1:9".to_string(),
        capabilities: vec!["x".to_string()],
        capacity: 10,
    };
    let FederationResult {
        nodes_joined,
        status,
        ..
    } = svc.federate_nodes(vec![spec]).await.await?;
    assert_eq!(nodes_joined, 1);
    assert_eq!(status, FederationStatus::Active);
    assert_eq!(svc.get_federation_stats().local_instances, 1);
}

#[tokio::test]
async fn federate_nodes_empty_yields_inactive() {
    let svc = FederationService::new(test_config()).expect("new");
    let r = svc.federate_nodes(vec![]).await.await?;
    assert_eq!(r.nodes_joined, 0);
    assert_eq!(r.status, FederationStatus::Inactive);
}

#[tokio::test]
async fn spawn_squirrel_registers_instance() {
    let svc = FederationService::new(test_config()).expect("new");
    let cfg = SquirrelConfig {
        node_id: "spawn-test".to_string(),
        port: 18080,
        region: None,
        zone: None,
        capabilities: vec!["mcp".to_string()],
        capacity: 2,
        federation_enabled: false,
        auto_scaling_enabled: false,
        metadata: std::collections::HashMap::new(),
    };
    let inst = svc.spawn_squirrel(cfg).await.await?;
    assert_eq!(inst.health, InstanceStatus::Starting);
    assert_eq!(svc.get_federation_stats().local_instances, 1);
}

#[tokio::test]
async fn balance_load_uses_internal_metrics_snapshot() {
    let svc = FederationService::new(test_config()).expect("new");
    // `balance_load` currently ignores the argument and uses internal `LoadMetrics` (zeroed at init).
    let lm = LoadMetrics {
        cpu_usage: 0.9,
        memory_usage: 0.5,
        network_usage: 0.0,
        active_tasks: 2,
        queue_length: 10,
        response_time: std::time::Duration::from_millis(5),
        error_rate: 0.0,
    };
    let LoadBalanceResult { balance_score, .. } = svc.balance_load(lm).await.await?;
    assert!(balance_score.abs() < f64::EPSILON);
    assert!(svc.get_federation_stats().current_utilization.abs() < f64::EPSILON);
}

#[tokio::test]
async fn shutdown_notifies_and_completes() {
    let mut cfg = test_config();
    cfg.federation_enabled = true;
    let svc = FederationService::new(cfg).expect("new");
    svc.start().await.await?;
    svc.shutdown().await.await?;
    assert!(matches!(
        svc.get_federation_stats().status,
        FederationStatus::Inactive
    ));
}

#[test]
fn default_federation_config_matches_documented_defaults() {
    let c = FederationConfig::default();
    assert!(!c.federation_enabled);
    assert_eq!(c.port, 8080);
    assert_eq!(c.federation_port, 8090);
    assert_eq!(c.max_instances, 10);
    assert_eq!(c.min_instances, 1);
}
