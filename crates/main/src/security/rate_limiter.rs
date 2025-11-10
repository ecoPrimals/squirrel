//! # Production Rate Limiting & DoS Protection
//! 
//! This module provides comprehensive rate limiting to protect against:
//! - Denial of Service (DoS) attacks
//! - API abuse and excessive requests
//! - Resource exhaustion attacks
//! - Brute force authentication attempts

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tracing::{warn, error, debug, info};
use serde::{Serialize, Deserialize};

use crate::observability::CorrelationId;

/// Rate limiting configuration for different endpoint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute for general API endpoints
    pub api_requests_per_minute: u32,
    
    /// Requests per minute for authentication endpoints
    pub auth_requests_per_minute: u32,
    
    /// Requests per minute for compute-intensive operations
    pub compute_requests_per_minute: u32,
    
    /// Maximum burst capacity
    pub burst_capacity: u32,
    
    /// Ban duration for repeat offenders
    pub ban_duration: Duration,
    
    /// Threshold for temporary ban (violations in time window)
    pub ban_threshold: u32,
    
    /// Time window for counting violations
    pub violation_window: Duration,
    
    /// Enable adaptive rate limiting based on system load
    pub adaptive_limiting: bool,
    
    /// Whitelist of IPs that bypass rate limiting
    pub whitelist: Vec<IpAddr>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            api_requests_per_minute: 100,
            auth_requests_per_minute: 10,
            compute_requests_per_minute: 20,
            burst_capacity: 150,
            ban_duration: Duration::from_secs(300), // 5 minutes
            ban_threshold: 5,
            violation_window: Duration::from_secs(60),
            adaptive_limiting: true,
            whitelist: vec![
                "127.0.0.1".parse().unwrap(),
                "::1".parse().unwrap(),
            ],
        }
    }
}

/// Rate limit bucket for token bucket algorithm
#[derive(Debug, Clone)]
struct RateLimitBucket {
    /// Current number of tokens
    tokens: f64,
    /// Maximum number of tokens
    capacity: f64,
    /// Token refill rate per second
    refill_rate: f64,
    /// Last refill timestamp
    last_refill: Instant,
    /// Request count in current window
    request_count: u32,
    /// Window start time
    window_start: Instant,
}

impl RateLimitBucket {
    fn new(capacity: u32, refill_rate: u32) -> Self {
        let now = Instant::now();
        Self {
            tokens: capacity as f64,
            capacity: capacity as f64,
            refill_rate: refill_rate as f64 / 60.0, // Convert per minute to per second
            last_refill: now,
            request_count: 0,
            window_start: now,
        }
    }
    
    /// Try to consume a token, returns true if allowed
    fn try_consume(&mut self, tokens_needed: f64) -> bool {
        self.refill_tokens();
        
        if self.tokens >= tokens_needed {
            self.tokens -= tokens_needed;
            self.request_count += 1;
            true
        } else {
            false
        }
    }
    
    /// Refill tokens based on elapsed time
    fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        // Add tokens based on refill rate
        let tokens_to_add = elapsed * self.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
        
        // Reset request count if window expired
        if now.duration_since(self.window_start) >= Duration::from_secs(60) {
            self.request_count = 0;
            self.window_start = now;
        }
    }
}

/// Security violation tracking
#[derive(Debug, Clone)]
struct SecurityViolation {
    timestamp: Instant,
    violation_type: ViolationType,
    severity: ViolationSeverity,
    details: String,
}

#[derive(Debug, Clone, PartialEq)]
enum ViolationType {
    RateLimitExceeded,
    SuspiciousActivity,
    RepeatedViolations,
    MaliciousRequest,
}

#[derive(Debug, Clone, PartialEq)]
enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Client tracking information
#[derive(Debug, Clone)]
struct ClientInfo {
    ip_address: IpAddr,
    user_agent: Option<String>,
    first_seen: Instant,
    last_activity: Instant,
    total_requests: u64,
    violations: Vec<SecurityViolation>,
    is_banned: bool,
    ban_expires_at: Option<Instant>,
}

#[derive(Debug)]
pub struct ClientRequestCounter {
    /// Current number of tokens
    tokens: f64,
    /// Maximum number of tokens
    capacity: f64,
    /// Token refill rate per second
    refill_rate: f64,
    /// Last refill timestamp
    last_refill: Instant,
    /// Request count in current window
    request_count: u32,
    /// Window start time
    window_start: Instant,
}

impl Default for ClientRequestCounter {
    fn default() -> Self {
        Self {
            tokens: 0.0,
            capacity: 0.0,
            refill_rate: 0.0,
            last_refill: Instant::now(),
            request_count: 0,
            window_start: Instant::now(),
        }
    }
}

