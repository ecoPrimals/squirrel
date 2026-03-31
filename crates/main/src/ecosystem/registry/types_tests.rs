// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Unit tests for the ecosystem registry [`types`](super) module.

use super::*;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::ecosystem::EcosystemPrimalType;

#[test]
fn test_discovered_service_new() {
    let service = DiscoveredService::new(
        "svc-123",
        EcosystemPrimalType::Squirrel,
        "unix:///tmp/svc.sock",
        "unix:///tmp/svc.sock",
        "1.0",
        vec!["storage", "compute"],
        HashMap::new(),
    );

    assert_eq!(service.service_id.as_ref(), "svc-123");
    assert!(service.has_capability("storage"));
    assert!(service.has_capability("compute"));
    assert!(!service.has_capability("unknown"));
}

#[test]
fn test_discovered_service_get_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("version", "1.0");
    metadata.insert("region", "us-west");

    let service = DiscoveredService::new(
        "test-svc",
        EcosystemPrimalType::Songbird,
        "http://localhost:8080",
        "http://localhost:8080/health",
        "1.0",
        vec!["service_mesh"],
        metadata,
    );

    assert!(service.get_metadata("version").is_some());
    assert!(service.get_metadata("region").is_some());
    assert!(service.get_metadata("missing").is_none());
}

#[test]
fn test_primal_api_request_new() {
    let request = PrimalApiRequest::new(
        "req-123",
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::Songbird,
        "discover",
        serde_json::json!({"capability": "storage"}),
        HashMap::new(),
        Duration::from_secs(30),
    );

    assert_eq!(request.request_id.as_ref(), "req-123");
    assert_eq!(request.operation.as_ref(), "discover");
}

#[test]
fn test_primal_api_request_get_header() {
    let mut headers = HashMap::new();
    headers.insert("x-correlation-id", "corr-123");

    let request = PrimalApiRequest::new(
        "req-1",
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::BearDog,
        "auth",
        serde_json::json!({}),
        headers,
        Duration::from_secs(5),
    );

    assert!(request.get_header("x-correlation-id").is_some());
    assert!(request.get_header("missing").is_none());
}

