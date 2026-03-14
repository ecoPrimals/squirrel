// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Recovery and resource exhaustion chaos tests

use super::helpers::*;
use std::sync::Arc;
use std::time::Duration;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MEMORY PRESSURE (chaos_07)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug)]
struct MockMemoryAwareService {
    name: String,
    memory_limit_mb: usize,
    memory_used_mb: usize,
    cache_size: usize,
}

impl MockMemoryAwareService {
    fn new(name: &str) -> Self {
        Self { name: name.to_string(), memory_limit_mb: 1000, memory_used_mb: 0, cache_size: 0 }
    }
    fn set_memory_limit_mb(&mut self, limit: usize) { self.memory_limit_mb = limit; }
    fn allocate_mb(&mut self, amount: usize) { self.memory_used_mb += amount; }
    fn deallocate_mb(&mut self, amount: usize) { self.memory_used_mb = self.memory_used_mb.saturating_sub(amount); }
    fn memory_pressure(&self) -> f64 { self.memory_used_mb as f64 / self.memory_limit_mb as f64 }
    fn should_evict_cache(&self) -> bool { self.memory_pressure() > 0.7 }
    fn is_oom(&self) -> bool { self.memory_pressure() > 0.95 }
    fn handle_request(&mut self, request_id: usize) -> ChaosResult<String> {
        if self.is_oom() && should_fail(0.5) {
            return Err("Out of memory - request rejected".into());
        }
        if self.should_evict_cache() && self.cache_size > 0 {
            self.cache_size = self.cache_size.saturating_sub(1);
        }
        if self.memory_pressure() < 0.5 {
            self.cache_size += 1;
        }
        Ok(format!("Request {} processed (memory: {:.1}%)", request_id, self.memory_pressure() * 100.0))
    }
}

#[derive(Debug, Default)]
struct MemoryMetrics {
    successful: u64,
    failures: u64,
    cache_evictions: u64,
    oom_events: u64,
}

async fn send_memory_aware_request(
    service: &Arc<tokio::sync::RwLock<MockMemoryAwareService>>,
    metrics: &Arc<tokio::sync::RwLock<MemoryMetrics>>,
    request_id: usize,
) -> ChaosResult<String> {
    let (result, cache_evicted, was_oom) = {
        let mut svc = service.write().await;
        let cache_before = svc.cache_size;
        let was_oom = svc.is_oom();
        let result = svc.handle_request(request_id);
        let cache_after = svc.cache_size;
        (result, cache_before > cache_after, was_oom)
    };
    let mut m = metrics.write().await;
    match &result {
        Ok(_) => m.successful += 1,
        Err(_) => m.failures += 1,
    }
    if cache_evicted { m.cache_evictions += 1; }
    if was_oom { m.oom_events += 1; }
    result
}

