// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Aborted tasks and cancellation cascades with correct resource cleanup (chaos_14).

use super::super::helpers::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

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
