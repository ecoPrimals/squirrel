// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Thundering herd: many concurrent clients against a rate-limited mock service (chaos_11).

use super::super::helpers::*;
use std::sync::Arc;
use std::time::Duration;

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