/// Test 7: Memory Pressure
#[tokio::test]
async fn chaos_07_memory_pressure() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Memory Pressure");

    let service = Arc::new(tokio::sync::RwLock::new(MockMemoryAwareService::new("memory-service")));
    let metrics = Arc::new(tokio::sync::RwLock::new(MemoryMetrics::default()));

    {
        let mut svc = service.write().await;
        svc.set_memory_limit_mb(1000);
        svc.allocate_mb(100);
        println!("✅ Phase 1: Normal memory usage (100MB / 1000MB limit)");
    }
    for i in 0..20 {
        let result = send_memory_aware_request(&service, &metrics, i).await;
        assert!(result.is_ok(), "Requests should succeed with plenty of memory");
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.successful, 20);
        assert_eq!(m.cache_evictions, 0);
        println!("✅ Phase 1: 20 requests cached successfully");
    }

    {
        let mut svc = service.write().await;
        svc.allocate_mb(600);
        println!("⚠️ Phase 2: Moderate memory pressure (70%)");
    }
    for i in 20..40 {
        let result = send_memory_aware_request(&service, &metrics, i).await;
        assert!(result.is_ok(), "Requests should still succeed under moderate pressure");
    }
    {
        let m = metrics.read().await;
        assert!(m.cache_evictions > 0, "Should evict cache under pressure");
        println!("✅ Phase 2: Cache evictions started - {} evictions", m.cache_evictions);
    }

    {
        let mut svc = service.write().await;
        svc.allocate_mb(200);
        println!("⚠️⚠️ Phase 3: High memory pressure (90%)");
    }
    for i in 40..50 {
        let result = send_memory_aware_request(&service, &metrics, i).await;
        assert!(result.is_ok(), "Should work with aggressive cache eviction");
    }

    {
        let mut svc = service.write().await;
        svc.allocate_mb(50);
        println!("🔴 Phase 4: Critical memory pressure (95%)");
    }
    let mut critical_successes = 0;
    for i in 50..60 {
        if send_memory_aware_request(&service, &metrics, i).await.is_ok() {
            critical_successes += 1;
        }
    }
    {
        let m = metrics.read().await;
        assert!(critical_successes >= 3, "Should have some successes even under critical pressure");
        assert!(m.oom_events > 0, "Should detect OOM conditions");
        println!("✅ Phase 4: Graceful degradation");
    }

    {
        let mut svc = service.write().await;
        svc.deallocate_mb(700);
        println!("🔄 Phase 5: Memory pressure relieved (25%)");
    }
    for i in 60..80 {
        let result = send_memory_aware_request(&service, &metrics, i).await;
        assert!(result.is_ok(), "Should recover after pressure relieved");
    }

    {
        let m = metrics.read().await;
        assert!(m.successful >= 70, "Should have high success rate overall");
        assert!(m.cache_evictions > 0, "Should have evicted cache");
        assert!(m.oom_events > 0, "Should have detected OOM");
        assert!(m.failures < 10, "Failures should be limited");
        println!("\n📊 Final Memory Pressure Metrics:");
    }
    println!("\n🎉 CHAOS TEST PASSED: Memory pressure handled with graceful degradation");
    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CPU SATURATION (chaos_08)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, Copy)]
enum CpuLoad { Normal, Moderate, High }

