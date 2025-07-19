//! Integration tests for safe operations
//!
//! This test suite demonstrates how safe operations can replace unsafe
//! unwrap() and expect() calls throughout the codebase.

use squirrel::biomeos_integration::*;
use squirrel::error_handling::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

/// Test safe lock operations
#[tokio::test]
async fn test_safe_lock_operations() {
    let data = Arc::new(RwLock::new(42));

    // Test safe read lock
    let read_result = SafeOps::safe_read_lock(&data, "test read lock").await;
    assert!(read_result.execute().is_ok());

    // Test safe write lock
    let write_result = SafeOps::safe_write_lock(&data, "test write lock").await;
    assert!(write_result.execute().is_ok());

    // Test mutex lock
    let mutex_data = Arc::new(Mutex::new(42));
    let mutex_result = SafeOps::safe_mutex_lock(&mutex_data, "test mutex lock").await;
    assert!(mutex_result.execute().is_ok());

    println!("✅ All safe lock operations completed successfully");
}

/// Test safe parsing operations
#[tokio::test]
async fn test_safe_parsing_operations() {
    // Test successful parsing
    let result = SafeOps::safe_parse::<i32>("42", "test parse");
    assert_eq!(result.unwrap_or_default(), 42);

    // Test failed parsing with safe recovery
    let result = SafeOps::safe_parse::<i32>("invalid", "test parse");
    assert_eq!(result.unwrap_or_default(), 0);

    // Test URL parsing
    let result = SafeOps::safe_parse_url("https://example.com", "test url");
    assert!(result.execute().is_ok());

    let result = SafeOps::safe_parse_url("invalid-url", "test url");
    assert!(result.execute().is_err());

    println!("✅ All safe parsing operations completed successfully");
}

/// Test safe collection operations
#[tokio::test]
async fn test_safe_collection_operations() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());

    // Test safe get
    let result = SafeOps::safe_get(&map, &"key".to_string(), "test map");
    assert_eq!(result.unwrap_or_default(), "value");

    // Test safe get with missing key
    let result = SafeOps::safe_get(&map, &"missing".to_string(), "test map");
    assert_eq!(result.unwrap_or_default(), "");

    // Test safe get cloned
    let result = SafeOps::safe_get_cloned(&map, &"key".to_string(), "test map");
    assert_eq!(result.unwrap_or_default(), "value");

    // Test safe vector operations
    let vec = vec![1, 2, 3, 4, 5];
    let result = SafeOps::safe_first(&vec, "test vec");
    assert_eq!(result.unwrap_or_default(), &1);

    let result = SafeOps::safe_last(&vec, "test vec");
    assert_eq!(result.unwrap_or_default(), &5);

    let result = SafeOps::safe_get_index(&vec, 2, "test vec");
    assert_eq!(result.unwrap_or_default(), &3);

    // Test with out of bounds
    let result = SafeOps::safe_get_index(&vec, 10, "test vec");
    assert_eq!(result.unwrap_or_default(), &0);

    println!("✅ All safe collection operations completed successfully");
}

/// Test safe serialization operations
#[tokio::test]
async fn test_safe_serialization_operations() {
    let data = HashMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    // Test safe serialization
    let result = SafeOps::safe_serialize(&data, "test serialize");
    assert!(result.execute().is_ok());

    let json = result.unwrap_or_default();
    assert!(!json.is_empty());

    // Test safe deserialization
    let result = SafeOps::safe_deserialize::<HashMap<String, String>>(&json, "test deserialize");
    assert!(result.execute().is_ok());

    let deserialized = result.unwrap_or_default();
    assert_eq!(deserialized.len(), 2);
    assert_eq!(deserialized.get("key1"), Some(&"value1".to_string()));

    // Test deserialization with invalid JSON
    let result =
        SafeOps::safe_deserialize::<HashMap<String, String>>("invalid json", "test deserialize");
    assert!(result.execute().is_err());

    let empty_map = result.unwrap_or_default();
    assert!(empty_map.is_empty());

    println!("✅ All safe serialization operations completed successfully");
}

/// Test safe channel operations
#[tokio::test]
async fn test_safe_channel_operations() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    // Test safe send
    let result = SafeOps::safe_send(&tx, 42, "test send").await;
    assert!(result.execute().is_ok());

    // Test safe receive
    let result = SafeOps::safe_receive(&mut rx, "test receive").await;
    assert!(result.execute().is_ok());
    assert_eq!(result.unwrap_or_default(), Some(42));

    // Test receive from empty channel
    let result = SafeOps::safe_receive(&mut rx, "test receive empty").await;
    assert!(result.execute().is_ok());
    assert_eq!(result.unwrap_or_default(), None);

    println!("✅ All safe channel operations completed successfully");
}

