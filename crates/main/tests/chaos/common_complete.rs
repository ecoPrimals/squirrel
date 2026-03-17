// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Common infrastructure for chaos testing
//!
//! This module provides shared test utilities, mock services, metrics tracking,
//! and helper functions used across all chaos tests.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Chaos test result type
pub type ChaosResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CONFIGURATION
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Configuration for chaos tests
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Test duration
    pub duration: Duration,
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f64,
    /// Number of concurrent clients
    pub num_clients: usize,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(60),
            failure_rate: 0.1,
            num_clients: 100,
            timeout: Duration::from_secs(10),
        }
    }
}

/// Chaos test metrics
#[derive(Debug, Default)]
pub struct ChaosMetrics {
    /// Total requests sent
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Timed out requests
    pub timeout_requests: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// P95 response time (ms)
    pub p95_response_time_ms: f64,
    /// P99 response time (ms)
    pub p99_response_time_ms: f64,
}

impl ChaosMetrics {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.failed_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Print metrics summary
    pub fn print_summary(&self) {
        println!("\n📊 CHAOS TEST METRICS:");
        println!("  Total Requests:    {}", self.total_requests);
        println!(
            "  Successful:        {} ({:.2}%)",
            self.successful_requests,
            self.success_rate()
        );
        println!(
            "  Failed:            {} ({:.2}%)",
            self.failed_requests,
            self.failure_rate()
        );
        println!("  Timed Out:         {}", self.timeout_requests);
        println!("  Avg Response:      {:.2}ms", self.avg_response_time_ms);
        println!("  P95 Response:      {:.2}ms", self.p95_response_time_ms);
        println!("  P99 Response:      {:.2}ms", self.p99_response_time_ms);
    }
}

/// Simulate random failures based on failure rate
pub fn should_fail(failure_rate: f64) -> bool {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.r#gen::<f64>() < failure_rate
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MOCK SERVICES
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceState {
    Healthy,
    Crashed,
    Recovering,
}

/// Mock service that can crash and recover
#[derive(Debug)]
pub struct MockService {
    name: String,
    state: ServiceState,
    request_count: u64,
}

impl MockService {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            state: ServiceState::Healthy,
            request_count: 0,
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.state, ServiceState::Healthy)
    }

    pub fn crash(&mut self) {
        self.state = ServiceState::Crashed;
    }

    pub fn recover(&mut self) {
        self.state = ServiceState::Healthy;
    }

    pub fn handle_request(
        &mut self,
        request_id: usize,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.state {
            ServiceState::Healthy => {
                self.request_count += 1;
                Ok(format!("Request {} processed by {}", request_id, self.name))
            }
            ServiceState::Crashed => Err("service unavailable - crashed".into()),
            ServiceState::Recovering => {
                // Simulate gradual recovery
                if self.request_count % 3 == 0 {
                    self.state = ServiceState::Healthy;
                    self.request_count += 1;
                    Ok(format!("Request {} processed after recovery", request_id))
                } else {
                    Err("service still recovering".into())
                }
            }
        }
    }
}

/// Mock service with configurable latency
#[derive(Debug)]
pub struct MockServiceWithLatency {
    name: String,
    latency: Duration,
    fallback_enabled: bool,
}

impl MockServiceWithLatency {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            latency: Duration::from_millis(10),
            fallback_enabled: false,
        }
    }

    pub fn set_latency(&mut self, latency: Duration) {
        self.latency = latency;
    }

    pub fn enable_fallback(&mut self, enabled: bool) {
        self.fallback_enabled = enabled;
    }

    pub async fn handle_request(
        &self,
        request_id: usize,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        tokio::time::sleep(self.latency).await;
        Ok(format!(
            "Request {} processed by {} (latency: {:?})",
            request_id, self.name, self.latency
        ))
    }

    pub fn handle_fallback(
        &self,
        request_id: usize,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(format!(
            "Request {} served from cache (fallback)",
            request_id
        ))
    }
}

/// Flaky service that randomly fails based on failure rate
#[derive(Debug)]
pub struct FlakyService {
    name: String,
    failure_rate: f64,
}

impl FlakyService {
    pub fn new(name: &str, failure_rate: f64) -> Self {
        Self {
            name: name.to_string(),
            failure_rate,
        }
    }

    pub fn set_failure_rate(&mut self, rate: f64) {
        self.failure_rate = rate;
    }

    pub fn handle_request(
        &self,
        request_id: usize,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if should_fail(self.failure_rate) {
            Err(format!("Network error - packet lost (request {})", request_id).into())
        } else {
            Ok(format!("Request {} processed by {}", request_id, self.name))
        }
    }
}

