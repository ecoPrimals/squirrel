// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Additional tests for the context tracker module

use super::tracker::{ContextTracker, ContextTrackerConfig};
use super::{ContextError, ContextState};
use serde_json::json;
use std::collections::HashMap;
use std::time::SystemTime;

#[tokio::test]
async fn test_tracker_get_state() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({"key": "value"}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state.clone());
    let retrieved = tracker.get_state().await.expect("Failed to get state");

    assert_eq!(retrieved.id, "test1");
    assert_eq!(retrieved.version, 1);
    assert_eq!(retrieved.data["key"], "value");
}

#[tokio::test]
async fn test_tracker_update_state_newer_version() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({"count": 1}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);

    let new_state = ContextState {
        id: "test1".to_string(),
        version: 2,
        timestamp: 2000,
        data: json!({"count": 2}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    tracker
        .update_state(new_state)
        .await
        .expect("Failed to update state");

    let retrieved = tracker.get_state().await.expect("Failed to get state");
    assert_eq!(retrieved.version, 2);
    assert_eq!(retrieved.data["count"], 2);
}

#[tokio::test]
async fn test_tracker_update_state_older_version() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 5,
        timestamp: 5000,
        data: json!({"count": 5}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);

    let old_state = ContextState {
        id: "test1".to_string(),
        version: 3,
        timestamp: 3000,
        data: json!({"count": 3}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    tracker
        .update_state(old_state)
        .await
        .expect("Update should succeed but not change state");

    let retrieved = tracker.get_state().await.expect("Failed to get state");
    assert_eq!(retrieved.version, 5);
    assert_eq!(retrieved.data["count"], 5);
}

#[tokio::test]
async fn test_tracker_update_state_same_version() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 3,
        timestamp: 3000,
        data: json!({"count": 3}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);

    let same_state = ContextState {
        id: "test1".to_string(),
        version: 3,
        timestamp: 3001,
        data: json!({"count": 30}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    tracker
        .update_state(same_state)
        .await
        .expect("Update should succeed but not change state");

    let retrieved = tracker.get_state().await.expect("Failed to get state");
    assert_eq!(retrieved.version, 3);
    assert_eq!(retrieved.data["count"], 3); // Original value unchanged
}

#[tokio::test]
async fn test_tracker_deactivate_context() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);

    let result = tracker.deactivate_context().await;
    assert!(result.is_ok());

    let active_id = tracker
        .get_active_context_id()
        .await
        .expect("Failed to get ID");
    assert!(active_id.is_none());
}

#[tokio::test]
async fn test_tracker_get_active_context_id_none() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);

    let active_id = tracker
        .get_active_context_id()
        .await
        .expect("Failed to get ID");
    assert!(active_id.is_none());
}

#[tokio::test]
async fn test_tracker_with_custom_config() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let config = ContextTrackerConfig {
        sync_interval_seconds: 120,
        auto_recovery: true,
        max_recovery_points: 20,
    };

    let tracker = ContextTracker::with_config_and_manager(state, config, None);

    let retrieved = tracker.get_state().await.expect("Failed to get state");
    assert_eq!(retrieved.id, "test1");
}

#[tokio::test]
async fn test_tracker_activate_context_without_manager() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);

    let result = tracker.activate_context("test2").await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(matches!(e, ContextError::NotInitialized));
    }
}

#[tokio::test]
async fn test_tracker_update_state_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("author".to_string(), json!("test_user"));

    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({"value": 100}),
        metadata,
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);

    let mut new_metadata = HashMap::new();
    new_metadata.insert("author".to_string(), json!("updated_user"));
    new_metadata.insert("updated_at".to_string(), json!("2024-01-01"));

    let new_state = ContextState {
        id: "test1".to_string(),
        version: 2,
        timestamp: 2000,
        data: json!({"value": 200}),
        metadata: new_metadata,
        synchronized: true,
        last_modified: SystemTime::now(),
    };

    tracker
        .update_state(new_state)
        .await
        .expect("Failed to update state");

    let retrieved = tracker.get_state().await.expect("Failed to get state");
    assert_eq!(retrieved.version, 2);
    assert_eq!(retrieved.data["value"], 200);
    assert!(retrieved.synchronized);
    assert_eq!(
        retrieved
            .metadata
            .get("author")
            .expect("test: should succeed"),
        "updated_user"
    );
    assert_eq!(
        retrieved
            .metadata
            .get("updated_at")
            .expect("test: should succeed"),
        "2024-01-01"
    );
}