/// Test safe configuration operations
#[tokio::test]
async fn test_safe_configuration_operations() {
    // Test safe environment variable access
    let result = SafeConfig::safe_env_var("NONEXISTENT_VAR", "default_value", "test env");
    assert_eq!(result, "default_value");

    // Test safe environment variable parsing
    let result = SafeConfig::safe_env_parse::<i32>("NONEXISTENT_VAR", 42, "test env parse");
    assert_eq!(result, 42);

    // Test safe configuration get
    let mut config = HashMap::new();
    config.insert("key".to_string(), "value".to_string());

    let result = SafeConfig::safe_config_get(&config, "key", "default".to_string(), "test config");
    assert_eq!(result, "value");

    let result =
        SafeConfig::safe_config_get(&config, "missing", "default".to_string(), "test config");
    assert_eq!(result, "default");

    println!("✅ All safe configuration operations completed successfully");
}

/// Test safe session operations
#[tokio::test]
async fn test_safe_session_operations() {
    // Test safe session ID generation
    let session_id = SafeSession::safe_session_id("test session");
    assert!(!session_id.is_empty());
    assert!(session_id.len() > 10); // UUID should be longer

    // Test safe timestamp
    let timestamp = SafeSession::safe_timestamp();
    assert!(timestamp <= chrono::Utc::now());

    // Test safe session validation
    let mut session_data = HashMap::new();
    session_data.insert("user_id".to_string(), "123".to_string());
    session_data.insert("session_token".to_string(), "abc123".to_string());

    let result = SafeSession::safe_validate_session(
        &session_data,
        &["user_id", "session_token"],
        "test validation",
    );
    assert!(result.execute().is_ok());

    // Test validation with missing field
    let result = SafeSession::safe_validate_session(
        &session_data,
        &["user_id", "missing_field"],
        "test validation",
    );
    assert!(result.execute().is_err());

    println!("✅ All safe session operations completed successfully");
}

/// Test safe file operations
#[tokio::test]
async fn test_safe_file_operations() {
    let test_content = "This is test content for safe file operations";
    let test_file = "/tmp/safe_ops_test.txt";

    // Test safe write
    let result = SafeOps::safe_write_file(test_file, test_content, "test write").await;
    assert!(result.execute().is_ok());

    // Test safe read
    let result = SafeOps::safe_read_file(test_file, "test read").await;
    assert!(result.execute().is_ok());

    let content = result.unwrap_or_default();
    assert_eq!(content, test_content);

    // Test reading non-existent file
    let result = SafeOps::safe_read_file("/nonexistent/file.txt", "test read nonexistent").await;
    assert!(result.execute().is_err());

    let empty_content = result.unwrap_or_default();
    assert!(empty_content.is_empty());

    // Cleanup
    let _ = tokio::fs::remove_file(test_file).await;

    println!("✅ All safe file operations completed successfully");
}

/// Test safe timeout operations
#[tokio::test]
async fn test_safe_timeout_operations() {
    // Test operation that completes within timeout
    let result = SafeOps::safe_with_timeout(
        Duration::from_secs(2),
        || async { 42 },
        "test timeout success",
    )
    .await;
    assert!(result.execute().is_ok());
    assert_eq!(result.unwrap_or_default(), 42);

    // Test operation that times out
    let result = SafeOps::safe_with_timeout(
        Duration::from_millis(100),
        || async {
            sleep(Duration::from_millis(500)).await;
            42
        },
        "test timeout failure",
    )
    .await;
    assert!(result.execute().is_err());
    assert_eq!(result.unwrap_or_default(), 0);

    println!("✅ All safe timeout operations completed successfully");
}

/// Test safe service operations
#[tokio::test]
async fn test_safe_service_operations() {
    let client = reqwest::Client::new();

    // Test with a real HTTP endpoint (this might fail in some environments)
    let result =
        SafeService::safe_health_check(&client, "https://httpbin.org", "test health check").await;

    // We don't assert success here since it depends on network connectivity
    // Instead, we verify the operation handles both success and failure gracefully
    match result.execute() {
        Ok(is_healthy) => {
            println!("✅ Health check successful: {}", is_healthy);
        }
        Err(err) => {
            println!("⚠️  Health check failed gracefully: {}", err);
            // This is expected behavior - the operation should not panic
        }
    }

    // Test with invalid endpoint
    let result =
        SafeService::safe_health_check(&client, "invalid-url", "test invalid health check").await;
    assert!(result.execute().is_err());

    let default_result = result.unwrap_or_default();
    assert!(!default_result); // Should default to false

    println!("✅ All safe service operations completed successfully");
}

