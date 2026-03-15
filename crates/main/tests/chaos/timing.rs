// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Timing and latency chaos tests

use super::helpers::*;
use std::sync::Arc;
use std::time::Duration;

/// Metrics for latency tests
#[derive(Debug, Default)]
struct LatencyMetrics {
    successful: u64,
    timeouts: u64,
    fallbacks: u64,
    total_latency_ms: f64,
    avg_latency_ms: f64,
}

/// Mock service with configurable latency
#[derive(Debug)]
struct MockServiceWithLatency {
    name: String,
    latency: Duration,
    fallback_enabled: bool,
}

impl MockServiceWithLatency {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            latency: Duration::from_millis(10),
            fallback_enabled: false,
        }
    }

    fn set_latency(&mut self, latency: Duration) {
        self.latency = latency;
    }

    fn enable_fallback(&mut self, enabled: bool) {
        self.fallback_enabled = enabled;
    }

    async fn handle_request(&self, request_id: usize) -> ChaosResult<String> {
        tokio::time::sleep(self.latency).await;
        Ok(format!(
            "Request {} processed by {} (latency: {:?})",
            request_id, self.name, self.latency
        ))
    }

    fn handle_fallback(&self, request_id: usize) -> ChaosResult<String> {
        Ok(format!(
            "Request {} served from cache (fallback)",
            request_id
        ))
    }
}

async fn send_request_with_timeout(
    service: &Arc<tokio::sync::RwLock<MockServiceWithLatency>>,
    metrics: &Arc<tokio::sync::RwLock<LatencyMetrics>>,
    request_id: usize,
    timeout: Duration,
) -> ChaosResult<String> {
    let start = std::time::Instant::now();

    let result = tokio::time::timeout(timeout, async {
        let svc = service.read().await;
        svc.handle_request(request_id).await
    })
    .await;

    let elapsed = start.elapsed();
    let latency_ms = elapsed.as_secs_f64() * 1000.0;

    let mut m = metrics.write().await;
    match result {
        Ok(Ok(_)) => {
            m.successful += 1;
            m.total_latency_ms += latency_ms;
            m.avg_latency_ms = m.total_latency_ms / m.successful as f64;
            Ok("Request completed".to_string())
        }
        _ => {
            m.timeouts += 1;
            Err("Request timed out".into())
        }
    }
}

async fn send_request_with_fallback(
    service: &Arc<tokio::sync::RwLock<MockServiceWithLatency>>,
    metrics: &Arc<tokio::sync::RwLock<LatencyMetrics>>,
    request_id: usize,
    timeout: Duration,
) -> ChaosResult<String> {
    let start = std::time::Instant::now();

    let result = tokio::time::timeout(timeout, async {
        let svc = service.read().await;
        svc.handle_request(request_id).await
    })
    .await;

    let elapsed = start.elapsed();
    let latency_ms = elapsed.as_secs_f64() * 1000.0;

    let mut m = metrics.write().await;
    match result {
        Ok(Ok(_)) => {
            m.successful += 1;
            m.total_latency_ms += latency_ms;
            m.avg_latency_ms = m.total_latency_ms / m.successful as f64;
            Ok("Request completed".to_string())
        }
        _ => {
            m.fallbacks += 1;
            let svc = service.read().await;
            svc.handle_fallback(request_id)
        }
    }
}

/// Test 3: Slow Service Response (Latency Injection)
#[tokio::test]
async fn chaos_03_slow_service_latency_injection() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Slow Service (Latency Injection)");

    let service = Arc::new(tokio::sync::RwLock::new(MockServiceWithLatency::new(
        "latency-test",
    )));
    let metrics = Arc::new(tokio::sync::RwLock::new(LatencyMetrics::default()));

    {
        let mut svc = service.write().await;
        svc.set_latency(Duration::from_millis(10));
    }

    for i in 0..5 {
        let result =
            send_request_with_timeout(&service, &metrics, i, Duration::from_millis(200)).await;
        assert!(result.is_ok(), "Fast requests should succeed");
    }

    {
        let m = metrics.read().await;
        assert_eq!(m.successful, 5);
        assert_eq!(m.timeouts, 0);
        println!(
            "✅ Phase 1: Fast responses completed (avg: {:.2}ms)",
            m.avg_latency_ms
        );
    }

    {
        let mut svc = service.write().await;
        svc.set_latency(Duration::from_millis(300));
        println!("🐌 Phase 2: Injected 300ms latency (timeout: 200ms)");
    }

    for i in 10..15 {
        let result =
            send_request_with_timeout(&service, &metrics, i, Duration::from_millis(200)).await;
        assert!(result.is_err(), "Slow requests should timeout");
    }

    {
        let m = metrics.read().await;
        assert!(m.timeouts >= 5, "Should have at least 5 timeouts");
        println!(
            "✅ Phase 2: Timeouts detected - {} requests timed out",
            m.timeouts
        );
    }

    {
        let mut svc = service.write().await;
        svc.enable_fallback(true);
        println!("🔄 Phase 3: Fallback strategy enabled");
    }

    for i in 20..25 {
        let result =
            send_request_with_fallback(&service, &metrics, i, Duration::from_millis(200)).await;
        assert!(result.is_ok(), "Requests should succeed via fallback");
    }

    {
        let m = metrics.read().await;
        assert!(m.fallbacks >= 5, "Should have used fallback");
        println!(
            "✅ Phase 3: Fallback provided degraded service - {} fallbacks",
            m.fallbacks
        );
    }

    {
        let mut svc = service.write().await;
        svc.set_latency(Duration::from_millis(10));
        svc.enable_fallback(false);
        println!("🔄 Phase 4: Normal latency restored");
    }

    for i in 30..35 {
        let result =
            send_request_with_timeout(&service, &metrics, i, Duration::from_millis(200)).await;
        assert!(result.is_ok(), "Requests should succeed normally");
    }

    {
        let m = metrics.read().await;
        println!("\n📊 Final Latency Metrics:");
        println!("  ✅ Successful: {}", m.successful);
        println!("  ⏱️  Timeouts: {}", m.timeouts);
        println!("  🔄 Fallbacks: {}", m.fallbacks);
        println!("  📈 Avg latency: {:.2}ms", m.avg_latency_ms);
        assert!(m.successful >= 10, "Should have successful requests");
        assert!(m.timeouts >= 5, "Should have detected timeouts");
        assert!(m.fallbacks >= 5, "Should have used fallbacks");
    }

    println!("\n🎉 CHAOS TEST PASSED: Latency handled with timeouts and fallbacks");
    Ok(())
}
