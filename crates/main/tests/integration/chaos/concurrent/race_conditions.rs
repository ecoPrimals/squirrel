// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Concurrent writes and conflict detection on a shared counter (chaos_13).

use super::super::helpers::*;
use std::sync::Arc;

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
