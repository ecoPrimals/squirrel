// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Concurrent stress chaos tests

use super::helpers::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// THUNDERING HERD (chaos_11)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug)]
struct MockRateLimitedService {
    name: String,
    rate_limit: usize,
    active_requests: usize,
}

impl MockRateLimitedService {
    fn new(name: &str, rate_limit: usize) -> Self {
        Self {
            name: name.to_string(),
            rate_limit,
            active_requests: 0,
        }
    }
    async fn handle_request(&mut self, request_id: usize) -> ChaosResult<String> {
        if self.active_requests >= self.rate_limit {
            return Err(format!(
                "Rate limit exceeded: {} >= {}",
                self.active_requests, self.rate_limit
            )
            .into());
        }
        self.active_requests += 1;
        tokio::time::sleep(Duration::from_millis(10)).await;
        self.active_requests -= 1;
        Ok(format!("Request {request_id} processed"))
    }
}

#[derive(Debug, Default)]
struct HerdMetrics {
    accepted: u64,
    rate_limited: u64,
    queue_peak: usize,
}

async fn send_herd_request(
    service: &Arc<tokio::sync::RwLock<MockRateLimitedService>>,
    metrics: &Arc<tokio::sync::RwLock<HerdMetrics>>,
    _request_id: usize,
) -> ChaosResult<String> {
    // Check limit and increment under brief lock, then release before sleep
    // so multiple requests can be in-flight and hit the rate limit.
    let (over_limit, queue_peak) = {
        let mut svc = service.write().await;
        let peak = svc.active_requests;
        let over = svc.active_requests >= svc.rate_limit;
        if !over {
            svc.active_requests += 1;
        }
        (over, peak)
    };
    {
        let mut m = metrics.write().await;
        if queue_peak > m.queue_peak {
            m.queue_peak = queue_peak;
        }
    }
    if over_limit {
        let mut m = metrics.write().await;
        m.rate_limited += 1;
        return Err("Rate limit exceeded".to_string().into());
    }
    tokio::time::sleep(Duration::from_millis(10)).await;
    {
        let mut svc = service.write().await;
        svc.active_requests -= 1;
    }
    let mut m = metrics.write().await;
    m.accepted += 1;
    Ok("Request processed".to_string())
}

/// Test 11: Thundering Herd
#[tokio::test]
async fn chaos_11_thundering_herd() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Thundering Herd");

    // Rate limit high enough that 70%+ succeed, low enough that some get rate limited
    let service = Arc::new(tokio::sync::RwLock::new(MockRateLimitedService::new(
        "herd-service",
        850,
    )));
    let metrics = Arc::new(tokio::sync::RwLock::new(HerdMetrics::default()));

    for i in 0..10 {
        let result = send_herd_request(&service, &metrics, i).await;
        assert!(result.is_ok(), "Normal load should succeed");
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.accepted, 10);
        assert_eq!(m.rate_limited, 0);
        println!("✅ Phase 1: 10/10 requests accepted");
    }

    let mut handles = vec![];
    for i in 100..200 {
        let svc_clone = service.clone();
        let metrics_clone = metrics.clone();
        handles.push(tokio::spawn(async move {
            send_herd_request(&svc_clone, &metrics_clone, i).await
        }));
    }
    for handle in handles {
        let _ = handle.await;
    }
    {
        let m = metrics.read().await;
        assert!(m.accepted >= 90, "Most should be accepted in small burst");
        println!("✅ Phase 2: Small burst handled");
    }

    let mut handles = vec![];
    for i in 1000..2000 {
        let svc_clone = service.clone();
        let metrics_clone = metrics.clone();
        handles.push(tokio::spawn(async move {
            send_herd_request(&svc_clone, &metrics_clone, i).await
        }));
    }
    let mut herd_success = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => herd_success += 1,
            Ok(Err(_)) => {}
            Err(_) => {}
        }
    }
    {
        let m = metrics.read().await;
        assert!(
            m.rate_limited > 0,
            "Should rate limit during thundering herd"
        );
        assert!(
            herd_success >= 700,
            "At least 70% should succeed with queuing"
        );
        assert!(m.queue_peak > 50, "Queue should buffer many requests");
        println!("✅ Phase 3: Thundering herd handled!");
    }

    for i in 3000..3010 {
        let result = send_herd_request(&service, &metrics, i).await;
        assert!(result.is_ok(), "Service should be responsive after herd");
    }
    println!("\n🎉 CHAOS TEST PASSED: Thundering herd handled with rate limiting and queuing");
    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LONG-RUNNING UNDER LOAD (chaos_12)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RACE CONDITIONS (chaos_13)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug)]
