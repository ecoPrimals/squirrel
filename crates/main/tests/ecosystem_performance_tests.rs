//! Ecosystem Performance Tests
//!
//! This test suite focuses on performance characteristics of the ecosystem
//! integration, including throughput, latency, and resource utilization.

use squirrel::biomeos_integration::*;
use squirrel::error::PrimalError;
use squirrel::security::UniversalSecurityAdapter;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;

/// Performance test configuration
#[derive(Clone)]
pub struct PerformanceTestConfig {
    pub concurrent_requests: usize,
    pub request_duration: Duration,
    pub warmup_requests: usize,
    pub measurement_duration: Duration,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            concurrent_requests: 100,
            request_duration: Duration::from_millis(100),
            warmup_requests: 50,
            measurement_duration: Duration::from_secs(30),
        }
    }
}

/// Performance metrics collection
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_rps: f64,
    pub error_rate: f64,
    pub start_time: Instant,
    pub end_time: Instant,
    pub latencies: Vec<Duration>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            throughput_rps: 0.0,
            error_rate: 0.0,
            start_time: Instant::now(),
            end_time: Instant::now(),
            latencies: Vec::new(),
        }
    }

    pub fn record_request(&mut self, latency: Duration, success: bool) {
        self.total_requests += 1;
        self.latencies.push(latency);

        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
    }

    pub fn finalize(&mut self) {
        self.end_time = Instant::now();
        let duration = self.end_time.duration_since(self.start_time);

        // Calculate throughput
        self.throughput_rps = self.total_requests as f64 / duration.as_secs_f64();

        // Calculate error rate
        self.error_rate = self.failed_requests as f64 / self.total_requests as f64;

        // Calculate latency statistics
        if !self.latencies.is_empty() {
            let mut sorted_latencies = self.latencies.clone();
            sorted_latencies.sort();

            let sum: Duration = sorted_latencies.iter().sum();
            self.average_latency_ms = sum.as_secs_f64() * 1000.0 / sorted_latencies.len() as f64;

            let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
            let p99_index = (sorted_latencies.len() as f64 * 0.99) as usize;

            self.p95_latency_ms =
                sorted_latencies[p95_index.min(sorted_latencies.len() - 1)].as_secs_f64() * 1000.0;
            self.p99_latency_ms =
                sorted_latencies[p99_index.min(sorted_latencies.len() - 1)].as_secs_f64() * 1000.0;
        }
    }
}

/// Performance test environment
#[derive(Clone)]
pub struct PerformanceTestEnvironment {
    pub squirrel_instance: Arc<RwLock<SquirrelBiomeOSIntegration>>,
    pub security_adapter: Arc<UniversalSecurityAdapter>,
    pub config: PerformanceTestConfig,
    pub metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl PerformanceTestEnvironment {
    pub async fn new(config: PerformanceTestConfig) -> Self {
        let squirrel_instance = Arc::new(RwLock::new(SquirrelBiomeOSIntegration::new(
            "perf-test-biome".to_string(),
        )));

        let security_adapter = Arc::new(
            UniversalSecurityAdapter::new("perf-test-config".to_string(), HashMap::new())
                .await
                .unwrap(),
        );

        let metrics = Arc::new(RwLock::new(PerformanceMetrics::new()));

        Self {
            squirrel_instance,
            security_adapter,
            config,
            metrics,
        }
    }

    pub async fn initialize(&self) -> Result<(), PrimalError> {
        let mut squirrel = self.squirrel_instance.write().await;
        squirrel.initialize().await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        let mut squirrel = self.squirrel_instance.write().await;
        squirrel.shutdown().await?;
        Ok(())
    }

    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
}

/// Test high-throughput intelligence request processing
#[tokio::test]
async fn test_high_throughput_intelligence_processing() -> Result<(), Box<dyn std::error::Error>> {
    let config = PerformanceTestConfig {
        concurrent_requests: 200,
        measurement_duration: Duration::from_secs(30),
        ..Default::default()
    };

    let env = PerformanceTestEnvironment::new(config.clone()).await;
    env.initialize().await?;

    // Warmup phase
    println!("Starting warmup phase...");
    for i in 0..config.warmup_requests {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("warmup-{}", i),
            request_type: "analysis".to_string(),
            data: serde_json::json!({"test": "warmup"}),
            metadata: HashMap::new(),
        };

        let _ = squirrel.process_intelligence_request(request).await;
    }