/// CPU-bound service with request queue
#[derive(Debug)]
pub struct CPUBoundService {
    queue_size: usize,
    active_requests: usize,
    cpu_intensive: bool,
}

impl CPUBoundService {
    pub fn new(queue_size: usize) -> Self {
        Self {
            queue_size,
            active_requests: 0,
            cpu_intensive: false,
        }
    }

    pub fn set_cpu_intensive(&mut self, intensive: bool) {
        self.cpu_intensive = intensive;
    }

    pub async fn process_request(
        &mut self,
        request_id: usize,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if self.active_requests >= self.queue_size {
            return Err("Queue full - request rejected".into());
        }

        self.active_requests += 1;

        // Simulate CPU-intensive work
        if self.cpu_intensive {
            tokio::time::sleep(Duration::from_millis(100)).await;
        } else {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        self.active_requests -= 1;
        Ok(format!("Request {} processed", request_id))
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// METRICS TRACKING
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Default)]
pub struct ServiceMetrics {
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_response_time_ms: f64,
    pub avg_response_time_ms: f64,
}

impl ServiceMetrics {
    pub fn record_success(&mut self, response_time_ms: f64) {
        self.successful_requests += 1;
        self.total_response_time_ms += response_time_ms;
        self.avg_response_time_ms = self.total_response_time_ms / self.successful_requests as f64;
    }

    pub fn record_failure(&mut self) {
        self.failed_requests += 1;
    }
}

#[derive(Debug, Default)]
pub struct CascadeMetrics {
    pub c_success: u64,
    pub c_failures: u64,
    pub cascade_prevented: u64,
}

#[derive(Debug, Default)]
pub struct LatencyMetrics {
    pub successful: u64,
    pub timeouts: u64,
    pub fallbacks: u64,
    pub total_latency_ms: f64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Default)]
pub struct PartitionMetrics {
    pub successful_cross_zone: u64,
    pub partition_detected: u64,
    pub zone_a_local: u64,
    pub zone_b_local: u64,
    pub reconciliations: u64,
}

#[derive(Debug, Default)]
pub struct RetryMetrics {
    pub successful: u64,
    pub retries: u64,
    pub total_attempts: u64,
    pub backoff_count: u64,
}

#[derive(Debug, Default)]
pub struct DNSMetrics {
    pub successful_connections: u64,
    pub dns_hits: u64,
    pub cache_hits: u64,
    pub dns_failures: u64,
    pub ip_fallbacks: u64,
}

#[derive(Debug, Default)]
pub struct MemoryMetrics {
    pub cache_entries: u64,
    pub evictions: u64,
    pub pressure_evictions: u64,
    pub memory_saved_bytes: u64,
}

#[derive(Debug, Default)]
pub struct CPUMetrics {
    pub processed: u64,
    pub queued: u64,
    pub queue_full: u64,
    pub timeouts: u64,
    pub total_processing_ms: f64,
    pub avg_processing_ms: f64,
}

#[derive(Debug, Default)]
pub struct FDMetrics {
    pub acquired: u64,
    pub fd_exhausted: u64,
    pub connections_reused: u64,
    pub cleanup_events: u64,
}

#[derive(Debug, Default)]
pub struct DiskMetrics {
    pub writes_succeeded: u64,
    pub writes_failed: u64,
    pub disk_full_errors: u64,
    pub cleanup_triggered: u64,
    pub space_freed: u64,
    pub critical_writes: u64,
    pub normal_writes_rejected: u64,
}

#[derive(Debug, Default)]
pub struct HerdMetrics {
    pub processed: u64,
    pub rate_limited: u64,
    pub queued: u64,
    pub queue_full: u64,
}

#[derive(Debug, Default)]
pub struct LoadMetrics {
    pub long_ops_completed: u64,
    pub short_ops_completed: u64,
    pub total_short_op_ms: f64,
    pub avg_short_op_ms: f64,
}

#[derive(Debug, Default)]
pub struct RaceMetrics {
    pub writes_completed: u64,
    pub reads_completed: u64,
    pub lock_contentions: u64,
}

#[derive(Debug, Default)]
pub struct CancellationMetrics {
    pub started: u64,
    pub completed: u64,
    pub cancelled: u64,
}

#[derive(Debug, Default)]
pub struct StormMetrics {
    pub reads: u64,
    pub writes: u64,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SPECIALIZED INFRASTRUCTURE
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Network controller for partition simulation
#[derive(Debug)]
pub struct NetworkController {
    partitioned: bool,
}

impl NetworkController {
    pub fn new() -> Self {
        Self { partitioned: false }
    }

    pub fn can_communicate(&self, _zone_a: &str, _zone_b: &str) -> bool {
        !self.partitioned
    }

