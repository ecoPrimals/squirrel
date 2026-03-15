// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service chaos tests - crash, recovery, cascading failures

use super::helpers::*;
use std::sync::Arc;
use std::time::Duration;

/// Test 1: Service Crash and Recovery
#[tokio::test]
async fn chaos_01_service_crash_recovery() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Service Crash and Recovery");

    let service = Arc::new(tokio::sync::RwLock::new(MockService::new("test-service")));
    let metrics = Arc::new(tokio::sync::RwLock::new(ServiceMetrics::default()));

    {
        let svc = service.read().await;
        assert!(svc.is_healthy(), "Service should start healthy");
        println!("✅ Phase 1: Service started and healthy");
    }

    let request_count = 10;
    for i in 0..request_count {
        let result = send_request(&service, &metrics, i).await;
        assert!(result.is_ok(), "Request {} should succeed", i);
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.successful_requests, request_count as u64);
        println!(
            "✅ Phase 2: {} successful requests completed",
            request_count
        );
    }

    {
        let mut svc = service.write().await;
        svc.crash();
        println!("💥 Phase 3: Service crashed (simulated)");
    }

    let start_fail_time = std::time::Instant::now();
    for i in 0..5 {
        let result = send_request(&service, &metrics, i).await;
        assert!(result.is_err(), "Request should fail when service is down");
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

    {
        let mut svc = service.write().await;
        svc.recover();
        println!("🔄 Phase 5: Service recovered (simulated)");
    }

    let recovery_start = std::time::Instant::now();
    let mut recovery_success = false;
    let mut backoff = Duration::from_millis(1);
    for attempt in 0..10 {
        let result = send_request(&service, &metrics, 100 + attempt).await;
        if result.is_ok() {
            recovery_success = true;
            println!(
                "✅ Phase 6: Recovery detected after {:?} (attempt {})",
                recovery_start.elapsed(),
                attempt + 1
            );
            break;
        }
        if attempt < 9 {
            tokio::time::sleep(backoff).await;
            backoff = backoff.saturating_mul(2).min(Duration::from_millis(100));
        }
    }

    assert!(
        recovery_success,
        "Service should recover and accept requests"
    );

    for i in 0..10 {
        let result = send_request(&service, &metrics, 200 + i).await;
        assert!(result.is_ok(), "Requests should succeed after recovery");
    }

    {
        let m = metrics.read().await;
        println!("\n📊 Final Metrics:");
        println!("  ✅ Successful requests: {}", m.successful_requests);
        println!("  ❌ Failed requests: {}", m.failed_requests);
        println!("  ⏱️  Avg response time: {:.2}ms", m.avg_response_time_ms);
        println!("  🔄 Recovery time: {:?}", start_fail_time.elapsed());
        assert!(
            m.successful_requests >= 20,
            "Should have >= 20 successful requests"
        );
        assert!(m.failed_requests >= 5, "Should have tracked failures");
        assert!(
            m.avg_response_time_ms < 100.0,
            "Response time should be reasonable"
        );
    }

    println!("\n🎉 CHAOS TEST PASSED: Service crash and recovery handled gracefully");
    Ok(())
}

/// Test 2: Cascading Service Failures
#[tokio::test]
async fn chaos_02_cascading_failures() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Cascading Failures");

    let service_a = Arc::new(tokio::sync::RwLock::new(MockService::new("service-a")));
    let service_b = Arc::new(tokio::sync::RwLock::new(MockService::new("service-b")));
    let service_c = Arc::new(tokio::sync::RwLock::new(MockService::new("service-c")));
    let metrics = Arc::new(tokio::sync::RwLock::new(CascadeMetrics::default()));

    {
        let a = service_a.read().await;
        let b = service_b.read().await;
        let c = service_c.read().await;
        assert!(a.is_healthy() && b.is_healthy() && c.is_healthy());
        println!("✅ Phase 1: All services healthy");
    }

    for i in 0..5 {
        let result = send_cascade_request(&service_a, &service_b, &service_c, &metrics, i).await;
        assert!(result.is_ok(), "Initial requests should succeed");
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.c_success, 5);
        println!("✅ Phase 2: 5 requests successful through full stack");
    }

    {
        let mut a = service_a.write().await;
        a.crash();
        println!("💥 Phase 3: Service A crashed");
    }

    for i in 0..3 {
        let result =
            send_cascade_request(&service_a, &service_b, &service_c, &metrics, 10 + i).await;
        assert!(result.is_err());
    }
    {
        let m = metrics.read().await;
        assert!(m.cascade_prevented > 0, "Cascade should be prevented");
        let b = service_b.read().await;
        let c = service_c.read().await;
        assert!(b.is_healthy(), "Service B should remain healthy");
        assert!(c.is_healthy(), "Service C should remain healthy");
        println!("✅ Phase 4: Cascade prevented - B and C remain healthy despite A failure");
    }

    {
        let mut a = service_a.write().await;
        a.recover();
        println!("🔄 Phase 5: Service A recovered");
    }

    for i in 0..5 {
        let result =
            send_cascade_request(&service_a, &service_b, &service_c, &metrics, 20 + i).await;
        assert!(result.is_ok(), "Requests should succeed after A recovery");
    }

    {
        let m = metrics.read().await;
        println!("\n📊 Final Cascade Metrics:");
        println!("  ✅ Service C successes: {}", m.c_success);
        println!("  ❌ Service C failures: {}", m.c_failures);
        println!("  🛡️  Cascades prevented: {}", m.cascade_prevented);
        assert!(m.c_success >= 10, "Should have successful requests");
        assert!(m.cascade_prevented >= 1, "Should have prevented cascade");
    }

    println!("\n🎉 CHAOS TEST PASSED: Cascading failures prevented via circuit breakers");
    Ok(())
}
