// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Long-running operations under mixed short-request load (chaos_12).

use super::super::helpers::*;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
struct MockLongRunningService {
    name: String,
    active_operations: usize,
}

impl MockLongRunningService {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            active_operations: 0,
        }
    }
}

#[derive(Debug, Default)]
struct LongRunningMetrics {
    long_completed: u64,
    short_completed: u64,
    total_long_duration_ms: u64,
    total_short_duration_ms: u64,
    max_concurrent: usize,
}

async fn send_long_request(
    service: &Arc<tokio::sync::RwLock<MockLongRunningService>>,
    metrics: &Arc<tokio::sync::RwLock<LongRunningMetrics>>,
    request_id: usize,
    duration: Duration,
) -> ChaosResult<String> {
    let start = std::time::Instant::now();
    {
        let mut svc = service.write().await;
        svc.active_operations += 1;
        let mut m = metrics.write().await;
        if svc.active_operations > m.max_concurrent {
            m.max_concurrent = svc.active_operations;
        }
    }
    tokio::time::sleep(duration).await;
    {
        let mut svc = service.write().await;
        svc.active_operations -= 1;
    }
    let elapsed = start.elapsed();
    let mut m = metrics.write().await;
    m.long_completed += 1;
    m.total_long_duration_ms += elapsed.as_millis() as u64;
    Ok(format!("Long request {request_id} completed"))
}

async fn send_short_request(
    service: &Arc<tokio::sync::RwLock<MockLongRunningService>>,
    metrics: &Arc<tokio::sync::RwLock<LongRunningMetrics>>,
    request_id: usize,
) -> ChaosResult<String> {
    let start = std::time::Instant::now();
    {
        let mut svc = service.write().await;
        svc.active_operations += 1;
        let mut m = metrics.write().await;
        if svc.active_operations > m.max_concurrent {
            m.max_concurrent = svc.active_operations;
        }
    }
    tokio::time::sleep(Duration::from_millis(10)).await;
    {
        let mut svc = service.write().await;
        svc.active_operations -= 1;
    }
    let elapsed = start.elapsed();
    let mut m = metrics.write().await;
    m.short_completed += 1;
    m.total_short_duration_ms += elapsed.as_millis() as u64;
    Ok(format!("Short request {request_id} completed"))
}

/// Test 12: Long-Running Operations Under Load
#[tokio::test]
async fn chaos_12_long_running_under_load() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Long-Running Operations Under Load");

    let service = Arc::new(tokio::sync::RwLock::new(MockLongRunningService::new(
        "load-service",
    )));
    let metrics = Arc::new(tokio::sync::RwLock::new(LongRunningMetrics::default()));

    let result = send_long_request(&service, &metrics, 1, Duration::from_millis(500)).await;
    assert!(result.is_ok(), "Long operation should succeed without load");
    println!("✅ Phase 1: Baseline long operation");

    let svc_clone = service.clone();
    let metrics_clone = metrics.clone();
    let long_handle = tokio::spawn(async move {
        send_long_request(&svc_clone, &metrics_clone, 100, Duration::from_secs(2)).await
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut short_handles = vec![];
    for i in 200..300 {
        let svc_clone = service.clone();
        let metrics_clone = metrics.clone();
        short_handles.push(tokio::spawn(async move {
            send_short_request(&svc_clone, &metrics_clone, i).await
        }));
    }
    let mut short_success = 0;
    for handle in short_handles {
        if let Ok(Ok(_)) = handle.await {
            short_success += 1;
        }
    }
    let long_result = long_handle.await.expect("should succeed");
    assert!(long_result.is_ok(), "Long operation should complete");
    assert!(short_success >= 90, "Most short operations should complete");
    println!("✅ Phase 2: Concurrent operations completed");

    let mut long_handles = vec![];
    for i in 500..505 {
        let svc_clone = service.clone();
        let metrics_clone = metrics.clone();
        long_handles.push(tokio::spawn(async move {
            send_long_request(&svc_clone, &metrics_clone, i, Duration::from_secs(1)).await
        }));
    }
    let mut short_handles = vec![];
    for i in 600..800 {
        let svc_clone = service.clone();
        let metrics_clone = metrics.clone();
        short_handles.push(tokio::spawn(async move {
            send_short_request(&svc_clone, &metrics_clone, i).await
        }));
    }
    let mut long_success = 0;
    for handle in long_handles {
        if let Ok(Ok(_)) = handle.await {
            long_success += 1;
        }
    }
    let mut short_success = 0;
    for handle in short_handles {
        if let Ok(Ok(_)) = handle.await {
            short_success += 1;
        }
    }
    assert_eq!(long_success, 5, "All long operations should complete");
    assert!(
        short_success >= 180,
        "Most short operations should complete"
    );
    println!("✅ Phase 3: Sustained load handled");

    {
        let m = metrics.read().await;
        assert!(m.long_completed >= 7, "All long operations should complete");
        assert!(
            m.short_completed >= 270,
            "Most short operations should complete"
        );
    }
    println!(
        "\n🎉 CHAOS TEST PASSED: Long-running operations complete without starving short operations"
    );
    Ok(())
}
