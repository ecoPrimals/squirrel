//! Tests for the circuit breaker implementation in the AI Agent
//!
//! These tests use a mock adapter to verify the circuit breaker pattern
//! works correctly under various success and failure scenarios.

use std::time::Duration;
use tokio::time::sleep;
use futures::future::join_all;

use squirrel_integration::ai_agent::{
    AIAgentConfig, 
    CircuitBreakerConfig,
    CircuitBreakerState, 
    AgentRequest,
    AIAgentError,
    ResourceLimits,
    GenerationOptions,
    OperationType,
};

// Import our mock adapter
mod mock_adapter;
use mock_adapter::MockAIAgent;

/// Create a test configuration with a short reset timeout for efficient testing
fn create_test_config() -> AIAgentConfig {
    AIAgentConfig {
        provider: "test-provider".to_string(),
        api_key: "test-api-key".to_string(),
        circuit_breaker: CircuitBreakerConfig {
            failure_threshold: 3,
            reset_timeout: 500,  // 500ms timeout for faster testing
            half_open_max_calls: 2,
        },
        resource_limits: ResourceLimits {
            max_tokens: 1000,
            max_requests_per_minute: 100,
            max_tokens_per_minute: 10000,
            max_concurrent_requests: 20,
            max_request_time_ms: 5000,
        },
        cache_size: Some(100),
        timeout_ms: 1000,
        max_retries: 1,
    }
}

/// Create a basic test request
fn create_test_request(prompt: &str) -> AgentRequest {
    AgentRequest {
        prompt: prompt.to_string(),
        system_message: Some("You are a helpful assistant".to_string()),
        options: GenerationOptions {
            max_tokens: Some(100),
            temperature: 0.7,
            top_p: 1.0,
            ..Default::default()
        },
        operation_type: OperationType::Generate,
        ..Default::default()
    }
}

/// Test the adapter under normal conditions (all successful requests)
#[tokio::test]
async fn test_circuit_breaker_normal_operation() {
    let config = create_test_config();
    let adapter = MockAIAgent::new(config);
    
    // Initialize the adapter
    adapter.initialize().await.expect("Failed to initialize adapter");
    
    // Ensure adapter is not in failure mode
    adapter.set_failure_mode(false);
    
    // Process multiple successful requests
    for i in 0..10 {
        let request = create_test_request(&format!("Test request {}", i));
        let result = adapter.process_request(request).await;
        assert!(result.is_ok(), "Request {} should succeed", i);
    }
    
    // Verify adapter status
    let status = adapter.get_status().await;
    assert!(status.operational, "Adapter should be operational");
    assert_eq!(status.circuit_breaker_state, CircuitBreakerState::Closed, 
               "Circuit breaker should remain closed");
    assert_eq!(adapter.get_success_count(), 10, "Should have 10 successful calls");
    assert_eq!(adapter.get_failure_count(), 0, "Should have 0 failed calls");
}

/// Test circuit breaker opening after threshold failures
#[tokio::test]
async fn test_circuit_breaker_opens_on_failures() {
    let config = create_test_config();
    let adapter = MockAIAgent::new(config);
    
    // Initialize the adapter
    adapter.initialize().await.expect("Failed to initialize adapter");
    
    // First, make some successful requests
    adapter.set_failure_mode(false);
    for i in 0..2 {
        let request = create_test_request(&format!("Successful request {}", i));
        let result = adapter.process_request(request).await;
        assert!(result.is_ok(), "Request {} should succeed", i);
    }
    
    // Then make failing requests to exceed the threshold (3)
    adapter.set_failure_mode(true);
    for i in 0..4 {
        let request = create_test_request(&format!("Failing request {}", i));
        let result = adapter.process_request(request).await;
        
        if i < 3 {
            // First 3 should be normal service errors
            assert!(matches!(result, Err(AIAgentError::ServiceError(_))), 
                   "Request {} should return ServiceError", i);
        } else {
            // After threshold, should immediately return CircuitBreakerOpen
            assert!(matches!(result, Err(AIAgentError::CircuitBreakerOpen(_))), 
                   "Request {} should return CircuitBreakerOpen", i);
        }
    }
    
    // Verify circuit breaker has opened
    let status = adapter.get_status().await;
    assert!(!status.operational, "Adapter should not be operational");
    assert_eq!(status.circuit_breaker_state, CircuitBreakerState::Open, 
               "Circuit breaker should be open");
    
    // Any further requests should immediately return circuit breaker error
    let request = create_test_request("This should fail fast");
    let result = adapter.process_request(request).await;
    assert!(matches!(result, Err(AIAgentError::CircuitBreakerOpen(_))), 
           "Request should immediately fail with CircuitBreakerOpen");
}