    pub fn partition(&mut self, _zone_a: &str, _zone_b: &str) {
        self.partitioned = true;
    }

    pub fn heal(&mut self, _zone_a: &str, _zone_b: &str) {
        self.partitioned = false;
    }
}

/// Mock DNS resolver with cache
#[derive(Debug)]
pub struct MockDNSResolver {
    records: HashMap<String, String>,
    cache: HashMap<String, String>,
    dns_working: bool,
}

impl MockDNSResolver {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            cache: HashMap::new(),
            dns_working: true,
        }
    }

    pub fn register(&mut self, hostname: &str, ip: &str) {
        self.records.insert(hostname.to_string(), ip.to_string());
    }

    pub fn break_dns(&mut self) {
        self.dns_working = false;
    }

    pub fn restore_dns(&mut self) {
        self.dns_working = true;
    }

    pub fn resolve(
        &mut self,
        hostname: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check cache first
        if let Some(ip) = self.cache.get(hostname) {
            return Ok(ip.clone());
        }

        // Try DNS if working
        if self.dns_working {
            if let Some(ip) = self.records.get(hostname) {
                // Add to cache
                self.cache.insert(hostname.to_string(), ip.clone());
                return Ok(ip.clone());
            }
        }

        Err(format!("DNS resolution failed for {}", hostname).into())
    }
}

/// Memory-aware cache with LRU eviction
#[derive(Debug)]
pub struct MemoryAwareCache {
    entries: HashMap<usize, Vec<u8>>,
    lru: VecDeque<usize>,
    max_size: usize,
    memory_pressure: bool,
}

impl MemoryAwareCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            lru: VecDeque::new(),
            max_size,
            memory_pressure: false,
        }
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }

    pub fn set_memory_pressure(&mut self, pressure: bool) {
        self.memory_pressure = pressure;
    }

    pub fn insert(&mut self, key: usize, value: Vec<u8>) -> Option<Vec<u8>> {
        // Evict if at capacity
        if self.entries.len() >= self.max_size {
            if let Some(old_key) = self.lru.pop_front() {
                self.entries.remove(&old_key);
            }
        }

        // Update LRU
        self.lru.retain(|k| *k != key);
        self.lru.push_back(key);

        self.entries.insert(key, value)
    }

    pub fn cleanup_under_pressure(&mut self) {
        if self.memory_pressure {
            // Evict 75% of entries under pressure
            let target_size = self.max_size / 4;
            while self.entries.len() > target_size {
                if let Some(key) = self.lru.pop_front() {
                    self.entries.remove(&key);
                }
            }
        }
    }
}

/// Connection pool with FD limits
#[derive(Debug)]
pub struct ConnectionPool {
    connections: Vec<usize>,
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Vec::new(),
            max_connections,
        }
    }

    pub fn acquire(&mut self, id: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.connections.len() >= self.max_connections {
            return Err("FD exhausted - max connections reached".into());
        }
        self.connections.push(id);
        Ok(())
    }

    pub fn release_oldest(&mut self, count: usize) {
        for _ in 0..count.min(self.connections.len()) {
            self.connections.remove(0);
        }
    }

    pub fn active_count(&self) -> usize {
        self.connections.len()
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections
    }
}

/// Mock storage with disk space simulation
#[derive(Debug)]
pub struct MockStorage {
    data: HashMap<usize, Vec<u8>>,
    used_space: usize,
    total_space: usize,
    critical_only: bool,
}

impl MockStorage {
    pub fn new(total_space: usize) -> Self {
        Self {
            data: HashMap::new(),
            used_space: 0,
            total_space,
            critical_only: false,
        }
    }

    pub fn set_critical_only(&mut self, critical_only: bool) {
        self.critical_only = critical_only;
    }

    pub fn write(
        &mut self,
        key: usize,
        data: Vec<u8>,
        critical: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data_size = data.len();

        if self.used_space + data_size > self.total_space {
            if !critical || self.critical_only {
                return Err("Disk full".into());
            }
        }

        if self.critical_only && !critical {
            return Err("Only critical writes allowed".into());
        }

        self.data.insert(key, data);
        self.used_space += data_size;
        Ok(())
    }

    pub fn cleanup(&mut self, target_keys: usize) -> usize {
        let mut freed = 0;
        let keys_to_remove: Vec<usize> = self.data.keys().take(target_keys).copied().collect();

        for key in keys_to_remove {
            if let Some(data) = self.data.remove(&key) {
                freed += data.len();
            }
        }

        self.used_space = self.used_space.saturating_sub(freed);
        freed
    }

    pub fn used_space(&self) -> usize {
        self.used_space
    }

