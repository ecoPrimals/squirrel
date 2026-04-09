// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;

#[tokio::test]
async fn test_session_creation() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    let session_id = manager
        .create_session(Some("test_client".to_string()))
        .await
        .expect("should succeed");
    assert!(!session_id.is_empty());

    let session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed");
    assert!(session.is_some());
}

#[tokio::test]
async fn test_session_update() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    let session_id = manager.create_session(None).await.expect("should succeed");

    let mut data = HashMap::new();
    data.insert(
        "test_key".to_string(),
        serde_json::Value::String("test_value".to_string()),
    );

    manager
        .update_session(&session_id, data)
        .await
        .expect("should succeed");

    let session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert_eq!(
        session.data.get("test_key"),
        Some(&serde_json::Value::String("test_value".to_string()))
    );
}

#[tokio::test]
async fn test_session_termination() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    let session_id = manager.create_session(None).await.expect("should succeed");
    manager
        .terminate_session(&session_id)
        .await
        .expect("should succeed");

    let session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert!(matches!(session.state, SessionState::Terminated));
}

#[tokio::test]
async fn test_session_manager_initialization_with_custom_config() {
    let config = SessionConfig {
        timeout: std::time::Duration::from_secs(600),
        max_connections: 50,
        enable_logging: false,
    };
    let manager = SessionManagerImpl::new(config.clone());

    // Verify config is stored correctly
    assert_eq!(manager.config.timeout, config.timeout);
    assert_eq!(manager.config.max_connections, config.max_connections);
    assert_eq!(manager.config.enable_logging, config.enable_logging);

    // Verify initial state
    assert_eq!(manager.get_active_session_count().await, 0);
}

#[tokio::test]
async fn test_session_manager_initialization_with_default_config() {
    let manager = SessionManagerImpl::new(SessionConfig::default());

    // Verify default config values
    assert_eq!(manager.config.timeout, std::time::Duration::from_secs(300));
    assert_eq!(manager.config.max_connections, 100);
    assert!(manager.config.enable_logging);

    // Verify initial state
    assert_eq!(manager.get_active_session_count().await, 0);
}

#[tokio::test]
async fn test_session_lifecycle_create_get_update_delete() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    // Create session
    let session_id = manager
        .create_session(Some("lifecycle_test_client".to_string()))
        .await
        .expect("should succeed");
    assert!(!session_id.is_empty());
    assert_eq!(manager.get_active_session_count().await, 1);

    // Get session
    let session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert_eq!(session.metadata.session_id, session_id);
    assert!(matches!(session.state, SessionState::Active));
    assert_eq!(
        session.metadata.client_info,
        Some("lifecycle_test_client".to_string())
    );

    // Update session
    let mut update_data = HashMap::new();
    update_data.insert(
        "user_id".to_string(),
        serde_json::Value::String("user_123".to_string()),
    );
    update_data.insert(
        "preferences".to_string(),
        serde_json::json!({"theme": "dark"}),
    );

    manager
        .update_session(&session_id, update_data.clone())
        .await
        .expect("should succeed");

    // Verify update
    let updated_session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert_eq!(
        updated_session.data.get("user_id"),
        Some(&serde_json::Value::String("user_123".to_string()))
    );
    assert!(updated_session.data.contains_key("preferences"));

    // Delete/terminate session
    manager
        .terminate_session(&session_id)
        .await
        .expect("should succeed");

    // Verify termination
    let terminated_session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert!(matches!(terminated_session.state, SessionState::Terminated));
    // Session still exists in map but is terminated
    assert_eq!(manager.get_active_session_count().await, 1);
}

#[tokio::test]
async fn test_session_timeout_cleanup() {
    let config = SessionConfig {
        timeout: std::time::Duration::from_millis(5), // Ultra-short timeout for fast tests
        max_connections: 100,
        enable_logging: true,
    };
    let manager = SessionManagerImpl::new(config);

    // Create multiple sessions
    let _session_id1 = manager.create_session(None).await.expect("should succeed");
    let _session_id2 = manager.create_session(None).await.expect("should succeed");
    let session_id3 = manager.create_session(None).await.expect("should succeed");

    assert_eq!(manager.get_active_session_count().await, 3);

    // Update one session to keep it active
    let mut keep_alive_data = HashMap::new();
    keep_alive_data.insert("keep_alive".to_string(), serde_json::json!(true));
    manager
        .update_session(&session_id3, keep_alive_data)
        .await
        .expect("should succeed");

    // Yield to let time pass for the ultra-short timeout (5ms)
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Cleanup expired sessions
    let removed_count = manager
        .cleanup_expired_sessions()
        .await
        .expect("should succeed");
    assert!(removed_count >= 2); // At least 2 sessions should be expired

    // Verify remaining session count
    let remaining_count = manager.get_active_session_count().await;
    assert!(remaining_count <= 1);
}

#[tokio::test]
async fn test_concurrent_session_access() {
    let config = SessionConfig::default();
    let manager = Arc::new(SessionManagerImpl::new(config));

    // Create a session
    let session_id = manager.create_session(None).await.expect("should succeed");

    // Spawn multiple concurrent tasks accessing the same session
    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let session_id_clone = session_id.clone();
        let handle = tokio::spawn(async move {
            let mut update_data = HashMap::new();
            update_data.insert(
                format!("concurrent_key_{i}"),
                serde_json::Value::Number(i.into()),
            );
            manager_clone
                .update_session(&session_id_clone, update_data)
                .await
                .expect("should succeed");

            let session = manager_clone
                .get_session(&session_id_clone)
                .await
                .expect("should succeed")
                .expect("should succeed");
            assert_eq!(session.metadata.session_id, session_id_clone);
            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.is_ok());
    }

    // Verify final session state
    let final_session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert_eq!(final_session.metadata.session_id, session_id);
    // Verify multiple updates were applied
    assert!(final_session.data.len() >= 10);
}

