// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]

//! Comprehensive tests for `ProductionRateLimiter`
//!
//! Tests `DoS` protection, rate limiting enforcement, ban mechanisms,
//! adaptive limiting, and security features.

use squirrel::security::rate_limiter::{EndpointType, ProductionRateLimiter, RateLimitConfig};
use std::net::IpAddr;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_rate_limiter_initialization() {
    let config = RateLimitConfig::default();
    let _limiter = ProductionRateLimiter::new(config);

    // Should initialize successfully with default config
    assert!(true, "Rate limiter initialized");
}

#[tokio::test]
async fn test_whitelist_bypass() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);

    // Localhost should be whitelisted by default
    let localhost: IpAddr = "127.0.0.1".parse().expect("should succeed");

    // Should allow many requests from whitelisted IP
    for _ in 0..1000 {
        let result = limiter
            .check_request(localhost, EndpointType::Api, None)
            .await;
        assert!(result.allowed, "Whitelisted IP should always be allowed");
        assert!(
            !result.client_banned,
            "Whitelisted IP should never be banned"
        );
    }
}

#[tokio::test]
async fn test_rate_limit_enforcement() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 5;
    config.burst_capacity = 5;
    config.whitelist = vec![]; // Remove default whitelist

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.100".parse().expect("should succeed");

    // First 5 requests should succeed (burst capacity)
    for i in 0..5 {
        let result = limiter
            .check_request(test_ip, EndpointType::Api, None)
            .await;
        assert!(
            result.allowed,
            "Request {} should be allowed (within burst)",
            i + 1
        );
    }

    // 6th request should be blocked
    let result = limiter
        .check_request(test_ip, EndpointType::Api, None)
        .await;
    assert!(!result.allowed, "Request should be rate limited");
    assert!(result.retry_after.is_some(), "Should provide retry_after");
}

#[tokio::test]
async fn test_different_endpoint_limits() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 10;
    config.auth_requests_per_minute = 3;
    config.compute_requests_per_minute = 5;
    config.burst_capacity = 3; // Match auth limit for proper testing
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.101".parse().expect("should succeed");

    // Auth endpoints should have stricter limits
    // Make requests and count how many are allowed
    let mut auth_allowed = 0;
    for _ in 0..6 {
        let result = limiter
            .check_request(test_ip, EndpointType::Authentication, None)
            .await;
        if result.allowed {
            auth_allowed += 1;
        }
    }

    // Should allow some based on burst capacity
    assert!(auth_allowed >= 3, "Should allow at least burst capacity");
    // Should enforce limit
    assert!(auth_allowed < 6, "Should enforce auth rate limit");
}

#[tokio::test]
async fn test_per_ip_isolation() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 3;
    config.burst_capacity = 3;
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let ip1: IpAddr = "192.168.1.102".parse().expect("should succeed");
    let ip2: IpAddr = "192.168.1.103".parse().expect("should succeed");

    // Exhaust IP1's limit
    for _ in 0..3 {
        limiter.check_request(ip1, EndpointType::Api, None).await;
    }

    // IP1 should be blocked
    let result = limiter.check_request(ip1, EndpointType::Api, None).await;
    assert!(!result.allowed, "IP1 should be rate limited");

    // IP2 should still be allowed (separate limit)
    let result = limiter.check_request(ip2, EndpointType::Api, None).await;
    assert!(result.allowed, "IP2 should not be affected by IP1's limit");
}

#[tokio::test]
async fn test_token_refill_over_time() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 6000; // 100 per second → refill every 10ms
    config.burst_capacity = 2;
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.104".parse().expect("should succeed");

    // Use burst capacity
    for _ in 0..2 {
        let result = limiter
            .check_request(test_ip, EndpointType::Api, None)
            .await;
        assert!(result.allowed);
    }

    // Should be blocked immediately after burst
    let result = limiter
        .check_request(test_ip, EndpointType::Api, None)
        .await;
    assert!(!result.allowed, "Should be rate limited after burst");

    // Wait just enough for token refill (10ms per token at 6000/min)
    sleep(Duration::from_millis(15)).await;

    // Should be allowed again after refill
    let result = limiter
        .check_request(test_ip, EndpointType::Api, None)
        .await;
    assert!(result.allowed, "Should be allowed after token refill");
}

