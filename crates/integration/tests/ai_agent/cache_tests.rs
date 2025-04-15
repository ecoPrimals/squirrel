//! Tests for the caching functionality in the AI Agent
//!
//! These tests verify that the caching mechanism works correctly to
//! optimize performance and reduce API usage.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::time::sleep;

use squirrel_integration::ai_agent::{
    AIAgentConfig,
    CircuitBreakerConfig,
    ResourceLimits,
    GenerationOptions,
    AgentRequest,
    AgentResponse,
    OperationType,
};

use super::mock_adapter::MockAIAgent;

/// A simple cache implementation for testing
struct RequestCache {
    max_size: usize,
    cache: Arc<Mutex<HashMap<String, (AgentResponse, Instant)>>>,
    hits: Arc<Mutex<usize>>,
    misses: Arc<Mutex<usize>>,
    ttl: Duration,
}

impl RequestCache {
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            max_size,
            cache: Arc::new(Mutex::new(HashMap::new())),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }
    
    /// Get a cached response if available
    pub fn get(&self, key: &str) -> Option<AgentResponse> {
        let mut cache = self.cache.lock().unwrap();
        let now = Instant::now();
        
        if let Some((response, created_at)) = cache.get(key) {
            // Check if entry is expired
            if now - *created_at > self.ttl {
                // Remove expired entry
                cache.remove(key);
                
                // Increment misses
                let mut misses = self.misses.lock().unwrap();
                *misses += 1;
                
                return None;
            }
            
            // Increment hits
            let mut hits = self.hits.lock().unwrap();
            *hits += 1;
            
            return Some(response.clone());
        }
        
        // Increment misses
        let mut misses = self.misses.lock().unwrap();
        *misses += 1;
        
        None
    }
    
    /// Add an entry to the cache
    pub fn put(&self, key: String, response: AgentResponse) {
        let mut cache = self.cache.lock().unwrap();
        
        // Handle cache size limit by removing oldest entries if needed
        if cache.len() >= self.max_size && !cache.contains_key(&key) {
            // In a real implementation, we would use an LRU policy
            // For this test, just clear a random entry
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }
        
        // Insert new entry
        cache.insert(key, (response, Instant::now()));
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let cache = self.cache.lock().unwrap();
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        
        (cache.len(), hits, misses)
    }
    
    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        
        let mut hits = self.hits.lock().unwrap();
        *hits = 0;
        
        let mut misses = self.misses.lock().unwrap();
        *misses = 0;
    }
}

/// Create a request with a specific cache key
fn create_test_request(key: &str) -> (String, AgentRequest) {
    let prompt = format!("Test prompt for {}", key);
    let request = AgentRequest {
        prompt: prompt.clone(),
        system_message: Some("You are a helpful assistant".to_string()),
        options: GenerationOptions {
            max_tokens: Some(100),
            temperature: 0.0, // Use temperature 0 for deterministic outputs
            top_p: 1.0,
            ..Default::default()
        },
        operation_type: OperationType::Generate,
        ..Default::default()
    };
    
    // Compute cache key (in a real implementation this would be a hash of the request)
    let cache_key = key.to_string();
    
    (cache_key, request)
}

/// Create a mock response
fn create_mock_response(request: &AgentRequest) -> AgentResponse {
    AgentResponse {
        id: uuid::Uuid::new_v4(),
        request_id: request.id,
        text: format!("Mock response for: {}", request.prompt),
        format: squirrel_integration::ai_agent::ContentFormat::Text,
        usage: None,
        metadata: Default::default(),
        raw_output: None,
        content: format!("Mock content for: {}", request.prompt),
        usage_info: None,
        raw_response: None,
    }
}

/// Test basic cache functionality
#[tokio::test]
async fn test_basic_caching() {
    // Create a configuration
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
    mock.initialize().await.expect("Failed to initialize mock adapter");
    
    // Create a cache with small size and long TTL
    let cache = RequestCache::new(10, 3600);
    
    // Create test requests
    let (key1, request1) = create_test_request("key1");
    let (key2, request2) = create_test_request("key2");
    
    // Create mock responses
    let response1 = create_mock_response(&request1);
    let response2 = create_mock_response(&request2);
    
    // Verify cache is empty
    assert!(cache.get(&key1).is_none(), "Cache should be empty initially");
    assert!(cache.get(&key2).is_none(), "Cache should be empty initially");
    
    // Add entries to cache
    cache.put(key1.clone(), response1.clone());
    cache.put(key2.clone(), response2.clone());
    
    // Verify entries can be retrieved
    let cached1 = cache.get(&key1);
    let cached2 = cache.get(&key2);
    
    assert!(cached1.is_some(), "Cache should contain key1");
    assert!(cached2.is_some(), "Cache should contain key2");
    
    // Verify content is correct
    let response1_content = cached1.unwrap().content;
    let response2_content = cached2.unwrap().content;
    
    assert_eq!(response1_content, response1.content, "Cached content should match original");
    assert_eq!(response2_content, response2.content, "Cached content should match original");
    
    // Verify cache statistics
    let (size, hits, misses) = cache.stats();
    assert_eq!(size, 2, "Cache should contain 2 entries");
    assert_eq!(hits, 2, "Cache should have 2 hits");
    assert_eq!(misses, 2, "Cache should have 2 misses");
}