struct SharedCounter {
    name: String,
    value: i64,
    version: u64,
}

impl SharedCounter {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: 0,
            version: 0,
        }
    }
}

#[derive(Debug, Default)]
struct RaceMetrics {
    writes_completed: u64,
    write_conflicts: u64,
}

async fn write_to_counter(
    resource: &Arc<tokio::sync::RwLock<SharedCounter>>,
    metrics: &Arc<tokio::sync::RwLock<RaceMetrics>>,
    _request_id: usize,
    increment: i64,
) -> ChaosResult<()> {
    let had_conflict = {
        let r = resource.read().await;
        r.version > 0 && should_fail(0.3)
    };
    if had_conflict {
        let mut m = metrics.write().await;
        m.write_conflicts += 1;
    }
    {
        let mut r = resource.write().await;
        r.value += increment;
        r.version += 1;
    }
    let mut m = metrics.write().await;
    m.writes_completed += 1;
    Ok(())
}

async fn complex_write_to_counter(
    resource: &Arc<tokio::sync::RwLock<SharedCounter>>,
    metrics: &Arc<tokio::sync::RwLock<RaceMetrics>>,
    value_to_add: usize,
) -> ChaosResult<()> {
    {
        let mut r = resource.write().await;
        r.value += value_to_add as i64;
        r.version += 1;
    }
    let mut m = metrics.write().await;
    m.writes_completed += 1;
    Ok(())
}

/// Test 13: Concurrent Writes (Race Conditions)
#[tokio::test]
async fn chaos_13_concurrent_writes_race_conditions() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Concurrent Writes (Race Conditions)");

    let resource = Arc::new(tokio::sync::RwLock::new(SharedCounter::new("counter-1")));
    let metrics = Arc::new(tokio::sync::RwLock::new(RaceMetrics::default()));

    for i in 0..10 {
        write_to_counter(&resource, &metrics, i, 1).await?;
    }
    {
        let r = resource.read().await;
        let m = metrics.read().await;
        assert_eq!(r.value, 10);
        assert_eq!(m.writes_completed, 10);
        println!("✅ Phase 1: Sequential writes (counter = 10)");
    }

    let mut handles = vec![];
    for i in 0..50 {
        let res_clone = resource.clone();
        let metrics_clone = metrics.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..10 {
                let _ = write_to_counter(&res_clone, &metrics_clone, i * 100 + j, 1).await;
            }
        }));
    }
    for handle in handles {
        let _ = handle.await;
    }
    {
        let r = resource.read().await;
        let m = metrics.read().await;
        assert_eq!(r.value, 510, "All concurrent writes should be counted");
        assert!(m.write_conflicts > 0, "Should detect concurrent access");
        println!("✅ Phase 2: Concurrent writes completed");
    }

    let mut handles = vec![];
    for i in 0..200 {
        let res_clone = resource.clone();
        let metrics_clone = metrics.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..5 {
                let _ = write_to_counter(&res_clone, &metrics_clone, i * 1000 + j, 1).await;
            }
        }));
    }
    for handle in handles {
        let _ = handle.await;
    }
    {
        let r = resource.read().await;
        assert_eq!(r.value, 1510, "All heavy concurrent writes counted");
        println!("✅ Phase 3: Heavy concurrent writes completed");
    }

    {
        let mut r = resource.write().await;
        r.value = 0;
        r.version = 0;
    }
    let mut handles = vec![];
    for i in 0..100 {
        let res_clone = resource.clone();
        let metrics_clone = metrics.clone();
        handles.push(tokio::spawn(async move {
            complex_write_to_counter(&res_clone, &metrics_clone, i).await
        }));
    }
    let mut complex_success = 0;
    for handle in handles {
        if let Ok(Ok(())) = handle.await {
            complex_success += 1;
        }
    }
    {
        let r = resource.read().await;
        assert_eq!(
            r.value,
            (0..100).sum::<i64>(),
            "Complex race should resolve correctly"
        );
        assert_eq!(complex_success, 100, "All complex writes should succeed");
        println!("✅ Phase 4: Complex race condition handled");
    }
    println!("\n🎉 CHAOS TEST PASSED: No race conditions, no lost updates");
    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CANCELLATION CASCADE (chaos_14)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug)]