#[tokio::test]
async fn test_tracker_update_state_synchronized_flag() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);

    let new_state = ContextState {
        id: "test1".to_string(),
        version: 2,
        timestamp: 2000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: true,
        last_modified: SystemTime::now(),
    };

    tracker
        .update_state(new_state)
        .await
        .expect("Failed to update state");

    let retrieved = tracker.get_state().await.expect("Failed to get state");
    assert!(retrieved.synchronized);
}

#[tokio::test]
async fn test_tracker_multiple_updates() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({"counter": 0}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);

    for i in 1..=10 {
        let new_state = ContextState {
            id: "test1".to_string(),
            version: i + 1,
            timestamp: 1000 + (i * 100),
            data: json!({"counter": i}),
            metadata: HashMap::new(),
            synchronized: false,
            last_modified: SystemTime::now(),
        };

        tracker
            .update_state(new_state)
            .await
            .expect("Failed to update state");
    }

    let retrieved = tracker.get_state().await.expect("Failed to get state");
    assert_eq!(retrieved.version, 11);
    assert_eq!(retrieved.data["counter"], 10);
}

#[tokio::test]
async fn test_tracker_config_default_values() {
    let config = ContextTrackerConfig::default();

    assert_eq!(config.sync_interval_seconds, 60);
    assert!(config.auto_recovery);
    assert_eq!(config.max_recovery_points, 10);
}

#[tokio::test]
async fn test_tracker_config_clone() {
    let config1 = ContextTrackerConfig {
        sync_interval_seconds: 30,
        auto_recovery: false,
        max_recovery_points: 15,
    };

    let config2 = config1.clone();

    assert_eq!(config1.sync_interval_seconds, config2.sync_interval_seconds);
    assert_eq!(config1.auto_recovery, config2.auto_recovery);
    assert_eq!(config1.max_recovery_points, config2.max_recovery_points);
}

#[tokio::test]
async fn test_tracker_state_clone() {
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), json!("value"));

    let state1 = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({"data": "value"}),
        metadata: metadata.clone(),
        synchronized: true,
        last_modified: SystemTime::now(),
    };

    let state2 = state1.clone();

    assert_eq!(state1.id, state2.id);
    assert_eq!(state1.version, state2.version);
    assert_eq!(state1.timestamp, state2.timestamp);
    assert_eq!(state1.data, state2.data);
    assert_eq!(state1.synchronized, state2.synchronized);
}

#[tokio::test]
async fn test_tracker_concurrent_reads() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({"shared": "data"}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = std::sync::Arc::new(ContextTracker::new(state));

    let mut handles = vec![];
    for _ in 0..10 {
        let tracker_clone = tracker.clone();
        let handle = tokio::spawn(async move {
            tracker_clone
                .get_state()
                .await
                .expect("Failed to get state")
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task failed");
        assert_eq!(result.id, "test1");
        assert_eq!(result.data["shared"], "data");
    }
}

#[tokio::test]
async fn test_tracker_concurrent_updates() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 0,
        timestamp: 0,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = std::sync::Arc::new(ContextTracker::new(state));

    let mut handles = vec![];
    for i in 1..=5 {
        let tracker_clone = tracker.clone();
        let handle = tokio::spawn(async move {
            let new_state = ContextState {
                id: "test1".to_string(),
                version: i,
                timestamp: i,
                data: json!({"version": i}),
                metadata: HashMap::new(),
                synchronized: false,
                last_modified: SystemTime::now(),
            };

            tracker_clone
                .update_state(new_state)
                .await
                .expect("Failed to update state");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task failed");
    }

    let final_state = tracker.get_state().await.expect("Failed to get state");
    assert!(final_state.version >= 1);
    assert!(final_state.version <= 5);
}

#[tokio::test]
async fn test_tracker_debug_impl() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let tracker = ContextTracker::new(state);
    let debug_str = format!("{:?}", tracker);
    assert!(debug_str.contains("ContextTracker"));
}

#[tokio::test]
async fn test_tracker_config_debug_impl() {
    let config = ContextTrackerConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ContextTrackerConfig"));
}

#[tokio::test]
async fn test_tracker_with_disabled_auto_recovery() {
    let state = ContextState {
        id: "test1".to_string(),
        version: 1,
        timestamp: 1000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let config = ContextTrackerConfig {
        sync_interval_seconds: 60,
        auto_recovery: false,
        max_recovery_points: 10,
    };

    let tracker = ContextTracker::with_config_and_manager(state, config, None);

    let new_state = ContextState {
        id: "test1".to_string(),
        version: 2,
        timestamp: 2000,
        data: json!({}),
        metadata: HashMap::new(),
        synchronized: false,
        last_modified: SystemTime::now(),
    };

    let result = tracker.update_state(new_state).await;
    assert!(result.is_ok());
}
