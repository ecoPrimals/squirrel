//! Tests for ecosystem registry configuration types

use super::config::RegistrySecurityConfig as SecurityConfig;
use super::config::*;
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// RETRY CONFIG TESTS
// ============================================================================

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();

    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay, Duration::from_millis(500));
    assert_eq!(config.max_delay, Duration::from_secs(30));
    assert_eq!(config.backoff_multiplier, 2.0);
    assert!(config.jitter);
}

#[test]
fn test_retry_config_custom() {
    let config = RetryConfig {
        max_retries: 5,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(60),
        backoff_multiplier: 1.5,
        jitter: false,
    };

    assert_eq!(config.max_retries, 5);
    assert_eq!(config.initial_delay, Duration::from_millis(100));
    assert!(!config.jitter);
}

#[test]
fn test_retry_config_serialization() {
    let config = RetryConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: RetryConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.max_retries, deserialized.max_retries);
    assert_eq!(config.backoff_multiplier, deserialized.backoff_multiplier);
}

#[test]
fn test_retry_config_clone() {
    let config = RetryConfig::default();
    let cloned = config.clone();

    assert_eq!(config.max_retries, cloned.max_retries);
    assert_eq!(config.initial_delay, cloned.initial_delay);
}

#[test]
fn test_retry_config_no_retries() {
    let config = RetryConfig {
        max_retries: 0,
        initial_delay: Duration::from_millis(0),
        max_delay: Duration::from_secs(0),
        backoff_multiplier: 1.0,
        jitter: false,
    };

    assert_eq!(config.max_retries, 0);
}

// ============================================================================
// HEALTH CONFIG TESTS
// ============================================================================

#[test]
fn test_health_config_default() {
    let config = HealthConfig::default();

    assert_eq!(config.check_interval, Duration::from_secs(30));
    assert_eq!(config.timeout, Duration::from_secs(10));
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.recovery_threshold, 2);
    assert_eq!(config.grace_period, Duration::from_secs(30));
}

#[test]
fn test_health_config_custom() {
    let config = HealthConfig {
        check_interval: Duration::from_secs(60),
        timeout: Duration::from_secs(5),
        failure_threshold: 5,
        recovery_threshold: 3,
        grace_period: Duration::from_secs(60),
    };

    assert_eq!(config.check_interval, Duration::from_secs(60));
    assert_eq!(config.failure_threshold, 5);
}

#[test]
fn test_health_config_serialization() {
    let config = HealthConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: HealthConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.failure_threshold, deserialized.failure_threshold);
    assert_eq!(config.recovery_threshold, deserialized.recovery_threshold);
}

#[test]
fn test_health_config_clone() {
    let config = HealthConfig::default();
    let cloned = config.clone();

    assert_eq!(config.check_interval, cloned.check_interval);
    assert_eq!(config.timeout, cloned.timeout);
}

#[test]
fn test_health_config_aggressive() {
    let config = HealthConfig {
        check_interval: Duration::from_secs(5),
        timeout: Duration::from_secs(1),
        failure_threshold: 1,
        recovery_threshold: 1,
        grace_period: Duration::from_secs(0),
    };

    assert_eq!(config.check_interval, Duration::from_secs(5));
    assert_eq!(config.failure_threshold, 1);
}

// ============================================================================
// DISCOVERY CONFIG TESTS
// ============================================================================

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();

    assert!(config.enabled);
    assert_eq!(config.discovery_interval, Duration::from_secs(60));
    assert_eq!(config.service_timeout, Duration::from_secs(5));
    assert!(config.auto_register);
    assert!(config.preferred_endpoints.is_empty());
}

#[test]
fn test_discovery_config_custom() {
    let mut preferred = HashMap::new();
    preferred.insert("api".to_string(), "http://api.local:8080".to_string());

    let config = DiscoveryConfig {
        enabled: false,
        discovery_interval: Duration::from_secs(120),
        service_timeout: Duration::from_secs(10),
        auto_register: false,
        preferred_endpoints: preferred.clone(),
    };

    assert!(!config.enabled);
    assert_eq!(config.discovery_interval, Duration::from_secs(120));
    assert_eq!(config.preferred_endpoints.len(), 1);
}

