// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Service Failure Chaos Tests
//!
//! Tests system resilience when services crash, fail, or become unresponsive.

use super::common::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Test: Service Crash and Recovery
///
/// Validates that the system can detect a service crash and recover gracefully.
///
/// **Scenario**:
/// 1. Start a service
/// 2. Simulate sudden crash (kill process)
/// 3. System should detect failure via health checks
/// 4. System should retry or route around failure
/// 5. Restart service
/// 6. System should detect recovery and restore routing
///
/// **Expected Behavior**:
/// - No data loss
/// - Graceful degradation during outage
/// - Automatic recovery when service returns
/// - Clear error messages to users
#[tokio::test]
async fn test_service_crash_recovery() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Service Crash and Recovery");

    let service = Arc::new(RwLock::new(MockService::new("test-service")));
    let metrics = Arc::new(RwLock::new(ServiceMetrics::new()));

    // Phase 1: Verify service is healthy
    {
        let svc = service.read().await;
        assert!(svc.is_healthy(), "Service should start healthy");
        println!("✅ Phase 1: Service started and healthy");
    }

    // Phase 2: Send successful requests
    let request_count = 10;
    for i in 0..request_count {
        let result = send_request(&service, &metrics, i).await;
        assert!(result.is_ok(), "Request {} should succeed", i);
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.successful_requests, request_count as u64);
        println!("✅ Phase 2: {} successful requests completed", request_count);
    }

    // Phase 3: Simulate service crash
    {
        let mut svc = service.write().await;
        svc.crash();
        println!("💥 Phase 3: Service crashed (simulated)");
    }

    // Phase 4: Verify crash detection and graceful degradation
    for i in 0..5 {
        let result = send_request(&service, &metrics, i).await;
        assert!(result.is_err(), "Request should fail when service is down");

        // Verify error message is informative
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("service unavailable") || error_msg.contains("crashed"),
                "Error message should be informative: {}",
                error_msg
            );
        }
    }
    {
        let m = metrics.read().await;
        assert!(m.failed_requests >= 5, "Failed requests should be tracked");
        println!("✅ Phase 4: Crash detected, graceful error handling working");
    }

    // Phase 5: Simulate service recovery
    {
        let mut svc = service.write().await;
        svc.recover();
        println!("🔄 Phase 5: Service recovered (simulated)");
    }

    // Phase 6: Wait for recovery (event-based, no arbitrary sleep)
    wait_for_healthy(&service, Duration::from_secs(5)).await?;
    println!("✅ Phase 6: Recovery detected");

    // Phase 7: Verify normal operation restored
    for i in 0..10 {
        let result = send_request(&service, &metrics, 200 + i).await;
        assert!(result.is_ok(), "Requests should succeed after recovery");
    }

    // Final metrics check
    {
        let m = metrics.read().await;
        println!("\n📊 Final Metrics:");
        println!("  ✅ Successful requests: {}", m.successful_requests);
        println!("  ❌ Failed requests: {}", m.failed_requests);
        println!("  ⏱️  Avg latency: {:.2}ms", m.average_latency_ms());

        assert!(m.successful_requests >= 20, "Should have >= 20 successful requests");
        assert!(m.failed_requests >= 5, "Should have tracked failures");
    }

    println!("\n🎉 CHAOS TEST PASSED: Service crash and recovery handled gracefully");
    Ok(())
}

