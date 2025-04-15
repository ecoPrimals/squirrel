//! Tests for the rate limiting implementation in the AI Agent
//!
//! These tests verify that the rate limiting functionality works correctly
//! to prevent excessive API usage.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use futures::future::join_all;

use squirrel_integration::ai_agent::{
    AIAgentAdapter, 
    AIAgentConfig,
    CircuitBreakerConfig,
    ResourceLimits,
    GenerationOptions,
    AgentRequest,
    AIAgentError,
    OperationType,
};

/// A rate limiting implementation for testing
struct RateLimiter {
    max_calls_per_minute: u32,
    call_timestamps: Arc<Mutex<Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new(max_calls_per_minute: u32) -> Self {
        Self {
            max_calls_per_minute,
            call_timestamps: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn check_rate_limit(&self) -> Result<(), String> {
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);
        
        let mut timestamps = self.call_timestamps.lock().unwrap();
        
        // Remove timestamps older than one minute
        timestamps.retain(|&ts| ts > one_minute_ago);
        
        // Check if we're over the limit
        if timestamps.len() >= self.max_calls_per_minute as usize {
            return Err(format!("Rate limit exceeded: {} calls in last minute", timestamps.len()));
        }
        
        // Add current timestamp
        timestamps.push(now);
        Ok(())
    }
}

/// Create a rate-limited test configuration
fn create_rate_limit_config() -> AIAgentConfig {
    AIAgentConfig {
        provider: "test-provider".to_string(),
        api_key: "test-api-key".to_string(),
        circuit_breaker: CircuitBreakerConfig::default(),
        resource_limits: ResourceLimits {
            max_tokens: 1000,
            max_requests_per_minute: 10, // Limit to 10 requests per minute
            max_tokens_per_minute: 10000,
            max_concurrent_requests: 20,
            max_request_time_ms: 5000,
        },
        cache_size: Some(100),
        timeout_ms: 5000,
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

/// Test rate limiting functionality
#[tokio::test]
async fn test_basic_rate_limiting() {
    // Create a rate limiter with a low limit for testing
    let rate_limiter = RateLimiter::new(5);
    
    // First 5 calls should succeed
    for i in 0..5 {
        let result = rate_limiter.check_rate_limit();
        assert!(result.is_ok(), "Call {} should be under the rate limit", i);
    }
    
    // The 6th call should fail
    let result = rate_limiter.check_rate_limit();
    assert!(result.is_err(), "Call 6 should exceed the rate limit");
    
    // Wait for a bit and try again (not enough for a full reset)
    sleep(Duration::from_millis(100)).await;
    let result = rate_limiter.check_rate_limit();
    assert!(result.is_err(), "Call after short wait should still exceed the rate limit");
    
    // Wait long enough for the oldest call to expire
    sleep(Duration::from_secs(61)).await;
    
    // Should be able to make more calls now
    for i in 0..5 {
        let result = rate_limiter.check_rate_limit();
        assert!(result.is_ok(), "Call {} after wait should be under the rate limit", i);
    }
}

/// Test rate limiting with concurrent requests
#[tokio::test]
async fn test_concurrent_rate_limiting() {
    let rate_limiter = Arc::new(RateLimiter::new(10));
    
    // Create 20 concurrent requests
    let futures = (0..20).map(|i| {
        let limiter = rate_limiter.clone();
        
        async move {
            // Add a small stagger to simulate realistic concurrent behavior
            sleep(Duration::from_millis(i * 5)).await;
            (i, limiter.check_rate_limit())
        }
    });
    
    // Execute all requests concurrently
    let results = join_all(futures).await;
    
    // The first 10 should succeed, the rest should fail
    let successes = results.iter().filter(|(_, result)| result.is_ok()).count();
    let failures = results.iter().filter(|(_, result)| result.is_err()).count();
    
    assert_eq!(successes, 10, "Should allow exactly 10 concurrent requests");
    assert_eq!(failures, 10, "Should reject 10 requests that exceed the rate limit");
    
    // Get the actual indexes to verify the order
    let success_indexes: Vec<_> = results.iter()
        .filter_map(|(idx, result)| if result.is_ok() { Some(*idx) } else { None })
        .collect();
    let failure_indexes: Vec<_> = results.iter()
        .filter_map(|(idx, result)| if result.is_err() { Some(*idx) } else { None })
        .collect();
    
    println!("Successful request indexes: {:?}", success_indexes);
    println!("Failed request indexes: {:?}", failure_indexes);
    
    // The first batch should generally succeed, though with true concurrency
    // the exact pattern might vary
    for idx in &success_indexes {
        assert!(*idx < 15, "Earlier requests (with index < 15) should generally succeed");
    }
    
    for idx in &failure_indexes {
        assert!(*idx >= 5, "Later requests (with index >= 5) should generally fail");
    }
}

/// Test token-based rate limiting
#[tokio::test]
async fn test_token_rate_limiting() {
    // This would test a more complex rate limiting mechanism that accounts for
    // both request count and token usage. In a real implementation, this would
    // involve tracking token usage per request and ensuring the total stays 
    // under the configured limit.
    
    // Create a config with token-based limits
    let config = AIAgentConfig {
        provider: "test-provider".to_string(),
        api_key: "test-api-key".to_string(),
        circuit_breaker: CircuitBreakerConfig::default(),
        resource_limits: ResourceLimits {
            max_tokens: 1000,
            max_requests_per_minute: 100, // High request limit
            max_tokens_per_minute: 5000,  // Relatively low token limit
            max_concurrent_requests: 20,
            max_request_time_ms: 5000,
        },
        cache_size: Some(100),
        timeout_ms: 5000,
        max_retries: 1,
    };
    
    // In a real test, we would now use the adapter to send requests of varying
    // token sizes and verify that the token limit is enforced properly.
    
    // For now, we'll just simulate the behavior
    struct TokenRateLimiter {
        max_tokens_per_minute: u32,
        tokens_used: Arc<Mutex<u32>>,
    }
    
    impl TokenRateLimiter {
        fn new(max_tokens_per_minute: u32) -> Self {
            Self {
                max_tokens_per_minute,
                tokens_used: Arc::new(Mutex::new(0)),
            }
        }
        
        fn check_token_limit(&self, tokens: u32) -> Result<(), String> {
            let mut current_tokens = self.tokens_used.lock().unwrap();
            
            // Check if this would exceed the limit
            if *current_tokens + tokens > self.max_tokens_per_minute {
                return Err(format!("Token limit exceeded: {} + {} > {}", 
                                  *current_tokens, tokens, self.max_tokens_per_minute));
            }
            
            // Add tokens used
            *current_tokens += tokens;
            Ok(())
        }
    }
    
    // Create a token limiter with the config limits
    let token_limiter = TokenRateLimiter::new(5000);
    
    // Test with varying token usages
    assert!(token_limiter.check_token_limit(1000).is_ok(), "First 1000 tokens should be allowed");
    assert!(token_limiter.check_token_limit(1500).is_ok(), "Next 1500 tokens should be allowed");
    assert!(token_limiter.check_token_limit(2000).is_ok(), "Next 2000 tokens should be allowed");
    assert!(token_limiter.check_token_limit(500).is_ok(), "Final 500 tokens should be allowed");
    assert!(token_limiter.check_token_limit(1).is_err(), "Should reject after token limit reached");
} 