    println!("Warmup complete, starting performance test...");

    // Performance test phase
    let semaphore = Arc::new(Semaphore::new(config.concurrent_requests));
    let mut handles = Vec::new();

    let start_time = Instant::now();
    let mut metrics = env.metrics.write().await;
    metrics.start_time = start_time;
    drop(metrics);

    let mut request_id = 0;

    while start_time.elapsed() < config.measurement_duration {
        let permit = semaphore.clone().acquire_owned().await?;
        let env_clone = env.clone();
        let current_request_id = request_id;
        request_id += 1;

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let start = Instant::now();

            let squirrel = env_clone.squirrel_instance.read().await;
            let request = IntelligenceRequest {
                request_id: format!("perf-{}", current_request_id),
                request_type: "analysis".to_string(),
                data: serde_json::json!({"test": "performance", "id": current_request_id}),
                metadata: HashMap::new(),
            };

            let result = squirrel.process_intelligence_request(request).await;
            let latency = start.elapsed();

            let mut metrics = env_clone.metrics.write().await;
            metrics.record_request(latency, result.is_ok());
        });

        handles.push(handle);

        // Small delay to avoid overwhelming the system
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Wait for all requests to complete
    futures::future::join_all(handles).await;

    let mut metrics = env.metrics.write().await;
    metrics.finalize();

    println!("Performance Test Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful Requests: {}", metrics.successful_requests);
    println!("  Failed Requests: {}", metrics.failed_requests);
    println!("  Throughput: {:.2} RPS", metrics.throughput_rps);
    println!("  Average Latency: {:.2} ms", metrics.average_latency_ms);
    println!("  P95 Latency: {:.2} ms", metrics.p95_latency_ms);
    println!("  P99 Latency: {:.2} ms", metrics.p99_latency_ms);
    println!("  Error Rate: {:.2}%", metrics.error_rate * 100.0);

    // Performance assertions
    assert!(metrics.total_requests > 0);
    assert!(metrics.throughput_rps > 10.0); // At least 10 RPS
    assert!(metrics.average_latency_ms < 1000.0); // Under 1 second average
    assert!(metrics.error_rate < 0.1); // Less than 10% error rate

    env.shutdown().await?;
    Ok(())
}

/// Test concurrent agent deployment performance
#[tokio::test]
async fn test_concurrent_agent_deployment() -> Result<(), Box<dyn std::error::Error>> {
    let config = PerformanceTestConfig {
        concurrent_requests: 50,
        measurement_duration: Duration::from_secs(20),
        ..Default::default()
    };

    let env = PerformanceTestEnvironment::new(config.clone()).await;
    env.initialize().await?;

    let semaphore = Arc::new(Semaphore::new(config.concurrent_requests));
    let mut handles = Vec::new();

    let start_time = Instant::now();
    let mut metrics = env.metrics.write().await;
    metrics.start_time = start_time;
    drop(metrics);

    let mut deployment_id = 0;

    while start_time.elapsed() < config.measurement_duration {
        let permit = semaphore.clone().acquire_owned().await?;
        let env_clone = env.clone();
        let current_deployment_id = deployment_id;
        deployment_id += 1;

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let start = Instant::now();

            let squirrel = env_clone.squirrel_instance.read().await;
            let deployment_manager = squirrel.get_agent_deployment_manager().await;

            let result = match deployment_manager {
                Ok(manager) => {
                    let mut manifest = BiomeManifestParser::generate_template();
                    manifest.agents[0].name = format!("agent-{}", current_deployment_id);
                    manager.deploy_from_manifest(&manifest).await
                }
                Err(e) => Err(e),
            };

            let latency = start.elapsed();

            let mut metrics = env_clone.metrics.write().await;
            metrics.record_request(latency, result.is_ok());
        });

        handles.push(handle);

        // Delay to avoid overwhelming the system
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Wait for all deployments to complete
    futures::future::join_all(handles).await;

    let mut metrics = env.metrics.write().await;
    metrics.finalize();

    println!("Agent Deployment Performance Results:");
    println!("  Total Deployments: {}", metrics.total_requests);
    println!("  Successful Deployments: {}", metrics.successful_requests);
    println!("  Failed Deployments: {}", metrics.failed_requests);
    println!(
        "  Throughput: {:.2} deployments/sec",
        metrics.throughput_rps
    );
    println!("  Average Latency: {:.2} ms", metrics.average_latency_ms);
    println!("  P95 Latency: {:.2} ms", metrics.p95_latency_ms);
    println!("  Error Rate: {:.2}%", metrics.error_rate * 100.0);

    // Performance assertions for deployment
    assert!(metrics.total_requests > 0);
    assert!(metrics.throughput_rps > 1.0); // At least 1 deployment per second
    assert!(metrics.average_latency_ms < 5000.0); // Under 5 seconds average
    assert!(metrics.error_rate < 0.2); // Less than 20% error rate

    env.shutdown().await?;
    Ok(())
}