/// Test circuit breaker transitions through half-open state to closed
#[tokio::test]
async fn test_circuit_breaker_transitions() {
    let config = create_test_config();
    let adapter = MockAIAgent::new(config);
    
    // Initialize the adapter
    adapter.initialize().await.expect("Failed to initialize adapter");
    
    // Cause the circuit to open
    adapter.set_failure_mode(true);
    for i in 0..3 {
        let request = create_test_request(&format!("Failing request {}", i));
        let _ = adapter.process_request(request).await;
    }
    
    let status = adapter.get_status().await;
    assert_eq!(status.circuit_breaker_state, CircuitBreakerState::Open, 
               "Circuit breaker should be open");
    
    // Wait for the reset timeout to transition to half-open
    sleep(Duration::from_millis(600)).await;  // Reset timeout is 500ms
    
    // Manually set to half-open (in a real implementation, this would happen automatically)
    adapter.set_circuit_breaker_state(CircuitBreakerState::HalfOpen);
    
    // Now switch to success mode to allow recovery
    adapter.set_failure_mode(false);
    
    // The first request in half-open state should succeed
    let request1 = create_test_request("Test request in half-open state 1");
    let result1 = adapter.process_request(request1).await;
    assert!(result1.is_ok(), "First request in half-open state should succeed");
    
    // The second request should succeed and close the circuit
    let request2 = create_test_request("Test request in half-open state 2");
    let result2 = adapter.process_request(request2).await;
    assert!(result2.is_ok(), "Second request in half-open state should succeed");
    
    // Verify the circuit is now closed
    let status = adapter.get_status().await;
    assert!(status.operational, "Adapter should be operational again");
    assert_eq!(status.circuit_breaker_state, CircuitBreakerState::Closed, 
               "Circuit breaker should be closed after successful half-open requests");
    
    // Further requests should succeed normally
    let request3 = create_test_request("Test request after circuit closed");
    let result3 = adapter.process_request(request3).await;
    assert!(result3.is_ok(), "Request after circuit closed should succeed");
}

/// Test failure during half-open state
#[tokio::test]
async fn test_circuit_breaker_half_open_failure() {
    let config = create_test_config();
    let adapter = MockAIAgent::new(config);
    
    // Initialize the adapter
    adapter.initialize().await.expect("Failed to initialize adapter");
    
    // Open the circuit
    adapter.set_failure_mode(true);
    for i in 0..3 {
        let request = create_test_request(&format!("Failing request {}", i));
        let _ = adapter.process_request(request).await;
    }
    
    // Verify circuit is open
    let status = adapter.get_status().await;
    assert_eq!(status.circuit_breaker_state, CircuitBreakerState::Open, 
               "Circuit breaker should be open");
    
    // Transition to half-open
    sleep(Duration::from_millis(600)).await;
    adapter.set_circuit_breaker_state(CircuitBreakerState::HalfOpen);
    
    // Keep in failure mode to cause the half-open test to fail
    adapter.set_failure_mode(true);
    
    // The request in half-open state should fail and reopen the circuit
    let request = create_test_request("Test request in half-open state");
    let result = adapter.process_request(request).await;
    assert!(matches!(result, Err(AIAgentError::ServiceError(_))), 
           "Request in half-open state should fail with ServiceError");
    
    // Verify the circuit is open again
    let status = adapter.get_status().await;
    assert_eq!(status.circuit_breaker_state, CircuitBreakerState::Open, 
               "Circuit breaker should reopen after failure in half-open state");
}