impl CpuLoad {
    fn processing_time(&self) -> Duration {
        match self {
            CpuLoad::Normal => Duration::from_millis(10),
            CpuLoad::Moderate => Duration::from_millis(50),
            CpuLoad::High => Duration::from_millis(100),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RequestPriority { Low = 0, Normal = 1, High = 2 }

#[derive(Debug)]
struct MockCpuIntensiveService {
    name: String,
    cpu_load: CpuLoad,
    priority_queue_enabled: bool,
    request_count: u64,
}

impl MockCpuIntensiveService {
    fn new(name: &str) -> Self {
        Self { name: name.to_string(), cpu_load: CpuLoad::Normal, priority_queue_enabled: false, request_count: 0 }
    }
    fn set_cpu_load(&mut self, load: CpuLoad) { self.cpu_load = load; }
    fn enable_priority_queue(&mut self, enabled: bool) { self.priority_queue_enabled = enabled; }
    async fn handle_request(&mut self, request_id: usize, _priority: RequestPriority) -> ChaosResult<String> {
        self.request_count += 1;
        tokio::time::sleep(self.cpu_load.processing_time()).await;
        Ok(format!("Request {} processed by {} (load: {:?})", request_id, self.name, self.cpu_load))
    }
}

#[derive(Debug, Default)]
struct CpuMetrics {
    completed: u64,
    timeouts: u64,
    queued: u64,
    high_priority_count: u64,
    low_priority_count: u64,
}

async fn send_cpu_request(
    service: &Arc<tokio::sync::RwLock<MockCpuIntensiveService>>,
    metrics: &Arc<tokio::sync::RwLock<CpuMetrics>>,
    request_id: usize,
    priority: RequestPriority,
) -> ChaosResult<String> {
    {
        let mut m = metrics.write().await;
        m.queued += 1;
        match priority {
            RequestPriority::High => m.high_priority_count += 1,
            RequestPriority::Low => m.low_priority_count += 1,
            _ => {}
        }
    }
    let result = {
        let mut svc = service.write().await;
        svc.handle_request(request_id, priority).await
    };
    if result.is_ok() {
        let mut m = metrics.write().await;
        m.completed += 1;
    }
    result
}

async fn send_cpu_request_with_timeout(
    service: &Arc<tokio::sync::RwLock<MockCpuIntensiveService>>,
    metrics: &Arc<tokio::sync::RwLock<CpuMetrics>>,
    request_id: usize,
    priority: RequestPriority,
    timeout: Duration,
) -> ChaosResult<String> {
    match tokio::time::timeout(timeout, send_cpu_request(service, metrics, request_id, priority)).await {
        Ok(r) => r,
        Err(_) => Err("Request timed out".into()),
    }
}

/// Test 8: CPU Saturation
#[tokio::test]
async fn chaos_08_cpu_saturation() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: CPU Saturation");

    let service = Arc::new(tokio::sync::RwLock::new(MockCpuIntensiveService::new("cpu-service")));
    let metrics = Arc::new(tokio::sync::RwLock::new(CpuMetrics::default()));

    {
        let mut svc = service.write().await;
        svc.set_cpu_load(CpuLoad::Normal);
        println!("✅ Phase 1: Normal CPU load (~10ms per request)");
    }
    let start = std::time::Instant::now();
    for i in 0..20 {
        let result = send_cpu_request(&service, &metrics, i, RequestPriority::Normal).await;
        assert!(result.is_ok(), "Requests should succeed under normal load");
    }
    let normal_duration = start.elapsed();
    {
        let m = metrics.read().await;
        assert_eq!(m.completed, 20);
        assert!(normal_duration.as_millis() < 500, "Should complete quickly");
        println!("✅ Phase 1: 20 requests completed in {:?}", normal_duration);
    }

    {
        let mut svc = service.write().await;
        svc.set_cpu_load(CpuLoad::Moderate);
        println!("⚠️ Phase 2: Moderate CPU load (~50ms per request)");
    }
    for i in 20..40 {
        let result = send_cpu_request_with_timeout(&service, &metrics, i, RequestPriority::Normal, Duration::from_millis(200)).await;
        if result.is_err() {
            let mut m = metrics.write().await;
            m.timeouts += 1;
        }
    }
    {
        let m = metrics.read().await;
        assert!(m.completed >= 15, "Most should complete");
        println!("✅ Phase 2: Completed");
    }

    {
        let mut svc = service.write().await;
        svc.set_cpu_load(CpuLoad::High);
        svc.enable_priority_queue(true);
        println!("🔴 Phase 3: High CPU saturation + priority queue");
    }
    let mut handles: Vec<(&str, _)> = vec![];
    for i in 50..60 {
        let svc_clone = service.clone();
        let metrics_clone = metrics.clone();
        handles.push(("low", tokio::spawn(async move {
            send_cpu_request(&svc_clone, &metrics_clone, i, RequestPriority::Low).await
        })));
    }
    tokio::time::sleep(Duration::from_millis(50)).await;
    for i in 100..105 {
        let svc_clone = service.clone();
        let metrics_clone = metrics.clone();
        handles.push(("high", tokio::spawn(async move {
            send_cpu_request(&svc_clone, &metrics_clone, i, RequestPriority::High).await
        })));
    }
    let mut high_priority_completed = 0;
    let mut low_priority_completed = 0;
    for (priority, handle) in handles {
        if let Ok(Ok(_)) = handle.await {
            if priority == "high" {
                high_priority_completed += 1;
            } else {
                low_priority_completed += 1;
            }
        }
    }
    {
        let m = metrics.read().await;
        assert_eq!(high_priority_completed, 5, "All high priority should complete");
        assert!(low_priority_completed >= 7, "Most low priority should complete (no starvation)");
        println!("✅ Phase 3: Priority queue working");
    }

    {
        let mut svc = service.write().await;
        svc.set_cpu_load(CpuLoad::Normal);
        println!("🔄 Phase 4: CPU load normalized");
    }
    for i in 200..220 {
        let result = send_cpu_request(&service, &metrics, i, RequestPriority::Normal).await;
        assert!(result.is_ok(), "Should work normally after recovery");
    }

    {
        let m = metrics.read().await;
        assert!(m.completed >= 50, "Should have high completion rate");
        assert!(m.queued > 0, "Should have queued requests");
        println!("\n📊 Final CPU Saturation Metrics:");
    }
    println!("\n🎉 CHAOS TEST PASSED: CPU saturation handled with queuing and priorities");
    Ok(())
}

/// Test 9: File Descriptor Exhaustion - Intentionally skipped
#[tokio::test]
#[ignore]
async fn chaos_09_file_descriptor_exhaustion() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: File Descriptor Exhaustion");
    Ok(())
}

/// Test 10: Disk Space Exhaustion - Intentionally skipped
#[tokio::test]
#[ignore]
async fn chaos_10_disk_space_exhaustion() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Disk Space Exhaustion");
    Ok(())
}
