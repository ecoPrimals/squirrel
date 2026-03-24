// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "Test code: explicit unwrap/expect and local lint noise")]
//! Common test utilities for chaos engineering tests
//!
//! This module provides shared infrastructure for chaos tests including:
//! - Mock services with controllable failure modes
//! - Metrics collection and verification
//! - Test helpers and utilities

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Chaos test result type
pub type ChaosResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Mock service that can simulate various failure modes
#[derive(Debug, Clone)]
pub struct MockService {
    pub name: String,
    pub healthy: bool,
    pub response_delay: Duration,
    pub error_rate: f64,
}

impl MockService {
    /// Create a new mock service
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            healthy: true,
            response_delay: Duration::from_millis(10),
            error_rate: 0.0,
        }
    }

    /// Check if service is healthy
    pub fn is_healthy(&self) -> bool {
        self.healthy
    }

    /// Simulate service crash
    pub fn crash(&mut self) {
        self.healthy = false;
    }

    /// Simulate service recovery
    pub fn recover(&mut self) {
        self.healthy = true;
    }

    /// Set response delay (simulates latency)
    pub fn set_delay(&mut self, delay: Duration) {
        self.response_delay = delay;
    }

    /// Set error rate (0.0 to 1.0)
    pub fn set_error_rate(&mut self, rate: f64) {
        self.error_rate = rate.clamp(0.0, 1.0);
    }

    /// Process a request with simulated behavior
    pub async fn handle_request(&self, request_id: usize) -> ChaosResult<String> {
        if !self.healthy {
            return Err("service unavailable - crashed".into());
        }

        // Simulate processing delay
        if self.response_delay > Duration::ZERO {
            tokio::time::sleep(self.response_delay).await;
        }

        // Simulate random errors based on error rate
        if self.error_rate > 0.0 {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            if rng.r#gen::<f64>() < self.error_rate {
                return Err(format!("simulated error for request {}", request_id).into());
            }
        }

        Ok(format!("Response for request {}", request_id))
    }
}

/// Metrics collector for chaos tests
#[derive(Debug, Default, Clone)]
pub struct ServiceMetrics {
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_latency_ms: u64,
    pub timeouts: u64,
    pub recoveries: u64,
}

impl ServiceMetrics {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record successful request
    pub fn record_success(&mut self, latency_ms: u64) {
        self.successful_requests += 1;
        self.total_latency_ms += latency_ms;
    }

    /// Record failed request
    pub fn record_failure(&mut self) {
        self.failed_requests += 1;
    }

    /// Record timeout
    pub fn record_timeout(&mut self) {
        self.timeouts += 1;
        self.failed_requests += 1;
    }

    /// Record service recovery
    pub fn record_recovery(&mut self) {
        self.recoveries += 1;
    }

    /// Get average latency
    pub fn average_latency_ms(&self) -> f64 {
        if self.successful_requests == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.successful_requests as f64
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_requests + self.failed_requests;
        if total == 0 {
            0.0
        } else {
            self.successful_requests as f64 / total as f64
        }
    }

    /// Get failure rate
    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
}

/// Send a request to a mock service
pub async fn send_request(
    service: &Arc<RwLock<MockService>>,
    metrics: &Arc<RwLock<ServiceMetrics>>,
    request_id: usize,
) -> ChaosResult<String> {
    let start = std::time::Instant::now();

    let result = {
        let svc = service.read().await;
        svc.handle_request(request_id).await
    };

    let latency = start.elapsed();

    // Record metrics
    {
        let mut m = metrics.write().await;
        match &result {
            Ok(_) => m.record_success(latency.as_millis() as u64),
            Err(_) => m.record_failure(),
        }
    }

    result
}

/// Send a request with timeout
pub async fn send_request_with_timeout(
    service: &Arc<RwLock<MockService>>,
    metrics: &Arc<RwLock<ServiceMetrics>>,
    request_id: usize,
    timeout: Duration,
) -> ChaosResult<String> {
    match tokio::time::timeout(timeout, send_request(service, metrics, request_id)).await {
        Ok(result) => result,
        Err(_) => {
            let mut m = metrics.write().await;
            m.record_timeout();
            Err("request timeout".into())
        }
    }
}

/// Wait for service health check to pass
pub async fn wait_for_healthy(
    service: &Arc<RwLock<MockService>>,
    max_wait: Duration,
) -> ChaosResult<()> {
    let start = std::time::Instant::now();
    loop {
        {
            let svc = service.read().await;
            if svc.is_healthy() {
                return Ok(());
            }
        }

        if start.elapsed() > max_wait {
            return Err("timeout waiting for service to become healthy".into());
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Verify metrics meet expected thresholds
pub fn verify_metrics(
    metrics: &ServiceMetrics,
    min_success_rate: f64,
    max_avg_latency_ms: u64,
) -> ChaosResult<()> {
    if metrics.success_rate() < min_success_rate {
        return Err(format!(
            "Success rate {} below threshold {}",
            metrics.success_rate(),
            min_success_rate
        )
        .into());
    }

    if metrics.average_latency_ms() > max_avg_latency_ms as f64 {
        return Err(format!(
            "Average latency {}ms exceeds threshold {}ms",
            metrics.average_latency_ms(),
            max_avg_latency_ms
        )
        .into());
    }

    Ok(())
}

/// Create a pool of mock services
pub fn create_service_pool(count: usize) -> Vec<Arc<RwLock<MockService>>> {
    (0..count)
        .map(|i| Arc::new(RwLock::new(MockService::new(format!("service-{}", i)))))
        .collect()
}

/// Simulate gradual load increase
pub async fn simulate_load_ramp(
    service: &Arc<RwLock<MockService>>,
    metrics: &Arc<RwLock<ServiceMetrics>>,
    initial_rps: usize,
    final_rps: usize,
    duration: Duration,
) -> ChaosResult<()> {
    let steps = 10;
    let step_duration = duration / steps;
    let rps_increment = (final_rps - initial_rps) / steps as usize;

    for step in 0..steps {
        let current_rps = initial_rps + (step as usize * rps_increment);
        let delay_between_requests = Duration::from_secs(1) / current_rps as u32;

        let step_start = std::time::Instant::now();
        while step_start.elapsed() < step_duration {
            let request_id = step * 1000 + (step_start.elapsed().as_millis() as usize);
            let _ = send_request(service, metrics, request_id).await;
            tokio::time::sleep(delay_between_requests).await;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_service_basic() {
        let mut service = MockService::new("test");
        assert!(service.is_healthy());

        service.crash();
        assert!(!service.is_healthy());

        service.recover();
        assert!(service.is_healthy());
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        let mut metrics = ServiceMetrics::new();

        metrics.record_success(100);
        metrics.record_success(200);
        metrics.record_failure();

        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.average_latency_ms(), 150.0);
        assert!((metrics.success_rate() - 0.666).abs() < 0.01);
    }
}