#[test]
fn test_discovery_config_serialization() {
    let config = DiscoveryConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: DiscoveryConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.auto_register, deserialized.auto_register);
}

#[test]
fn test_discovery_config_clone() {
    let config = DiscoveryConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.discovery_interval, cloned.discovery_interval);
}

#[test]
fn test_discovery_config_with_endpoints() {
    let mut preferred = HashMap::new();
    preferred.insert("storage".to_string(), "http://storage:9000".to_string());
    preferred.insert("compute".to_string(), "http://compute:8000".to_string());

    let config = DiscoveryConfig {
        enabled: true,
        discovery_interval: Duration::from_secs(60),
        service_timeout: Duration::from_secs(5),
        auto_register: true,
        preferred_endpoints: preferred,
    };

    assert_eq!(config.preferred_endpoints.len(), 2);
    assert!(config.preferred_endpoints.contains_key("storage"));
}

// ============================================================================
// SECURITY CONFIG TESTS
// ============================================================================

#[test]
fn test_security_config_default() {
    let config = SecurityConfig::default();

    assert!(config.tls_enabled);
    assert!(!config.mtls_required);
    assert!(config.auth_token.is_none());
    assert_eq!(config.trust_domain, "squirrel");
    assert!(config.certificate_path.is_none());
    assert!(config.key_path.is_none());
}

#[test]
fn test_security_config_custom() {
    let config = SecurityConfig {
        tls_enabled: false,
        mtls_required: true,
        auth_token: Some("secret-token".to_string()),
        trust_domain: "production".to_string(),
        certificate_path: Some("/etc/certs/cert.pem".to_string()),
        key_path: Some("/etc/certs/key.pem".to_string()),
    };

    assert!(!config.tls_enabled);
    assert!(config.mtls_required);
    assert_eq!(config.auth_token.as_ref().unwrap(), "secret-token");
}

#[test]
fn test_security_config_serialization() {
    let config = SecurityConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: SecurityConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.tls_enabled, deserialized.tls_enabled);
    assert_eq!(config.trust_domain, deserialized.trust_domain);
}

#[test]
fn test_security_config_clone() {
    let config = SecurityConfig::default();
    let cloned = config.clone();

    assert_eq!(config.tls_enabled, cloned.tls_enabled);
    assert_eq!(config.trust_domain, cloned.trust_domain);
}

#[test]
fn test_security_config_with_mtls() {
    let config = SecurityConfig {
        tls_enabled: true,
        mtls_required: true,
        auth_token: None,
        trust_domain: "secure".to_string(),
        certificate_path: Some("/certs/client.crt".to_string()),
        key_path: Some("/certs/client.key".to_string()),
    };

    assert!(config.tls_enabled);
    assert!(config.mtls_required);
    assert!(config.certificate_path.is_some());
    assert!(config.key_path.is_some());
}

// ============================================================================
// ECOSYSTEM REGISTRY CONFIG TESTS
// ============================================================================

#[test]
fn test_ecosystem_registry_config_default() {
    let config = EcosystemRegistryConfig::default();

    assert!(!config.songbird_endpoint.is_empty());
    assert_eq!(config.retry_config.max_retries, 3);
    assert_eq!(config.health_config.failure_threshold, 3);
    assert!(config.discovery_config.enabled);
    assert!(config.security_config.tls_enabled);
}

#[test]
fn test_ecosystem_registry_config_custom() {
    let config = EcosystemRegistryConfig {
        songbird_endpoint: "http://custom-songbird:8080".to_string(),
        retry_config: RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        },
        health_config: HealthConfig::default(),
        discovery_config: DiscoveryConfig::default(),
        security_config: SecurityConfig::default(),
    };

    assert_eq!(config.songbird_endpoint, "http://custom-songbird:8080");
    assert_eq!(config.retry_config.max_retries, 5);
}