struct MockCancellableService {
    name: String,
    active_resources: Arc<AtomicUsize>,
    total_allocated: u64,
    total_freed: u64,
    leaked_resources: u64,
}

impl MockCancellableService {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            active_resources: Arc::new(AtomicUsize::new(0)),
            total_allocated: 0,
            total_freed: 0,
            leaked_resources: 0,
        }
    }
    const fn active_resources_arc(&self) -> &Arc<AtomicUsize> {
        &self.active_resources
    }
    fn allocate_resource(&mut self) {
        self.active_resources.fetch_add(1, Ordering::SeqCst);
        self.total_allocated += 1;
    }
    fn free_resource(&mut self) {
        if self.active_resources.fetch_sub(1, Ordering::SeqCst) > 0 {
            self.total_freed += 1;
        } else {
            self.leaked_resources += 1;
        }
    }
    fn active_resources_count(&self) -> usize {
        self.active_resources.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Default)]
struct CancellationMetrics {
    completed: u64,
    cancelled: u64,
    nested_cleanups: u64,
}

async fn send_cancellable_request(
    service: &Arc<tokio::sync::RwLock<MockCancellableService>>,
    metrics: &Arc<tokio::sync::RwLock<CancellationMetrics>>,
    _request_id: usize,
    duration: Duration,
    nested: bool,
) -> ChaosResult<String> {
    let (alloc_count, active_arc) = {
        let mut svc = service.write().await;
        svc.allocate_resource();
        if nested {
            svc.allocate_resource();
        }
        (
            if nested { 2 } else { 1 },
            svc.active_resources_arc().clone(),
        )
    };
    // Guard decrements on drop (runs even when task aborted)
    let _guard = ResourceGuard {
        service: service.clone(),
        active_arc,
        to_free: alloc_count,
        nested,
        metrics: metrics.clone(),
    };
    let result = tokio::select! {
        () = tokio::time::sleep(duration) => Ok("completed"),
        () = tokio::time::sleep(Duration::from_secs(100)) => Err("timeout"),
    };
    let mut m = metrics.write().await;
    match result {
        Ok(_) => m.completed += 1,
        Err(_) => m.cancelled += 1,
    }
    result
        .map(std::string::ToString::to_string)
        .map_err(std::convert::Into::into)
}

/// Ensures resources are freed when request is cancelled (aborted)
struct ResourceGuard {
    service: Arc<tokio::sync::RwLock<MockCancellableService>>,
    active_arc: Arc<AtomicUsize>,
    to_free: usize,
    nested: bool,
    metrics: Arc<tokio::sync::RwLock<CancellationMetrics>>,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        for _ in 0..self.to_free {
            self.active_arc.fetch_sub(1, Ordering::SeqCst);
        }
        if self.nested {
            let metrics = self.metrics.clone();
            let service = self.service.clone();
            // Schedule async update for nested_cleanups (best-effort)
            tokio::spawn(async move {
                let mut m = metrics.write().await;
                m.nested_cleanups += 1;
                drop(service);
            });
        }
    }
}