/// Rate limiting result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub retry_after: Option<Duration>,
    pub remaining_tokens: Option<u32>,
    pub client_banned: bool,
}

/// Endpoint classification for different rate limits
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EndpointType {
    /// General API endpoints
    Api,
    /// Authentication endpoints (stricter limits)
    Authentication,
    /// Compute-intensive operations
    Compute,
    /// Health check endpoints (more lenient)
    HealthCheck,
    /// Administrative endpoints (most restrictive)
    Admin,
}

/// Production-grade rate limiter with DoS protection
pub struct ProductionRateLimiter {
    /// Configuration
    config: RateLimitConfig,
    
    /// Rate limit buckets by client IP
    client_buckets: Arc<RwLock<HashMap<IpAddr, HashMap<EndpointType, RateLimitBucket>>>>,
    
    /// Client information tracking
    client_info: Arc<RwLock<HashMap<IpAddr, ClientInfo>>>,
    
    /// Global system metrics
    global_metrics: Arc<Mutex<GlobalRateLimitMetrics>>,
    
    /// Adaptive rate limiting state
    adaptive_state: Arc<RwLock<AdaptiveRateLimitState>>,
}

#[derive(Debug)]
struct GlobalRateLimitMetrics {
    total_requests: u64,
    blocked_requests: u64,
    banned_clients: u64,
    suspicious_activities: u64,
    last_reset: Instant,
}

impl Default for GlobalRateLimitMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            blocked_requests: 0,
            banned_clients: 0,
            suspicious_activities: 0,
            last_reset: Instant::now(),
        }
    }
}

#[derive(Debug)]
struct AdaptiveRateLimitState {
    system_load: f64,
    active_connections: u32,
    memory_usage: f64,
    cpu_usage: f64,
    rate_multiplier: f64, // 1.0 = normal, < 1.0 = stricter, > 1.0 = more lenient
    last_update: Instant,
}

impl Default for AdaptiveRateLimitState {
    fn default() -> Self {
        Self {
            system_load: 0.0,
            active_connections: 0,
            memory_usage: 0.0,
            cpu_usage: 0.0,
            rate_multiplier: 1.0,
            last_update: Instant::now(),
        }
    }
}

