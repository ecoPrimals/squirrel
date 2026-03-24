// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: Sync mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#[cfg(test)]
mod tests {
    use crate::ContextState;
    use crate::sync::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();

        assert_eq!(config.sync_timeout_seconds, 30);
        assert_eq!(config.heartbeat_interval_seconds, 10);
        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.retry_delay_seconds, 2);
        assert_eq!(config.max_pending_operations, 1000);
        assert!(config.auto_resolve_conflicts);
        assert_eq!(config.partition_detection_timeout_seconds, 60);
        assert_eq!(config.max_message_age_seconds, 300);
    }

    #[test]
    fn test_sync_config_custom() {
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

        assert_eq!(config.sync_timeout_seconds, 60);
        assert!(!config.auto_resolve_conflicts);
        assert_eq!(config.max_pending_operations, 500);
    }

    #[test]
    fn test_sync_status_variants() {
        let healthy = SyncStatus::Healthy;
        let degraded = SyncStatus::Degraded;
        let unhealthy = SyncStatus::Unhealthy;
        let offline = SyncStatus::Offline;
        let partitioned = SyncStatus::Partitioned;

        assert!(matches!(healthy, SyncStatus::Healthy));
        assert!(matches!(degraded, SyncStatus::Degraded));
        assert!(matches!(unhealthy, SyncStatus::Unhealthy));
        assert!(matches!(offline, SyncStatus::Offline));
        assert!(matches!(partitioned, SyncStatus::Partitioned));

        assert_ne!(healthy, degraded);
        assert_ne!(unhealthy, offline);
    }

    #[test]
    fn test_sync_result_creation() {
        let result = SyncResult {
            success: true,
            message: "Sync completed successfully".to_string(),
            timestamp: SystemTime::now(),
            retry_count: 0,
        };

        assert!(result.success);
        assert_eq!(result.message, "Sync completed successfully");
        assert_eq!(result.retry_count, 0);
    }

    #[test]
    fn test_sync_result_with_retries() {
        let result = SyncResult {
            success: true,
            message: "Sync completed after retries".to_string(),
            timestamp: SystemTime::now(),
            retry_count: 2,
        };

        assert!(result.success);
        assert_eq!(result.retry_count, 2);
    }

    #[test]
    fn test_sync_result_failure() {
        let result = SyncResult {
            success: false,
            message: "Sync failed: network error".to_string(),
            timestamp: SystemTime::now(),
            retry_count: 3,
        };

        assert!(!result.success);
        assert!(result.message.contains("failed"));
        assert_eq!(result.retry_count, 3);
    }

    #[test]
    fn test_partition_recovery_strategies() {
        let wait = PartitionRecoveryStrategy::WaitForHealing;
        let reconnect = PartitionRecoveryStrategy::AttemptReconnection;
        let cached = PartitionRecoveryStrategy::UseCachedState;
        let failover = PartitionRecoveryStrategy::FailoverToBackup;

        assert!(matches!(wait, PartitionRecoveryStrategy::WaitForHealing));
        assert!(matches!(
            reconnect,
            PartitionRecoveryStrategy::AttemptReconnection
        ));
        assert!(matches!(cached, PartitionRecoveryStrategy::UseCachedState));
        assert!(matches!(
            failover,
            PartitionRecoveryStrategy::FailoverToBackup
        ));
    }

    #[test]
    fn test_partition_info_creation() {
        let partition = PartitionInfo {
            detected_at: SystemTime::now(),
            affected_peers: vec!["node1".to_string(), "node2".to_string()],
            partition_duration: Duration::from_secs(30),
            recovery_strategy: PartitionRecoveryStrategy::AttemptReconnection,
        };

        assert_eq!(partition.affected_peers.len(), 2);
        assert_eq!(partition.partition_duration.as_secs(), 30);
        assert!(matches!(
            partition.recovery_strategy,
            PartitionRecoveryStrategy::AttemptReconnection
        ));
    }

    #[test]
    fn test_sync_message_new() {
        let state = ContextState::default();
        let operation = SyncOperation::StateUpdate(state);
        let message = SyncMessage::new(operation, "node-1".to_string());

        assert_eq!(message.source, "node-1");
        assert_eq!(message.priority, 0);
        assert_eq!(message.retry_count, 0);
        assert!(message.checksum.is_none());
        assert!(!message.id.is_empty());
    }

    #[test]
    fn test_sync_message_high_priority() {
        let state = ContextState::default();
        let operation = SyncOperation::StateUpdate(state);
        let message = SyncMessage::high_priority(operation, "node-1".to_string());

        assert_eq!(message.source, "node-1");
        assert_eq!(message.priority, 10);
        assert_eq!(message.retry_count, 0);
    }

    #[test]
    fn test_sync_message_is_expired_fresh() {
        let state = ContextState::default();
        let operation = SyncOperation::StateUpdate(state);
        let message = SyncMessage::new(operation, "node-1".to_string());
        let config = SyncConfig::default();

        // Fresh message should not be expired
        assert!(!message.is_expired(&config));
    }

    #[test]
    fn test_sync_message_is_expired_old() {
        let state = ContextState::default();
        let operation = SyncOperation::StateUpdate(state);
        let mut message = SyncMessage::new(operation, "node-1".to_string());

        // Set timestamp to 10 minutes ago
        message.timestamp = SystemTime::now() - Duration::from_secs(600);

        let config = SyncConfig::default(); // max_message_age_seconds = 300 (5 minutes)

        // Old message should be expired
        assert!(message.is_expired(&config));
    }

    #[test]
    fn test_sync_message_increment_retry() {
        let state = ContextState::default();
        let operation = SyncOperation::StateUpdate(state);
        let mut message = SyncMessage::new(operation, "node-1".to_string());

        assert_eq!(message.retry_count, 0);

        message.increment_retry();
        assert_eq!(message.retry_count, 1);

        message.increment_retry();
        assert_eq!(message.retry_count, 2);

        message.increment_retry();
        assert_eq!(message.retry_count, 3);
    }

    #[test]
    fn test_sync_message_with_checksum() {
        let state = ContextState::default();
        let operation = SyncOperation::StateUpdate(state);
        let mut message = SyncMessage::new(operation, "node-1".to_string());

        assert!(message.checksum.is_none());

        message.checksum = Some("abc123".to_string());
        assert_eq!(message.checksum.expect("should succeed"), "abc123");
    }

    #[test]
    fn test_sync_operation_state_update() {
        let state = ContextState::default();
        let operation = SyncOperation::StateUpdate(state);

        assert!(matches!(operation, SyncOperation::StateUpdate(_)));
    }

    #[test]
    fn test_sync_config_serialization() {
        let config = SyncConfig::default();

        // Test that config can be serialized
        let serialized = serde_json::to_string(&config).expect("should succeed");
        assert!(!serialized.is_empty());

        // Test that it can be deserialized
        let deserialized: SyncConfig = serde_json::from_str(&serialized).expect("should succeed");
        assert_eq!(
            deserialized.sync_timeout_seconds,
            config.sync_timeout_seconds
        );
        assert_eq!(deserialized.max_retry_attempts, config.max_retry_attempts);
    }

    #[test]
    fn test_partition_info_serialization() {
        let partition = PartitionInfo {
            detected_at: SystemTime::now(),
            affected_peers: vec!["node1".to_string()],
            partition_duration: Duration::from_secs(60),
            recovery_strategy: PartitionRecoveryStrategy::WaitForHealing,
        };

        // Test serialization
        let serialized = serde_json::to_string(&partition).expect("should succeed");
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized: PartitionInfo =
            serde_json::from_str(&serialized).expect("should succeed");
        assert_eq!(deserialized.affected_peers.len(), 1);
        assert_eq!(deserialized.partition_duration.as_secs(), 60);
    }

    #[test]
    fn test_sync_message_serialization() {
        let state = ContextState::default();
        let operation = SyncOperation::StateUpdate(state);
        let message = SyncMessage::new(operation, "node-1".to_string());

        // Test serialization
        let serialized = serde_json::to_string(&message).expect("should succeed");
        assert!(!serialized.is_empty());
        assert!(serialized.contains("node-1"));

        // Test deserialization
        let deserialized: SyncMessage = serde_json::from_str(&serialized).expect("should succeed");
        assert_eq!(deserialized.source, "node-1");
        assert_eq!(deserialized.retry_count, 0);
    }

    #[test]
    fn test_sync_config_clone() {
        let config1 = SyncConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.sync_timeout_seconds, config2.sync_timeout_seconds);
        assert_eq!(config1.max_retry_attempts, config2.max_retry_attempts);
    }

    #[test]
    fn test_sync_message_priority_ordering() {
        let state = ContextState::default();
        let low = SyncMessage::new(
            SyncOperation::StateUpdate(state.clone()),
            "node-1".to_string(),
        );
        let high =
            SyncMessage::high_priority(SyncOperation::StateUpdate(state), "node-2".to_string());

        assert!(high.priority > low.priority);
        assert_eq!(low.priority, 0);
        assert_eq!(high.priority, 10);
    }

    #[test]
    fn test_multiple_retry_increments() {
        let state = ContextState::default();
        let operation = SyncOperation::StateUpdate(state);
        let mut message = SyncMessage::new(operation, "node-1".to_string());

        let max_retries = 10;
        for i in 0..max_retries {
            assert_eq!(message.retry_count, i);
            message.increment_retry();
        }

        assert_eq!(message.retry_count, max_retries);
    }

    #[test]
    fn test_sync_config_timeout_validation() {
        let config = SyncConfig {
            sync_timeout_seconds: 0,
            ..Default::default()
        };

        // Zero timeout should still be represented correctly
        assert_eq!(config.sync_timeout_seconds, 0);
    }

    #[test]
    fn test_partition_with_no_peers() {
        let partition = PartitionInfo {
            detected_at: SystemTime::now(),
            affected_peers: vec![],
            partition_duration: Duration::from_secs(0),
            recovery_strategy: PartitionRecoveryStrategy::UseCachedState,
        };

        assert_eq!(partition.affected_peers.len(), 0);
        assert_eq!(partition.partition_duration.as_secs(), 0);
    }

    #[test]
    fn test_partition_with_many_peers() {
        let peers: Vec<String> = (0..100).map(|i| format!("node-{}", i)).collect();
        let partition = PartitionInfo {
            detected_at: SystemTime::now(),
            affected_peers: peers.clone(),
            partition_duration: Duration::from_secs(300),
            recovery_strategy: PartitionRecoveryStrategy::FailoverToBackup,
        };

        assert_eq!(partition.affected_peers.len(), 100);
        assert!(partition.affected_peers.contains(&"node-50".to_string()));
    }

    #[test]
    fn test_sync_result_clone() {
        let result1 = SyncResult {
            success: true,
            message: "Test".to_string(),
            timestamp: SystemTime::now(),
            retry_count: 1,
        };

        let result2 = result1.clone();
        assert_eq!(result1.success, result2.success);
        assert_eq!(result1.message, result2.message);
        assert_eq!(result1.retry_count, result2.retry_count);
    }

    #[test]
    fn test_sync_message_unique_ids() {
        let state = ContextState::default();
        let msg1 = SyncMessage::new(
            SyncOperation::StateUpdate(state.clone()),
            "node-1".to_string(),
        );
        let msg2 = SyncMessage::new(SyncOperation::StateUpdate(state), "node-1".to_string());

        // Each message should have a unique ID
        assert_ne!(msg1.id, msg2.id);
    }

    #[test]
    fn test_sync_status_equality() {
        let status1 = SyncStatus::Healthy;
        let status2 = SyncStatus::Healthy;
        let status3 = SyncStatus::Degraded;

        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
    }
}