    pub fn total_space(&self) -> usize {
        self.total_space
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HELPER FUNCTIONS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Send a request to a service
pub async fn send_request(
    service: &Arc<tokio::sync::RwLock<MockService>>,
    metrics: &Arc<tokio::sync::RwLock<ServiceMetrics>>,
    request_id: usize,
) -> ChaosResult<String> {
    let start = Instant::now();

    let result = {
        let mut svc = service.write().await;
        svc.handle_request(request_id)
    };

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    let mut m = metrics.write().await;
    match &result {
        Ok(_) => m.record_success(elapsed_ms),
        Err(_) => m.record_failure(),
    }

    result
}

/// Send a cascade request through service chain
pub async fn send_cascade_request(
    service_a: &Arc<tokio::sync::RwLock<MockService>>,
    service_b: &Arc<tokio::sync::RwLock<MockService>>,
    service_c: &Arc<tokio::sync::RwLock<MockService>>,
    metrics: &Arc<tokio::sync::RwLock<CascadeMetrics>>,
    request_id: usize,
) -> ChaosResult<String> {
    // Try service A
    let a_result = {
        let mut svc_a = service_a.write().await;
        svc_a.handle_request(request_id)
    };

    if a_result.is_err() {
        let mut m = metrics.write().await;
        m.cascade_prevented += 1;
        return Err("Service A failed - cascade prevented".into());
    }

    // Try service B
    let b_result = {
        let mut svc_b = service_b.write().await;
        svc_b.handle_request(request_id)
    };

    if b_result.is_err() {
        let mut m = metrics.write().await;
        m.c_failures += 1;
        return Err("Service B failed".into());
    }

    // Try service C
    let c_result = {
        let mut svc_c = service_c.write().await;
        svc_c.handle_request(request_id)
    };

    let mut m = metrics.write().await;
    match c_result {
        Ok(response) => {
            m.c_success += 1;
            Ok(response)
        }
        Err(e) => {
            m.c_failures += 1;
            Err(e)
        }
    }
}

/// Send request with timeout
pub async fn send_request_with_timeout(
    service: &Arc<tokio::sync::RwLock<MockServiceWithLatency>>,
    metrics: &Arc<tokio::sync::RwLock<LatencyMetrics>>,
    request_id: usize,
    timeout: Duration,
) -> ChaosResult<String> {
    let start = Instant::now();

    let result = {
        let svc = service.read().await;
        tokio::time::timeout(timeout, svc.handle_request(request_id)).await
    };

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    let mut m = metrics.write().await;
    match result {
        Ok(Ok(response)) => {
            m.successful += 1;
            m.total_latency_ms += elapsed_ms;
            m.avg_latency_ms = m.total_latency_ms / m.successful as f64;
            Ok(response)
        }
        Ok(Err(e)) => {
            m.timeouts += 1;
            Err(e)
        }
        Err(_) => {
            m.timeouts += 1;
            Err("Timeout".into())
        }
    }
}

/// Send request with fallback
pub async fn send_request_with_fallback(
    service: &Arc<tokio::sync::RwLock<MockServiceWithLatency>>,
    metrics: &Arc<tokio::sync::RwLock<LatencyMetrics>>,
    request_id: usize,
    timeout: Duration,
) -> ChaosResult<String> {
    let svc = service.read().await;
    let result = tokio::time::timeout(timeout, svc.handle_request(request_id)).await;

    let mut m = metrics.write().await;
    match result {
        Ok(Ok(response)) => {
            m.successful += 1;
            Ok(response)
        }
        _ => {
            m.fallbacks += 1;
            svc.handle_fallback(request_id)
        }
    }
}

/// Send cross-zone request
pub async fn send_cross_zone_request(
    zone_a: &Arc<tokio::sync::RwLock<MockService>>,
    zone_b: &Arc<tokio::sync::RwLock<MockService>>,
    network: &Arc<tokio::sync::RwLock<NetworkController>>,
    metrics: &Arc<tokio::sync::RwLock<PartitionMetrics>>,
    request_id: usize,
) -> ChaosResult<String> {
    // Check if zones can communicate
    let can_communicate = {
        let net = network.read().await;
        net.can_communicate("zone-a", "zone-b")
    };

    if !can_communicate {
        let mut m = metrics.write().await;
        m.partition_detected += 1;
        return Err("Network partition - zones isolated".into());
    }

    // Send request from zone A to zone B
    let result = {
        let mut svc_b = zone_b.write().await;
        svc_b.handle_request(request_id)
    };

    let mut m = metrics.write().await;
    match result {
        Ok(response) => {
            m.successful_cross_zone += 1;
            Ok(response)
        }
        Err(e) => Err(e),
    }
}

