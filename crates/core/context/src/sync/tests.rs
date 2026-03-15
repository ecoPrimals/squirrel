// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Sync mechanics licensed under ORC
// Copyright (C) 2026 DataScienceBioLab

use super::*;
use crate::ContextState;
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::mpsc;

fn make_state(id: &str, version: u64, last_modified: SystemTime) -> ContextState {
    ContextState {
        id: id.to_string(),
        version,
        timestamp: version * 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified,
    }
}

#[tokio::test]
async fn test_sync_manager_new() {
    let manager = SyncManager::new();
    assert_eq!(manager.get_status(), SyncStatus::Healthy);
}

#[tokio::test]
async fn test_sync_manager_with_config() {
    let config = SyncConfig {
        sync_timeout_seconds: 60,
        heartbeat_interval_seconds: 5,
        max_retry_attempts: 5,
        retry_delay_seconds: 1,
        max_pending_operations: 500,
        auto_resolve_conflicts: false,
        partition_detection_timeout_seconds: 120,
        max_message_age_seconds: 600,
    };
    let manager = SyncManager::with_config(config);
    assert_eq!(manager.get_status(), SyncStatus::Healthy);
}

#[tokio::test]
async fn test_sync_manager_default() {
    let manager = SyncManager::default();
    assert_eq!(manager.get_status(), SyncStatus::Healthy);
}

#[tokio::test]
async fn test_sync_manager_subscribe_unsubscribe() {
    let mut manager = SyncManager::new();
    let (tx, _rx) = mpsc::channel::<SyncEvent>(10);
    let id = manager.subscribe(tx);
    assert!(!id.is_empty());
    assert!(manager.unsubscribe(&id).is_ok());
    assert!(manager.unsubscribe(&id).is_err());
}

#[tokio::test]
async fn test_sync_manager_unsubscribe_nonexistent() {
    let mut manager = SyncManager::new();
    assert!(manager.unsubscribe("nonexistent").is_err());
}

#[tokio::test]
async fn test_sync_manager_get_statistics() {
    let manager = SyncManager::new();
    let stats = manager.get_statistics();
    assert_eq!(stats.status, SyncStatus::Healthy);
    assert_eq!(stats.pending_operations, 0);
    assert_eq!(stats.failed_operations, 0);
    assert_eq!(stats.active_partitions, 0);
}

#[tokio::test]
async fn test_sync_manager_process_heartbeat() {
    let mut manager = SyncManager::new();
    let msg = SyncMessage::new(
        SyncOperation::Heartbeat {
            node_id: "node-1".to_string(),
            timestamp: SystemTime::now(),
        },
        "node-1".to_string(),
    );
    let result = manager.process_message_with_retry(msg).await.unwrap();
    assert!(result.success);
    let stats = manager.get_statistics();
    assert_eq!(stats.connected_peers, 1);
}

#[tokio::test]
async fn test_sync_manager_process_state_update() {
    let mut manager = SyncManager::new();
    let state = make_state("s1", 1, SystemTime::now());
    let msg = SyncMessage::new(SyncOperation::StateUpdate(state), "node-1".to_string());
    let result = manager.process_message_with_retry(msg).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_sync_manager_resolve_conflict() {
    let manager = SyncManager::new();
    let now = SystemTime::now();
    let state1 = make_state("s1", 1, now);
    let state2 = make_state("s1", 2, now);
    let resolved = manager.resolve_conflict(&state1, &state2);
    assert_eq!(resolved.version, 2);
}

#[tokio::test]
async fn test_sync_manager_resolve_conflict_prefer_newer_version() {
    let manager = SyncManager::new();
    let now = SystemTime::now();
    let state1 = make_state("s1", 2, now);
    let state2 = make_state("s1", 1, now);
    let resolved = manager.resolve_conflict(&state1, &state2);
    assert_eq!(resolved.version, 2);
}

#[tokio::test]
async fn test_sync_manager_detect_partitions() {
    let mut manager = SyncManager::new();
    let result = manager.detect_partitions().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sync_manager_update_config() {
    let mut manager = SyncManager::new();
    let config = SyncConfig {
        sync_timeout_seconds: 120,
        ..Default::default()
    };
    manager.update_config(config);
    assert_eq!(manager.get_status(), SyncStatus::Healthy);
}

#[tokio::test]
async fn test_sync_manager_process_expired_message() {
    let mut manager = SyncManager::new();
    let state = make_state("s1", 1, SystemTime::now());
    let mut msg = SyncMessage::new(SyncOperation::StateUpdate(state), "node-1".to_string());
    msg.timestamp = SystemTime::now() - Duration::from_secs(400);
    let config = SyncConfig {
        max_message_age_seconds: 300,
        ..Default::default()
    };
    manager.update_config(config);
    let result = manager.process_message_with_retry(msg).await.unwrap();
    assert!(!result.success);
    assert!(result.message.contains("expired") || result.message.contains("Expired"));
}

#[tokio::test]
async fn test_sync_manager_broadcast_event() {
    let mut manager = SyncManager::new();
    let (tx, mut rx) = mpsc::channel::<SyncEvent>(10);
    manager.subscribe(tx);
    let event = SyncEvent::StateUpdated {
        version: 1,
        timestamp: SystemTime::now(),
        source: "test".to_string(),
    };
    manager.broadcast_event(event.clone()).await.unwrap();
    let received = rx.recv().await.unwrap();
    match (&received, &event) {
        (
            SyncEvent::StateUpdated { version: v1, .. },
            SyncEvent::StateUpdated { version: v2, .. },
        ) => {
            assert_eq!(v1, v2);
        }
        _ => panic!("Event mismatch"),
    }
}

#[test]
fn test_sync_statistics_fields() {
    let stats = SyncStatistics {
        status: SyncStatus::Healthy,
        pending_operations: 0,
        failed_operations: 0,
        active_partitions: 0,
        connected_peers: 0,
        subscribers: 0,
    };
    assert_eq!(stats.pending_operations, 0);
    assert_eq!(stats.subscribers, 0);
}
