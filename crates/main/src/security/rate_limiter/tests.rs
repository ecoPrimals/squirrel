// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::bucket::RateLimitBucket;
use super::config::RateLimitConfig;
use super::production::ProductionRateLimiter;
use super::types::{
    ClientRequestCounter, EndpointType, RateLimitResult, ViolationSeverity, ViolationType,
};
use std::net::IpAddr;
use std::time::Duration;

#[test]
fn test_rate_limit_config_default() {
    let config = RateLimitConfig::default();
    assert_eq!(config.api_requests_per_minute, 100);
    assert_eq!(config.auth_requests_per_minute, 10);
    assert_eq!(config.compute_requests_per_minute, 20);
    assert_eq!(config.burst_capacity, 150);
    assert_eq!(config.ban_duration, Duration::from_secs(300));
    assert_eq!(config.ban_threshold, 5);
    assert_eq!(config.violation_window, Duration::from_secs(60));
    assert!(config.adaptive_limiting);
    assert_eq!(config.whitelist.len(), 2); // localhost IPv4 and IPv6
}

#[test]
fn test_rate_limit_bucket_new() {
    let bucket = RateLimitBucket::new(100, 60);
    assert_eq!(bucket.tokens, 100.0);
    assert_eq!(bucket.capacity, 100.0);
    assert_eq!(bucket.refill_rate, 1.0); // 60 per minute = 1 per second
    assert_eq!(bucket.request_count, 0);
}

#[test]
fn test_rate_limit_bucket_try_consume_success() {
    let mut bucket = RateLimitBucket::new(100, 60);
    assert!(bucket.try_consume(1.0));
    assert_eq!(bucket.tokens, 99.0);
    assert_eq!(bucket.request_count, 1);
}

#[test]
fn test_rate_limit_bucket_try_consume_failure() {
    let mut bucket = RateLimitBucket::new(10, 60);
    // Consume all tokens
    for _ in 0..10 {
        assert!(bucket.try_consume(1.0));
    }
    // Should fail on 11th request
    assert!(!bucket.try_consume(1.0));
    assert_eq!(bucket.request_count, 10);
}

#[test]
fn test_client_request_counter_default() {
    let counter = ClientRequestCounter::default();
    assert_eq!(counter.tokens, 0.0);
    assert_eq!(counter.capacity, 0.0);
    assert_eq!(counter.refill_rate, 0.0);
    assert_eq!(counter.request_count, 0);
}

#[tokio::test]
async fn test_production_rate_limiter_new() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config.clone());

    let stats = limiter.get_statistics().await;
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.blocked_requests, 0);
    assert_eq!(stats.banned_clients, 0);
}

#[tokio::test]
async fn test_production_rate_limiter_check_request_success() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);

    let ip: IpAddr = "192.168.1.100".parse().unwrap();

    let result = limiter.check_request(ip, EndpointType::Api, None).await;

    assert!(result.allowed);
}

#[tokio::test]
async fn test_production_rate_limiter_check_request_whitelist() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);

    let ip: IpAddr = "127.0.0.1".parse().unwrap(); // localhost, in whitelist

    let result = limiter.check_request(ip, EndpointType::Api, None).await;

    assert!(result.allowed);
    // Whitelist IPs have unlimited tokens
}

#[tokio::test]
async fn test_production_rate_limiter_different_endpoint_types() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);

    let ip: IpAddr = "192.168.1.100".parse().unwrap();

    // Test API request
    let api_result = limiter.check_request(ip, EndpointType::Api, None).await;
    assert!(api_result.allowed);

    // Test Auth request (lower limit)
    let auth_result = limiter
        .check_request(ip, EndpointType::Authentication, None)
        .await;
    assert!(auth_result.allowed);

    // Test Compute request
    let compute_result = limiter.check_request(ip, EndpointType::Compute, None).await;
    assert!(compute_result.allowed);
}

#[tokio::test]
async fn test_production_rate_limiter_update_system_metrics() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);

    limiter.update_system_metrics(50.0, 60.0, 10).await;

    let stats = limiter.get_statistics().await;
    assert_eq!(stats.system_cpu_usage, 50.0);
    assert_eq!(stats.system_memory_usage, 60.0);
}

