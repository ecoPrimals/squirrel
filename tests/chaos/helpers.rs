// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Shared test utilities and mock infrastructure for chaos tests

use std::sync::Arc;
use std::time::Duration;

/// Chaos test result
pub type ChaosResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Chaos test configuration
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Test duration
    pub duration: Duration,
    /// Failure injection rate (0.0-1.0)
    pub failure_rate: f64,
    /// Number of concurrent clients
    pub num_clients: usize,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(60),
            failure_rate: 0.1,
            num_clients: 100,
            timeout: Duration::from_secs(10),
        }
    }
}

/// Chaos test metrics
#[derive(Debug, Default)]
pub struct ChaosMetrics {
    /// Total requests sent
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Timed out requests
    pub timeout_requests: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// P95 response time (ms)
    pub p95_response_time_ms: f64,
    /// P99 response time (ms)
    pub p99_response_time_ms: f64,
}

impl ChaosMetrics {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.failed_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Print metrics summary
    pub fn print_summary(&self) {
        println!("\n📊 CHAOS TEST METRICS:");
        println!("  Total Requests:    {}", self.total_requests);
        println!("  Successful:        {} ({:.2}%)", self.successful_requests, self.success_rate());
        println!("  Failed:            {} ({:.2}%)", self.failed_requests, self.failure_rate());
        println!("  Timed Out:         {}", self.timeout_requests);
        println!("  Avg Response:      {:.2}ms", self.avg_response_time_ms);
        println!("  P95 Response:      {:.2}ms", self.p95_response_time_ms);
        println!("  P99 Response:      {:.2}ms", self.p99_response_time_ms);
    }
}

/// Simulate random failures based on failure rate
pub fn should_fail(failure_rate: f64) -> bool {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() < failure_rate
}

/// Simulate network delay
pub async fn simulate_network_delay(min_ms: u64, max_ms: u64) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let delay_ms = rng.gen_range(min_ms..=max_ms);
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}

/// Measure operation duration
pub async fn measure_duration<F, Fut, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed();
    (result, duration)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MOCK SERVICE INFRASTRUCTURE
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Mock service that can crash and recover
#[derive(Debug)]
pub struct MockService {
    pub name: String,
    pub state: ServiceState,
    pub request_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceState {
    Healthy,
    Crashed,
    Recovering,
}

impl MockService {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            state: ServiceState::Healthy,
            request_count: 0,
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.state, ServiceState::Healthy)
    }

    pub fn crash(&mut self) {
        self.state = ServiceState::Crashed;
    }

    pub fn recover(&mut self) {
        self.state = ServiceState::Healthy;
    }

    pub fn handle_request(&mut self, request_id: usize) -> ChaosResult<String> {
        match self.state {
            ServiceState::Healthy => {
                self.request_count += 1;
                Ok(format!("Request {} processed by {}", request_id, self.name))
            }
            ServiceState::Crashed => Err("service unavailable - crashed".into()),
            ServiceState::Recovering => {
                if self.request_count % 3 == 0 {
                    self.state = ServiceState::Healthy;
                    self.request_count += 1;
                    Ok(format!("Request {} processed after recovery", request_id))
                } else {
                    Err("service still recovering".into())
                }
            }
        }
    }
}

/// Service metrics tracking
#[derive(Debug, Default)]
pub struct ServiceMetrics {
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_response_time_ms: f64,
    pub avg_response_time_ms: f64,
}

impl ServiceMetrics {
    pub fn record_success(&mut self, response_time_ms: f64) {
        self.successful_requests += 1;
        self.total_response_time_ms += response_time_ms;
        self.avg_response_time_ms = self.total_response_time_ms / self.successful_requests as f64;
    }

    pub fn record_failure(&mut self) {
        self.failed_requests += 1;
    }
}

/// Metrics for cascading failure tests
#[derive(Debug, Default)]
pub struct CascadeMetrics {
    pub c_success: u64,
    pub c_failures: u64,
    pub cascade_prevented: u64,
}

/// Send a request to the mock service
pub async fn send_request(
    service: &Arc<tokio::sync::RwLock<MockService>>,
    metrics: &Arc<tokio::sync::RwLock<ServiceMetrics>>,
    request_id: usize,
) -> ChaosResult<String> {
    let start = std::time::Instant::now();

    let result = {
        let mut svc = service.write().await;
        svc.handle_request(request_id)
    };

    let elapsed = start.elapsed();
    let response_time_ms = elapsed.as_secs_f64() * 1000.0;

    let mut m = metrics.write().await;
    match &result {
        Ok(_) => m.record_success(response_time_ms),
        Err(_) => m.record_failure(),
    }

    result
}