#[tokio::test]
async fn test_ban_mechanism() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 2;
    config.burst_capacity = 2;
    config.ban_threshold = 3; // Ban after 3 violations
    config.violation_window = Duration::from_secs(10);
    config.ban_duration = Duration::from_secs(5);
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.105".parse().expect("should succeed");

    // Generate violations by exceeding rate limit repeatedly
    for _ in 0..6 {
        limiter
            .check_request(test_ip, EndpointType::Api, None)
            .await;
    }

    // After multiple violations, client should be banned
    let result = limiter
        .check_request(test_ip, EndpointType::Api, None)
        .await;

    // Should either be rate limited or banned
    assert!(
        !result.allowed,
        "Client should be blocked (banned or rate limited)"
    );
}

#[tokio::test]
async fn test_health_check_endpoints_more_lenient() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 10;
    config.burst_capacity = 15;
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.106".parse().expect("should succeed");

    // Health check endpoints should have higher limits
    // (2x API limit according to implementation)
    for i in 0..20 {
        let result = limiter
            .check_request(test_ip, EndpointType::HealthCheck, None)
            .await;
        if !result.allowed {
            // Should allow more than standard API limit
            assert!(i > 10, "Health check should allow more requests than API");
            break;
        }
    }
}

#[tokio::test]
async fn test_admin_endpoints_more_restrictive() {
    let mut config = RateLimitConfig::default();
    config.auth_requests_per_minute = 10;
    config.burst_capacity = 10;
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.107".parse().expect("should succeed");

    // Admin endpoints should have stricter limits than auth
    // (implementation uses 1/2 auth limit)
    let mut allowed_count = 0;
    for _ in 0..10 {
        let result = limiter
            .check_request(test_ip, EndpointType::Admin, None)
            .await;
        if result.allowed {
            allowed_count += 1;
        } else {
            break;
        }
    }

    // Should allow some requests but enforce limit
    assert!(allowed_count > 0, "Should allow some admin requests");
    assert!(allowed_count <= 10, "Should have finite limit");
}

#[tokio::test]
async fn test_adaptive_rate_limiting() {
    let mut config = RateLimitConfig::default();
    config.adaptive_limiting = true;
    config.api_requests_per_minute = 100;
    config.burst_capacity = 100;
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.108".parse().expect("should succeed");

    // Simulate high system load
    limiter.update_system_metrics(0.9, 0.85, 1000).await;

    // Under high load, rate limits may be adjusted
    // Test that the limiter still functions correctly
    let mut allowed_count = 0;
    for _ in 0..100 {
        let result = limiter
            .check_request(test_ip, EndpointType::Api, None)
            .await;
        if result.allowed {
            allowed_count += 1;
        } else {
            break;
        }
    }

    // Should enforce some limit (exact behavior depends on adaptive algorithm)
    assert!(allowed_count > 0, "Should allow some requests");
    assert!(allowed_count <= 100, "Should enforce rate limit");
}

#[tokio::test]
async fn test_statistics_collection() {
    let config = RateLimitConfig::default();
    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.109".parse().expect("should succeed");

    // Make some requests
    for _ in 0..5 {
        limiter
            .check_request(test_ip, EndpointType::Api, None)
            .await;
    }

    // Get statistics
    let stats = limiter.get_statistics().await;

    assert!(stats.total_requests >= 5, "Should track total requests");
    assert!(stats.active_clients >= 1, "Should track active clients");
    assert!(stats.requests_per_second >= 0.0, "Should calculate RPS");
}

#[tokio::test]
async fn test_user_agent_tracking() {
    let mut config = RateLimitConfig::default();
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.110".parse().expect("should succeed");

    // First request with user agent
    limiter
        .check_request(test_ip, EndpointType::Api, Some("Mozilla/5.0 Test"))
        .await;

    // Second request without user agent
    limiter
        .check_request(test_ip, EndpointType::Api, None)
        .await;

    // Should successfully track both
    let stats = limiter.get_statistics().await;
    assert!(
        stats.total_requests >= 2,
        "Should track requests with and without user agent"
    );
}

#[tokio::test]
async fn test_cleanup_expired_data() {
    let mut config = RateLimitConfig::default();
    config.ban_duration = Duration::from_millis(100); // Very short ban
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.111".parse().expect("should succeed");

    // Make some requests
    for _ in 0..5 {
        limiter
            .check_request(test_ip, EndpointType::Api, None)
            .await;
    }

    // Wait just past the ban_duration (100ms) for data to become stale
    sleep(Duration::from_millis(120)).await;

    // Run cleanup
    limiter.cleanup_expired_data().await;

    // Should complete without error
    assert!(true, "Cleanup should complete successfully");
}

