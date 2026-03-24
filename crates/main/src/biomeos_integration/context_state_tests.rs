// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::error::PrimalError;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_context_state_creation() {
    let context_state = ContextState::new();
    assert!(context_state.active_sessions.is_empty());
    assert!(context_state.persistent_contexts.is_empty());
}

#[tokio::test]
async fn test_session_context_creation() {
    let mut context_state = ContextState::new();
    let session_id = "test-session-001".to_string();

    context_state
        .create_session_context(
            session_id.clone(),
            Some("user-123".to_string()),
            "test_context".to_string(),
        )
        .await
        .expect("should succeed");

    assert!(context_state.active_sessions.contains_key(&session_id));
    assert_eq!(context_state.get_active_sessions(), 1);
}

#[tokio::test]
async fn test_context_state_request_handling() {
    let context_state = ContextState::new();
    let request = ContextStateRequest {
        request_id: "test-request-001".to_string(),
        session_id: "test-session".to_string(),
        request_type: "get_context".to_string(),
        context_data: None,
        query: None,
    };

    let response = context_state
        .handle_state_request(request)
        .await
        .expect("should succeed");
    assert_eq!(response.session_id, "test-session");
}

#[tokio::test]
async fn test_context_search() {
    let context_state = ContextState::new();
    let search_results = context_state
        .search_context_data("test")
        .await
        .expect("should succeed");
    assert!(search_results.is_empty()); // No contexts to search initially
}

#[test]
fn context_state_default_matches_new() {
    let a = ContextState::default();
    let b = ContextState::new();
    assert_eq!(a.get_active_sessions(), b.get_active_sessions());
}

#[tokio::test]
async fn initialize_manage_shutdown_lifecycle() {
    let mut cs = ContextState::new();
    cs.initialize().await.expect("should succeed");
    cs.manage_ecosystem_context().await.expect("should succeed");
    cs.shutdown().await.expect("should succeed");
}

#[tokio::test]
async fn health_check_empty_errors() {
    let cs = ContextState::new();
    let err = cs.health_check().await.unwrap_err();
    assert!(matches!(err, PrimalError::General(_)));
}

#[tokio::test]
async fn health_check_with_session_ok() {
    let mut cs = ContextState::new();
    cs.create_session_context("s1".to_string(), None, "t".to_string())
        .await
        .expect("should succeed");
    cs.health_check().await.expect("should succeed");
}

#[tokio::test]
async fn update_session_and_search_analyze() {
    let mut cs = ContextState::new();
    cs.create_session_context("s2".to_string(), None, "workflow".to_string())
        .await
        .expect("should succeed");
    let mut upd = HashMap::new();
    upd.insert("k".to_string(), serde_json::json!(1));
    cs.update_session_context("s2", upd)
        .await
        .expect("should succeed");

    let hits = cs
        .search_context_data("workflow")
        .await
        .expect("should succeed");
    assert!(!hits.is_empty());

    let combined = cs
        .search_and_analyze("workflow")
        .await
        .expect("should succeed");
    assert_eq!(combined.len(), hits.len());

    let recs = cs
        .get_session_recommendations("s2")
        .await
        .expect("should succeed");
    assert!(recs.iter().any(|r| r.contains("linking")));
}

#[tokio::test]
async fn analyze_session_not_found() {
    let cs = ContextState::new();
    let err = cs.analyze_session("nope").await.unwrap_err();
    assert!(matches!(err, PrimalError::Internal(_)));
}

#[tokio::test]
async fn get_context_analytics_empty_map_when_no_sessions() {
    let cs = ContextState::new();
    let m = cs.get_context_analytics().await.expect("should succeed");
    assert!(m.is_empty());
}

#[test]
fn serde_roundtrip_session_context() {
    let sc = SessionContext {
        session_id: "sid".to_string(),
        user_id: Some("u".to_string()),
        context_data: HashMap::from([("a".to_string(), serde_json::json!(true))]),
        created_at: Utc::now(),
        last_accessed: Utc::now(),
        access_count: 3,
        context_type: "conversation".to_string(),
        tags: vec!["t1".to_string()],
        related_sessions: vec![],
    };
    let json = serde_json::to_string(&sc).expect("should succeed");
    let back: SessionContext = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(back.session_id, sc.session_id);
}

#[test]
fn serde_roundtrip_search_result_and_context_analysis() {
    let sr = SearchResult {
        result_id: "r".to_string(),
        result_type: "active_session".to_string(),
        relevance_score: 0.5,
        context_snippet: "x".to_string(),
    };
    let s = serde_json::to_string(&sr).expect("should succeed");
    let sr2: SearchResult = serde_json::from_str(&s).expect("should succeed");
    assert_eq!(sr2.relevance_score, 0.5);

    let ca = ContextAnalysis {
        session_id: "s".to_string(),
        data_complexity: 1.0,
        usage_frequency: 2.0,
        recency_score: 0.5,
        relationship_strength: 0.0,
    };
    let caj = serde_json::to_string(&ca).expect("should succeed");
    let ca2: ContextAnalysis = serde_json::from_str(&caj).expect("should succeed");
    assert!((ca2.recency_score - 0.5).abs() < f64::EPSILON);
}

#[test]
fn serde_roundtrip_persistent_context_and_policies() {
    let pc = PersistentContext {
        context_id: "c1".to_string(),
        context_name: "MyCtx".to_string(),
        context_data: HashMap::new(),
        created_at: Utc::now(),
        last_updated: Utc::now(),
        version: 1,
        context_type: "t".to_string(),
        retention_policy: RetentionPolicy {
            retention_duration: Duration::from_secs(3600),
            cleanup_strategy: "archive".to_string(),
            archival_rules: vec![ArchivalRule {
                rule_id: "r1".to_string(),
                condition: "old".to_string(),
                archive_location: "/tmp/a".to_string(),
                compression_enabled: true,
            }],
            deletion_protection: false,
        },
        access_permissions: AccessPermissions {
            read_permissions: vec!["a".to_string()],
            write_permissions: vec![],
            admin_permissions: vec![],
            time_based_access: vec![],
        },
    };
    let j = serde_json::to_string(&pc).expect("should succeed");
    let pc2: PersistentContext = serde_json::from_str(&j).expect("should succeed");
    assert_eq!(pc2.context_name, "MyCtx");
}
