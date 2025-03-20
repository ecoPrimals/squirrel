#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::test;
    use chrono::Duration;

    // ... existing code ...

    #[test]
    async fn test_state_recovery() {
        let temp_dir = tempdir().unwrap();
        let manager = StateManager::with_storage_path(temp_dir.path());

        // Register initial state
        let state = State {
            name: "test_state".to_string(),
            data: serde_json::json!({"value": 1}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        manager.register_state("test_state".to_string(), state.clone()).await.unwrap();

        // Make some transitions
        let state2 = State {
            name: "test_state".to_string(),
            data: serde_json::json!({"value": 2}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        manager.transition_state("test_state", "test_state", Some(serde_json::json!({"reason": "update"}))).await.unwrap();

        // List recovery points
        let points = manager.list_recovery_points("test_state").await.unwrap();
        assert_eq!(points.len(), 2); // Initial + transition

        // Verify state integrity
        assert!(manager.verify_state_integrity("test_state").await.unwrap());

        // Recover to initial state
        let initial_point_id = points[0].id;
        let recovered_state = manager.recover_state("test_state", Some(initial_point_id)).await.unwrap();
        assert_eq!(recovered_state.data.as_object().unwrap()["value"], 1);

        // Recover to latest state
        let recovered_state = manager.recover_state("test_state", None).await.unwrap();
        assert_eq!(recovered_state.data.as_object().unwrap()["value"], 2);

        // Clean up old points
        let cleaned = manager.cleanup_recovery_points(1).await.unwrap();
        assert_eq!(cleaned, 0); // No points should be cleaned as they're recent

        // Test cleanup with older points
        tokio::time::sleep(Duration::days(2).to_std().unwrap()).await;
        let cleaned = manager.cleanup_recovery_points(1).await.unwrap();
        assert_eq!(cleaned, 2); // Both points should be cleaned
    }

    #[test]
    async fn test_recovery_error_handling() {
        let temp_dir = tempdir().unwrap();
        let manager = StateManager::with_storage_path(temp_dir.path());

        // Test recovery of non-existent state
        let result = manager.recover_state("nonexistent", None).await;
        assert!(matches!(result, Err(StateError::Recovery(_))));

        // Test recovery with invalid point ID
        let state = State {
            name: "test_state".to_string(),
            data: serde_json::json!({"value": 1}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        manager.register_state("test_state".to_string(), state).await.unwrap();

        let result = manager.recover_state("test_state", Some(Uuid::new_v4())).await;
        assert!(matches!(result, Err(StateError::Recovery(_))));

        // Test integrity verification of non-existent state
        let result = manager.verify_state_integrity("nonexistent").await;
        assert!(matches!(result, Err(StateError::Recovery(_))));
    }

    // ... rest of the tests ...
} 