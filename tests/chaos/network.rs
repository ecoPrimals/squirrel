// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Network failure chaos tests

use super::helpers::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Test 4: Network Partition (Split Brain)
#[tokio::test]
async fn chaos_04_network_partition_split_brain() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Network Partition (Split Brain)");

    let zone_a_service = Arc::new(tokio::sync::RwLock::new(MockService::new("zone-a")));
    let zone_b_service = Arc::new(tokio::sync::RwLock::new(MockService::new("zone-b")));
    let network = Arc::new(tokio::sync::RwLock::new(NetworkController::new()));
    let metrics = Arc::new(tokio::sync::RwLock::new(PartitionMetrics::default()));

    {
        let net = network.read().await;
        assert!(net.can_communicate("zone-a", "zone-b"));
        println!("✅ Phase 1: Both zones connected and healthy");
    }

    for i in 0..5 {
        let result = send_cross_zone_request(&zone_a_service, &zone_b_service, &network, &metrics, i).await;
        assert!(result.is_ok(), "Cross-zone requests should succeed");
    }

    {
        let m = metrics.read().await;
        assert_eq!(m.successful_cross_zone, 5);
        println!("✅ Phase 1: 5 cross-zone requests successful");
    }

    {
        let mut net = network.write().await;
        net.partition("zone-a", "zone-b");
        println!("🚧 Phase 2: Network partition created - zones isolated");
    }

    for i in 10..15 {
        let result = send_cross_zone_request(&zone_a_service, &zone_b_service, &network, &metrics, i).await;
        assert!(result.is_err(), "Cross-zone requests should fail during partition");
    }

    {
        let m = metrics.read().await;
        assert!(m.partition_detected >= 1, "Partition should be detected");
        println!("✅ Phase 2: Partition detected - cross-zone communication blocked");
    }

    {
        for i in 20..25 {
            let mut svc_a = zone_a_service.write().await;
            let result = svc_a.handle_request(i);
            assert!(result.is_ok(), "Zone A local requests should work");
        }
        for i in 30..35 {
            let mut svc_b = zone_b_service.write().await;
            let result = svc_b.handle_request(i);
            assert!(result.is_ok(), "Zone B local requests should work");
        }
        let mut m = metrics.write().await;
        m.zone_a_local += 5;
        m.zone_b_local += 5;
        println!("✅ Phase 3: Both zones operating independently");
    }

    {
        let mut net = network.write().await;
        net.heal("zone-a", "zone-b");
        println!("🔄 Phase 4: Network partition healed");
    }

    for i in 40..45 {
        let result = send_cross_zone_request(&zone_a_service, &zone_b_service, &network, &metrics, i).await;
        assert!(result.is_ok(), "Cross-zone requests should succeed after heal");
    }

    {
        let m = metrics.read().await;
        println!("\n📊 Final Partition Metrics:");
        assert!(m.successful_cross_zone >= 10, "Should have cross-zone successes");
        assert!(m.partition_detected >= 1, "Should detect partition");
        assert!(m.zone_a_local >= 5 && m.zone_b_local >= 5, "Zones should operate independently");
    }

    println!("\n🎉 CHAOS TEST PASSED: Network partition handled with independent operation");
    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// INTERMITTENT FAILURES (chaos_05)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug)]
struct MockFlakeyService {
    name: String,
    failure_rate: f64,
    request_count: u64,
}

impl MockFlakeyService {
    fn new(name: &str, failure_rate: f64) -> Self {
        Self { name: name.to_string(), failure_rate, request_count: 0 }
    }
    fn set_failure_rate(&mut self, rate: f64) {
        self.failure_rate = rate;
    }
    fn handle_request(&mut self, request_id: usize) -> ChaosResult<String> {
        self.request_count += 1;
        if should_fail(self.failure_rate) {
            Err(format!("Transient network failure - request {}", request_id).into())
        } else {
            Ok(format!("Request {} processed by {} (attempt {})", request_id, self.name, self.request_count))
        }
    }
}

#[derive(Debug, Default)]
struct IntermittentMetrics {
    successful: u64,
    total_attempts: u64,
    transient_failures: u64,
    permanent_failures: u64,
    total_backoff_ms: u64,
}

async fn send_flakey_request(
    service: &Arc<tokio::sync::RwLock<MockFlakeyService>>,
    metrics: &Arc<tokio::sync::RwLock<IntermittentMetrics>>,
    request_id: usize,
    max_retries: usize,
) -> ChaosResult<String> {
    let mut backoff = Duration::from_millis(10);
    for attempt in 0..=max_retries {
        {
            let mut m = metrics.write().await;
            m.total_attempts += 1;
        }
        let result = {
            let mut svc = service.write().await;
            svc.handle_request(request_id)
        };
        match result {
            Ok(response) => {
                let mut m = metrics.write().await;
                m.successful += 1;
                return Ok(response);
            }
            Err(e) => {
                {
                    let mut m = metrics.write().await;
                    m.transient_failures += 1;
                }
                if attempt < max_retries {
                    tokio::time::sleep(backoff).await;
                    {
                        let mut m = metrics.write().await;
                        m.total_backoff_ms += backoff.as_millis() as u64;
                    }
                    backoff = backoff.saturating_mul(2).min(Duration::from_millis(1000));
                } else {
                    let mut m = metrics.write().await;
                    m.permanent_failures += 1;
                    return Err(format!("Permanent failure after {} retries: {}", max_retries, e).into());
                }
            }
        }
    }
    unreachable!()
}