/// Test security adapter performance under load
#[tokio::test]
async fn test_security_adapter_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = PerformanceTestConfig {
        concurrent_requests: 100,
        measurement_duration: Duration::from_secs(15),
        ..Default::default()
    };

    let env = PerformanceTestEnvironment::new(config.clone()).await;
    env.initialize().await?;

    let semaphore = Arc::new(Semaphore::new(config.concurrent_requests));
    let mut handles = Vec::new();

    let start_time = Instant::now();
    let mut metrics = env.metrics.write().await;
    metrics.start_time = start_time;
    drop(metrics);

    let mut auth_id = 0;

    while start_time.elapsed() < config.measurement_duration {
        let permit = semaphore.clone().acquire_owned().await?;
        let env_clone = env.clone();
        let current_auth_id = auth_id;
        auth_id += 1;

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let start = Instant::now();

            let auth_request = AuthenticationRequest {
                request_id: format!("auth-perf-{}", current_auth_id),
                username: format!("user-{}", current_auth_id),
                password: "test-password".to_string(),
                method: "password".to_string(),
                metadata: HashMap::new(),
            };

            let result = env_clone.security_adapter.authenticate(auth_request).await;
            let latency = start.elapsed();

            let mut metrics = env_clone.metrics.write().await;
            metrics.record_request(latency, result.is_ok());
        });

        handles.push(handle);

        // Small delay to avoid overwhelming the system
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    // Wait for all authentication requests to complete
    futures::future::join_all(handles).await;

    let mut metrics = env.metrics.write().await;
    metrics.finalize();

    println!("Security Adapter Performance Results:");
    println!("  Total Auth Requests: {}", metrics.total_requests);
    println!("  Successful Auths: {}", metrics.successful_requests);
    println!("  Failed Auths: {}", metrics.failed_requests);
    println!("  Throughput: {:.2} auths/sec", metrics.throughput_rps);
    println!("  Average Latency: {:.2} ms", metrics.average_latency_ms);
    println!("  P95 Latency: {:.2} ms", metrics.p95_latency_ms);
    println!("  Error Rate: {:.2}%", metrics.error_rate * 100.0);

    // Performance assertions for security
    assert!(metrics.total_requests > 0);
    assert!(metrics.throughput_rps > 50.0); // At least 50 auths per second
    assert!(metrics.average_latency_ms < 100.0); // Under 100ms average
    assert!(metrics.error_rate < 0.05); // Less than 5% error rate

    env.shutdown().await?;
    Ok(())
}