impl ProductionRateLimiter {
    /// Create a new production rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            client_buckets: Arc::new(RwLock::new(HashMap::new())),
            client_info: Arc::new(RwLock::new(HashMap::new())),
            global_metrics: Arc::new(Mutex::new(GlobalRateLimitMetrics {
                last_reset: Instant::now(),
                ..Default::default()
            })),
            adaptive_state: Arc::new(RwLock::new(AdaptiveRateLimitState::default())),
        }
    }
    
    /// Check if a request should be allowed
    pub async fn check_request(
        &self,
        client_ip: IpAddr,
        endpoint_type: EndpointType,
        user_agent: Option<String>,
    ) -> RateLimitResult {
        let correlation_id = CorrelationId::new();
        
        // Check if IP is whitelisted
        if self.config.whitelist.contains(&client_ip) {
            debug!(
                correlation_id = %correlation_id,
                client_ip = %client_ip,
                operation = "rate_limit_whitelist_bypass",
                "Request allowed - IP is whitelisted"
            );
            return RateLimitResult {
                allowed: true,
                reason: None,
                retry_after: None,
                remaining_tokens: None,
                client_banned: false,
            };
        }
        
        // Update client info
        self.update_client_info(client_ip, user_agent.clone()).await;
        
        // Check if client is banned
        if let Some(ban_result) = self.check_client_ban(client_ip).await {
            warn!(
                correlation_id = %correlation_id,
                client_ip = %client_ip,
                ban_expires_in_seconds = ban_result.retry_after.map(|d| d.as_secs()),
                operation = "rate_limit_banned_client",
                "Request blocked - client is banned"
            );
            return ban_result;
        }
        
        // Get rate limit for endpoint type
        let base_limit = self.get_rate_limit_for_endpoint(&endpoint_type);
        
        // Apply adaptive rate limiting
        let adjusted_limit = if self.config.adaptive_limiting {
            self.apply_adaptive_rate_limiting(base_limit).await
        } else {
            base_limit
        };
        
        // Check rate limit
        let allowed = self.check_rate_limit(client_ip, endpoint_type.clone(), adjusted_limit).await;
        
        // Update global metrics
        {
            let mut metrics = self.global_metrics.lock().await;
            metrics.total_requests += 1;
            if !allowed {
                metrics.blocked_requests += 1;
            }
        }
        
        if !allowed {
            // Record violation
            self.record_violation(
                client_ip,
                ViolationType::RateLimitExceeded,
                ViolationSeverity::Medium,
                format!("Rate limit exceeded for {:?} endpoint", endpoint_type),
            ).await;
            
            warn!(
                correlation_id = %correlation_id,
                client_ip = %client_ip,
                endpoint_type = ?endpoint_type,
                rate_limit = adjusted_limit,
                operation = "rate_limit_exceeded",
                "Request blocked - rate limit exceeded"
            );
            
            RateLimitResult {
                allowed: false,
                reason: Some("Rate limit exceeded".to_string()),
                retry_after: Some(Duration::from_secs(60)),
                remaining_tokens: Some(0),
                client_banned: false,
            }
        } else {
            debug!(
                correlation_id = %correlation_id,
                client_ip = %client_ip,
                endpoint_type = ?endpoint_type,
                operation = "rate_limit_allowed",
                "Request allowed within rate limits"
            );
            
            RateLimitResult {
                allowed: true,
                reason: None,
                retry_after: None,
                remaining_tokens: None,
                client_banned: false,
            }
        }
    }
    
    /// Update client information
    async fn update_client_info(&self, client_ip: IpAddr, user_agent: Option<String>) {
        let mut clients = self.client_info.write().await;
        let now = Instant::now();
        
        let client_info = clients.entry(client_ip).or_insert_with(|| ClientInfo {
            ip_address: client_ip,
            user_agent: user_agent.clone(),
            first_seen: now,
            last_activity: now,
            total_requests: 0,
            violations: Vec::new(),
            is_banned: false,
            ban_expires_at: None,
        });
        
        client_info.last_activity = now;
        client_info.total_requests += 1;
        
        if client_info.user_agent.is_none() && user_agent.is_some() {
            client_info.user_agent = user_agent;
        }
    }
    
    /// Check if client is currently banned
    async fn check_client_ban(&self, client_ip: IpAddr) -> Option<RateLimitResult> {
        let clients = self.client_info.read().await;
        
        if let Some(client_info) = clients.get(&client_ip) {
            if client_info.is_banned {
                if let Some(ban_expires_at) = client_info.ban_expires_at {
                    if Instant::now() < ban_expires_at {
                        // Still banned
                        let retry_after = ban_expires_at.duration_since(Instant::now());
                        return Some(RateLimitResult {
                            allowed: false,
                            reason: Some("Client temporarily banned due to violations".to_string()),
                            retry_after: Some(retry_after),
                            remaining_tokens: None,
                            client_banned: true,
                        });
                    } else {
                        // Ban expired - will be cleaned up in next check
                    }
                }
            }
        }
        
        None
    }
    
    /// Get rate limit for specific endpoint type
    fn get_rate_limit_for_endpoint(&self, endpoint_type: &EndpointType) -> u32 {
        match endpoint_type {
            EndpointType::Api => self.config.api_requests_per_minute,
            EndpointType::Authentication => self.config.auth_requests_per_minute,
            EndpointType::Compute => self.config.compute_requests_per_minute,
            EndpointType::HealthCheck => self.config.api_requests_per_minute * 2, // More lenient
            EndpointType::Admin => self.config.auth_requests_per_minute / 2, // More restrictive
        }
    }
    
    /// Apply adaptive rate limiting based on system load
    async fn apply_adaptive_rate_limiting(&self, base_limit: u32) -> u32 {
        let adaptive_state = self.adaptive_state.read().await;
        let adjusted_limit = (base_limit as f64 * adaptive_state.rate_multiplier) as u32;
        
        // Ensure minimum limit
        adjusted_limit.max(1)
    }
    
    /// Check rate limit using token bucket algorithm
    async fn check_rate_limit(
        &self,
        client_ip: IpAddr,
        endpoint_type: EndpointType,
        rate_limit: u32,
    ) -> bool {
        let mut buckets = self.client_buckets.write().await;
        
        let client_buckets = buckets.entry(client_ip).or_insert_with(HashMap::new);
        
        let bucket = client_buckets.entry(endpoint_type).or_insert_with(|| {
            RateLimitBucket::new(self.config.burst_capacity, rate_limit)
        });
        
        bucket.try_consume(1.0)
    }
    
    /// Record a security violation
    async fn record_violation(
        &self,
        client_ip: IpAddr,
        violation_type: ViolationType,
        severity: ViolationSeverity,
        details: String,
    ) {
        let mut clients = self.client_info.write().await;
        
        if let Some(client_info) = clients.get_mut(&client_ip) {
            let violation = SecurityViolation {
                timestamp: Instant::now(),
                violation_type: violation_type.clone(),
                severity: severity.clone(),
                details: details.clone(),
            };
            
            client_info.violations.push(violation);
            
            // Check if client should be banned
            let recent_violations = client_info.violations.iter()
                .filter(|v| v.timestamp.elapsed() < self.config.violation_window)
                .count();
            
            if recent_violations >= self.config.ban_threshold as usize {
                client_info.is_banned = true;
                client_info.ban_expires_at = Some(Instant::now() + self.config.ban_duration);
                
                error!(
                    client_ip = %client_ip,
                    violation_count = recent_violations,
                    ban_duration_seconds = self.config.ban_duration.as_secs(),
                    operation = "client_banned",
                    "Client banned due to repeated violations"
                );
                
                // Update global metrics
                let mut metrics = self.global_metrics.lock().await;
                metrics.banned_clients += 1;
                if severity == ViolationSeverity::High || severity == ViolationSeverity::Critical {
                    metrics.suspicious_activities += 1;
                }
            }
        }
    }
    
    /// Update system metrics for adaptive rate limiting
    pub async fn update_system_metrics(
        &self,
        cpu_usage: f64,
        memory_usage: f64,
        active_connections: u32,
    ) {
        let mut adaptive_state = self.adaptive_state.write().await;
        
        adaptive_state.cpu_usage = cpu_usage;
        adaptive_state.memory_usage = memory_usage;
        adaptive_state.active_connections = active_connections;
        adaptive_state.last_update = Instant::now();
        
        // Calculate rate multiplier based on system load
        let load_factor = (cpu_usage + memory_usage) / 2.0;
        
        adaptive_state.rate_multiplier = if load_factor > 0.8 {
            0.5 // Strict limiting under high load
        } else if load_factor > 0.6 {
            0.7 // Moderate limiting
        } else if load_factor < 0.3 {
            1.2 // More lenient under low load
        } else {
            1.0 // Normal limiting
        };
        
        debug!(
            cpu_usage = %format!("{:.1}%", cpu_usage * 100.0),
            memory_usage = %format!("{:.1}%", memory_usage * 100.0),
            active_connections = active_connections,
            rate_multiplier = adaptive_state.rate_multiplier,
            operation = "adaptive_rate_limit_update",
            "Updated adaptive rate limiting parameters"
        );
    }
    
    /// Get rate limiting statistics
    pub async fn get_statistics(&self) -> RateLimitStatistics {
        let metrics = self.global_metrics.lock().await;
        let clients = self.client_info.read().await;
        let adaptive_state = self.adaptive_state.read().await;
        
        let uptime = metrics.last_reset.elapsed();
        let requests_per_second = if uptime.as_secs() > 0 {
            metrics.total_requests as f64 / uptime.as_secs_f64()
        } else {
            0.0
        };
        
        let block_rate = if metrics.total_requests > 0 {
            metrics.blocked_requests as f64 / metrics.total_requests as f64
        } else {
            0.0
        };
        
        RateLimitStatistics {
            total_requests: metrics.total_requests,
            blocked_requests: metrics.blocked_requests,
            banned_clients: metrics.banned_clients,
            suspicious_activities: metrics.suspicious_activities,
            active_clients: clients.len(),
            requests_per_second,
            block_rate,
            uptime,
            adaptive_rate_multiplier: adaptive_state.rate_multiplier,
            system_cpu_usage: adaptive_state.cpu_usage,
            system_memory_usage: adaptive_state.memory_usage,
        }
    }
    
    /// Cleanup expired data
    pub async fn cleanup_expired_data(&self) {
        let now = Instant::now();
        
        // Cleanup expired bans and old violations
        {
            let mut clients = self.client_info.write().await;
            clients.retain(|_, client_info| {
                // Remove expired bans
                if client_info.is_banned {
                    if let Some(ban_expires_at) = client_info.ban_expires_at {
                        if now >= ban_expires_at {
                            client_info.is_banned = false;
                            client_info.ban_expires_at = None;
                        }
                    }
                }
                
                // Remove old violations (keep only last 24 hours)
                client_info.violations.retain(|violation| {
                    violation.timestamp.elapsed() < Duration::from_secs(24 * 3600)
                });
                
                // Keep client info if active in last hour or has recent violations
                now.duration_since(client_info.last_activity) < Duration::from_secs(3600)
                    || !client_info.violations.is_empty()
            });
        }
        
        // Cleanup old rate limit buckets
        {
            let mut buckets = self.client_buckets.write().await;
            buckets.retain(|ip, _| {
                let clients = futures::executor::block_on(self.client_info.read());
                clients.contains_key(ip)
            });
        }
        
        info!(
            operation = "rate_limiter_cleanup",
            "Completed rate limiter data cleanup"
        );
    }
}

/// Rate limiting statistics
#[derive(Debug, Clone, Serialize)]
pub struct RateLimitStatistics {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub banned_clients: u64,
    pub suspicious_activities: u64,
    pub active_clients: usize,
    pub requests_per_second: f64,
    pub block_rate: f64,
    pub uptime: Duration,
    pub adaptive_rate_multiplier: f64,
    pub system_cpu_usage: f64,
    pub system_memory_usage: f64,
} 