/// Test 5: Intermittent Network Failures
#[tokio::test]
async fn chaos_05_intermittent_network_failures() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: Intermittent Network Failures");

    let service = Arc::new(tokio::sync::RwLock::new(MockFlakeyService::new("flakey-service", 0.3)));
    let metrics = Arc::new(tokio::sync::RwLock::new(IntermittentMetrics::default()));

    {
        let mut svc = service.write().await;
        svc.set_failure_rate(0.0);
        println!("✅ Phase 1: Baseline - no failures");
    }
    for i in 0..10 {
        let result = send_flakey_request(&service, &metrics, i, 3).await;
        assert!(result.is_ok(), "Baseline requests should succeed");
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.successful, 10);
        println!("✅ Phase 1: {} requests succeeded without retries", m.successful);
    }

    {
        let mut svc = service.write().await;
        svc.set_failure_rate(0.3);
        println!("🌩️ Phase 2: Injected 30% failure rate (flaky network)");
    }
    for i in 10..60 {
        let result = send_flakey_request(&service, &metrics, i, 5).await;
        assert!(result.is_ok(), "Request {} should succeed after retries", i);
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.successful, 60, "All requests should eventually succeed");
        assert!(m.transient_failures > 0, "Should have encountered transient failures");
        let retry_ratio = (m.total_attempts - m.successful) as f64 / m.successful as f64;
        assert!(retry_ratio < 2.0, "Retry ratio should be reasonable (< 2x)");
        println!("✅ Phase 2: Flaky network handled");
    }

    {
        let mut svc = service.write().await;
        svc.set_failure_rate(0.6);
        println!("⚠️ Phase 3: Increased to 60% failure rate (severe flakiness)");
    }
    let mut severe_successes = 0;
    for i in 100..120 {
        if send_flakey_request(&service, &metrics, i, 5).await.is_ok() {
            severe_successes += 1;
        }
    }
    assert!(severe_successes >= 15, "At least 75% should succeed with retries");
    println!("✅ Phase 3: Severe flakiness handled");

    {
        let mut svc = service.write().await;
        svc.set_failure_rate(0.1);
        println!("🔄 Phase 4: Reduced to 10% failure rate (realistic)");
    }
    for i in 200..230 {
        let result = send_flakey_request(&service, &metrics, i, 3).await;
        assert!(result.is_ok(), "Realistic failures should be handled");
    }

    {
        let m = metrics.read().await;
        assert!(m.total_backoff_ms < 60000, "Total backoff should be < 60s");
        println!("\n📊 Final Intermittent Failure Metrics:");
    }
    println!("\n🎉 CHAOS TEST PASSED: Intermittent failures handled with retry logic");
    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DNS RESOLUTION (chaos_06)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug)]
struct MockDnsResolver {
    records: HashMap<String, String>,
    cache: HashMap<String, String>,
    failure_mode: bool,
}

impl MockDnsResolver {
    fn new() -> Self {
        Self { records: HashMap::new(), cache: HashMap::new(), failure_mode: false }
    }
    fn register(&mut self, hostname: &str, ip: &str) {
        self.records.insert(hostname.to_string(), ip.to_string());
    }
    fn set_failure_mode(&mut self, fail: bool) {
        self.failure_mode = fail;
    }
    fn expire_cache(&mut self) {
        self.cache.clear();
    }
    fn resolve(&mut self, hostname: &str) -> ChaosResult<String> {
        if let Some(ip) = self.cache.get(hostname) {
            return Ok(ip.clone());
        }
        if self.failure_mode {
            return Err(format!("DNS resolution failed for {}", hostname).into());
        }
        if let Some(ip) = self.records.get(hostname) {
            self.cache.insert(hostname.to_string(), ip.clone());
            Ok(ip.clone())
        } else {
            Err(format!("Hostname not found: {}", hostname).into())
        }
    }
}

#[derive(Debug, Default)]
struct DnsMetrics {
    cache_hits: u64,
    cache_misses: u64,
    dns_queries: u64,
    dns_failures: u64,
    ip_fallbacks: u64,
}

