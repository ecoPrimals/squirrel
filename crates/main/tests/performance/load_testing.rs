// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Load and Stress Testing Framework
//!
//! Validates system performance under various load conditions.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Load test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestConfig {
    /// Number of concurrent users
    pub concurrent_users: usize,
    /// Duration of test
    pub duration: Duration,
    /// Ramp-up time
    pub ramp_up: Duration,
    /// Request rate per second
    pub target_rps: f64,
}

/// Load test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestResults {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub max_latency_ms: f64,
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub duration: Duration,
}

/// Load test engine
pub struct LoadTestEngine {
    config: LoadTestConfig,
    results: Arc<RwLock<Vec<RequestMetric>>>,
}

#[derive(Debug, Clone)]
struct RequestMetric {
    latency: Duration,
    success: bool,
    timestamp: Instant,
}

impl LoadTestEngine {
    pub fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Run load test
    pub async fn run<F, Fut>(&self, request_fn: F) -> LoadTestResults
    where
        F: Fn() -> Fut + Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send,
    {
        let start = Instant::now();
        let mut handles = vec![];

        // Spawn concurrent workers
        for _ in 0..self.config.concurrent_users {
            let request_fn = request_fn.clone();
            let results = self.results.clone();
            let duration = self.config.duration;

            let handle = tokio::spawn(async move {
                let worker_start = Instant::now();
                
                while worker_start.elapsed() < duration {
                    let req_start = Instant::now();
                    let success = request_fn().await.is_ok();
                    let latency = req_start.elapsed();

                    results.write().await.push(RequestMetric {
                        latency,
                        success,
                        timestamp: req_start,
                    });

                    // Legitimate: Rate limiting for controlled load simulation
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            });

            handles.push(handle);
        }

        // Wait for all workers
        for handle in handles {
            let _ = handle.await;
        }

        self.calculate_results(start.elapsed()).await
    }

    async fn calculate_results(&self, total_duration: Duration) -> LoadTestResults {
        let metrics = self.results.read().await;
        let total_requests = metrics.len() as u64;
        let successful_requests = metrics.iter().filter(|m| m.success).count() as u64;
        let failed_requests = total_requests - successful_requests;

        let mut latencies: Vec<f64> = metrics
            .iter()
            .map(|m| m.latency.as_secs_f64() * 1000.0)
            .collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let average_latency_ms = if !latencies.is_empty() {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        } else {
            0.0
        };

        let p50_latency_ms = percentile(&latencies, 0.50);
        let p95_latency_ms = percentile(&latencies, 0.95);
        let p99_latency_ms = percentile(&latencies, 0.99);
        let max_latency_ms = latencies.last().copied().unwrap_or(0.0);

        let requests_per_second = total_requests as f64 / total_duration.as_secs_f64();
        let error_rate = if total_requests > 0 {
            failed_requests as f64 / total_requests as f64
        } else {
            0.0
        };

        LoadTestResults {
            total_requests,
            successful_requests,
            failed_requests,
            average_latency_ms,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            max_latency_ms,
            requests_per_second,
            error_rate,
            duration: total_duration,
        }
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let idx = ((sorted_values.len() as f64) * p) as usize;
    sorted_values[idx.min(sorted_values.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_test_engine() {
        let config = LoadTestConfig {
            concurrent_users: 10,
            duration: Duration::from_secs(1),
            ramp_up: Duration::from_millis(100),
            target_rps: 100.0,
        };

        let engine = LoadTestEngine::new(config);

        let results = engine
            .run(|| async {
                tokio::time::sleep(Duration::from_millis(10)).await; // Legitimate: simulating work
                Ok(())
            })
            .await;

        assert!(results.total_requests > 0);
        assert!(results.requests_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_load_test_with_failures() {
        let config = LoadTestConfig {
            concurrent_users: 5,
            duration: Duration::from_millis(500),
            ramp_up: Duration::from_millis(50),
            target_rps: 50.0,
        };

        let engine = LoadTestEngine::new(config);

        let results = engine
            .run(|| async {
                // Simulate 20% failure rate
                if rand::random::<f64>() < 0.2 {
                    Err("Simulated failure".into())
                } else {
                    Ok(())
                }
            })
            .await;

        assert!(results.total_requests > 0);
        assert!(results.error_rate > 0.0);
    }

    #[test]
    fn test_percentile_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        
        assert_eq!(percentile(&values, 0.50), 5.0);
        assert_eq!(percentile(&values, 0.95), 9.0);
        assert_eq!(percentile(&values, 0.99), 10.0);
    }

    #[test]
    fn test_percentile_empty() {
        let values = vec![];
        assert_eq!(percentile(&values, 0.50), 0.0);
    }
}