#[test]
fn test_ecosystem_registry_config_serialization() {
    let config = EcosystemRegistryConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: EcosystemRegistryConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.songbird_endpoint, deserialized.songbird_endpoint);
}

#[test]
fn test_ecosystem_registry_config_clone() {
    let config = EcosystemRegistryConfig::default();
    let cloned = config.clone();

    assert_eq!(config.songbird_endpoint, cloned.songbird_endpoint);
    assert_eq!(
        config.retry_config.max_retries,
        cloned.retry_config.max_retries
    );
}

#[test]
fn test_ecosystem_registry_config_components() {
    let config = EcosystemRegistryConfig::default();

    // Verify all components are initialized
    assert!(config.retry_config.max_retries > 0);
    assert!(config.health_config.check_interval > Duration::from_secs(0));
    assert!(config.discovery_config.discovery_interval > Duration::from_secs(0));
    assert!(!config.security_config.trust_domain.is_empty());
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_full_config_setup() {
    let retry = RetryConfig::default();
    let health = HealthConfig::default();
    let discovery = DiscoveryConfig::default();
    let security = SecurityConfig::default();

    let config = EcosystemRegistryConfig {
        songbird_endpoint: "http://songbird:8080".to_string(),
        retry_config: retry,
        health_config: health,
        discovery_config: discovery,
        security_config: security,
    };

    assert!(!config.songbird_endpoint.is_empty());
}

#[test]
fn test_config_modification() {
    let mut config = EcosystemRegistryConfig::default();

    config.songbird_endpoint = "http://new-endpoint:9090".to_string();
    config.retry_config.max_retries = 10;
    config.health_config.failure_threshold = 5;
    config.discovery_config.enabled = false;
    config.security_config.tls_enabled = false;

    assert_eq!(config.songbird_endpoint, "http://new-endpoint:9090");
    assert_eq!(config.retry_config.max_retries, 10);
    assert_eq!(config.health_config.failure_threshold, 5);
    assert!(!config.discovery_config.enabled);
    assert!(!config.security_config.tls_enabled);
}

#[test]
fn test_production_config() {
    let config = EcosystemRegistryConfig {
        songbird_endpoint: "https://songbird.production.local:443".to_string(),
        retry_config: RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        },
        health_config: HealthConfig {
            check_interval: Duration::from_secs(15),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
            grace_period: Duration::from_secs(30),
        },
        discovery_config: DiscoveryConfig {
            enabled: true,
            discovery_interval: Duration::from_secs(30),
            service_timeout: Duration::from_secs(5),
            auto_register: true,
            preferred_endpoints: HashMap::new(),
        },
        security_config: SecurityConfig {
            tls_enabled: true,
            mtls_required: true,
            auth_token: Some("prod-token".to_string()),
            trust_domain: "production".to_string(),
            certificate_path: Some("/etc/ssl/cert.pem".to_string()),
            key_path: Some("/etc/ssl/key.pem".to_string()),
        },
    };

    assert!(config.songbird_endpoint.contains("production"));
    assert!(config.security_config.mtls_required);
}

#[test]
fn test_development_config() {
    let config = EcosystemRegistryConfig {
        songbird_endpoint: "http://localhost:8080".to_string(),
        retry_config: RetryConfig {
            max_retries: 1,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 1.0,
            jitter: false,
        },
        health_config: HealthConfig {
            check_interval: Duration::from_secs(60),
            timeout: Duration::from_secs(10),
            failure_threshold: 10,
            recovery_threshold: 1,
            grace_period: Duration::from_secs(60),
        },
        discovery_config: DiscoveryConfig {
            enabled: true,
            discovery_interval: Duration::from_secs(120),
            service_timeout: Duration::from_secs(10),
            auto_register: true,
            preferred_endpoints: HashMap::new(),
        },
        security_config: SecurityConfig {
            tls_enabled: false,
            mtls_required: false,
            auth_token: None,
            trust_domain: "development".to_string(),
            certificate_path: None,
            key_path: None,
        },
    };

    assert!(config.songbird_endpoint.contains("localhost"));
    assert!(!config.security_config.tls_enabled);
}