/// Send cascade request through service chain
pub async fn send_cascade_request(
    service_a: &Arc<tokio::sync::RwLock<MockService>>,
    service_b: &Arc<tokio::sync::RwLock<MockService>>,
    service_c: &Arc<tokio::sync::RwLock<MockService>>,
    metrics: &Arc<tokio::sync::RwLock<CascadeMetrics>>,
    request_id: usize,
) -> ChaosResult<String> {
    let a_result = {
        let mut svc = service_a.write().await;
        svc.handle_request(request_id)
    };

    if a_result.is_err() {
        let mut m = metrics.write().await;
        m.cascade_prevented += 1;
        m.c_failures += 1;
        return Err("Service A unavailable - circuit breaker activated".into());
    }

    let b_result = {
        let mut svc = service_b.write().await;
        svc.handle_request(request_id)
    };

    if b_result.is_err() {
        let mut m = metrics.write().await;
        m.c_failures += 1;
        return Err("Service B failed".into());
    }

    let c_result = {
        let mut svc = service_c.write().await;
        svc.handle_request(request_id)
    };

    let mut m = metrics.write().await;
    if c_result.is_ok() {
        m.c_success += 1;
        Ok("Request processed through full stack".to_string())
    } else {
        m.c_failures += 1;
        Err("Service C failed".into())
    }
}

/// Network controller for partition simulation
#[derive(Debug)]
pub struct NetworkController {
    partitioned: bool,
}

impl NetworkController {
    pub fn new() -> Self {
        Self { partitioned: false }
    }

    pub fn can_communicate(&self, _zone_a: &str, _zone_b: &str) -> bool {
        !self.partitioned
    }

    pub fn partition(&mut self, _zone_a: &str, _zone_b: &str) {
        self.partitioned = true;
    }

    pub fn heal(&mut self, _zone_a: &str, _zone_b: &str) {
        self.partitioned = false;
    }
}

/// Metrics for partition tests
#[derive(Debug, Default)]
pub struct PartitionMetrics {
    pub successful_cross_zone: u64,
    pub partition_detected: u64,
    pub zone_a_local: u64,
    pub zone_b_local: u64,
    pub reconciliations: u64,
}

/// Send cross-zone request
pub async fn send_cross_zone_request(
    zone_a: &Arc<tokio::sync::RwLock<MockService>>,
    zone_b: &Arc<tokio::sync::RwLock<MockService>>,
    network: &Arc<tokio::sync::RwLock<NetworkController>>,
    metrics: &Arc<tokio::sync::RwLock<PartitionMetrics>>,
    request_id: usize,
) -> ChaosResult<String> {
    let can_communicate = {
        let net = network.read().await;
        net.can_communicate("zone-a", "zone-b")
    };

    if !can_communicate {
        let mut m = metrics.write().await;
        m.partition_detected += 1;
        return Err("Network partition - cannot reach zone".into());
    }

    {
        let mut svc_a = zone_a.write().await;
        svc_a.handle_request(request_id)?;
    }

    {
        let mut svc_b = zone_b.write().await;
        svc_b.handle_request(request_id)?;
    }

    let mut m = metrics.write().await;
    m.successful_cross_zone += 1;
    Ok("Cross-zone request completed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_config_default() {
        let config = ChaosConfig::default();
        assert_eq!(config.duration, Duration::from_secs(60));
        assert_eq!(config.failure_rate, 0.1);
        assert_eq!(config.num_clients, 100);
    }

    #[test]
    fn test_chaos_metrics_success_rate() {
        let mut metrics = ChaosMetrics::default();
        metrics.total_requests = 100;
        metrics.successful_requests = 95;
        metrics.failed_requests = 5;
        assert_eq!(metrics.success_rate(), 95.0);
        assert_eq!(metrics.failure_rate(), 5.0);
    }

    #[test]
    fn test_should_fail_probability() {
        assert!(!should_fail(0.0));
        assert!(should_fail(1.0));
        let mut failures = 0;
        for _ in 0..1000 {
            if should_fail(0.5) {
                failures += 1;
            }
        }
        assert!(failures > 400 && failures < 600);
    }

    #[tokio::test]
    async fn test_simulate_network_delay() {
        let start = std::time::Instant::now();
        simulate_network_delay(10, 20).await;
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
        assert!(elapsed <= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn test_measure_duration() {
        let (result, duration) = measure_duration(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        })
        .await;
        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(100));
        assert!(duration <= Duration::from_millis(150));
    }
}
