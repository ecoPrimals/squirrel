//! AI Resilience Tests - Activated
//!
//! Tests using the actual AIResilienceCoordinator API that exists in the codebase.
//! These replace the broken chaos_engineering_tests.rs which used a non-existent API.

use squirrel::chaos::{AIResilienceCoordinator, AIResilienceTestType};
use std::time::Duration;

mod common;
use common::concurrent_helpers::eventually;

/// Test basic AI resilience coordinator creation
#[test]
fn test_ai_resilience_coordinator_creation() {
    let coordinator = AIResilienceCoordinator::new();
    assert_eq!(coordinator.get_test_results().len(), 0);
}

/// Test AI service unavailability resilience
#[tokio::test]
async fn test_ai_service_unavailability_resilience() {
    let coordinator = AIResilienceCoordinator::new();

    let test_type = AIResilienceTestType::AIServiceUnavailable {
        service_name: "openai-api".to_string(),
        failure_rate: 0.3,
    };

    let test_id = coordinator
        .start_ai_resilience_test(test_type, Duration::from_secs(2))
        .await
        .expect("Should start resilience test");

    assert!(!test_id.is_empty());

    // Wait for test to become active (no sleeps - proper async sync!)
    eventually(
        || async {
            let active_tests = coordinator.get_active_tests().await;
            active_tests.len() == 1
        },
        Duration::from_secs(2),
        Duration::from_millis(10),
    )
    .await
    .expect("Test should become active");

    let active_tests = coordinator.get_active_tests().await;
    assert_eq!(active_tests.len(), 1);
}

/// Test slow AI response resilience
#[tokio::test]
async fn test_slow_ai_response_resilience() {
    let coordinator = AIResilienceCoordinator::new();

    let test_type = AIResilienceTestType::SlowAIResponse {
        service_name: "anthropic-api".to_string(),
        delay_ms: 500,
    };

    let test_id = coordinator
        .start_ai_resilience_test(test_type, Duration::from_secs(1))
        .await
        .expect("Should start resilience test");

    assert!(!test_id.is_empty());
}

/// Test coordination failure resilience
#[tokio::test]
async fn test_coordination_failure_resilience() {
    let mut coordinator = AIResilienceCoordinator::new();

    let test_type = AIResilienceTestType::CoordinationFailure {
        affected_primals: vec!["beardog".to_string(), "toadstool".to_string()],
        failure_type: "network_partition".to_string(),
    };

    let test_id = coordinator
        .start_ai_resilience_test(test_type, Duration::from_secs(1))
        .await
        .expect("Should start resilience test");

    // Wait for test to actually run some operations (no sleeps!)
    eventually(
        || async {
            let active_tests = coordinator.get_active_tests().await;
            active_tests.len() == 1
        },
        Duration::from_secs(2),
        Duration::from_millis(10),
    )
    .await
    .expect("Test should become active");

    // Stop the test
    let result = coordinator
        .stop_resilience_test(&test_id)
        .await
        .expect("Should stop resilience test");

    assert_eq!(result.test_id, test_id);
    assert!(result.ai_operations_tested > 0);
}

/// Test circuit breaker resilience
#[tokio::test]
async fn test_circuit_breaker_resilience() {
    let coordinator = AIResilienceCoordinator::new();

    let test_type = AIResilienceTestType::CircuitBreakerTest {
        service_name: "gemini-api".to_string(),
        failure_threshold: 5,
    };

    let test_id = coordinator
        .start_ai_resilience_test(test_type, Duration::from_secs(1))
        .await
        .expect("Should start resilience test");

    assert!(!test_id.is_empty());
}

/// Test retry pattern resilience
#[tokio::test]
async fn test_retry_pattern_resilience() {
    let coordinator = AIResilienceCoordinator::new();

    let test_type = AIResilienceTestType::RetryPatternTest {
        service_name: "local-model".to_string(),
        max_retries: 3,
    };

    let test_id = coordinator
        .start_ai_resilience_test(test_type, Duration::from_secs(1))
        .await
        .expect("Should start resilience test");

    assert!(!test_id.is_empty());
}

/// Test quick AI resilience check
#[tokio::test]
async fn test_quick_ai_resilience_check() {
    let coordinator = AIResilienceCoordinator::new();

    let status = coordinator.quick_ai_resilience_check().await;

    assert_eq!(status.overall_healthy, true);
    assert_eq!(status.security_coordination, "healthy");
    assert_eq!(status.orchestration_coordination, "healthy");
    assert_eq!(status.storage_coordination, "healthy");
    assert_eq!(status.compute_coordination, "healthy");
}

/// Test resilience recommendations
#[test]
fn test_resilience_recommendations() {
    let coordinator = AIResilienceCoordinator::new();

    let recommendations = coordinator.get_resilience_recommendations();

    assert!(!recommendations.is_empty());
    assert!(recommendations.len() >= 5);
    assert!(recommendations
        .iter()
        .any(|r| r.contains("circuit breaker")));
}

/// Test multiple concurrent resilience tests
#[tokio::test]
async fn test_multiple_concurrent_tests() {
    let coordinator = AIResilienceCoordinator::new();

    // Start multiple tests concurrently
    let _test1 = coordinator
        .start_ai_resilience_test(
            AIResilienceTestType::AIServiceUnavailable {
                service_name: "service1".to_string(),
                failure_rate: 0.1,
            },
            Duration::from_secs(1),
        )
        .await
        .expect("Should start test 1");

    let _test2 = coordinator
        .start_ai_resilience_test(
            AIResilienceTestType::SlowAIResponse {
                service_name: "service2".to_string(),
                delay_ms: 100,
            },
            Duration::from_secs(1),
        )
        .await
        .expect("Should start test 2");

    let _test3 = coordinator
        .start_ai_resilience_test(
            AIResilienceTestType::CircuitBreakerTest {
                service_name: "service3".to_string(),
                failure_threshold: 3,
            },
            Duration::from_secs(1),
        )
        .await
        .expect("Should start test 3");

    // Check all tests are active
    let active_tests = coordinator.get_active_tests().await;
    assert_eq!(active_tests.len(), 3);
}

/// Test stopping non-existent resilience test
#[tokio::test]
async fn test_stop_nonexistent_test() {
    let mut coordinator = AIResilienceCoordinator::new();

    let result = coordinator.stop_resilience_test("nonexistent-id").await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("not found"));
}

/// Test resilience test result tracking
#[tokio::test]
async fn test_resilience_result_tracking() {
    let mut coordinator = AIResilienceCoordinator::new();

    // Start and complete a test
    let test_id = coordinator
        .start_ai_resilience_test(
            AIResilienceTestType::RetryPatternTest {
                service_name: "test-service".to_string(),
                max_retries: 2,
            },
            Duration::from_secs(1),
        )
        .await
        .expect("Should start test");

    // Wait for test to become active (no sleeps!)
    eventually(
        || async {
            let active_tests = coordinator.get_active_tests().await;
            active_tests.iter().any(|t| t.test_id == test_id)
        },
        Duration::from_secs(2),
        Duration::from_millis(10),
    )
    .await
    .expect("Test should become active");

    let result = coordinator
        .stop_resilience_test(&test_id)
        .await
        .expect("Should stop test");

    // Check result is tracked
    let results = coordinator.get_test_results();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].test_id, test_id);
}