async fn resolve_with_cache(
    resolver: &Arc<tokio::sync::RwLock<MockDnsResolver>>,
    metrics: &Arc<tokio::sync::RwLock<DnsMetrics>>,
    hostname: &str,
) -> ChaosResult<String> {
    let mut dns = resolver.write().await;
    let mut m = metrics.write().await;
    let in_cache = dns.cache.contains_key(hostname);
    let result = dns.resolve(hostname);
    match &result {
        Ok(_) => {
            if in_cache {
                m.cache_hits += 1;
            } else {
                m.cache_misses += 1;
                m.dns_queries += 1;
            }
        }
        Err(_) => m.dns_failures += 1,
    }
    result
}

async fn resolve_ip_directly(metrics: &Arc<tokio::sync::RwLock<DnsMetrics>>, ip: &str) -> ChaosResult<String> {
    let mut m = metrics.write().await;
    m.ip_fallbacks += 1;
    Ok(ip.to_string())
}

/// Test 6: DNS Resolution Failures
#[tokio::test]
async fn chaos_06_dns_resolution_failures() -> ChaosResult<()> {
    println!("🔥 CHAOS TEST: DNS Resolution Failures");

    let resolver = Arc::new(tokio::sync::RwLock::new(MockDnsResolver::new()));
    let metrics = Arc::new(tokio::sync::RwLock::new(DnsMetrics::default()));

    {
        let mut dns = resolver.write().await;
        dns.register("service-a.local", "192.168.1.10");
        dns.register("service-b.local", "192.168.1.20");
        dns.register("service-c.local", "192.168.1.30");
        println!("✅ Phase 1: DNS configured with 3 services");
    }
    for hostname in &["service-a.local", "service-b.local", "service-c.local"] {
        let result = resolve_with_cache(&resolver, &metrics, hostname).await;
        assert!(result.is_ok(), "DNS should resolve {}", hostname);
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.cache_hits, 0);
        assert_eq!(m.cache_misses, 3);
        assert_eq!(m.dns_queries, 3);
        println!("✅ Phase 1: {} DNS queries successful", m.dns_queries);
    }

    for _ in 0..5 {
        for hostname in &["service-a.local", "service-b.local"] {
            let result = resolve_with_cache(&resolver, &metrics, hostname).await;
            assert!(result.is_ok(), "Cached lookups should succeed");
        }
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.cache_hits, 10);
        assert_eq!(m.dns_queries, 3);
        println!("✅ Phase 2: {} cache hits (DNS cache working)", m.cache_hits);
    }

    {
        let mut dns = resolver.write().await;
        dns.set_failure_mode(true);
        println!("💥 Phase 3: DNS resolution broken");
    }
    for hostname in &["service-a.local", "service-b.local", "service-c.local"] {
        let result = resolve_with_cache(&resolver, &metrics, hostname).await;
        assert!(result.is_ok(), "Cached entries should work despite DNS failure");
    }
    {
        let m = metrics.read().await;
        assert_eq!(m.cache_hits, 13);
        assert_eq!(m.dns_failures, 0);
        println!("✅ Phase 3: Cache protected against DNS failure");
    }

    let new_hostname = "service-d.local";
    let result = resolve_with_cache(&resolver, &metrics, new_hostname).await;
    assert!(result.is_err(), "New lookups should fail when DNS is broken");
    {
        let m = metrics.read().await;
        assert_eq!(m.dns_failures, 1);
        println!("✅ Phase 4: DNS failure detected for uncached lookup");
    }

    let ip_address = "192.168.1.40";
    let result = resolve_ip_directly(&metrics, ip_address).await;
    assert!(result.is_ok(), "IP addresses should work without DNS");
    {
        let m = metrics.read().await;
        assert_eq!(m.ip_fallbacks, 1);
        println!("✅ Phase 5: IP fallback working");
    }

    {
        let mut dns = resolver.write().await;
        dns.set_failure_mode(false);
        dns.register("service-d.local", "192.168.1.40");
        println!("🔄 Phase 6: DNS restored");
    }
    let result = resolve_with_cache(&resolver, &metrics, "service-d.local").await;
    assert!(result.is_ok(), "DNS should work after restoration");

    {
        let mut dns = resolver.write().await;
        dns.expire_cache();
        println!("⏱️ Phase 7: Cache expired");
    }
    for hostname in &["service-a.local", "service-b.local"] {
        let result = resolve_with_cache(&resolver, &metrics, hostname).await;
        assert!(result.is_ok(), "Should re-resolve after cache expiration");
    }
    {
        let m = metrics.read().await;
        assert!(m.dns_queries > 3, "Should have made new DNS queries after expiration");
        assert!(m.cache_hits > 10, "Should have many cache hits");
        assert_eq!(m.dns_failures, 1, "Should have handled DNS failure");
        assert!(m.ip_fallbacks >= 1, "Should have used IP fallback");
        println!("\n📊 Final DNS Metrics:");
    }
    println!("\n🎉 CHAOS TEST PASSED: DNS failures handled with caching and fallbacks");
    Ok(())
}