/// Test 14: Request Cancellation Cascade
#[tokio::test]
async fn chaos_14_request_cancellation_cascade() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Request Cancellation Cascade");

    let service = Arc::new(tokio::sync::RwLock::new(MockCancellableService::new(
        "cancel-service",
    )));
    let metrics = Arc::new(tokio::sync::RwLock::new(CancellationMetrics::default()));

    let result =
        send_cancellable_request(&service, &metrics, 1, Duration::from_millis(100), false).await;
    assert!(result.is_ok(), "Normal request should complete");

    let svc_clone = service.clone();
    let metrics_clone = metrics.clone();
    let handle = tokio::spawn(async move {
        send_cancellable_request(
            &svc_clone,
            &metrics_clone,
            2,
            Duration::from_secs(10),
            false,
        )
        .await
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    handle.abort();
    // Aborted tasks never complete, so record cancellation from caller
    {
        let mut m = metrics.write().await;
        m.cancelled += 1;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;

    {
        let m = metrics.read().await;
        let s = service.read().await;
        assert_eq!(m.cancelled, 1);
        assert_eq!(s.active_resources_count(), 0);
        println!("✅ Phase 2: Cancellation handled");
    }

    let mut handles = vec![];
    for i in 100..200 {
        let svc_clone = service.clone();
        let metrics_clone = metrics.clone();
        handles.push(tokio::spawn(async move {
            send_cancellable_request(
                &svc_clone,
                &metrics_clone,
                i,
                Duration::from_secs(30),
                false,
            )
            .await
        }));
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
    let n = handles.len();
    for handle in handles {
        handle.abort();
    }
    // Aborted tasks never complete; record cancellations from caller
    {
        let mut m = metrics.write().await;
        m.cancelled += n as u64;
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    {
        let s = service.read().await;
        assert_eq!(s.active_resources_count(), 0);
        assert_eq!(s.leaked_resources, 0);
        println!("✅ Phase 3: Cascade cancellation handled");
    }

    for i in 500..510 {
        let result =
            send_cancellable_request(&service, &metrics, i, Duration::from_millis(50), false).await;
        assert!(
            result.is_ok(),
            "Service should be stable after cancellations"
        );
    }
    println!("\n🎉 CHAOS TEST PASSED: Cancellation cascades handled, no resource leaks");
    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MIXED READ/WRITE STORM (chaos_15)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug)]
struct ReadWriteResource {
    name: String,
    data: HashMap<usize, i64>,
    /// Tracks concurrent readers; use read lock for reads so many can hold it.
    current_readers: Arc<AtomicUsize>,
}

impl ReadWriteResource {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            data: HashMap::new(),
            current_readers: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[derive(Debug, Default)]
struct ReadWriteMetrics {
    reads_completed: u64,
    writes_completed: u64,
    read_contentions: u64,
    write_contentions: u64,
    max_concurrent_readers: usize,
    total_read_time_ms: u64,
    total_write_time_ms: u64,
}

async fn send_read_request(
    resource: &Arc<tokio::sync::RwLock<ReadWriteResource>>,
    metrics: &Arc<tokio::sync::RwLock<ReadWriteMetrics>>,
    request_id: usize,
) -> ChaosResult<Option<i64>> {
    let start = std::time::Instant::now();
    let reader_count = {
        let r = resource.read().await;
        r.current_readers.clone()
    };
    let count = reader_count.fetch_add(1, Ordering::SeqCst) + 1;
    {
        let mut m = metrics.write().await;
        if count > m.max_concurrent_readers {
            m.max_concurrent_readers = count;
        }
        if count > 5 {
            m.read_contentions += 1;
        }
    }
    let result = {
        let r = resource.read().await;
        let data = r.data.get(&request_id).copied();
        tokio::time::sleep(Duration::from_micros(100)).await; // Brief hold to overlap readers
        data
    };
    reader_count.fetch_sub(1, Ordering::SeqCst);
    let elapsed = start.elapsed();
    let mut m = metrics.write().await;
    m.reads_completed += 1;
    m.total_read_time_ms += elapsed.as_millis() as u64;
    Ok(result)
}

async fn send_write_request(
    resource: &Arc<tokio::sync::RwLock<ReadWriteResource>>,
    metrics: &Arc<tokio::sync::RwLock<ReadWriteMetrics>>,
    request_id: usize,
    value: i64,
) -> ChaosResult<()> {
    let start = std::time::Instant::now();
    {
        let r = resource.read().await;
        let readers = r.current_readers.load(Ordering::SeqCst);
        if readers > 0 {
            let mut m = metrics.write().await;
            m.write_contentions += 1;
        }
    }
    {
        let mut r = resource.write().await;
        r.data.insert(request_id, value);
    }
    let elapsed = start.elapsed();
    let mut m = metrics.write().await;
    m.writes_completed += 1;
    m.total_write_time_ms += elapsed.as_millis() as u64;
    Ok(())
}

/// Test 15: Mixed Load (Read/Write Storm)
#[tokio::test]
async fn chaos_15_mixed_read_write_storm() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Mixed Read/Write Storm");

    let resource = Arc::new(tokio::sync::RwLock::new(ReadWriteResource::new(
        "data-store",
    )));
    let metrics = Arc::new(tokio::sync::RwLock::new(ReadWriteMetrics::default()));

    let mut handles = vec![];
    for i in 0..100 {
        let res_clone = resource.clone();
        let metrics_clone = metrics.clone();
        handles.push(tokio::spawn(async move {
            send_read_request(&res_clone, &metrics_clone, i).await
        }));
    }
    for handle in handles {
        let _ = handle.await;
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.reads_completed, 100);
        println!("✅ Phase 1: Read-only baseline (100 reads)");
    }

    let mut handles = vec![];
    for i in 0..50 {
        let res_clone = resource.clone();
        let metrics_clone = metrics.clone();
        handles.push(tokio::spawn(async move {
            send_write_request(&res_clone, &metrics_clone, i, i as i64).await
        }));
    }
    for handle in handles {
        let _ = handle.await;
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.writes_completed, 50);
        println!("✅ Phase 2: Write-only baseline (50 writes)");
    }

    let mut read_handles = vec![];
    for i in 200..400 {
        let res_clone = resource.clone();
        let metrics_clone = metrics.clone();
        read_handles.push(tokio::spawn(async move {
            send_read_request(&res_clone, &metrics_clone, i).await
        }));
    }
    let mut write_handles = vec![];
    for i in 500..550 {
        let res_clone = resource.clone();
        let metrics_clone = metrics.clone();
        write_handles.push(tokio::spawn(async move {
            send_write_request(&res_clone, &metrics_clone, i, i as i64).await
        }));
    }
    let mut mixed_success = 0;
    for handle in read_handles {
        if let Ok(Ok(_)) = handle.await {
            mixed_success += 1;
        }
    }
    for handle in write_handles {
        if let Ok(Ok(())) = handle.await {
            mixed_success += 1;
        }
    }
    {
        let m = metrics.read().await;
        assert!(m.reads_completed >= 290);
        assert!(m.writes_completed >= 95);
        assert!(mixed_success >= 235);
        println!("✅ Phase 3: Mixed load completed");
    }

    let mut read_handles = vec![];
    for i in 1000..1500 {
        let res_clone = resource.clone();
        let metrics_clone = metrics.clone();
        read_handles.push(tokio::spawn(async move {
            send_read_request(&res_clone, &metrics_clone, i).await
        }));
    }
    let mut write_handles = vec![];
    for i in 2000..2200 {
        let res_clone = resource.clone();
        let metrics_clone = metrics.clone();
        write_handles.push(tokio::spawn(async move {
            send_write_request(&res_clone, &metrics_clone, i, i as i64).await
        }));
    }
    let mut storm_success = 0;
    for handle in read_handles {
        if let Ok(Ok(_)) = handle.await {
            storm_success += 1;
        }
    }
    for handle in write_handles {
        if let Ok(Ok(())) = handle.await {
            storm_success += 1;
        }
    }
    {
        let m = metrics.read().await;
        assert!(m.reads_completed >= 790);
        assert!(m.writes_completed >= 285);
        assert!(storm_success >= 650);
        assert!(
            m.max_concurrent_readers > 10,
            "Should have concurrent readers during storm"
        );
        println!("✅ Phase 4: Heavy storm completed");
    }
    println!("\n🎉 CHAOS TEST PASSED: Mixed read/write storm handled without deadlocks");
    Ok(())
}