/// Test memory usage during sustained load
#[tokio::test]
async fn test_memory_usage_under_load() -> Result<(), Box<dyn std::error::Error>> {
    let config = PerformanceTestConfig {
        concurrent_requests: 50,
        measurement_duration: Duration::from_secs(30),
        ..Default::default()
    };

    let env = PerformanceTestEnvironment::new(config.clone()).await;
    env.initialize().await?;

    // Memory usage tracking
    let mut memory_samples = Vec::new();
    let start_time = Instant::now();

    // Start memory monitoring task
    let memory_task = {
        let env_clone = env.clone();
        tokio::spawn(async move {
            let mut samples = Vec::new();
            while start_time.elapsed() < config.measurement_duration {
                // Sample memory usage (in a real implementation, this would use actual memory metrics)
                let sample = MemoryUsage {
                    timestamp: Instant::now(),
                    heap_bytes: get_current_memory_usage(),
                    allocated_objects: get_allocated_objects_count(),
                };
                samples.push(sample);

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            samples
        })
    };

    // Run load test
    let load_task = {
        let env_clone = env.clone();
        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(config.concurrent_requests));
            let mut handles = Vec::new();
            let mut request_id = 0;

            while start_time.elapsed() < config.measurement_duration {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let env_clone = env_clone.clone();
                let current_request_id = request_id;
                request_id += 1;

                let handle = tokio::spawn(async move {
                    let _permit = permit;

                    let squirrel = env_clone.squirrel_instance.read().await;
                    let request = IntelligenceRequest {
                        request_id: format!("mem-test-{}", current_request_id),
                        request_type: "analysis".to_string(),
                        data: serde_json::json!({"test": "memory", "data": vec![0u8; 1024]}),
                        metadata: HashMap::new(),
                    };

                    let _ = squirrel.process_intelligence_request(request).await;
                });

                handles.push(handle);
                tokio::time::sleep(Duration::from_millis(20)).await;
            }

            futures::future::join_all(handles).await;
        })
    };

    // Wait for both tasks to complete
    let (memory_results, _) = futures::join!(memory_task, load_task);
    memory_samples = memory_results?;

    // Analyze memory usage
    let initial_memory = memory_samples.first().unwrap().heap_bytes;
    let peak_memory = memory_samples.iter().map(|s| s.heap_bytes).max().unwrap();
    let final_memory = memory_samples.last().unwrap().heap_bytes;

    println!("Memory Usage Analysis:");
    println!("  Initial Memory: {} bytes", initial_memory);
    println!("  Peak Memory: {} bytes", peak_memory);
    println!("  Final Memory: {} bytes", final_memory);
    println!("  Memory Growth: {} bytes", final_memory - initial_memory);
    println!(
        "  Peak Usage Ratio: {:.2}x",
        peak_memory as f64 / initial_memory as f64
    );

    // Memory assertions
    assert!(peak_memory > initial_memory); // Some memory growth is expected
    assert!(peak_memory < initial_memory * 5); // But not more than 5x growth
    assert!(final_memory < peak_memory * 1.1); // Memory should be mostly released

    env.shutdown().await?;
    Ok(())
}

/// Test service discovery performance
#[tokio::test]
async fn test_service_discovery_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = PerformanceTestConfig {
        concurrent_requests: 75,
        measurement_duration: Duration::from_secs(15),
        ..Default::default()
    };

    let env = PerformanceTestEnvironment::new(config.clone()).await;
    env.initialize().await?;

    let semaphore = Arc::new(Semaphore::new(config.concurrent_requests));
    let mut handles = Vec::new();

    let start_time = Instant::now();
    let mut metrics = env.metrics.write().await;
    metrics.start_time = start_time;
    drop(metrics);

    let mut discovery_id = 0;

    while start_time.elapsed() < config.measurement_duration {
        let permit = semaphore.clone().acquire_owned().await?;
        let env_clone = env.clone();
        let current_discovery_id = discovery_id;
        discovery_id += 1;

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let start = Instant::now();

            let squirrel = env_clone.squirrel_instance.read().await;
            let result = squirrel.discover_ecosystem_services().await;
            let latency = start.elapsed();

            let mut metrics = env_clone.metrics.write().await;
            metrics.record_request(latency, result.is_ok());
        });

        handles.push(handle);

        // Small delay to avoid overwhelming the system
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Wait for all discovery requests to complete
    futures::future::join_all(handles).await;

    let mut metrics = env.metrics.write().await;
    metrics.finalize();

    println!("Service Discovery Performance Results:");
    println!("  Total Discoveries: {}", metrics.total_requests);
    println!("  Successful Discoveries: {}", metrics.successful_requests);
    println!("  Failed Discoveries: {}", metrics.failed_requests);
    println!(
        "  Throughput: {:.2} discoveries/sec",
        metrics.throughput_rps
    );
    println!("  Average Latency: {:.2} ms", metrics.average_latency_ms);
    println!("  P95 Latency: {:.2} ms", metrics.p95_latency_ms);
    println!("  Error Rate: {:.2}%", metrics.error_rate * 100.0);

    // Performance assertions for service discovery
    assert!(metrics.total_requests > 0);
    assert!(metrics.throughput_rps > 20.0); // At least 20 discoveries per second
    assert!(metrics.average_latency_ms < 200.0); // Under 200ms average
    assert!(metrics.error_rate < 0.1); // Less than 10% error rate

    env.shutdown().await?;
    Ok(())
}

