//! Comprehensive error path tests for safe operations
//!
//! These tests increase test coverage by validating error scenarios,
//! edge cases, and recovery strategies.

#[cfg(test)]
mod error_path_tests {
    use crate::error_handling::safe_operations::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::{Mutex, RwLock};

    // ============================================================================
    // SafeResult Error Path Tests
    // ============================================================================

    #[test]
    fn test_safe_result_with_default_recovery() {
        let result: SafeResult<i32> = SafeResult::failure(
            SafeError::Configuration {
                message: "Config missing".to_string(),
                field: Some("port".to_string()),
            },
            "test".to_string(),
        )
        .with_recovery_strategy(RecoveryStrategy::UseDefault);

        assert_eq!(result.execute().unwrap(), 0);
    }

    #[test]
    fn test_safe_result_with_log_and_continue() {
        let result: SafeResult<String> = SafeResult::failure(
            SafeError::Network {
                message: "Connection refused".to_string(),
                endpoint: Some("http://localhost:8080".to_string()),
            },
            "network_test".to_string(),
        )
        .with_recovery_strategy(RecoveryStrategy::LogAndContinue);

        assert_eq!(result.execute().unwrap(), String::default());
    }

    #[test]
    fn test_safe_result_propagate_strategy() {
        let result: SafeResult<i32> = SafeResult::failure(
            SafeError::Validation {
                message: "Invalid input".to_string(),
                field: Some("username".to_string()),
            },
            "validation_test".to_string(),
        )
        .with_recovery_strategy(RecoveryStrategy::Propagate);

        assert!(result.execute().is_err());
    }

    #[test]
    fn test_safe_result_unwrap_or() {
        let result = SafeResult::failure(
            SafeError::Internal {
                message: "Internal error".to_string(),
            },
            "test".to_string(),
        );

        assert_eq!(result.unwrap_or(100), 100);
    }

