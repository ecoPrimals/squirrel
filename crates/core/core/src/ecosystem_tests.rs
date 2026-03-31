// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::{EcosystemService, ServiceStatus};
use crate::monitoring::MonitoringConfig;
use crate::{
    DiscoveryConfig, EcosystemConfig, EcosystemMode, Error, HealthStatus, MonitoringService,
    PrimalCoordinator, Task, TaskPriority, TaskRequirements, TaskType,
};
use chrono::Duration as ChronoDuration;
use ecosystem_api::PrimalType;
use std::collections::HashMap;
use std::sync::Arc;

fn monitoring() -> Arc<MonitoringService> {
    Arc::new(MonitoringService::new(MonitoringConfig::default()))
}

fn service(cfg: EcosystemConfig) -> EcosystemService {
    EcosystemService::new(cfg, monitoring()).expect("ecosystem new")
}

fn sample_task() -> Task {
    Task {
        id: "task-1".to_string(),
        task_type: TaskType::McpCoordination,
        priority: TaskPriority::Normal,
        requirements: TaskRequirements {
            cpu: None,
            memory: None,
            storage: None,
            network: None,
            required_capabilities: vec!["nonexistent-cap".to_string()],
            preferred_primals: vec![PrimalType::Squirrel],
            constraints: HashMap::new(),
        },
        context: serde_json::json!({}),
        deadline: None,
    }
}

#[tokio::test]
async fn start_disabled_sets_standalone_and_skips_background() {
    let cfg = EcosystemConfig {
        enabled: false,
        mode: EcosystemMode::Coordinated,
        discovery: DiscoveryConfig::default(),
    };
    let eco = service(cfg);
    eco.start().await.expect("start");
    assert!(matches!(eco.get_status(), ServiceStatus::Standalone));
}

#[tokio::test]
async fn start_standalone_mode_sets_status() {
    let cfg = EcosystemConfig {
        enabled: true,
        mode: EcosystemMode::Standalone,
        discovery: DiscoveryConfig {
            auto_discovery: false,
            songbird_endpoint: None,
            direct_endpoints: HashMap::new(),
            probe_interval: ChronoDuration::seconds(60),
            health_check_timeout: ChronoDuration::seconds(5),
        },
    };
    let eco = service(cfg);
    eco.start().await.expect("start");
    assert!(matches!(eco.get_status(), ServiceStatus::Standalone));
}

#[tokio::test]
async fn start_sovereign_reaches_coordinating() {
    let cfg = EcosystemConfig {
        enabled: true,
        mode: EcosystemMode::Sovereign,
        discovery: DiscoveryConfig {
            auto_discovery: false,
            songbird_endpoint: None,
            direct_endpoints: HashMap::new(),
            probe_interval: ChronoDuration::seconds(60),
            health_check_timeout: ChronoDuration::seconds(5),
        },
    };
    let eco = service(cfg);
    eco.start().await.expect("start");
    assert!(matches!(eco.get_status(), ServiceStatus::Coordinating));
}

#[tokio::test]
async fn start_coordinated_succeeds_when_discovery_succeeds() {
    let cfg = EcosystemConfig {
        enabled: true,
        mode: EcosystemMode::Coordinated,
        discovery: DiscoveryConfig {
            auto_discovery: false,
            songbird_endpoint: None,
            direct_endpoints: HashMap::new(),
            probe_interval: ChronoDuration::seconds(60),
            health_check_timeout: ChronoDuration::seconds(5),
        },
    };
    let eco = service(cfg);
    eco.start().await.expect("start");
    assert!(matches!(eco.get_status(), ServiceStatus::Coordinating));
}

#[test]
fn get_endpoint_resolves_localhost_http_url() {
    let eco = service(EcosystemConfig::default());
    let ep = eco.get_endpoint();
    assert!(ep.starts_with("http://"), "{ep}");
    assert!(ep.contains("localhost"), "{ep}");
}

#[test]
fn get_service_metadata_contains_core_keys() {
    let eco = service(EcosystemConfig::default());
    let m = eco.get_service_metadata();
    assert!(m.contains_key("version"));
    assert!(m.contains_key("node_id"));
    assert!(m.contains_key("started_at"));
    assert!(m.contains_key("mode"));
}

#[tokio::test]
async fn discover_primals_returns_empty_without_real_probes() {
    let eco = service(EcosystemConfig::default());
    let v = eco.discover_primals().await.expect("discover");
    assert!(v.is_empty());
    assert!(eco.get_discovered_primals().is_empty());
}

#[tokio::test]
async fn coordinate_task_sovereign_falls_back_locally() {
    let cfg = EcosystemConfig {
        enabled: true,
        mode: EcosystemMode::Sovereign,
        discovery: DiscoveryConfig {
            auto_discovery: false,
            songbird_endpoint: None,
            direct_endpoints: HashMap::new(),
            probe_interval: ChronoDuration::seconds(60),
            health_check_timeout: ChronoDuration::seconds(5),
        },
    };
    let eco = service(cfg);
    eco.start().await.expect("start");
    let res = eco
        .coordinate_task(sample_task())
        .await
        .expect("coordinate");
    assert_eq!(res.status, crate::TaskStatus::Completed);
    assert!(res.result.is_some());
    assert!(res.executed_by.is_none());
}

#[tokio::test]
async fn coordinate_task_coordinated_errors_without_route() {
    let cfg = EcosystemConfig {
        enabled: true,
        mode: EcosystemMode::Coordinated,
        discovery: DiscoveryConfig {
            auto_discovery: false,
            songbird_endpoint: None,
            direct_endpoints: HashMap::new(),
            probe_interval: ChronoDuration::seconds(60),
            health_check_timeout: ChronoDuration::seconds(5),
        },
    };
    let eco = service(cfg);
    eco.start().await.expect("start");
    let err = eco
        .coordinate_task(sample_task())
        .await
        .expect_err("no primal");
    assert!(matches!(err, Error::Routing(_)));
}

#[tokio::test]
async fn health_check_returns_mapped_status() {
    let eco = service(EcosystemConfig::default());
    eco.start().await.expect("start");
    let h = eco.health_check().await.expect("health");
    assert!(matches!(
        h,
        HealthStatus::Healthy | HealthStatus::Unknown | HealthStatus::Degraded
    ));
}

#[tokio::test]
async fn register_with_ecosystem_no_songbird_is_ok() {
    let eco = service(EcosystemConfig::default());
    eco.register_with_ecosystem().await.expect("register");
}

#[tokio::test]
async fn register_with_ecosystem_with_songbird_endpoint_is_ok() {
    let cfg = EcosystemConfig {
        enabled: true,
        mode: EcosystemMode::Sovereign,
        discovery: DiscoveryConfig {
            auto_discovery: false,
            songbird_endpoint: Some("unix:///tmp/songbird.sock".to_string()),
            direct_endpoints: HashMap::new(),
            probe_interval: ChronoDuration::seconds(60),
            health_check_timeout: ChronoDuration::seconds(5),
        },
    };
    let eco = service(cfg);
    eco.register_with_ecosystem().await.expect("register");
}

#[tokio::test]
async fn shutdown_sets_stopping_status() {
    let eco = service(EcosystemConfig::default());
    eco.start().await.expect("start");
    eco.shutdown().await.expect("shutdown");
    assert!(matches!(eco.get_status(), ServiceStatus::Stopping));
}