/// Test concurrent requests with circuit breaker
#[tokio::test]
async fn test_circuit_breaker_concurrency() {
    let config = create_test_config();
    let adapter = MockAIAgent::new(config);
    
    // Initialize the adapter
    adapter.initialize().await.expect("Failed to initialize adapter");
    
    // Set a longer delay to ensure true concurrency
    adapter.set_delay(50);
    adapter.set_failure_mode(false);
    
    // Create 20 concurrent requests
    let futures = (0..20).map(|i| {
        let adapter_clone = adapter.clone();
        let request = create_test_request(&format!("Concurrent request {}", i));
        
        async move {
            adapter_clone.process_request(request).await
        }
    });
    
    // Execute all requests concurrently
    let results = join_all(futures).await;
    
    // All requests should succeed
    assert_eq!(results.len(), 20, "Should process all 20 requests");
    assert!(results.iter().all(|r| r.is_ok()), "All concurrent requests should succeed");
    assert_eq!(adapter.get_success_count(), 20, "Should have 20 successful calls");
    
    // Reset the adapter for the next test
    adapter.set_circuit_breaker_state(CircuitBreakerState::Closed);
    
    // Now test with some failures
    adapter.set_failure_mode(true);
    
    // Create a mix of requests that will cause circuit to open during processing
    let futures = (0..10).map(|i| {
        let adapter_clone = adapter.clone();
        let request = create_test_request(&format!("Concurrent failing request {}", i));
        
        async move {
            adapter_clone.process_request(request).await
        }
    });
    
    // Execute all requests concurrently
    let results = join_all(futures).await;
    
    // Some requests should fail with ServiceError, some with CircuitBreakerOpen
    let service_errors = results.iter()
        .filter(|r| matches!(r, Err(AIAgentError::ServiceError(_))))
        .count();
    let circuit_open_errors = results.iter()
        .filter(|r| matches!(r, Err(AIAgentError::CircuitBreakerOpen(_))))
        .count();
    
    // Verify error counts (at least 3 service errors to open circuit, rest should be circuit open)
    assert!(service_errors >= 3, "Should have at least 3 service errors to open circuit");
    assert!(circuit_open_errors > 0, "Should have some circuit open errors");
    assert_eq!(service_errors + circuit_open_errors, 10, "All requests should fail");
    
    // Verify circuit is open
    let status = adapter.get_status().await;
    assert_eq!(status.circuit_breaker_state, CircuitBreakerState::Open, 
               "Circuit breaker should be open after concurrent failures");
}

/// Test that the circuit breaker opens after reaching the failure threshold
#[tokio::test]
async fn test_circuit_breaker_opens_after_failures() {
    // Configure a circuit breaker with low thresholds for testing
    let config = AIAgentConfig {
        circuit_breaker: CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 3,
            reset_timeout: Duration::from_secs(30),
            half_open_max_calls: 2,
        },
        // Other configurations can be kept at default or minimal values
        ..Default::default()
    };
    
    // Create our mock adapter
    let mock = MockAIAgent::new(config);
    
    // Initialize the adapter
    mock.initialize().await.expect("Failed to initialize mock adapter");
    
    // Verify the circuit is initially closed
    let initial_status = mock.get_status().await;
    assert_eq!(initial_status.circuit_breaker_state, CircuitBreakerState::Closed);
    
    // Set failure mode and make enough requests to trigger the circuit breaker
    mock.set_failure_mode(true);
    
    // Make 3 requests (matching our failure threshold)
    for i in 0..3 {
        let result = mock.process_request(&format!("test prompt {}", i)).await;
        assert!(result.is_err());
    }
    
    // Verify that circuit breaker opened
    let status_after_failures = mock.get_status().await;
    assert_eq!(status_after_failures.circuit_breaker_state, CircuitBreakerState::Open);
    assert_eq!(mock.get_failure_count(), 3);
}

/// Test that the circuit breaker halts processing when open
#[tokio::test]
async fn test_circuit_breaker_rejects_requests_when_open() {
    // Create a mock with default config
    let config = AIAgentConfig::default();
    let mock = MockAIAgent::new(config);
    
    // Initialize the adapter
    mock.initialize().await.expect("Failed to initialize mock adapter");
    
    // Force the circuit into open state
    mock.set_circuit_breaker_state(CircuitBreakerState::Open);
    
    // Verify requests are rejected with circuit breaker error
    let result = mock.process_request("test prompt").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circuit breaker is open"));
    
    // Verify the error was not counted as a failure (since it's a fast fail)
    assert_eq!(mock.get_failure_count(), 0);
}

/// Test circuit breaker reset and half-open state
#[tokio::test]
async fn test_circuit_breaker_transition_to_closed() {
    // Configure a circuit breaker with test settings
    let config = AIAgentConfig {
        circuit_breaker: CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 3,
            reset_timeout: Duration::from_secs(30),
            half_open_max_calls: 2,
        },
        ..Default::default()
    };
    
    // Create our mock adapter
    let mock = MockAIAgent::new(config);
    
    // Initialize the adapter
    mock.initialize().await.expect("Failed to initialize mock adapter");
    
    // Set the circuit breaker to half-open state
    mock.set_circuit_breaker_state(CircuitBreakerState::HalfOpen);
    
    // Set the adapter to succeed
    mock.set_failure_mode(false);
    
    // Process 2 successful requests (matching half_open_max_calls)
    for i in 0..2 {
        let result = mock.process_request(&format!("test prompt {}", i)).await;
        assert!(result.is_ok());
    }
    
    // Verify that the circuit breaker closed
    let status = mock.get_status().await;
    assert_eq!(status.circuit_breaker_state, CircuitBreakerState::Closed);
    assert_eq!(mock.get_success_count(), 2);
} 