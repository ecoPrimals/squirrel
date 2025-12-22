//! Comprehensive tests for API module
//!
//! Tests cover response types, server state, and API structures.

use crate::api::*;
use chrono::Utc;
use std::collections::HashMap;

// ============================================================================
// HEALTH RESPONSE TESTS
// ============================================================================

#[test]
fn test_health_response_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        uptime_seconds: 3600,
        service_mesh: ServiceMeshHealthStatus {
            registered: true,
            last_heartbeat: Some(Utc::now()),
            connection_status: "connected".to_string(),
            load_balancing_active: true,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: 5,
            active_integrations: vec!["beardog".to_string(), "nestgate".to_string()],
            cross_primal_status: "operational".to_string(),
            ecosystem_health_score: 0.95,
        },
        metadata,
    };

    assert_eq!(response.status, "healthy");
    assert_eq!(response.uptime_seconds, 3600);
    assert!(response.service_mesh.registered);
    assert_eq!(response.ecosystem.discovered_primals, 5);
}

#[test]
fn test_health_response_serialization() {
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        uptime_seconds: 100,
        service_mesh: ServiceMeshHealthStatus {
            registered: true,
            last_heartbeat: None,
            connection_status: "disconnected".to_string(),
            load_balancing_active: false,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: 0,
            active_integrations: vec![],
            cross_primal_status: "initializing".to_string(),
            ecosystem_health_score: 0.5,
        },
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&response).expect("Should serialize");
    assert!(json.contains("healthy"));
    assert!(json.contains("disconnected"));

    let deserialized: HealthResponse = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.status, "healthy");
    assert_eq!(deserialized.uptime_seconds, 100);
}

// ============================================================================
// SERVICE MESH HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_service_mesh_health_status_registered() {
    let status = ServiceMeshHealthStatus {
        registered: true,
        last_heartbeat: Some(Utc::now()),
        connection_status: "active".to_string(),
        load_balancing_active: true,
    };

    assert!(status.registered);
    assert!(status.last_heartbeat.is_some());
    assert!(status.load_balancing_active);
}

#[test]
fn test_service_mesh_health_status_not_registered() {
    let status = ServiceMeshHealthStatus {
        registered: false,
        last_heartbeat: None,
        connection_status: "not_registered".to_string(),
        load_balancing_active: false,
    };

    assert!(!status.registered);
    assert!(status.last_heartbeat.is_none());
    assert!(!status.load_balancing_active);
}

#[test]
fn test_service_mesh_health_status_serialization() {
    let status = ServiceMeshHealthStatus {
        registered: true,
        last_heartbeat: Some(Utc::now()),
        connection_status: "connected".to_string(),
        load_balancing_active: true,
    };

    let json = serde_json::to_string(&status).expect("Should serialize");
    let deserialized: ServiceMeshHealthStatus =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.registered, status.registered);
    assert_eq!(deserialized.connection_status, status.connection_status);
    assert_eq!(
        deserialized.load_balancing_active,
        status.load_balancing_active
    );
}

// ============================================================================
// ECOSYSTEM HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_ecosystem_health_status_creation() {
    let status = EcosystemHealthStatus {
        discovered_primals: 10,
        active_integrations: vec![
            "beardog".to_string(),
            "nestgate".to_string(),
            "toadstool".to_string(),
        ],
        cross_primal_status: "fully_operational".to_string(),
        ecosystem_health_score: 1.0,
    };

    assert_eq!(status.discovered_primals, 10);
    assert_eq!(status.active_integrations.len(), 3);
    assert_eq!(status.ecosystem_health_score, 1.0);
}

#[test]
fn test_ecosystem_health_status_no_integrations() {
    let status = EcosystemHealthStatus {
        discovered_primals: 0,
        active_integrations: vec![],
        cross_primal_status: "isolated".to_string(),
        ecosystem_health_score: 0.1,
    };

    assert_eq!(status.discovered_primals, 0);
    assert!(status.active_integrations.is_empty());
    assert!(status.ecosystem_health_score < 0.5);
}

#[test]
fn test_ecosystem_health_status_serialization() {
    let status = EcosystemHealthStatus {
        discovered_primals: 5,
        active_integrations: vec!["test".to_string()],
        cross_primal_status: "operational".to_string(),
        ecosystem_health_score: 0.85,
    };

    let json = serde_json::to_string(&status).expect("Should serialize");
    let deserialized: EcosystemHealthStatus =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.discovered_primals, status.discovered_primals);
    assert_eq!(deserialized.active_integrations, status.active_integrations);
    assert_eq!(
        deserialized.ecosystem_health_score,
        status.ecosystem_health_score
    );
}

// Note: ServerState is private, so we test through public API

// ============================================================================
// HEALTH RESPONSE EDGE CASES
// ============================================================================

#[test]
fn test_health_response_with_empty_metadata() {
    let response = HealthResponse {
        status: "degraded".to_string(),
        timestamp: Utc::now(),
        uptime_seconds: 0,
        service_mesh: ServiceMeshHealthStatus {
            registered: false,
            last_heartbeat: None,
            connection_status: "initializing".to_string(),
            load_balancing_active: false,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: 0,
            active_integrations: vec![],
            cross_primal_status: "starting".to_string(),
            ecosystem_health_score: 0.0,
        },
        metadata: HashMap::new(),
    };

    assert!(response.metadata.is_empty());
    assert_eq!(response.uptime_seconds, 0);
    assert_eq!(response.ecosystem.ecosystem_health_score, 0.0);
}