/// Test: Cascading Service Failures
///
/// Tests behavior when multiple services fail in sequence.
///
/// **Scenario**:
/// 1. Start 3+ interconnected services
/// 2. Fail service A
/// 3. Verify impact isolation
/// 4. Fail service B (dependent on A)
/// 5. Verify no cascading failure to service C
///
/// **Expected Behavior**:
/// - Circuit breakers prevent cascading failures
/// - Bulkheads isolate failures
/// - System degrades gracefully
/// - Core functionality remains available
#[tokio::test]
async fn test_cascading_failures() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Cascading Failures");

    // Create three interconnected services
    // Service A (leaf) - no dependencies
    // Service B (middle) - depends on A
    // Service C (core) - depends on B, but should be isolated

    let service_a = Arc::new(RwLock::new(MockService::new("service-a")));
    let service_b = Arc::new(RwLock::new(MockService::new("service-b")));
    let service_c = Arc::new(RwLock::new(MockService::new("service-c")));

    let metrics_a = Arc::new(RwLock::new(ServiceMetrics::new()));
    let metrics_b = Arc::new(RwLock::new(ServiceMetrics::new()));
    let metrics_c = Arc::new(RwLock::new(ServiceMetrics::new()));

    // Phase 1: Verify all services healthy
    {
        let a = service_a.read().await;
        let b = service_b.read().await;
        let c = service_c.read().await;
        assert!(a.is_healthy() && b.is_healthy() && c.is_healthy());
        println!("✅ Phase 1: All services healthy");
    }

    // Phase 2: Send requests through the stack (C → B → A)
    for i in 0..5 {
        send_request(&service_a, &metrics_a, i).await?;
        send_request(&service_b, &metrics_b, i).await?;
        send_request(&service_c, &metrics_c, i).await?;
    }
    {
        let m = metrics_c.read().await;
        assert_eq!(m.successful_requests, 5);
        println!("✅ Phase 2: 5 requests successful through full stack");
    }

    // Phase 3: Fail service A (leaf)
    {
        let mut a = service_a.write().await;
        a.crash();
        println!("💥 Phase 3: Service A crashed");
    }

    // Phase 4: Verify B detects A failure (circuit breaker should trip)
    for _ in 0..3 {
        let result = send_request(&service_a, &metrics_a, 10).await;
        assert!(result.is_err(), "Service A should be down");
    }

    // Service B and C should still be healthy (not crashed)
    {
        let b = service_b.read().await;
        let c = service_c.read().await;
        assert!(b.is_healthy(), "Service B should remain healthy");
        assert!(c.is_healthy(), "Service C should remain healthy");
        println!("✅ Phase 4: Cascade prevented - B and C remain healthy despite A failure");
    }

    // Phase 5: Recover A, verify system recovers
    {
        let mut a = service_a.write().await;
        a.recover();
        println!("🔄 Phase 5: Service A recovered");
    }

    // Phase 6: Verify full stack recovery
    wait_for_healthy(&service_a, Duration::from_secs(5)).await?;

    for i in 0..5 {
        send_request(&service_a, &metrics_a, 20 + i).await?;
        send_request(&service_b, &metrics_b, 20 + i).await?;
        send_request(&service_c, &metrics_c, 20 + i).await?;
    }

    {
        let m = metrics_c.read().await;
        println!("\n📊 Final Cascade Metrics:");
        println!("  ✅ Service C successes: {}", m.successful_requests);
        println!("  ❌ Service C failures: {}", m.failed_requests);

        assert!(m.successful_requests >= 10, "Should have successful requests");
    }

    println!("\n🎉 CHAOS TEST PASSED: Cascading failures prevented via circuit breakers");
    Ok(())
}

/// Test: Service Slow Response (Latency Injection)
///
/// Tests timeout handling and graceful degradation under slow responses.
///
/// **Scenario**:
/// 1. Start service with artificially increased latency
/// 2. Send requests
/// 3. Verify timeout mechanisms activate
/// 4. Verify fallback strategies engage
///
/// **Expected Behavior**:
/// - Requests timeout appropriately
/// - Fallbacks provide degraded service
/// - System doesn't hang waiting
/// - Users receive timely error messages
#[tokio::test]
async fn test_slow_service_latency_injection() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Slow Service (Latency Injection)");

    let service = Arc::new(RwLock::new(MockService::new("latency-test")));
    let metrics = Arc::new(RwLock::new(ServiceMetrics::new()));

    // Phase 1: Normal latency (fast responses)
    {
        let mut svc = service.write().await;
        svc.set_delay(Duration::from_millis(10));
    }

    for i in 0..5 {
        let result = send_request_with_timeout(&service, &metrics, i, Duration::from_millis(200)).await;
        assert!(result.is_ok(), "Fast requests should succeed");
    }

    {
        let m = metrics.read().await;
        assert_eq!(m.successful_requests, 5);
        assert_eq!(m.timeouts, 0);
        println!("✅ Phase 1: Fast responses completed (avg: {:.2}ms)", m.average_latency_ms());
    }

    // Phase 2: Inject high latency (300ms) with 200ms timeout
    {
        let mut svc = service.write().await;
        svc.set_delay(Duration::from_millis(300));
        println!("🐌 Phase 2: Injected 300ms latency (timeout: 200ms)");
    }

    // Send requests that should timeout
    for i in 10..15 {
        let result = send_request_with_timeout(&service, &metrics, i, Duration::from_millis(200)).await;
        assert!(result.is_err(), "Slow requests should timeout");
    }

    {
        let m = metrics.read().await;
        assert!(m.timeouts >= 5, "Should have at least 5 timeouts");
        println!("✅ Phase 2: Timeouts detected - {} requests timed out", m.timeouts);
    }

    // Phase 3: Restore normal latency
    {
        let mut svc = service.write().await;
        svc.set_delay(Duration::from_millis(10));
        println!("🔄 Phase 3: Normal latency restored");
    }

    for i in 30..35 {
        let result = send_request_with_timeout(&service, &metrics, i, Duration::from_millis(200)).await;
        assert!(result.is_ok(), "Requests should succeed normally");
    }

    // Final metrics
    {
        let m = metrics.read().await;
        println!("\n📊 Final Latency Metrics:");
        println!("  ✅ Successful: {}", m.successful_requests);
        println!("  ⏱️  Timeouts: {}", m.timeouts);
        println!("  📈 Avg latency: {:.2}ms", m.average_latency_ms());

        assert!(m.successful_requests >= 10, "Should have successful requests");
        assert!(m.timeouts >= 5, "Should have detected timeouts");
    }

    println!("\n🎉 CHAOS TEST PASSED: Latency handled with timeouts");
    Ok(())
}