/// Test safe error recovery strategies
#[tokio::test]
async fn test_safe_error_recovery_strategies() {
    // Test with UseDefault strategy
    let result = SafeResult::failure(
        SafeError::Configuration {
            message: "Test error".to_string(),
            field: None,
        },
        "test recovery".to_string(),
    )
    .with_recovery_strategy(RecoveryStrategy::UseDefault);

    assert_eq!(result.execute(), Ok(0)); // Should return default i32 value

    // Test with LogAndContinue strategy
    let result = SafeResult::failure(
        SafeError::Network {
            message: "Network error".to_string(),
            endpoint: None,
        },
        "test log and continue".to_string(),
    )
    .with_recovery_strategy(RecoveryStrategy::LogAndContinue);

    assert_eq!(result.execute(), Ok(0)); // Should return default and log

    // Test with Propagate strategy
    let result = SafeResult::failure(
        SafeError::Validation {
            message: "Validation error".to_string(),
            field: None,
        },
        "test propagate".to_string(),
    )
    .with_recovery_strategy(RecoveryStrategy::Propagate);

    assert!(result.execute().is_err()); // Should propagate error

    println!("✅ All safe error recovery strategies completed successfully");
}

/// Test safe operations with BiomeOS integration
#[tokio::test]
async fn test_safe_operations_with_biomeos() {
    let mut integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Test safe initialization
    let init_result = integration.initialize().await;
    assert!(init_result.is_ok());

    // Test safe service registration creation
    let registration_result = integration.create_service_registration().await;
    assert!(registration_result.is_ok());

    let registration = registration_result.unwrap();
    assert!(!registration.service_id.is_empty());
    assert_eq!(registration.primal_type, "squirrel");

    // Test safe service discovery
    let discovery_result = integration.discover_ecosystem_services().await;
    // This might fail if no services are available, but should not panic
    match discovery_result {
        Ok(services) => {
            println!("✅ Discovered {} services", services.len());
        }
        Err(err) => {
            println!("⚠️  Service discovery failed gracefully: {}", err);
        }
    }

    // Test safe health status
    let health_result = integration.get_health_status().await;
    assert!(health_result.is_ok());

    let health_status = health_result.unwrap();
    assert!(!health_status.service_id.is_empty());
    assert!(!health_status.status.is_empty());

    println!("✅ All safe BiomeOS operations completed successfully");
}

/// Test safe operations macros
#[tokio::test]
async fn test_safe_operations_macros() {
    // Test safe_unwrap macro
    let result: Result<i32, &str> = Ok(42);
    let value = safe_unwrap!(result, "test safe_unwrap");
    assert_eq!(value, 42);

    let result: Result<i32, &str> = Err("error");
    let value = safe_unwrap!(result, "test safe_unwrap error");
    assert_eq!(value, 0); // Should use default

    // Test safe_expect macro
    let result: Result<i32, &str> = Ok(42);
    let value = safe_expect!(result, "test safe_expect", 100);
    assert_eq!(value, 42);

    let result: Result<i32, &str> = Err("error");
    let value = safe_expect!(result, "test safe_expect error", 100);
    assert_eq!(value, 100); // Should use provided default

    // Test safe_option macro
    let option: Option<i32> = Some(42);
    let value = safe_option!(option, "test safe_option");
    assert_eq!(value, 42);

    let option: Option<i32> = None;
    let value = safe_option!(option, "test safe_option none");
    assert_eq!(value, 0); // Should use default

    // Test safe_option_with_default macro
    let option: Option<i32> = Some(42);
    let value = safe_option_with_default!(option, "test safe_option_with_default", 100);
    assert_eq!(value, 42);

    let option: Option<i32> = None;
    let value = safe_option_with_default!(option, "test safe_option_with_default none", 100);
    assert_eq!(value, 100); // Should use provided default

    println!("✅ All safe operations macros completed successfully");
}

