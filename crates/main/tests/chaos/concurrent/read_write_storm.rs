// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Heavy mixed concurrent reads and writes with contention metrics (chaos_15).

use super::super::helpers::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

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
