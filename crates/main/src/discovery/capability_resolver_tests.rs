//! Tests for capability resolver

use super::capability_resolver::{CapabilityResolver, DiscoveryMethod};
use super::mechanisms::RegistryType;
use super::types::CapabilityRequest;
use std::env;
use std::time::Duration;

#[test]
fn test_capability_resolver_new() {
    let resolver = CapabilityResolver::new();
    // Verify resolver was created successfully
    assert!(format!("{:?}", resolver).contains("CapabilityResolver"));
}

#[test]
fn test_capability_resolver_with_registry() {
    let resolver = CapabilityResolver::with_registry(
        RegistryType::Consul,
        "http://localhost:8500".to_string(),
    );
    assert!(format!("{:?}", resolver).contains("CapabilityResolver"));
}

#[test]
fn test_capability_resolver_default() {
    let resolver = CapabilityResolver::default();
    assert!(format!("{:?}", resolver).contains("CapabilityResolver"));
}

#[test]
fn test_discovery_method_variants() {
    let methods = vec![
        DiscoveryMethod::EnvironmentVariable,
        DiscoveryMethod::ServiceRegistry,
        DiscoveryMethod::MDns,
        DiscoveryMethod::DnsSd,
        DiscoveryMethod::P2PMulticast,
    ];

    for method in methods {
        let debug_str = format!("{:?}", method);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_discovery_method_equality() {
    assert_eq!(
        DiscoveryMethod::EnvironmentVariable,
        DiscoveryMethod::EnvironmentVariable
    );
    assert_ne!(
        DiscoveryMethod::EnvironmentVariable,
        DiscoveryMethod::ServiceRegistry
    );
}

#[test]
fn test_discovery_method_clone() {
    let method = DiscoveryMethod::MDns;
    let cloned = method.clone();
    assert_eq!(method, cloned);
}

#[tokio::test]
async fn test_discover_from_env_found() {
    let resolver = CapabilityResolver::new();

    // Set environment variable with flexible test port
    let test_port = env::var("TEST_AI_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8000);
    let test_endpoint = format!("http://localhost:{}", test_port);
    env::set_var("AI_COMPLETE_ENDPOINT", &test_endpoint);

    let request = CapabilityRequest {
        capability: "ai.complete".to_string(),
        features: vec![],
        preference: None,
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    let result = resolver.discover_provider(request).await;

    // Clean up
    env::remove_var("AI_COMPLETE_ENDPOINT");

    // Verify discovery succeeded
    assert!(result.is_ok());
    let service = result.unwrap();
    assert_eq!(service.endpoint, test_endpoint);
    assert!(service.capabilities.contains(&"ai.complete".to_string()));
    assert_eq!(service.priority, 100); // Highest priority for env vars
}

#[tokio::test]
async fn test_discover_from_env_not_found() {
    let resolver = CapabilityResolver::new();

    // Ensure env var is not set
    env::remove_var("NONEXISTENT_CAPABILITY_ENDPOINT");

    let request = CapabilityRequest {
        capability: "nonexistent.capability".to_string(),
        features: vec![],
        preference: None,
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    let result = resolver.discover_provider(request).await;

    // Should fail - no discovery mechanisms will find this
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discover_provider_with_dots_in_capability() {
    let resolver = CapabilityResolver::new();

    // Set environment variable with dots converted to underscores
    env::set_var("HTTP_REQUEST_ENDPOINT", "unix:///tmp/songbird.sock");

    let request = CapabilityRequest {
        capability: "http.request".to_string(),
        features: vec![],
        preference: None,
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    let result = resolver.discover_provider(request).await;

    // Clean up
    env::remove_var("HTTP_REQUEST_ENDPOINT");

    assert!(result.is_ok());
    let service = result.unwrap();
    assert_eq!(service.endpoint, "unix:///tmp/songbird.sock");
    assert!(service.capabilities.contains(&"http.request".to_string()));
}

#[tokio::test]
async fn test_discover_provider_priority() {
    let resolver = CapabilityResolver::new();

    // Set environment variable (should have highest priority)
    env::set_var("TEST_CAPABILITY_ENDPOINT", "http://env-provider:8000");

    let request = CapabilityRequest {
        capability: "test.capability".to_string(),
        features: vec![],
        preference: None,
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    let result = resolver.discover_provider(request).await;

    // Clean up
    env::remove_var("TEST_CAPABILITY_ENDPOINT");

    // Environment variable should be discovered first (priority 100)
    assert!(result.is_ok());
    let service = result.unwrap();
    assert_eq!(service.endpoint, "http://env-provider:8000");
    assert_eq!(service.priority, 100);
}

#[tokio::test]
async fn test_capability_request_with_preference() {
    let resolver = CapabilityResolver::new();

    env::set_var("PREFERRED_SERVICE_ENDPOINT", "http://localhost:9000");

    let request = CapabilityRequest {
        capability: "preferred.service".to_string(),
        features: vec![],
        preference: Some("performance".to_string()),
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    let result = resolver.discover_provider(request).await;

    env::remove_var("PREFERRED_SERVICE_ENDPOINT");

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_capability_request_with_features() {
    let resolver = CapabilityResolver::new();

    env::set_var("FEATURED_SERVICE_ENDPOINT", "http://localhost:10000");

    let request = CapabilityRequest {
        capability: "featured.service".to_string(),
        features: vec!["feature1".to_string(), "feature2".to_string()],
        preference: None,
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    let result = resolver.discover_provider(request).await;

    env::remove_var("FEATURED_SERVICE_ENDPOINT");

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_capability_request_with_timeout() {
    let resolver = CapabilityResolver::new();

    let request = CapabilityRequest {
        capability: "timeout.test".to_string(),
        features: vec![],
        preference: None,
        timeout: Duration::from_millis(100), // Very short timeout
        use_cache: true,
    };

    // This will timeout since we didn't set env var and other mechanisms won't find it
    let result = resolver.discover_provider(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resolver_clone() {
    let resolver1 = CapabilityResolver::new();
    let resolver2 = resolver1.clone();

    env::set_var("CLONE_TEST_ENDPOINT", "http://localhost:11000");

    let request = CapabilityRequest {
        capability: "clone.test".to_string(),
        features: vec![],
        preference: None,
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    // Both resolvers should work
    let result1 = resolver1.discover_provider(request.clone()).await;
    let result2 = resolver2.discover_provider(request).await;

    env::remove_var("CLONE_TEST_ENDPOINT");

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_discover_service_metadata() {
    let resolver = CapabilityResolver::new();

    env::set_var("METADATA_TEST_ENDPOINT", "http://localhost:12000");

    let request = CapabilityRequest {
        capability: "metadata.test".to_string(),
        features: vec![],
        preference: None,
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    let result = resolver.discover_provider(request).await;

    env::remove_var("METADATA_TEST_ENDPOINT");

    assert!(result.is_ok());
    let service = result.unwrap();
    assert_eq!(service.name, "metadata.test-provider");
    assert_eq!(service.discovery_method, "environment_variable");
    assert!(service.healthy.unwrap_or(false));
}

#[tokio::test]
async fn test_uppercase_capability_env_conversion() {
    let resolver = CapabilityResolver::new();

    // Test that lowercase capability is converted to uppercase env var
    env::set_var("LOWERCASE_TEST_ENDPOINT", "http://localhost:13000");

    let request = CapabilityRequest {
        capability: "lowercase.test".to_string(),
        features: vec![],
        preference: None,
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    let result = resolver.discover_provider(request).await;

    env::remove_var("LOWERCASE_TEST_ENDPOINT");

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_dots_in_capability() {
    let resolver = CapabilityResolver::new();

    // Test capability with multiple dots: "ai.neural.inference"
    env::set_var("AI_NEURAL_INFERENCE_ENDPOINT", "http://localhost:14000");

    let request = CapabilityRequest {
        capability: "ai.neural.inference".to_string(),
        features: vec![],
        preference: None,
        timeout: Duration::from_secs(5),
        use_cache: true,
    };

    let result = resolver.discover_provider(request).await;

    env::remove_var("AI_NEURAL_INFERENCE_ENDPOINT");

    assert!(result.is_ok());
    let service = result.unwrap();
    assert!(service
        .capabilities
        .contains(&"ai.neural.inference".to_string()));
}