#[tokio::test]
async fn test_production_rate_limiter_cleanup_expired_data() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);

    let ip: IpAddr = "192.168.1.100".parse().unwrap();

    // Make a request to create client info
    limiter.check_request(ip, EndpointType::Api, None).await;

    // Cleanup shouldn't affect recent data
    limiter.cleanup_expired_data().await;

    let stats = limiter.get_statistics().await;
    assert_eq!(stats.active_clients, 1);
}

#[tokio::test]
async fn test_production_rate_limiter_multiple_clients() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);

    let ip1: IpAddr = "192.168.1.100".parse().unwrap();
    let ip2: IpAddr = "192.168.1.101".parse().unwrap();

    // Make requests from different IPs
    limiter.check_request(ip1, EndpointType::Api, None).await;
    limiter.check_request(ip2, EndpointType::Api, None).await;

    let stats = limiter.get_statistics().await;
    assert_eq!(stats.total_requests, 2);
    assert_eq!(stats.active_clients, 2);
}

#[tokio::test]
async fn test_rate_limit_result_fields() {
    let result = RateLimitResult {
        allowed: true,
        remaining_tokens: Some(50),
        retry_after: None,
        reason: None,
        client_banned: false,
    };

    // Test fields
    assert!(result.allowed);
    assert_eq!(result.remaining_tokens, Some(50));
    assert!(!result.client_banned);
}

#[tokio::test]
async fn test_production_rate_limiter_with_user_agent() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);

    let ip: IpAddr = "192.168.1.100".parse().unwrap();
    let user_agent = Some("TestClient/1.0");

    let result = limiter
        .check_request(ip, EndpointType::Api, user_agent)
        .await;

    assert!(result.allowed);
}

#[test]
fn test_violation_type_equality() {
    assert_eq!(
        ViolationType::RateLimitExceeded,
        ViolationType::RateLimitExceeded
    );
    assert_ne!(
        ViolationType::RateLimitExceeded,
        ViolationType::SuspiciousActivity
    );
}

#[test]
fn test_violation_severity_equality() {
    assert_eq!(ViolationSeverity::High, ViolationSeverity::High);
    assert_ne!(ViolationSeverity::Low, ViolationSeverity::Critical);
}

#[tokio::test]
async fn test_rate_limit_ban_after_repeated_violations() {
    let mut config = RateLimitConfig::default();
    config.whitelist.clear();
    config.burst_capacity = 1;
    config.api_requests_per_minute = 1;
    config.ban_threshold = 3;
    config.violation_window = Duration::from_secs(3600);
    config.adaptive_limiting = false;

    let limiter = ProductionRateLimiter::new(config);
    let ip: IpAddr = "10.0.0.42".parse().unwrap();

    let _ = limiter.check_request(ip, EndpointType::Api, None).await;

    let mut saw_ban = false;
    for _ in 0..64 {
        let r = limiter.check_request(ip, EndpointType::Api, None).await;
        if r.client_banned {
            assert!(!r.allowed);
            saw_ban = true;
            break;
        }
    }
    assert!(
        saw_ban,
        "expected client to be banned after repeated violations"
    );

    let banned = limiter.check_request(ip, EndpointType::Api, None).await;
    assert!(banned.client_banned);
    assert!(!banned.allowed);
}

#[tokio::test]
async fn test_adaptive_rate_multiplier_high_load() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);
    limiter.update_system_metrics(0.9, 0.85, 100).await;
    let stats = limiter.get_statistics().await;
    assert!((stats.adaptive_rate_multiplier - 0.5).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_adaptive_rate_multiplier_low_load() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);
    limiter.update_system_metrics(0.1, 0.2, 1).await;
    let stats = limiter.get_statistics().await;
    assert!((stats.adaptive_rate_multiplier - 1.2).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_health_check_endpoint_uses_lenient_limit() {
    let mut config = RateLimitConfig::default();
    config.whitelist.clear();
    config.burst_capacity = 200;
    config.api_requests_per_minute = 10;
    config.adaptive_limiting = false;

    let limiter = ProductionRateLimiter::new(config);
    let ip: IpAddr = "10.0.0.7".parse().unwrap();

    for _ in 0..25 {
        let r = limiter
            .check_request(ip, EndpointType::HealthCheck, None)
            .await;
        assert!(
            r.allowed,
            "HealthCheck limit is 2x API; should stay within burst for this loop"
        );
    }
}
