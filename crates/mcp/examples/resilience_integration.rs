use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use squirrel_mcp::resilience::{
    bulkhead::{Bulkhead, BulkheadConfig},
    rate_limiter::{RateLimiter, RateLimiterConfig},
    ResilienceError,
};

// A simple service to protect with resilience patterns
struct ExampleService {
    name: String,
    failure_rate: RwLock<f64>,
    latency: RwLock<Duration>,
}

impl ExampleService {
    fn new(name: &str, failure_rate: f64, latency: Duration) -> Self {
        Self {
            name: name.to_string(),
            failure_rate: RwLock::new(failure_rate),
            latency: RwLock::new(latency),
        }
    }
    
    async fn set_failure_rate(&self, rate: f64) {
        let mut lock = self.failure_rate.write().await;
        *lock = rate.clamp(0.0, 1.0);
    }
    
    async fn set_latency(&self, duration: Duration) {
        let mut lock = self.latency.write().await;
        *lock = duration;
    }
    
    async fn call(&self, request_id: u64) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Simulate processing time
        let latency = *self.latency.read().await;
        tokio::time::sleep(latency).await;
        
        // Simulate failures based on failure rate
        let failure_rate = *self.failure_rate.read().await;
        let random_val = rand::random::<f64>();
        
        if random_val < failure_rate {
            Err(format!("Service '{}' failed processing request {}", self.name, request_id).into())
        } else {
            Ok(format!("Service '{}' processed request {}", self.name, request_id))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize a tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    // Create the example service with a moderate failure rate
    let service = Arc::new(ExampleService::new("example", 0.3, Duration::from_millis(50)));
    let service_clone = service.clone();
    
    // Initialize resilience components
    
    // 1. Bulkhead for limiting concurrent calls
    let bulkhead = Bulkhead::new(BulkheadConfig {
        name: "example-service-bulkhead".to_string(),
        max_concurrent_calls: 5,
        max_queue_size: 10,
        call_timeout: Duration::from_millis(200),
        queue_timeout: Duration::from_millis(100),
    });
    println!("📊 Bulkhead initialized with {} max concurrent calls and {} max queue size",
        bulkhead.config().max_concurrent_calls, bulkhead.config().max_queue_size);
    
    // 2. Rate limiter for controlling throughput
    let rate_limiter = RateLimiter::new(RateLimiterConfig {
        name: "example-service-rate-limiter".to_string(),
        limit_for_period: 20,
        limit_refresh_period: Duration::from_secs(1),
        timeout_duration: Some(Duration::from_millis(50)),
        wait_for_permits: true,
    });
    println!("🚦 Rate limiter initialized with {} operations per {} ms",
        rate_limiter.config().limit_for_period, rate_limiter.config().limit_refresh_period.as_millis());
    
    // Run a series of requests using bulkhead and rate limiter
    println!("\n🚀 Starting resilience integration test with normal service conditions...\n");
    
    // Test 1: Normal operation with moderate load
    let mut successful = 0;
    let mut failed = 0;
    let total_requests = 50;
    
    println!("📋 TEST 1: Normal operation ({}% failure rate)", *service.failure_rate.read().await * 100.0);
    for i in 0..total_requests {
        let service = service_clone.clone();
        // Use the bulkhead to protect the service call
        let result = bulkhead.execute(async move {
            service.call(i).await
        }).await;
        
        match result {
            Ok(message) => {
                println!("✅ Request {} succeeded: {}", i, message);
                successful += 1;
            }
            Err(e) => {
                println!("❌ Request {} failed: {}", i, e);
                failed += 1;
            }
        }
        
        // Add a small delay between requests
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    println!("\n📊 TEST 1 RESULTS: {} successful, {} failed requests", successful, failed);
    println!("📉 Bulkhead metrics: available_permits={}, rejection_count={}, timeout_count={}",
        bulkhead.metrics().available_permits,
        bulkhead.metrics().rejection_count,
        bulkhead.metrics().timeout_count);
    
    // Test 2: High failure rate
    service.set_failure_rate(0.8).await;
    successful = 0;
    failed = 0;
    
    println!("\n📋 TEST 2: High failure rate ({}% failure rate)", *service.failure_rate.read().await * 100.0);
    for i in 0..total_requests {
        let service = service_clone.clone();
        // Use the rate limiter to protect the service call
        let result = rate_limiter.execute(async move {
            service.call(i + 100).await
        }).await;
        
        match result {
            Ok(message) => {
                println!("✅ Request {} succeeded: {}", i, message);
                successful += 1;
            }
            Err(e) => {
                println!("❌ Request {} failed: {}", i, e);
                failed += 1;
            }
        }
        
        // Add a small delay between requests
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    println!("\n📊 TEST 2 RESULTS: {} successful, {} failed requests", successful, failed);
    println!("📈 Rate Limiter metrics: available_permits={}, waiting_threads={}, rejection_count={}, timeout_count={}",
        rate_limiter.metrics().await.available_permits,
        rate_limiter.metrics().await.waiting_threads,
        rate_limiter.metrics().await.rejection_count,
        rate_limiter.metrics().await.timeout_count);
    
    // Test 3: Concurrent stress test with bulkhead
    println!("\n📋 TEST 3: Concurrent stress test with bulkhead");
    
    // Create a vec to hold our task handles
    let mut handles = Vec::with_capacity(20);
    successful = 0;
    failed = 0;
    
    // Launch 20 concurrent tasks, which exceeds our bulkhead's capacity
    for i in 0..20 {
        let bulkhead = bulkhead.clone();
        let service = service_clone.clone();
        
        let handle = tokio::spawn(async move {
            let result = bulkhead.execute(async move {
                // Add some variable latency to simulate different processing times
                let extra_latency = Duration::from_millis((i * 10) as u64);
                tokio::time::sleep(extra_latency).await;
                
                service.call(i + 200).await
            }).await;
            
            (i, result)
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete and gather results
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await?);
    }
    
    // Count successes and failures
    let successful = results.iter().filter(|(_i, r)| r.is_ok()).count();
    let failed = results.len() - successful;
    
    println!("\n📊 TEST 3 RESULTS: {} successful, {} failed requests", successful, failed);
    println!("📉 Bulkhead metrics: available_permits={}, rejection_count={}, timeout_count={}",
        bulkhead.metrics().available_permits,
        bulkhead.metrics().rejection_count,
        bulkhead.metrics().timeout_count);
    
    // Success!
    println!("\n✨ Resilience integration test completed successfully!");
    
    Ok(())
} 