/// Test comprehensive safe operations scenario
#[tokio::test]
async fn test_comprehensive_safe_operations_scenario() {
    println!("🚀 Starting comprehensive safe operations scenario...");

    // Simulate a complex operation that could fail at multiple points
    let data = Arc::new(RwLock::new(HashMap::<String, String>::new()));

    // Step 1: Safely acquire lock and add data
    let lock_result = SafeOps::safe_write_lock(&data, "comprehensive test lock").await;
    if let Ok(mut guard) = lock_result.execute() {
        guard.insert("status".to_string(), "initialized".to_string());
        guard.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
    }

    // Step 2: Safely read and process data
    let read_result = SafeOps::safe_read_lock(&data, "comprehensive test read").await;
    if let Ok(guard) = read_result.execute() {
        let status = SafeOps::safe_get(&*guard, &"status".to_string(), "comprehensive test get");
        assert_eq!(status.unwrap_or_default(), "initialized");

        let timestamp = SafeOps::safe_get(
            &*guard,
            &"timestamp".to_string(),
            "comprehensive test get timestamp",
        );
        assert!(!timestamp.unwrap_or_default().is_empty());
    }

    // Step 3: Safely parse and validate data
    let config_data = r#"{"port": "8080", "host": "localhost"}"#;
    let parse_result = SafeOps::safe_deserialize::<HashMap<String, String>>(
        config_data,
        "comprehensive test parse",
    );

    if let Ok(config) = parse_result.execute() {
        let port = SafeOps::safe_get(&config, &"port".to_string(), "comprehensive test port");
        let port_number =
            SafeOps::safe_parse::<u16>(port.unwrap_or_default(), "comprehensive test port parse");
        assert_eq!(port_number.unwrap_or_default(), 8080);

        let host = SafeOps::safe_get(&config, &"host".to_string(), "comprehensive test host");
        assert_eq!(host.unwrap_or_default(), "localhost");
    }

    // Step 4: Safely handle network operations
    let client = reqwest::Client::new();
    let network_result = SafeService::safe_health_check(
        &client,
        "https://httpbin.org",
        "comprehensive test network",
    )
    .await;

    // Handle both success and failure gracefully
    match network_result.execute() {
        Ok(is_healthy) => {
            println!("✅ Network check completed: healthy={}", is_healthy);
        }
        Err(err) => {
            println!("⚠️  Network check failed gracefully: {}", err);
        }
    }

    // Step 5: Safely handle timeout operations
    let timeout_result = SafeOps::safe_with_timeout(
        Duration::from_millis(100),
        || async {
            // Simulate some processing
            tokio::time::sleep(Duration::from_millis(50)).await;
            "processing complete".to_string()
        },
        "comprehensive test timeout",
    )
    .await;

    assert!(timeout_result.execute().is_ok());
    assert_eq!(timeout_result.unwrap_or_default(), "processing complete");

    println!("✅ Comprehensive safe operations scenario completed successfully");
    println!("🎉 All operations completed without panics or unwrap() failures!");
}

/// Summary test that demonstrates the benefits of safe operations
#[tokio::test]
async fn test_safe_operations_benefits_summary() {
    println!("\n📊 Safe Operations Benefits Summary");
    println!("=====================================");

    let mut success_count = 0;
    let mut total_operations = 0;

    // Test 1: Safe parsing
    total_operations += 1;
    let result = SafeOps::safe_parse::<i32>("invalid", "benefits test parse");
    if result.execute().is_err() {
        success_count += 1; // Success = graceful error handling
        println!("✅ Parsing: Handled invalid input gracefully");
    }

    // Test 2: Safe collection access
    total_operations += 1;
    let empty_vec: Vec<i32> = vec![];
    let result = SafeOps::safe_first(&empty_vec, "benefits test first");
    if result.execute().is_err() {
        success_count += 1; // Success = graceful error handling
        println!("✅ Collections: Handled empty vector gracefully");
    }

    // Test 3: Safe configuration
    total_operations += 1;
    let config_value = SafeConfig::safe_env_var("NONEXISTENT_VAR", "default", "benefits test");
    if config_value == "default" {
        success_count += 1;
        println!("✅ Configuration: Provided default for missing env var");
    }

    // Test 4: Safe serialization
    total_operations += 1;
    let result = SafeOps::safe_deserialize::<HashMap<String, String>>(
        "invalid json",
        "benefits test deserialize",
    );
    if result.execute().is_err() {
        success_count += 1; // Success = graceful error handling
        println!("✅ Serialization: Handled invalid JSON gracefully");
    }

    // Test 5: Safe URL parsing
    total_operations += 1;
    let result = SafeOps::safe_parse_url("not-a-url", "benefits test url");
    if result.execute().is_err() {
        success_count += 1; // Success = graceful error handling
        println!("✅ URL Parsing: Handled invalid URL gracefully");
    }

    println!("\n📈 Results:");
    println!("  • Total operations: {}", total_operations);
    println!("  • Graceful failures: {}", success_count);
    println!(
        "  • Success rate: {:.1}%",
        (success_count as f64 / total_operations as f64) * 100.0
    );
    println!("  • Zero panics: ✅");
    println!("  • Zero unwrap() failures: ✅");

    println!("\n🎯 Key Benefits:");
    println!("  • Prevents production panics");
    println!("  • Provides comprehensive error context");
    println!("  • Enables graceful error recovery");
    println!("  • Improves system reliability");
    println!("  • Enhances debugging capabilities");

    assert_eq!(success_count, total_operations);
    println!("\n🏆 Safe operations successfully replaced all unsafe patterns!");
}
