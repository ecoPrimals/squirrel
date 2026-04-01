// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use chrono::Utc;

use super::*;
use crate::universal::PrimalContext;

fn sample_service(id: &str, endpoint: &str, capabilities: Vec<&str>) -> DiscoveredService {
    DiscoveredService {
        service_id: format!("svc-{id}"),
        instance_id: format!("inst-{id}"),
        endpoint: endpoint.to_string(),
        capabilities: capabilities.iter().map(|s| (*s).to_string()).collect(),
        health: ServiceHealth::Healthy,
        discovered_at: Utc::now(),
        last_health_check: Some(Utc::now()),
    }
}

#[tokio::test]
async fn new_and_initialize_default_context() {
    let mut eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    eco.initialize().await.expect("should succeed");
    assert!(eco.get_discovered_primals().await.is_empty());
}

#[tokio::test]
async fn with_cache_config_disables_caching_uncached_path() {
    let mut cfg = CacheConfig::default();
    cfg.enable_caching = false;
    let eco = UniversalPrimalEcosystem::with_cache_config(PrimalContext::default(), cfg);
    let req = CapabilityRequest {
        required_capabilities: vec!["data-persistence".to_string()],
        optional_capabilities: vec![],
        context: PrimalContext::default(),
        metadata: HashMap::new(),
    };
    let matches = eco
        .find_services_by_capability(&req)
        .await
        .expect("should succeed");
    assert!(matches.is_empty());
}

#[tokio::test]
async fn find_services_matches_required_and_optional_scoring() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    // One capability key so the service is not duplicated in the outer iteration map.
    eco.test_only_register_service(sample_service(
        "a",
        "unix:///tmp/x",
        vec!["data-persistence"],
    ))
    .await;

    let req = CapabilityRequest {
        required_capabilities: vec!["data-persistence".to_string()],
        optional_capabilities: vec!["high-availability".to_string(), "encryption".to_string()],
        context: PrimalContext::default(),
        metadata: HashMap::new(),
    };
    let matches = eco
        .find_services_by_capability(&req)
        .await
        .expect("should succeed");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].score, 0.0);
    assert!(matches[0].missing_capabilities.is_empty());
}

#[tokio::test]
async fn find_by_capability_alias() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    eco.test_only_register_service(sample_service(
        "b",
        "unix:///tmp/y",
        vec!["container-runtime"],
    ))
    .await;
    let m = eco
        .find_by_capability("container-runtime")
        .await
        .expect("should succeed");
    assert_eq!(m.len(), 1);
}

#[tokio::test]
async fn capability_discovery_cache_hit_second_call() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    eco.test_only_register_service(sample_service("c", "unix:///tmp/z", vec!["task-execution"]))
        .await;

    let req = CapabilityRequest {
        required_capabilities: vec!["task-execution".to_string()],
        optional_capabilities: vec![],
        context: PrimalContext::default(),
        metadata: HashMap::new(),
    };
    let first = eco
        .find_services_by_capability(&req)
        .await
        .expect("should succeed");
    let second = eco
        .find_services_by_capability(&req)
        .await
        .expect("should succeed");
    assert_eq!(first.len(), second.len());
}

#[tokio::test]
async fn clear_caches_and_stats() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    eco.test_only_register_service(sample_service("d", "unix:///tmp/w", vec!["x-cap"]))
        .await;
    let req = CapabilityRequest {
        required_capabilities: vec!["x-cap".to_string()],
        optional_capabilities: vec![],
        context: PrimalContext::default(),
        metadata: HashMap::new(),
    };
    let _ = eco
        .find_services_by_capability(&req)
        .await
        .expect("should succeed");
    let stats_before = eco.get_cache_stats().await;
    eco.clear_caches().await;
    let stats_after = eco.get_cache_stats().await;
    assert!(stats_after.discovery_cache_size <= stats_before.discovery_cache_size);
}

#[tokio::test]
async fn send_to_primal_returns_success_json() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    let req = PrimalRequest::new(
        "squirrel",
        "any",
        "ping",
        serde_json::json!({}),
        PrimalContext::default(),
    );
    let resp = eco
        .send_to_primal("any", req)
        .await
        .expect("should succeed");
    assert!(resp.success);
}

#[tokio::test]
async fn match_capabilities_delegates_to_find() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    let req = CapabilityRequest {
        required_capabilities: vec!["none".to_string()],
        optional_capabilities: vec![],
        context: PrimalContext::default(),
        metadata: HashMap::new(),
    };
    let m = eco.match_capabilities(&req).await.expect("should succeed");
    assert!(m.is_empty());
}

#[test]
fn generate_cache_key_stable_for_sorted_caps() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    let mut ctx = PrimalContext::default();
    ctx.user_id = "u1".to_string();
    let req = CapabilityRequest {
        required_capabilities: vec!["b".to_string(), "a".to_string()],
        optional_capabilities: vec!["z".to_string(), "y".to_string()],
        context: ctx,
        metadata: HashMap::new(),
    };
    let k1 = eco.generate_cache_key(&req);
    let k2 = eco.generate_cache_key(&req);
    assert_eq!(k1, k2);
    assert!(k1.contains("req:a,b"));
}

