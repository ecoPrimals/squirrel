// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Additional tests for sync module

use super::sync::{
    ConflictInfo, ConflictResolutionStrategy, PartitionInfo, PartitionRecoveryStrategy, SyncConfig,
    SyncEvent, SyncMessage, SyncOperation,
};
use crate::ContextState;
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[test]
fn test_sync_message_new_creates_uuid() {
    let op = SyncOperation::Heartbeat {
        node_id: "node-001".to_string(),
        timestamp: SystemTime::now(),
    };

    let msg1 = SyncMessage::new(op.clone(), "source".to_string());
    let msg2 = SyncMessage::new(op, "source".to_string());

    assert_ne!(msg1.id, msg2.id); // UUIDs should be unique
}

#[test]
fn test_sync_message_serialize_deserialize() {
    let op = SyncOperation::Heartbeat {
        node_id: "node-001".to_string(),
        timestamp: SystemTime::now(),
    };

    let msg = SyncMessage::new(op, "source".to_string());
    let json = serde_json::to_string(&msg).expect("Failed to serialize");
    let deserialized: SyncMessage = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(msg.id, deserialized.id);
    assert_eq!(msg.source, deserialized.source);
}

#[test]
fn test_sync_operation_state_update() {
    let state = ContextState {
        id: "test".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let op = SyncOperation::StateUpdate(state.clone());

    match op {
        SyncOperation::StateUpdate(s) => {
            assert_eq!(s.id, "test");
        }
        _ => panic!("Wrong operation type"),
    }
}

#[test]
fn test_sync_operation_conflict() {
    let conflict = ConflictInfo {
        state_id: "conflict-state".to_string(),
        conflicting_versions: vec![],
        resolution_strategy: ConflictResolutionStrategy::KeepLatest,
        detected_at: SystemTime::now(),
        involved_nodes: vec!["node1".to_string(), "node2".to_string()],
    };

    let op = SyncOperation::Conflict(conflict);

    match op {
        SyncOperation::Conflict(c) => {
            assert_eq!(c.state_id, "conflict-state");
            assert_eq!(c.involved_nodes.len(), 2);
        }
        _ => panic!("Wrong operation type"),
    }
}

#[test]
fn test_conflict_info_multiple_versions() {
    let state1 = ContextState {
        id: "test".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({"v": 1}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let state2 = ContextState {
        id: "test".to_string(),
        version: 2,
        timestamp: 2000,
        data: json!({"v": 2}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let conflict = ConflictInfo {
        state_id: "test".to_string(),
        conflicting_versions: vec![state1, state2],
        resolution_strategy: ConflictResolutionStrategy::Merge,
        detected_at: SystemTime::now(),
        involved_nodes: vec!["node1".to_string()],
    };

    assert_eq!(conflict.conflicting_versions.len(), 2);
}

#[test]
fn test_sync_event_state_updated() {
    let event = SyncEvent::StateUpdated {
        version: 5,
        timestamp: SystemTime::now(),
        source: "node-001".to_string(),
    };

    match event {
        SyncEvent::StateUpdated {
            version, source, ..
        } => {
            assert_eq!(version, 5);
            assert_eq!(source, "node-001");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_sync_event_conflict_detected() {
    let conflict = ConflictInfo {
        state_id: "conflict".to_string(),
        conflicting_versions: vec![],
        resolution_strategy: ConflictResolutionStrategy::Manual,
        detected_at: SystemTime::now(),
        involved_nodes: vec![],
    };

    let event = SyncEvent::ConflictDetected {
        state_id: "conflict".to_string(),
        conflict: Box::new(conflict.clone()),
    };

    match event {
        SyncEvent::ConflictDetected { state_id, conflict } => {
            assert_eq!(state_id, "conflict");
            assert!(matches!(
                conflict.resolution_strategy,
                ConflictResolutionStrategy::Manual
            ));
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_sync_event_conflict_resolved() {
    let resolved_state = ContextState {
        id: "resolved".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let event = SyncEvent::ConflictResolved {
        state_id: "resolved".to_string(),
        strategy: ConflictResolutionStrategy::VectorClock,
        resolved_state,
    };

    match event {
        SyncEvent::ConflictResolved {
            state_id, strategy, ..
        } => {
            assert_eq!(state_id, "resolved");
            assert!(matches!(strategy, ConflictResolutionStrategy::VectorClock));
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_partition_info_serialize_deserialize() {
    let info = PartitionInfo {
        detected_at: SystemTime::now(),
        affected_peers: vec!["peer1".to_string(), "peer2".to_string()],
        partition_duration: Duration::from_secs(120),
        recovery_strategy: PartitionRecoveryStrategy::AttemptReconnection,
    };

    let json = serde_json::to_string(&info).expect("Failed to serialize");
    let deserialized: PartitionInfo = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(info.affected_peers.len(), deserialized.affected_peers.len());
    assert_eq!(info.partition_duration, deserialized.partition_duration);
}

#[test]
fn test_sync_operation_partition_detected() {
    let partition_info = PartitionInfo {
        detected_at: SystemTime::now(),
        affected_peers: vec!["peer1".to_string()],
        partition_duration: Duration::from_secs(60),
        recovery_strategy: PartitionRecoveryStrategy::WaitForHealing,
    };

    let op = SyncOperation::PartitionDetected(partition_info);

    match op {
        SyncOperation::PartitionDetected(info) => {
            assert_eq!(info.affected_peers.len(), 1);
        }
        _ => panic!("Wrong operation type"),
    }
}

#[test]
fn test_sync_operation_full_sync_response() {
    let op = SyncOperation::FullSyncResponse {
        states: vec![],
        snapshots: vec![],
    };

    match op {
        SyncOperation::FullSyncResponse { states, snapshots } => {
            assert!(states.is_empty());
            assert!(snapshots.is_empty());
        }
        _ => panic!("Wrong operation type"),
    }
}

#[test]
fn test_conflict_resolution_strategy_serialize() {
    let strategies = vec![
        ConflictResolutionStrategy::KeepLatest,
        ConflictResolutionStrategy::KeepOldest,
        ConflictResolutionStrategy::Merge,
        ConflictResolutionStrategy::Manual,
        ConflictResolutionStrategy::VectorClock,
        ConflictResolutionStrategy::Consensus,
    ];

    for strategy in strategies {
        let json = serde_json::to_string(&strategy).expect("Failed to serialize");
        let _: ConflictResolutionStrategy =
            serde_json::from_str(&json).expect("Failed to deserialize");
    }
}

#[test]
fn test_partition_recovery_strategy_serialize() {
    let strategies = vec![
        PartitionRecoveryStrategy::WaitForHealing,
        PartitionRecoveryStrategy::AttemptReconnection,
        PartitionRecoveryStrategy::UseCachedState,
        PartitionRecoveryStrategy::FailoverToBackup,
    ];

    for strategy in strategies {
        let json = serde_json::to_string(&strategy).expect("Failed to serialize");
        let _: PartitionRecoveryStrategy =
            serde_json::from_str(&json).expect("Failed to deserialize");
    }
}

#[test]
fn test_sync_message_priority_levels() {
    let op = SyncOperation::Heartbeat {
        node_id: "node".to_string(),
        timestamp: SystemTime::now(),
    };

    let normal_msg = SyncMessage::new(op.clone(), "source".to_string());
    let high_msg = SyncMessage::high_priority(op, "source".to_string());

    assert_eq!(normal_msg.priority, 0);
    assert_eq!(high_msg.priority, 10);
    assert!(high_msg.priority > normal_msg.priority);
}

#[test]
fn test_sync_event_serialize() {
    let event = SyncEvent::StateUpdated {
        version: 1,
        timestamp: SystemTime::now(),
        source: "test".to_string(),
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    let _: SyncEvent = serde_json::from_str(&json).expect("Failed to deserialize");
}

#[test]
fn test_conflict_info_serialize() {
    let conflict = ConflictInfo {
        state_id: "test".to_string(),
        conflicting_versions: vec![],
        resolution_strategy: ConflictResolutionStrategy::KeepLatest,
        detected_at: SystemTime::now(),
        involved_nodes: vec!["node1".to_string()],
    };

    let json = serde_json::to_string(&conflict).expect("Failed to serialize");
    let deserialized: ConflictInfo = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(conflict.state_id, deserialized.state_id);
}

#[test]
fn test_sync_message_with_checksum() {
    let op = SyncOperation::Heartbeat {
        node_id: "node".to_string(),
        timestamp: SystemTime::now(),
    };

    let mut msg = SyncMessage::new(op, "source".to_string());
    msg.checksum = Some("abc123".to_string());

    assert_eq!(msg.checksum, Some("abc123".to_string()));
}

#[test]
fn test_sync_message_multiple_increments() {
    let op = SyncOperation::Heartbeat {
        node_id: "node".to_string(),
        timestamp: SystemTime::now(),
    };

    let mut msg = SyncMessage::new(op, "source".to_string());

    for i in 1..=10 {
        msg.increment_retry();
        assert_eq!(msg.retry_count, i);
    }
}

#[test]
fn test_sync_config_edge_values() {
    let config = SyncConfig {
        sync_timeout_seconds: 0,
        heartbeat_interval_seconds: 0,
        max_retry_attempts: 0,
        retry_delay_seconds: 0,
        max_pending_operations: 0,
        auto_resolve_conflicts: false,
        partition_detection_timeout_seconds: 0,
        max_message_age_seconds: 0,
    };

    assert_eq!(config.max_retry_attempts, 0);
    assert_eq!(config.max_pending_operations, 0);
}

#[test]
fn test_sync_operation_clone_all_variants() {
    let operations = vec![
        SyncOperation::Heartbeat {
            node_id: "node".to_string(),
            timestamp: SystemTime::now(),
        },
        SyncOperation::FullSyncRequest {
            requesting_node: "node".to_string(),
        },
        SyncOperation::FullSyncResponse {
            states: vec![],
            snapshots: vec![],
        },
        SyncOperation::PartitionRecovered {
            recovered_at: SystemTime::now(),
            affected_peers: vec![],
        },
    ];

    for op in operations {
        let cloned = op.clone();
        let _ = format!("{:?}", cloned);
    }
}
