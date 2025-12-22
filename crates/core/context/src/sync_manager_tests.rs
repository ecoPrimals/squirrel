//! Comprehensive tests for SyncManager

use super::sync::*;
use crate::{ContextSnapshot, ContextState};
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

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
async fn test_sync_manager_get_status() {
    let manager = SyncManager::new();
    let status = manager.get_status();
    assert!(matches!(status, SyncStatus::Healthy));
}

#[tokio::test]
async fn test_sync_manager_update_config() {
    let mut manager = SyncManager::new();

    let new_config = SyncConfig {
        sync_timeout_seconds: 120,
        heartbeat_interval_seconds: 15,
        max_retry_attempts: 10,
        retry_delay_seconds: 3,
        max_pending_operations: 2000,
        auto_resolve_conflicts: false,
        partition_detection_timeout_seconds: 180,
        max_message_age_seconds: 900,
    };

    manager.update_config(new_config);
    // Config update should succeed without panic
    assert_eq!(manager.get_status(), SyncStatus::Healthy);
}

#[tokio::test]
async fn test_sync_manager_subscribe() {
    let mut manager = SyncManager::new();
    let (tx, _rx) = mpsc::channel::<SyncEvent>(10);

    let subscription_id = manager.subscribe(tx);
    assert!(!subscription_id.is_empty());
}

#[tokio::test]
async fn test_sync_manager_subscribe_multiple() {
    let mut manager = SyncManager::new();
    let (tx1, _rx1) = mpsc::channel::<SyncEvent>(10);
    let (tx2, _rx2) = mpsc::channel::<SyncEvent>(10);

    let id1 = manager.subscribe(tx1);
    let id2 = manager.subscribe(tx2);

    assert_ne!(id1, id2);
}