#[tokio::test]
async fn store_data_fails_without_storage_service() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    let err = eco.store_data("k", b"data").await.unwrap_err();
    assert!(matches!(err, PrimalError::OperationFailed(_)));
}

#[tokio::test]
async fn query_service_capabilities_non_unix_returns_empty() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    let caps = eco
        .query_service_capabilities("http://localhost:8080")
        .await
        .expect("should succeed");
    assert!(caps.is_empty());
}

#[tokio::test]
async fn discover_service_mesh_sets_endpoint_from_match() {
    let mut eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    eco.test_only_register_service(sample_service(
        "mesh",
        "unix:///tmp/mesh.sock",
        vec![
            "service-discovery",
            "service-registration",
            "load-balancing",
        ],
    ))
    .await;
    eco.discover_service_mesh().await.expect("should succeed");
    assert_eq!(
        eco.service_mesh_endpoint.as_deref(),
        Some("unix:///tmp/mesh.sock")
    );
}

#[tokio::test]
async fn send_capability_request_rejects_unknown_scheme() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    let svc = sample_service("bad", "ftp://example/wrong", vec!["data-persistence"]);
    let req = PrimalRequest::new(
        "squirrel",
        &svc.service_id,
        "store",
        serde_json::json!({}),
        PrimalContext::default(),
    );
    let err = eco.send_capability_request(&svc, req).await.unwrap_err();
    assert!(matches!(err, PrimalError::InvalidEndpoint(_)));
}

#[tokio::test]
async fn send_capability_request_https_delegates_not_implemented() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    let svc = sample_service("http", "https://songbird/proxy", vec!["data-persistence"]);
    let req = PrimalRequest::new(
        "squirrel",
        &svc.service_id,
        "store",
        serde_json::json!({}),
        PrimalContext::default(),
    );
    let err = eco.send_capability_request(&svc, req).await.unwrap_err();
    assert!(matches!(err, PrimalError::NotImplemented(_)));
}

#[tokio::test]
async fn retrieve_and_execute_fail_without_capabilities() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    let e1 = eco.retrieve_data("k").await.unwrap_err();
    assert!(matches!(e1, PrimalError::OperationFailed(_)));
    let e2 = eco
        .execute_computation(serde_json::json!({}))
        .await
        .unwrap_err();
    assert!(matches!(e2, PrimalError::OperationFailed(_)));
    let e3 = eco
        .perform_security_operation("encrypt", serde_json::json!({}))
        .await
        .unwrap_err();
    assert!(matches!(e3, PrimalError::OperationFailed(_)));
}

#[tokio::test]
async fn find_services_skips_when_required_capabilities_missing() {
    let eco = UniversalPrimalEcosystem::new(PrimalContext::default());
    eco.test_only_register_service(sample_service(
        "only-other",
        "unix:///tmp/o",
        vec!["other-only"],
    ))
    .await;
    let req = CapabilityRequest {
        required_capabilities: vec!["data-persistence".to_string()],
        optional_capabilities: vec![],
        context: PrimalContext::default(),
        metadata: HashMap::new(),
    };
    assert!(
        eco.find_services_by_capability(&req)
            .await
            .expect("should succeed")
            .is_empty()
    );
}

#[tokio::test]
async fn cache_ttl_zero_marks_expired_in_stats_before_next_lookup() {
    let mut cfg = CacheConfig::default();
    cfg.capability_discovery_ttl = 0;
    let eco = UniversalPrimalEcosystem::with_cache_config(PrimalContext::default(), cfg);
    eco.test_only_register_service(sample_service("ttl0", "unix:///tmp/ttl", vec!["ttl-cap"]))
        .await;
    let req = CapabilityRequest {
        required_capabilities: vec!["ttl-cap".to_string()],
        optional_capabilities: vec![],
        context: PrimalContext::default(),
        metadata: HashMap::new(),
    };
    let _ = eco
        .find_services_by_capability(&req)
        .await
        .expect("should succeed");
    let stats = eco.get_cache_stats().await;
    assert!(stats.expired_cache_entries >= 1);
}

#[tokio::test]
async fn cache_evicts_oldest_when_at_capacity() {
    let mut cfg = CacheConfig::default();
    cfg.max_cache_entries = 10;
    cfg.capability_discovery_ttl = 3600;
    let eco = UniversalPrimalEcosystem::with_cache_config(PrimalContext::default(), cfg);
    for i in 0..11 {
        eco.test_only_register_service(sample_service(
            &format!("svc{i}"),
            "unix:///tmp/x",
            vec![&format!("cap{i}")],
        ))
        .await;
        let req = CapabilityRequest {
            required_capabilities: vec![format!("cap{i}")],
            optional_capabilities: vec![],
            context: PrimalContext::default(),
            metadata: HashMap::new(),
        };
        let _ = eco
            .find_services_by_capability(&req)
            .await
            .expect("should succeed");
    }
    let stats = eco.get_cache_stats().await;
    assert!(stats.discovery_cache_size <= 10);
}

#[test]
fn discover_ecosystem_services_empty_ports_skips_well_known() {
    temp_env::with_var("SERVICE_DISCOVERY_PORTS", Some(""), || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(async {
                let mut eco = UniversalPrimalEcosystem::new(PrimalContext::default());
                eco.discover_ecosystem_services()
                    .await
                    .expect("should succeed");
            });
    });
}