#[test]
fn test_health_response_with_many_integrations() {
    let integrations: Vec<String> = (0..100).map(|i| format!("primal_{}", i)).collect();

    let status = EcosystemHealthStatus {
        discovered_primals: 100,
        active_integrations: integrations.clone(),
        cross_primal_status: "fully_meshed".to_string(),
        ecosystem_health_score: 1.0,
    };

    assert_eq!(status.active_integrations.len(), 100);
    assert_eq!(status.discovered_primals, 100);
}

#[test]
fn test_health_response_health_score_boundaries() {
    // Test minimum score
    let min_status = EcosystemHealthStatus {
        discovered_primals: 0,
        active_integrations: vec![],
        cross_primal_status: "critical".to_string(),
        ecosystem_health_score: 0.0,
    };
    assert_eq!(min_status.ecosystem_health_score, 0.0);

    // Test maximum score
    let max_status = EcosystemHealthStatus {
        discovered_primals: 100,
        active_integrations: vec!["all".to_string()],
        cross_primal_status: "optimal".to_string(),
        ecosystem_health_score: 1.0,
    };
    assert_eq!(max_status.ecosystem_health_score, 1.0);

    // Test mid-range score
    let mid_status = EcosystemHealthStatus {
        discovered_primals: 5,
        active_integrations: vec!["some".to_string()],
        cross_primal_status: "nominal".to_string(),
        ecosystem_health_score: 0.5,
    };
    assert_eq!(mid_status.ecosystem_health_score, 0.5);
}

// ============================================================================
// SERVICE MESH EDGE CASES
// ============================================================================

#[test]
fn test_service_mesh_status_various_connection_states() {
    let states = vec![
        "connected",
        "disconnected",
        "reconnecting",
        "initializing",
        "degraded",
        "failed",
    ];

    for state_str in states {
        let status = ServiceMeshHealthStatus {
            registered: true,
            last_heartbeat: Some(Utc::now()),
            connection_status: state_str.to_string(),
            load_balancing_active: state_str == "connected",
        };

        assert_eq!(status.connection_status, state_str);
    }
}

#[test]
fn test_service_mesh_heartbeat_timing() {
    let now = Utc::now();

    let status_with_heartbeat = ServiceMeshHealthStatus {
        registered: true,
        last_heartbeat: Some(now),
        connection_status: "active".to_string(),
        load_balancing_active: true,
    };

    let status_no_heartbeat = ServiceMeshHealthStatus {
        registered: false,
        last_heartbeat: None,
        connection_status: "never_connected".to_string(),
        load_balancing_active: false,
    };

    assert!(status_with_heartbeat.last_heartbeat.is_some());
    assert!(status_no_heartbeat.last_heartbeat.is_none());
}

// ============================================================================
// RESPONSE SERIALIZATION ROUND-TRIP TESTS
// ============================================================================

#[test]
fn test_health_response_round_trip() {
    let original = HealthResponse {
        status: "test_status".to_string(),
        timestamp: Utc::now(),
        uptime_seconds: 12345,
        service_mesh: ServiceMeshHealthStatus {
            registered: true,
            last_heartbeat: Some(Utc::now()),
            connection_status: "test".to_string(),
            load_balancing_active: true,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: 7,
            active_integrations: vec!["test1".to_string(), "test2".to_string()],
            cross_primal_status: "test_status".to_string(),
            ecosystem_health_score: 0.75,
        },
        metadata: {
            let mut m = HashMap::new();
            m.insert("key1".to_string(), "value1".to_string());
            m
        },
    };

    let json = serde_json::to_string(&original).expect("Should serialize");
    let deserialized: HealthResponse = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.status, original.status);
    assert_eq!(deserialized.uptime_seconds, original.uptime_seconds);
    assert_eq!(deserialized.metadata.len(), original.metadata.len());
}

// Note: Server state stress tests omitted (private structure)

// ============================================================================
// METADATA TESTS
// ============================================================================

#[test]
fn test_health_response_metadata_operations() {
    let mut metadata = HashMap::new();

    // Add various metadata
    metadata.insert("version".to_string(), "2.0.0".to_string());
    metadata.insert("environment".to_string(), "production".to_string());
    metadata.insert("region".to_string(), "us-east-1".to_string());

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        uptime_seconds: 1000,
        service_mesh: ServiceMeshHealthStatus {
            registered: true,
            last_heartbeat: Some(Utc::now()),
            connection_status: "active".to_string(),
            load_balancing_active: true,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: 1,
            active_integrations: vec![],
            cross_primal_status: "ok".to_string(),
            ecosystem_health_score: 0.9,
        },
        metadata: metadata.clone(),
    };

    assert_eq!(response.metadata.len(), 3);
    assert_eq!(response.metadata.get("version"), Some(&"2.0.0".to_string()));
    assert_eq!(
        response.metadata.get("environment"),
        Some(&"production".to_string())
    );
}

#[test]
fn test_ecosystem_status_unicode_strings() {
    let status = EcosystemHealthStatus {
        discovered_primals: 1,
        active_integrations: vec![
            "日本語".to_string(),
            "中文".to_string(),
            "العربية".to_string(),
        ],
        cross_primal_status: "🚀 operational".to_string(),
        ecosystem_health_score: 1.0,
    };

    assert_eq!(status.active_integrations.len(), 3);
    assert!(status.cross_primal_status.contains("🚀"));

    // Test serialization with unicode
    let json = serde_json::to_string(&status).expect("Should serialize");
    let deserialized: EcosystemHealthStatus =
        serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.active_integrations, status.active_integrations);
}

#[test]
fn test_uptime_calculation_logic() {
    use chrono::Duration;

    let started_at = Utc::now() - Duration::hours(2);
    let uptime = Utc::now().signed_duration_since(started_at).num_seconds();

    // Should be approximately 2 hours (7200 seconds)
    assert!((7199..=7201).contains(&uptime));
}