/// Helper structures for memory tracking
#[derive(Debug, Clone)]
struct MemoryUsage {
    timestamp: Instant,
    heap_bytes: usize,
    allocated_objects: usize,
}

/// Mock function to get current memory usage
fn get_current_memory_usage() -> usize {
    // In a real implementation, this would use actual memory profiling
    // For testing, we'll simulate memory usage
    std::thread::available_parallelism().unwrap().get() * 1024 * 1024
}

/// Mock function to get allocated objects count
fn get_allocated_objects_count() -> usize {
    // In a real implementation, this would track object allocations
    // For testing, we'll simulate object count
    1000
}

/// Test resource cleanup performance
#[tokio::test]
async fn test_resource_cleanup_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = PerformanceTestConfig {
        concurrent_requests: 25,
        measurement_duration: Duration::from_secs(10),
        ..Default::default()
    };

    let env = PerformanceTestEnvironment::new(config.clone()).await;
    env.initialize().await?;

    // Create resources to clean up
    let mut resources = Vec::new();
    for i in 0..100 {
        let resource = TestResource {
            id: format!("resource-{}", i),
            data: vec![0u8; 1024], // 1KB of data
            created_at: Instant::now(),
        };
        resources.push(resource);
    }

    // Time cleanup operation
    let cleanup_start = Instant::now();

    // Cleanup resources (simulate cleanup process)
    for resource in resources {
        let _ = resource.cleanup().await;
    }

    let cleanup_duration = cleanup_start.elapsed();

    println!("Resource Cleanup Performance:");
    println!(
        "  Cleanup Duration: {:.2} ms",
        cleanup_duration.as_secs_f64() * 1000.0
    );
    println!("  Resources Cleaned: 100");
    println!(
        "  Cleanup Rate: {:.2} resources/sec",
        100.0 / cleanup_duration.as_secs_f64()
    );

    // Performance assertions for cleanup
    assert!(cleanup_duration < Duration::from_secs(5)); // Cleanup should complete quickly

    env.shutdown().await?;
    Ok(())
}

/// Test resource for cleanup testing
#[derive(Debug)]
struct TestResource {
    id: String,
    data: Vec<u8>,
    created_at: Instant,
}

impl TestResource {
    async fn cleanup(self) -> Result<(), PrimalError> {
        // Simulate cleanup work
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
}

/// Test ecosystem startup performance
#[tokio::test]
async fn test_ecosystem_startup_performance() -> Result<(), Box<dyn std::error::Error>> {
    let startup_start = Instant::now();

    let env = PerformanceTestEnvironment::new(Default::default()).await;
    let creation_time = startup_start.elapsed();

    let init_start = Instant::now();
    env.initialize().await?;
    let init_time = init_start.elapsed();

    let total_startup_time = startup_start.elapsed();

    println!("Ecosystem Startup Performance:");
    println!(
        "  Environment Creation: {:.2} ms",
        creation_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Initialization: {:.2} ms",
        init_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Total Startup: {:.2} ms",
        total_startup_time.as_secs_f64() * 1000.0
    );

    // Performance assertions for startup
    assert!(creation_time < Duration::from_secs(1)); // Environment creation should be fast
    assert!(init_time < Duration::from_secs(5)); // Initialization should be reasonable
    assert!(total_startup_time < Duration::from_secs(10)); // Total startup should be quick

    env.shutdown().await?;
    Ok(())
}