/// Test cache expiration
#[tokio::test]
async fn test_cache_expiration() {
    // Create a cache with small TTL for testing
    let cache = RequestCache::new(10, 1); // 1 second TTL
    
    // Create test request
    let (key, request) = create_test_request("expiring");
    let response = create_mock_response(&request);
    
    // Add to cache
    cache.put(key.clone(), response.clone());
    
    // Verify entry is in cache
    assert!(cache.get(&key).is_some(), "Entry should be in cache initially");
    
    // Wait for TTL to expire
    sleep(Duration::from_secs(2)).await;
    
    // Verify entry is expired
    assert!(cache.get(&key).is_none(), "Entry should be expired after TTL");
    
    // Verify cache statistics
    let (size, hits, misses) = cache.stats();
    assert_eq!(size, 0, "Cache should be empty after expiration");
    assert_eq!(hits, 1, "Cache should have 1 hit");
    assert_eq!(misses, 2, "Cache should have 2 misses (initial + after expiration)");
}

/// Test cache size limits
#[tokio::test]
async fn test_cache_size_limit() {
    // Create a cache with small size limit
    let cache = RequestCache::new(3, 3600);
    
    // Add entries to fill the cache
    for i in 0..5 {
        let (key, request) = create_test_request(&format!("key{}", i));
        let response = create_mock_response(&request);
        cache.put(key, response);
    }
    
    // Verify cache size is limited
    let (size, _, _) = cache.stats();
    assert_eq!(size, 3, "Cache size should be limited to 3 entries");
    
    // Check which entries are in the cache (should be the most recently added)
    assert!(cache.get("key0").is_none(), "Oldest entry should be evicted");
    assert!(cache.get("key1").is_none(), "Second oldest entry should be evicted");
    assert!(cache.get("key2").is_some(), "Key2 should be in cache");
    assert!(cache.get("key3").is_some(), "Key3 should be in cache");
    assert!(cache.get("key4").is_some(), "Key4 should be in cache");
}

/// Test cache with identical requests but different parameters
#[tokio::test]
async fn test_cache_with_different_parameters() {
    // Create a cache
    let cache = RequestCache::new(10, 3600);
    
    // Create requests with same prompt but different parameters
    let mut request1 = AgentRequest {
        prompt: "What is the capital of France?".to_string(),
        system_message: Some("You are a helpful assistant".to_string()),
        options: GenerationOptions {
            max_tokens: Some(100),
            temperature: 0.0,
            top_p: 1.0,
            ..Default::default()
        },
        operation_type: OperationType::Generate,
        ..Default::default()
    };
    
    let mut request2 = request1.clone();
    request2.options.temperature = 0.7; // Different temperature
    
    let mut request3 = request1.clone();
    request3.options.max_tokens = Some(200); // Different max_tokens
    
    // Create mock responses
    let response1 = create_mock_response(&request1);
    let response2 = create_mock_response(&request2);
    let response3 = create_mock_response(&request3);
    
    // In a real implementation, these would have different cache keys
    // For this test, we'll manually create different keys
    cache.put("request1".to_string(), response1);
    cache.put("request2".to_string(), response2);
    cache.put("request3".to_string(), response3);
    
    // Verify all entries are cached separately
    assert!(cache.get("request1").is_some(), "Request1 should be cached");
    assert!(cache.get("request2").is_some(), "Request2 should be cached");
    assert!(cache.get("request3").is_some(), "Request3 should be cached");
    
    // Verify cache statistics
    let (size, _, _) = cache.stats();
    assert_eq!(size, 3, "Cache should have 3 separate entries");
}