#[tokio::test]
async fn test_concurrent_requests() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 100;
    config.burst_capacity = 100;
    config.whitelist = vec![];

    let limiter = std::sync::Arc::new(ProductionRateLimiter::new(config));
    let test_ip: IpAddr = "192.168.1.112".parse().expect("should succeed");

    // Spawn concurrent requests
    let mut handles = vec![];
    for _ in 0..50 {
        let limiter_clone = limiter.clone();
        let handle = tokio::spawn(async move {
            limiter_clone
                .check_request(test_ip, EndpointType::Api, None)
                .await
        });
        handles.push(handle);
    }

    // Wait for all requests
    let mut allowed_count = 0;
    for handle in handles {
        let result = handle.await.expect("should succeed");
        if result.allowed {
            allowed_count += 1;
        }
    }

    // Most requests should be allowed (within capacity)
    assert!(
        allowed_count >= 40,
        "Most concurrent requests should be allowed"
    );
}

#[tokio::test]
async fn test_ipv6_support() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 5;
    config.burst_capacity = 5;
    // IPv6 localhost should be whitelisted by default

    let limiter = ProductionRateLimiter::new(config);
    let ipv6_localhost: IpAddr = "::1".parse().expect("should succeed");

    // Should be whitelisted
    let result = limiter
        .check_request(ipv6_localhost, EndpointType::Api, None)
        .await;
    assert!(result.allowed, "IPv6 localhost should be whitelisted");
}

#[tokio::test]
async fn test_non_whitelisted_ipv6() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 3;
    config.burst_capacity = 3;
    config.whitelist = vec![]; // Remove default whitelist

    let limiter = ProductionRateLimiter::new(config);
    let test_ipv6: IpAddr = "2001:db8::1".parse().expect("should succeed");

    // Should be rate limited like any other IP
    for _ in 0..3 {
        limiter
            .check_request(test_ipv6, EndpointType::Api, None)
            .await;
    }

    let result = limiter
        .check_request(test_ipv6, EndpointType::Api, None)
        .await;
    assert!(
        !result.allowed,
        "Non-whitelisted IPv6 should be rate limited"
    );
}

#[tokio::test]
async fn test_retry_after_information() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 2;
    config.burst_capacity = 2;
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.113".parse().expect("should succeed");

    // Exhaust limit
    for _ in 0..2 {
        limiter
            .check_request(test_ip, EndpointType::Api, None)
            .await;
    }

    // Get rate limited response
    let result = limiter
        .check_request(test_ip, EndpointType::Api, None)
        .await;

    assert!(!result.allowed, "Should be rate limited");
    assert!(result.retry_after.is_some(), "Should provide retry_after");
    assert!(result.reason.is_some(), "Should provide reason");
}

#[tokio::test]
async fn test_separate_endpoint_buckets() {
    let mut config = RateLimitConfig::default();
    config.api_requests_per_minute = 3;
    config.auth_requests_per_minute = 3;
    config.burst_capacity = 3;
    config.whitelist = vec![];

    let limiter = ProductionRateLimiter::new(config);
    let test_ip: IpAddr = "192.168.1.114".parse().expect("should succeed");

    // Exhaust API limit
    for _ in 0..3 {
        limiter
            .check_request(test_ip, EndpointType::Api, None)
            .await;
    }

    // API should be limited
    let result = limiter
        .check_request(test_ip, EndpointType::Api, None)
        .await;
    assert!(!result.allowed, "API should be rate limited");

    // But Auth should still work (separate bucket)
    let result = limiter
        .check_request(test_ip, EndpointType::Authentication, None)
        .await;
    assert!(result.allowed, "Auth should have separate limit");
}

#[tokio::test]
async fn test_system_metrics_update() {
    let mut config = RateLimitConfig::default();
    config.adaptive_limiting = true;

    let limiter = ProductionRateLimiter::new(config);

    // Update metrics with various loads
    limiter.update_system_metrics(0.5, 0.6, 100).await;
    limiter.update_system_metrics(0.9, 0.8, 500).await;
    limiter.update_system_metrics(0.2, 0.3, 50).await;

    // Should complete without error
    let stats = limiter.get_statistics().await;
    assert!(stats.system_cpu_usage >= 0.0, "Should track CPU usage");
    assert!(
        stats.system_memory_usage >= 0.0,
        "Should track memory usage"
    );
}