#[test]
fn test_primal_api_response_new() {
    let request_id = Arc::from("req-123");
    let response = PrimalApiResponse::new(
        request_id,
        true,
        Some(serde_json::json!({"result": "ok"})),
        None,
        HashMap::new(),
        Duration::from_millis(50),
    );

    assert!(response.success);
    assert!(response.data.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_intern_registry_string() {
    let s = intern_registry_string("storage");
    assert_eq!(s.as_ref(), "storage");

    let s2 = intern_registry_string("squirrel");
    assert_eq!(s2.as_ref(), "squirrel");
}

#[test]
fn test_service_health_status_variants() {
    let _ = ServiceHealthStatus::Unknown;
    let _ = ServiceHealthStatus::Healthy;
    let _ = ServiceHealthStatus::Degraded;
    let _ = ServiceHealthStatus::Unhealthy;
    let _ = ServiceHealthStatus::Offline;
}

#[test]
fn test_service_status_variants() {
    let _ = ServiceStatus::Unknown;
    let _ = ServiceStatus::Healthy;
    let _ = ServiceStatus::Degraded;
}

#[test]
fn intern_registry_string_covers_capability_and_fallback_branches() {
    use universal_constants::capabilities;

    assert_eq!(intern_registry_string("storage").as_ref(), "storage");
    assert_eq!(intern_registry_string("compute").as_ref(), "compute");
    assert_eq!(intern_registry_string("security").as_ref(), "security");
    assert_eq!(intern_registry_string("discovery").as_ref(), "discovery");
    assert_eq!(intern_registry_string("active").as_ref(), "active");
    assert_eq!(intern_registry_string("inactive").as_ref(), "inactive");
    assert_eq!(intern_registry_string("error").as_ref(), "error");
    assert_eq!(
        intern_registry_string(capabilities::COMPUTE_CAPABILITY).as_ref(),
        capabilities::COMPUTE_CAPABILITY
    );
    let custom = intern_registry_string("unique-custom-id-xyz");
    assert_eq!(custom.as_ref(), "unique-custom-id-xyz");
}

#[test]
fn discovered_service_serde_roundtrip() {
    let s = DiscoveredService::new(
        "svc-serde",
        EcosystemPrimalType::Squirrel,
        "unix:///tmp/x.sock",
        "unix:///tmp/x.sock",
        "2.0",
        vec!["storage"],
        HashMap::from([("region", "eu")]),
    );
    let json = serde_json::to_string(&s).expect("ser");
    let back: DiscoveredService = serde_json::from_str(&json).expect("de");
    assert_eq!(back.service_id.as_ref(), "svc-serde");
    assert_eq!(
        back.get_metadata("region").map(std::convert::AsRef::as_ref),
        Some("eu")
    );
}

#[test]
fn primal_api_response_with_error_roundtrip() {
    let rid = Arc::from("r1");
    let resp = PrimalApiResponse::new(
        Arc::clone(&rid),
        false,
        None,
        Some("failed"),
        HashMap::from([("x-trace", "t1")]),
        Duration::from_millis(1),
    );
    let json = serde_json::to_string(&resp).expect("ser");
    let back: PrimalApiResponse = serde_json::from_str(&json).expect("de");
    assert!(!back.success);
    assert_eq!(
        back.error.as_ref().map(std::convert::AsRef::as_ref),
        Some("failed")
    );
}

#[test]
fn ecosystem_status_serde_roundtrip() {
    let st = EcosystemStatus {
        overall_health: 0.75,
        primal_statuses: vec![],
        registered_services: 2,
        active_coordinations: 1,
        last_full_sync: Some(Utc::now()),
        discovery_cache_size: 3,
    };
    let v = serde_json::to_value(&st).expect("should succeed");
    let back: EcosystemStatus = serde_json::from_value(v).expect("should succeed");
    assert!((back.overall_health - 0.75).abs() < f64::EPSILON);
    assert_eq!(back.discovery_cache_size, 3);
}

#[test]
fn registry_state_default_is_empty() {
    let rs = RegistryState::default();
    assert!(rs.registered_services.is_empty());
    assert!(rs.service_discovery_cache.is_empty());
    assert_eq!(rs.registration_attempts, 0);
}

#[test]
fn ecosystem_registry_event_debug_smoke() {
    let ev = EcosystemRegistryEvent::ServiceDiscovered {
        service_id: Arc::from("s1"),
        primal_type: EcosystemPrimalType::Squirrel,
        endpoint: Arc::from("unix:///a"),
        capabilities: vec![Arc::from("compute")],
    };
    let s = format!("{ev:?}");
    assert!(s.contains("ServiceDiscovered"));
}

#[test]
fn service_status_all_variants_serde() {
    for st in [
        ServiceStatus::Discovering,
        ServiceStatus::Registering,
        ServiceStatus::Recovering,
        ServiceStatus::Offline,
        ServiceStatus::Unhealthy,
    ] {
        let j = serde_json::to_string(&st).expect("should succeed");
        let _: ServiceStatus = serde_json::from_str(&j).expect("should succeed");
    }
}

#[test]
fn intern_registry_string_additional_branches() {
    assert_eq!(
        intern_registry_string("ai_coordination").as_ref(),
        "ai_coordination"
    );
    assert_eq!(intern_registry_string("network").as_ref(), "network");
    assert_eq!(intern_registry_string("songbird").as_ref(), "songbird");
    assert_eq!(intern_registry_string("nestgate").as_ref(), "nestgate");
}

#[test]
fn ecosystem_registry_event_variants_debug() {
    let ev = EcosystemRegistryEvent::ServiceRegistered {
        service_id: Arc::from("s1"),
        primal_type: EcosystemPrimalType::Songbird,
        endpoint: Arc::from("unix:///a"),
    };
    assert!(format!("{ev:?}").contains("ServiceRegistered"));
    let ev2 = EcosystemRegistryEvent::ServiceError {
        service_id: Arc::from("s2"),
        error: Arc::from("e"),
        timestamp: Utc::now(),
    };
    assert!(format!("{ev2:?}").contains("ServiceError"));
    let ev3 = EcosystemRegistryEvent::ServiceHealthChanged {
        service_id: Arc::from("s3"),
        primal_type: EcosystemPrimalType::Squirrel,
        old_status: ServiceHealthStatus::Unknown,
        new_status: ServiceHealthStatus::Healthy,
    };
    assert!(format!("{ev3:?}").contains("ServiceHealthChanged"));
    let ev4 = EcosystemRegistryEvent::ServiceOffline {
        service_id: Arc::from("s4"),
        primal_type: EcosystemPrimalType::BearDog,
        reason: Arc::from("shutdown"),
    };
    assert!(format!("{ev4:?}").contains("ServiceOffline"));
}

#[test]
fn health_check_result_debug() {
    let h = HealthCheckResult {
        status: ServiceHealthStatus::Degraded,
        processing_time: Duration::from_millis(5),
        error: Some("x".to_string()),
    };
    let s = format!("{h:?}");
    assert!(s.contains("Degraded"));
}

#[test]
fn primal_status_roundtrip() {
    let ps = PrimalStatus {
        primal_type: EcosystemPrimalType::Squirrel,
        status: ServiceStatus::Healthy,
        endpoint: "e".to_string(),
        version: "1".to_string(),
        capabilities: vec!["c".to_string()],
        health_score: 0.9,
        response_time: Duration::from_millis(1),
        last_seen: Utc::now(),
        error_count: 0,
        coordination_features: vec![],
    };
    let j = serde_json::to_value(&ps).expect("should succeed");
    let back: PrimalStatus = serde_json::from_value(j).expect("should succeed");
    assert_eq!(back.endpoint, "e");
}