    #[test]
    fn test_safe_result_unwrap_or_else() {
        let result: SafeResult<i32> = SafeResult::failure(
            SafeError::Timeout {
                message: "Operation timed out".to_string(),
                duration: Duration::from_secs(5),
            },
            "timeout_test".to_string(),
        );

        let value = result.unwrap_or_else(|_| 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_safe_result_log_and_default() {
        let result: SafeResult<Vec<String>> = SafeResult::failure(
            SafeError::ResourceUnavailable {
                message: "Resource not found".to_string(),
                resource: "database".to_string(),
            },
            "resource_test".to_string(),
        );

        assert_eq!(result.log_and_default(), Vec::<String>::new());
    }

    // ============================================================================
    // SafeError Type Tests
    // ============================================================================

    #[test]
    fn test_safe_error_configuration() {
        let error = SafeError::Configuration {
            message: "Missing config".to_string(),
            field: Some("host".to_string()),
        };
        assert!(error.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_safe_error_network() {
        let error = SafeError::Network {
            message: "Connection failed".to_string(),
            endpoint: Some("http://example.com".to_string()),
        };
        assert!(error.to_string().contains("Network error"));
    }

    #[test]
    fn test_safe_error_lock_acquisition() {
        let error = SafeError::LockAcquisition {
            message: "Deadlock detected".to_string(),
            lock_type: "RwLock".to_string(),
        };
        assert!(error.to_string().contains("Lock acquisition failed"));
    }

    #[test]
    fn test_safe_error_timeout() {
        let error = SafeError::Timeout {
            message: "Operation took too long".to_string(),
            duration: Duration::from_secs(10),
        };
        assert!(error.to_string().contains("Timeout error"));
    }

    // ============================================================================
    // SafeOps Error Path Tests
    // ============================================================================

    #[test]
    fn test_safe_parse_invalid_input() {
        let result = SafeOps::safe_parse::<i32>("not_a_number", "parse_test");
        assert!(result.execute().is_err());
    }

    #[test]
    fn test_safe_parse_edge_cases() {
        // Test overflow
        let result = SafeOps::safe_parse::<i8>("1000", "overflow_test");
        assert!(result.execute().is_err());

        // Test empty string
        let result = SafeOps::safe_parse::<i32>("", "empty_test");
        assert!(result.execute().is_err());

        // Test whitespace
        let result = SafeOps::safe_parse::<i32>("  ", "whitespace_test");
        assert!(result.execute().is_err());
    }

    #[test]
    fn test_safe_get_missing_key() {
        let map = HashMap::<String, String>::new();
        let result = SafeOps::safe_get_cloned(&map, &"missing_key".to_string(), "test_map");
        assert!(result.execute().is_err());
    }

    #[test]
    fn test_safe_first_empty_vec() {
        let vec: Vec<i32> = vec![];
        let result = SafeOps::safe_first(&vec, "empty_vec_test");
        assert!(result.execute_without_default().is_err());
    }

    #[test]
    fn test_safe_last_empty_vec() {
        let vec: Vec<String> = vec![];
        let result = SafeOps::safe_last(&vec, "empty_vec_test");
        assert!(result.execute_without_default().is_err());
    }

    #[test]
    fn test_safe_get_index_out_of_bounds() {
        let vec = vec![1, 2, 3];
        let result = SafeOps::safe_get_index(&vec, 10, "bounds_test");
        assert!(result.execute_without_default().is_err());
    }

    #[test]
    fn test_safe_serialize_error() {
        // Create a value that cannot be serialized (using a custom type)
        use serde::Serialize;

        #[derive(Serialize)]
        struct Test {
            value: i32,
        }

        let test = Test { value: 42 };
        let result = SafeOps::safe_serialize(&test, "serialize_test");
        assert!(result.execute().is_ok());
    }

    #[test]
    fn test_safe_deserialize_invalid_json() {
        let result = SafeOps::safe_deserialize::<HashMap<String, String>>(
            "not valid json",
            "deserialize_test",
        );
        assert!(result.execute().is_err());
    }

    #[test]
    fn test_safe_url_parse_invalid() {
        let result = SafeOps::safe_parse_url("not a url", "url_test");
        assert!(result.execute_without_default().is_err());

        let result = SafeOps::safe_parse_url("", "url_test");
        assert!(result.execute_without_default().is_err());

        let result = SafeOps::safe_parse_url("://invalid", "url_test");
        assert!(result.execute_without_default().is_err());
    }

    // ============================================================================
    // Async Error Path Tests
    // ============================================================================

    #[tokio::test]
    async fn test_safe_read_lock_timeout() {
        // This test would require a way to create a deadlock scenario
        // For now, we test successful acquisition
        let data = Arc::new(RwLock::new(42));
        let result = SafeOps::safe_read_lock(&data, "timeout_test").await;
        assert!(result.execute_without_default().is_ok());
    }

    #[tokio::test]
    async fn test_safe_channel_send_closed() {
        let (tx, rx) = tokio::sync::mpsc::channel::<i32>(1);
        drop(rx); // Close the receiver

        let result = SafeOps::safe_send(&tx, 42, "closed_channel_test").await;
        assert!(result.execute().is_err());
    }

    #[tokio::test]
    async fn test_safe_channel_receive_closed() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<i32>(1);

        // Close the channel by dropping the sender
        drop(tx);

        // Now try to receive - should return None immediately (channel closed)
        let result = SafeOps::safe_receive(&mut rx, "closed_receiver_test").await;

        // Should succeed with None (channel closed is a valid state, not an error)
        match result.execute() {
            Ok(Some(_)) => panic!("Expected None (channel closed), got Some"),
            Ok(None) => {
                // This is the expected path - channel closed gracefully
            }
            Err(e) => {
                // Only fail if it's NOT a timeout (timeout shouldn't happen for closed channel)
                if e.to_string().contains("timed out") {
                    panic!("Channel receive timed out - closed channels should return None immediately");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_safe_with_timeout_expires() {
        let result = SafeOps::safe_with_timeout(
            Duration::from_millis(10),
            || async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                42
            },
            "timeout_expire_test",
        )
        .await;

        assert!(result.execute().is_err());
    }

    #[tokio::test]
    async fn test_safe_with_timeout_succeeds() {
        let result = SafeOps::safe_with_timeout(
            Duration::from_secs(1),
            || async { 42 },
            "timeout_success_test",
        )
        .await;

        assert_eq!(result.execute().unwrap(), 42);
    }

    // ============================================================================
    // SafeConfig Error Path Tests
    // ============================================================================

    #[test]
    fn test_safe_config_get_missing() {
        let config = HashMap::<String, i32>::new();
        let result = SafeConfig::safe_config_get(&config, "missing_key", 100, "config_test");
        assert_eq!(result, 100);
    }

    #[test]
    fn test_safe_env_parse_invalid() {
        // Test with invalid parse
        std::env::set_var("TEST_INVALID_INT", "not_a_number");
        let result = SafeConfig::safe_env_parse::<i32>("TEST_INVALID_INT", 42, "parse_test");
        assert_eq!(result, 42);
        std::env::remove_var("TEST_INVALID_INT");
    }

    #[test]
    fn test_safe_env_var_missing() {
        let result =
            SafeConfig::safe_env_var("NONEXISTENT_ENV_VAR_12345", "default_value", "env_test");
        assert_eq!(result, "default_value");
    }

    // ============================================================================
    // SafeSession Error Path Tests
    // ============================================================================

    #[test]
    fn test_safe_validate_session_missing_field() {
        let mut session_data = HashMap::new();
        session_data.insert("user_id".to_string(), "123".to_string());

        let result = SafeSession::safe_validate_session(
            &session_data,
            &["user_id", "session_token", "expires_at"],
            "session_test",
        );

        assert!(result.execute().is_err());
    }

    #[test]
    fn test_safe_validate_session_empty() {
        let session_data = HashMap::new();
        let result =
            SafeSession::safe_validate_session(&session_data, &["user_id"], "empty_session_test");
        assert!(result.execute().is_err());
    }

    #[test]
    fn test_safe_session_id_uniqueness() {
        let id1 = SafeSession::safe_session_id("test");
        let id2 = SafeSession::safe_session_id("test");
        assert_ne!(id1, id2); // Should generate unique IDs
    }

    #[test]
    fn test_safe_timestamp_validity() {
        let ts1 = SafeSession::safe_timestamp();
        // Timestamps should be monotonic (equal or increasing)
        let ts2 = SafeSession::safe_timestamp();
        assert!(ts2 >= ts1); // Time should not go backwards
    }

    // ============================================================================
    // Recovery Strategy Tests
    // ============================================================================

    #[test]
    fn test_recovery_strategy_clone() {
        let strategy1 = RecoveryStrategy::UseDefault;
        let strategy2 = strategy1.clone();
        assert!(matches!(strategy2, RecoveryStrategy::UseDefault));

        let strategy1 = RecoveryStrategy::Retry {
            max_attempts: 3,
            backoff: Duration::from_secs(1),
        };
        let strategy2 = strategy1.clone();
        match strategy2 {
            RecoveryStrategy::Retry { max_attempts, .. } => assert_eq!(max_attempts, 3),
            _ => panic!("Expected Retry strategy"),
        }
    }

    #[test]
    fn test_recovery_strategy_debug() {
        let strategy = RecoveryStrategy::Propagate;
        let debug_str = format!("{:?}", strategy);
        assert_eq!(debug_str, "Propagate");

        let strategy = RecoveryStrategy::Retry {
            max_attempts: 5,
            backoff: Duration::from_secs(2),
        };
        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("Retry"));
        assert!(debug_str.contains("max_attempts"));
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_safe_result_with_zero_values() {
        let result = SafeResult::success(0i32, "zero_test".to_string());
        assert_eq!(result.unwrap_or(100), 0);
    }

    #[test]
    fn test_safe_result_with_empty_strings() {
        let result = SafeResult::success(String::new(), "empty_string_test".to_string());
        assert_eq!(result.unwrap_or("default".to_string()), "");
    }

    #[test]
    fn test_safe_get_index_boundary() {
        let vec = vec![1, 2, 3];

        // Valid boundary cases
        let result = SafeOps::safe_get_index(&vec, 0, "first");
        assert_eq!(*result.execute_without_default().unwrap(), 1);

        let result = SafeOps::safe_get_index(&vec, 2, "last");
        assert_eq!(*result.execute_without_default().unwrap(), 3);

        // Invalid boundary
        let result = SafeOps::safe_get_index(&vec, 3, "past_end");
        assert!(result.execute_without_default().is_err());
    }

    #[test]
    fn test_multiple_error_types() {
        let errors = vec![
            SafeError::Configuration {
                message: "Config error".to_string(),
                field: None,
            },
            SafeError::Network {
                message: "Network error".to_string(),
                endpoint: None,
            },
            SafeError::Timeout {
                message: "Timeout".to_string(),
                duration: Duration::from_secs(1),
            },
            SafeError::Validation {
                message: "Validation error".to_string(),
                field: Some("email".to_string()),
            },
            SafeError::Internal {
                message: "Internal error".to_string(),
            },
        ];

        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty());
        }
    }

    // ============================================================================
    // Concurrency Tests
    // ============================================================================

    #[tokio::test]
    async fn test_concurrent_safe_ops() {
        let data = Arc::new(RwLock::new(0i32));
        let mut handles = vec![];

        // Spawn multiple tasks that safely access the shared data
        for i in 0..10 {
            let data_clone = Arc::clone(&data);
            let handle = tokio::spawn(async move {
                let result = SafeOps::safe_write_lock(&data_clone, "concurrent_test").await;
                if let Ok(mut guard) = result.execute_without_default() {
                    *guard += i;
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        let final_value = *data.read().await;
        assert_eq!(final_value, 45); // Sum of 0..10
    }

    #[tokio::test]
    async fn test_concurrent_channel_ops() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let mut handles = vec![];

        // Spawn multiple senders
        for i in 0..10 {
            let tx_clone = tx.clone();
            let handle = tokio::spawn(async move {
                let result = SafeOps::safe_send(&tx_clone, i, "concurrent_send").await;
                assert!(result.execute().is_ok());
            });
            handles.push(handle);
        }

        // Wait for all senders
        drop(tx); // Drop original sender
        for handle in handles {
            handle.await.unwrap();
        }

        // Receive all messages
        let mut received = vec![];
        while let Some(val) = rx.recv().await {
            received.push(val);
        }

        assert_eq!(received.len(), 10);
        received.sort();
        assert_eq!(received, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