#[tokio::test]
async fn test_concurrent_session_creation() {
    let config = SessionConfig::default();
    let manager = Arc::new(SessionManagerImpl::new(config));

    // Create multiple sessions concurrently
    let mut handles = vec![];
    for i in 0..20 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let client_info = Some(format!("concurrent_client_{i}"));
            manager_clone
                .create_session(client_info)
                .await
                .expect("should succeed")
        });
        handles.push(handle);
    }

    // Wait for all sessions to be created
    let session_ids: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("should succeed"))
        .collect();

    // Verify all sessions are unique
    let unique_ids: std::collections::HashSet<_> = session_ids.iter().collect();
    assert_eq!(unique_ids.len(), 20);

    // Verify session count
    assert_eq!(manager.get_active_session_count().await, 20);
}

#[tokio::test]
async fn test_session_metadata_operations() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    // Create session with client info
    let client_info = Some("metadata_test_client".to_string());
    let session_id = manager
        .create_session(client_info.clone())
        .await
        .expect("should succeed");

    // Get session and verify metadata
    let session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert_eq!(session.metadata.session_id, session_id);
    assert_eq!(session.metadata.client_info, client_info);
    assert!(session.metadata.capabilities.contains(&"mcp".to_string()));
    assert!(
        session
            .metadata
            .capabilities
            .contains(&"ai_intelligence".to_string())
    );
    assert!(matches!(session.state, SessionState::Active));

    // Verify timestamps
    let created_at = session.metadata.created_at;
    let last_activity = session.metadata.last_activity;
    assert!(created_at <= last_activity);

    // Update session and verify last_activity is updated
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    let mut update_data = HashMap::new();
    update_data.insert("test".to_string(), serde_json::json!("value"));
    manager
        .update_session(&session_id, update_data)
        .await
        .expect("should succeed");

    let updated_session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert!(updated_session.metadata.last_activity > last_activity);
    assert_eq!(updated_session.metadata.created_at, created_at); // Created at shouldn't change
}

#[tokio::test]
async fn test_session_data_operations() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    let session_id = manager.create_session(None).await.expect("should succeed");

    // Test adding data
    let mut data1 = HashMap::new();
    data1.insert("key1".to_string(), serde_json::json!("value1"));
    data1.insert("key2".to_string(), serde_json::json!(42));
    manager
        .update_session(&session_id, data1)
        .await
        .expect("should succeed");

    let session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert_eq!(session.data.len(), 2);
    assert_eq!(session.data.get("key1"), Some(&serde_json::json!("value1")));
    assert_eq!(session.data.get("key2"), Some(&serde_json::json!(42)));

    // Test updating existing data
    let mut data2 = HashMap::new();
    data2.insert("key1".to_string(), serde_json::json!("updated_value1"));
    data2.insert("key3".to_string(), serde_json::json!({"nested": "object"}));
    manager
        .update_session(&session_id, data2)
        .await
        .expect("should succeed");

    let updated_session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert_eq!(updated_session.data.len(), 3);
    assert_eq!(
        updated_session.data.get("key1"),
        Some(&serde_json::json!("updated_value1"))
    );
    assert_eq!(
        updated_session.data.get("key3"),
        Some(&serde_json::json!({"nested": "object"}))
    );
}

#[tokio::test]
async fn test_session_state_transitions() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    // Create session - should be Active
    let session_id = manager.create_session(None).await.expect("should succeed");
    let session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert!(matches!(session.state, SessionState::Active));

    // Terminate session - should transition to Terminated
    manager
        .terminate_session(&session_id)
        .await
        .expect("should succeed");
    let terminated_session = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert!(matches!(terminated_session.state, SessionState::Terminated));

    // Verify terminated session can still be retrieved but state remains Terminated
    let still_terminated = manager
        .get_session(&session_id)
        .await
        .expect("should succeed")
        .expect("should succeed");
    assert!(matches!(still_terminated.state, SessionState::Terminated));
}

#[tokio::test]
async fn test_get_nonexistent_session() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    // Try to get a session that doesn't exist
    let result = manager
        .get_session("nonexistent_session_id")
        .await
        .expect("should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_nonexistent_session() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    // Try to update a session that doesn't exist
    let mut data = HashMap::new();
    data.insert("test".to_string(), serde_json::json!("value"));
    let result = manager.update_session("nonexistent_session_id", data).await;
    // Should not error, just do nothing
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_terminate_nonexistent_session() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    // Try to terminate a session that doesn't exist
    let result = manager.terminate_session("nonexistent_session_id").await;
    // Should not error, just do nothing
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_session_metadata() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    let session_id = manager
        .create_session(Some("metadata_test".to_string()))
        .await
        .expect("should succeed");

    let metadata = manager
        .get_session_metadata(&session_id)
        .await
        .expect("should succeed");
    assert_eq!(metadata.session_id, session_id);
    assert_eq!(metadata.client_info, Some("metadata_test".to_string()));
    assert!(metadata.capabilities.contains(&"mcp".to_string()));
}

#[tokio::test]
async fn test_get_session_metadata_nonexistent() {
    let config = SessionConfig::default();
    let manager = SessionManagerImpl::new(config);

    let result = manager.get_session_metadata("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_manager_trait_dispatch() {
    async fn exercise_manager<M: super::SessionManager>(m: &M) {
        let session_id = m.create_session(None).await.expect("should succeed");
        let metadata = m
            .get_session_metadata(&session_id)
            .await
            .expect("should succeed");
        assert_eq!(metadata.session_id, session_id);
    }

    let manager = SessionManagerImpl::new(SessionConfig::default());
    exercise_manager(&manager).await;
}