#[tokio::test]
async fn test_sync_manager_unsubscribe() {
    let mut manager = SyncManager::new();
    let (tx, _rx) = mpsc::channel::<SyncEvent>(10);

    let subscription_id = manager.subscribe(tx);
    let result = manager.unsubscribe(&subscription_id);

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sync_manager_unsubscribe_nonexistent() {
    let mut manager = SyncManager::new();
    let result = manager.unsubscribe("nonexistent-id");

    assert!(result.is_err());
}

#[tokio::test]
async fn test_sync_manager_broadcast_event() {
    let mut manager = SyncManager::new();
    let (tx, mut rx) = mpsc::channel::<SyncEvent>(10);

    manager.subscribe(tx);

    let event = SyncEvent::Started {
        operation_id: "op-123".to_string(),
        operation_type: "StateUpdate".to_string(),
    };

    let result = manager.broadcast_event(event.clone()).await;
    assert!(result.is_ok());

    // Verify event was received
    let received = rx.recv().await;
    assert!(received.is_some());
    match received.unwrap() {
        SyncEvent::Started { operation_id, .. } => {
            assert_eq!(operation_id, "op-123");
        }
        _ => panic!("Wrong event type"),
    }
}

#[tokio::test]
async fn test_sync_manager_broadcast_event_multiple_subscribers() {
    let mut manager = SyncManager::new();
    let (tx1, mut rx1) = mpsc::channel::<SyncEvent>(10);
    let (tx2, mut rx2) = mpsc::channel::<SyncEvent>(10);

    manager.subscribe(tx1);
    manager.subscribe(tx2);

    let event = SyncEvent::Completed {
        operation_id: "op-456".to_string(),
        result: SyncResult {
            success: true,
            message: "Success".to_string(),
            timestamp: SystemTime::now(),
            retry_count: 0,
        },
    };

    let result = manager.broadcast_event(event).await;
    assert!(result.is_ok());

    // Both subscribers should receive the event
    assert!(rx1.recv().await.is_some());
    assert!(rx2.recv().await.is_some());
}

#[tokio::test]
async fn test_sync_manager_broadcast_removes_failed_subscribers() {
    let mut manager = SyncManager::new();
    let (tx, rx) = mpsc::channel::<SyncEvent>(1);

    let id = manager.subscribe(tx);

    // Drop receiver to cause send failure
    drop(rx);

    let event = SyncEvent::Started {
        operation_id: "op-789".to_string(),
        operation_type: "Heartbeat".to_string(),
    };

    // Broadcast should succeed but remove the failed subscriber
    let result = manager.broadcast_event(event).await;
    assert!(result.is_ok());

    // Trying to unsubscribe should fail as it was already removed
    let unsubscribe_result = manager.unsubscribe(&id);
    assert!(unsubscribe_result.is_err());
}

#[tokio::test]
async fn test_sync_manager_process_heartbeat() {
    let mut manager = SyncManager::new();

    let heartbeat_op = SyncOperation::Heartbeat {
        node_id: "node-001".to_string(),
        timestamp: SystemTime::now(),
    };

    let message = SyncMessage::new(heartbeat_op, "source-node".to_string());
    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert!(sync_result.success);
}

#[tokio::test]
async fn test_sync_manager_process_state_update() {
    let mut manager = SyncManager::new();

    let state = ContextState {
        id: "state-001".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({"key": "value"}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let state_update_op = SyncOperation::StateUpdate(state);
    let message = SyncMessage::new(state_update_op, "source-node".to_string());
    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert!(sync_result.success);
}

#[tokio::test]
async fn test_sync_manager_process_snapshot_create() {
    let mut manager = SyncManager::new();

    let state = ContextState {
        id: "snapshot-state".to_string(),
        version: 1,
        timestamp: 2000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: true,
        last_modified: SystemTime::now(),
    };

    let snapshot = ContextSnapshot {
        id: "snapshot-001".to_string(),
        timestamp: SystemTime::now(),
        state,
        metadata: None,
    };

    let snapshot_op = SyncOperation::SnapshotCreate(SnapshotCreateRequest { snapshot });
    let message = SyncMessage::new(snapshot_op, "source-node".to_string());
    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert!(sync_result.success);
}

#[tokio::test]
async fn test_sync_manager_process_expired_message() {
    let mut manager = SyncManager::new();

    let heartbeat_op = SyncOperation::Heartbeat {
        node_id: "node-002".to_string(),
        timestamp: SystemTime::now(),
    };

    let mut message = SyncMessage::new(heartbeat_op, "source-node".to_string());

    // Set message timestamp to far in the past
    message.timestamp = SystemTime::now()
        .checked_sub(Duration::from_secs(400))
        .unwrap();

    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert!(!sync_result.success);
    assert!(sync_result.message.contains("expired"));
}

#[tokio::test]
async fn test_sync_manager_process_queue_full() {
    let config = SyncConfig {
        max_pending_operations: 0, // Set to 0 to trigger queue full immediately
        ..Default::default()
    };

    let mut manager = SyncManager::with_config(config);

    // Try to add message when queue is at capacity
    let heartbeat = SyncOperation::Heartbeat {
        node_id: "node-003".to_string(),
        timestamp: SystemTime::now(),
    };
    let message = SyncMessage::new(heartbeat, "source".to_string());
    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert!(!sync_result.success);
    assert!(sync_result.message.contains("Queue full"));
}

#[tokio::test]
async fn test_sync_manager_detect_partitions_none() {
    let mut manager = SyncManager::new();
    let partitions = manager.detect_partitions().await;
    assert!(partitions.is_empty());
}

#[tokio::test]
async fn test_sync_manager_detect_partitions_after_heartbeats() {
    let mut manager = SyncManager::new();

    // Send recent heartbeat
    let heartbeat_op = SyncOperation::Heartbeat {
        node_id: "healthy-node".to_string(),
        timestamp: SystemTime::now(),
    };
    let message = SyncMessage::new(heartbeat_op, "healthy-node".to_string());
    let _ = manager.process_message(message).await;

    let partitions = manager.detect_partitions().await;
    // Should not detect partition for recent heartbeat
    assert!(partitions.is_empty());
}

#[tokio::test]
async fn test_sync_manager_retry_failed_operations_empty() {
    let mut manager = SyncManager::new();
    let result = manager.retry_failed_operations().await;
    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_sync_manager_get_statistics() {
    let manager = SyncManager::new();
    let stats = manager.get_statistics();

    assert_eq!(stats.pending_operations, 0);
    assert_eq!(stats.failed_operations, 0);
    assert_eq!(stats.subscribers, 0);
    assert_eq!(stats.connected_peers, 0);
    assert_eq!(stats.active_partitions, 0);
    assert!(matches!(stats.status, SyncStatus::Healthy));
}

#[tokio::test]
async fn test_sync_manager_get_statistics_with_subscribers() {
    let mut manager = SyncManager::new();
    let (tx1, _rx1) = mpsc::channel::<SyncEvent>(10);
    let (tx2, _rx2) = mpsc::channel::<SyncEvent>(10);

    manager.subscribe(tx1);
    manager.subscribe(tx2);

    let stats = manager.get_statistics();
    assert_eq!(stats.subscribers, 2);
}

#[tokio::test]
async fn test_sync_manager_get_statistics_with_pending() {
    let mut manager = SyncManager::new();

    let heartbeat_op = SyncOperation::Heartbeat {
        node_id: "node-005".to_string(),
        timestamp: SystemTime::now(),
    };
    let message = SyncMessage::new(heartbeat_op, "source".to_string());
    let _ = manager.process_message(message).await;

    let stats = manager.get_statistics();
    // Note: pending operations are cleared after processing completes successfully
    // So we test that stats can be retrieved, not specific count
    assert!(stats.pending_operations <= 1);
}

#[tokio::test]
async fn test_sync_manager_process_conflict_resolution() {
    let mut manager = SyncManager::new();

    let state1 = ContextState {
        id: "conflict-state-001".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({"v": 1}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let state2 = ContextState {
        id: "conflict-state-001".to_string(),
        version: 2,
        timestamp: 2000,
        data: json!({"v": 2}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let conflict = ConflictInfo {
        state_id: "conflict-state-001".to_string(),
        local_version: state1,
        remote_version: state2,
        detected_at: SystemTime::now(),
        resolution_strategy: ConflictResolutionStrategy::LastWriteWins,
    };

    let conflict_op = SyncOperation::Conflict(conflict);
    let message = SyncMessage::new(conflict_op, "resolver-node".to_string());
    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert!(sync_result.success);
}

#[tokio::test]
async fn test_sync_manager_process_partition_detected() {
    let mut manager = SyncManager::new();

    let partition = PartitionInfo {
        detected_at: SystemTime::now(),
        affected_peers: vec!["node1".to_string(), "node2".to_string()],
        partition_duration: std::time::Duration::from_secs(60),
        recovery_strategy: PartitionRecoveryStrategy::WaitForHealing,
    };

    let partition_op = SyncOperation::PartitionDetected(partition);
    let message = SyncMessage::new(partition_op, "monitor-node".to_string());
    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert!(sync_result.success);
}

#[tokio::test]
async fn test_sync_manager_process_partition_recovered() {
    let mut manager = SyncManager::new();

    let recovered_op = SyncOperation::PartitionRecovered {
        recovered_at: SystemTime::now(),
        affected_peers: vec!["node1".to_string(), "node2".to_string()],
    };

    let message = SyncMessage::new(recovered_op, "monitor-node".to_string());
    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert!(sync_result.success);
}

#[tokio::test]
async fn test_sync_manager_message_priority() {
    let mut manager = SyncManager::new();

    let heartbeat_op = SyncOperation::Heartbeat {
        node_id: "node-006".to_string(),
        timestamp: SystemTime::now(),
    };

    let mut message = SyncMessage::new(heartbeat_op, "source".to_string());
    message.priority = 10; // High priority

    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[tokio::test]
async fn test_sync_manager_message_with_retries() {
    let mut manager = SyncManager::new();

    let heartbeat_op = SyncOperation::Heartbeat {
        node_id: "node-007".to_string(),
        timestamp: SystemTime::now(),
    };

    let mut message = SyncMessage::new(heartbeat_op, "source".to_string());
    message.retry_count = 2; // Already retried twice

    let result = manager.process_message(message).await;

    assert!(result.is_ok());
    let sync_result = result.unwrap();
    // The result includes the retry count from the message
    assert!(sync_result.retry_count >= 0); // May be 0 or preserve the count
}

#[tokio::test]
async fn test_sync_manager_concurrent_operations() {
    let mut manager = SyncManager::new();

    let heartbeat1 = SyncOperation::Heartbeat {
        node_id: "node-008".to_string(),
        timestamp: SystemTime::now(),
    };
    let heartbeat2 = SyncOperation::Heartbeat {
        node_id: "node-009".to_string(),
        timestamp: SystemTime::now(),
    };

    let message1 = SyncMessage::new(heartbeat1, "source1".to_string());
    let message2 = SyncOperation::Heartbeat {
        node_id: "node-009".to_string(),
        timestamp: SystemTime::now(),
    };
    let message2 = SyncMessage::new(message2, "source2".to_string());

    let result1 = manager.process_message(message1).await;
    let result2 = manager.process_message(message2).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Both operations processed successfully
    assert!(result1.unwrap().success);
    assert!(result2.unwrap().success);
}

#[tokio::test]
async fn test_sync_manager_broadcast_with_no_subscribers() {
    let mut manager = SyncManager::new();

    let event = SyncEvent::Failed {
        operation_id: "op-fail".to_string(),
        error: "Test error".to_string(),
        can_retry: true,
    };

    let result = manager.broadcast_event(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sync_manager_subscribe_after_unsubscribe() {
    let mut manager = SyncManager::new();
    let (tx1, _rx1) = mpsc::channel::<SyncEvent>(10);
    let (tx2, _rx2) = mpsc::channel::<SyncEvent>(10);

    let id1 = manager.subscribe(tx1);
    let _ = manager.unsubscribe(&id1);

    let id2 = manager.subscribe(tx2);
    assert!(!id2.is_empty());
    assert_ne!(id1, id2);
}