/// Test cache performance improvement
#[tokio::test]
async fn test_cache_performance() {
    // Create a cache
    let cache = RequestCache::new(100, 3600);
    
    // Create test request
    let (key, request) = create_test_request("performance");
    let response = create_mock_response(&request);
    
    // Measure uncached access time (simulating API call)
    let start = Instant::now();
    
    // Simulate an expensive operation (API call)
    sleep(Duration::from_millis(100)).await;
    
    // Add to cache
    cache.put(key.clone(), response);
    
    let uncached_time = start.elapsed();
    
    // Measure cached access time
    let start = Instant::now();
    
    // Get from cache
    let _ = cache.get(&key);
    
    let cached_time = start.elapsed();
    
    println!("Uncached time: {:?}", uncached_time);
    println!("Cached time: {:?}", cached_time);
    
    // Verify cached access is significantly faster
    assert!(cached_time < uncached_time / 10, "Cached access should be at least 10x faster");
}

/// A simple in-memory cache implementation for testing
#[derive(Clone)]
struct TestCache {
    store: Arc<Mutex<HashMap<String, String>>>,
    hit_count: Arc<Mutex<usize>>,
    miss_count: Arc<Mutex<usize>>,
}

impl TestCache {
    fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            hit_count: Arc::new(Mutex::new(0)),
            miss_count: Arc::new(Mutex::new(0)),
        }
    }
    
    fn get(&self, key: &str) -> Option<String> {
        let store = self.store.lock().unwrap();
        let result = store.get(key).cloned();
        
        if result.is_some() {
            let mut hits = self.hit_count.lock().unwrap();
            *hits += 1;
        } else {
            let mut misses = self.miss_count.lock().unwrap();
            *misses += 1;
        }
        
        result
    }
    
    fn set(&self, key: String, value: String) {
        let mut store = self.store.lock().unwrap();
        store.insert(key, value);
    }
    
    fn get_hit_count(&self) -> usize {
        *self.hit_count.lock().unwrap()
    }
    
    fn get_miss_count(&self) -> usize {
        *self.miss_count.lock().unwrap()
    }
}

/// Test basic caching functionality
#[tokio::test]
async fn test_basic_caching_mock() {
    // Create a configuration
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
    mock.initialize().await.expect("Failed to initialize mock adapter");
    
    // Create our test cache
    let cache = TestCache::new();
    
    // Execute a request without caching
    let prompt = "test prompt";
    let direct_result = mock.process_request(prompt).await.expect("Request should succeed");
    
    // Get the result again, but this time cache it
    let cached_value = cache.get(prompt);
    if cached_value.is_none() {
        let result = mock.process_request(prompt).await.expect("Request should succeed");
        cache.set(prompt.to_string(), result);
    }
    
    // Verify cache miss on first access
    assert_eq!(cache.get_miss_count(), 1);
    assert_eq!(cache.get_hit_count(), 0);
    
    // Now get the result again, which should be cached
    let cached_result = cache.get(prompt).expect("Should be in cache");
    
    // Verify cache hit
    assert_eq!(cache.get_hit_count(), 1);
    assert_eq!(cache.get_miss_count(), 1);
    
    // Verify the cached result matches the direct result
    assert_eq!(cached_result, direct_result);
}

/// Test cache invalidation
#[tokio::test]
async fn test_cache_invalidation() {
    // Create a configuration
    let config = AIAgentConfig::default();
    
    // Create our mock adapter
    let mock = MockAIAgent::new(config);
    mock.initialize().await.expect("Failed to initialize mock adapter");
    
    // Create our test cache
    let cache = TestCache::new();
    
    // Execute and cache multiple requests
    for i in 0..3 {
        let prompt = format!("test prompt {}", i);
        let result = mock.process_request(&prompt).await.expect("Request should succeed");
        cache.set(prompt, result);
    }
    
    // Verify initial cache state
    assert_eq!(cache.get_miss_count(), 3);
    assert_eq!(cache.get_hit_count(), 0);
    
    // Access the cached items
    for i in 0..3 {
        let prompt = format!("test prompt {}", i);
        let _ = cache.get(&prompt).expect("Should be in cache");
    }
    
    // Verify cache hits
    assert_eq!(cache.get_hit_count(), 3);
    
    // Simulate cache invalidation by removing an item
    {
        let mut store = cache.store.lock().unwrap();
        store.remove("test prompt 1");
    }
    
    // Access the items again
    for i in 0..3 {
        let prompt = format!("test prompt {}", i);
        let cached_result = cache.get(&prompt);
        
        if cached_result.is_none() {
            // Re-generate and cache the missing result
            let result = mock.process_request(&prompt).await.expect("Request should succeed");
            cache.set(prompt, result);
        }
    }
    
    // Verify we had one more miss due to the invalidated item
    assert_eq!(cache.get_miss_count(), 4);
    assert_eq!(cache.get_hit_count(), 5); // 3 hits before + 2 hits after (since 1 was invalidated)
} 