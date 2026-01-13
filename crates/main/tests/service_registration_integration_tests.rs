//! Integration tests for ecosystem service registration
//!
//! Tests the complete registration flow with environment configuration

mod common;

use common::create_test_provider;
use squirrel::ecosystem::{EcosystemServiceRegistration, ServiceCapabilities, ServiceEndpoints};
use squirrel::primal_provider::{EcosystemIntegration, SquirrelPrimalProvider};
use std::env;

#[tokio::test]
async fn test_service_registration_with_default_config() -> Result<(), Box<dyn std::error::Error>> {
    // Clear any environment variables
    env::remove_var("SERVER_BIND_ADDRESS");
    env::remove_var("SERVER_PORT");

    let provider = create_test_provider().await?;
    let registration = EcosystemIntegration::create_service_registration(&provider);

    // Should use defaults when no env vars set
    assert!(registration.endpoints.primary.contains("0.0.0.0"));
    assert!(registration.endpoints.primary.contains("8080"));
    assert!(!registration.endpoints.secondary.is_empty());
    assert!(registration.endpoints.health.is_some());

    Ok(())
}

#[tokio::test]
async fn test_service_registration_with_custom_address() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("SERVER_BIND_ADDRESS", "192.168.1.100");
    env::set_var("SERVER_PORT", "9090");

    let provider = create_test_provider().await?;
    let registration = EcosystemIntegration::create_service_registration(&provider);

    assert!(registration.endpoints.primary.contains("192.168.1.100"));
    assert!(registration.endpoints.primary.contains("9090"));

    // Verify all secondary endpoints also use custom config
    for endpoint in &registration.endpoints.secondary {
        assert!(endpoint.contains("192.168.1.100"));
        assert!(endpoint.contains("9090"));
    }

    // Cleanup
    env::remove_var("SERVER_BIND_ADDRESS");
    env::remove_var("SERVER_PORT");

    Ok(())
}

#[tokio::test]
async fn test_service_registration_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let provider = create_test_provider().await?;
    let registration = EcosystemIntegration::create_service_registration(&provider);

    assert_eq!(registration.name, "Squirrel AI Primal");
    assert_eq!(registration.version, "1.0.0");
    assert!(!registration.description.is_empty());
    assert!(registration.tags.contains(&"ai".to_string()));
    assert!(registration.tags.contains(&"coordination".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_service_registration_capabilities() -> Result<(), Box<dyn std::error::Error>> {
    let provider = create_test_provider().await?;
    let registration = EcosystemIntegration::create_service_registration(&provider);

    assert!(!registration.capabilities.core.is_empty());
    assert!(registration
        .capabilities
        .core
        .contains(&"ai_coordination".to_string()));
    assert!(registration
        .capabilities
        .core
        .contains(&"context_analysis".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_service_registration_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let provider = create_test_provider().await?;
    let registration = EcosystemIntegration::create_service_registration(&provider);

    assert!(registration.health_check.enabled);
    assert_eq!(registration.health_check.interval_secs, 30);
    assert_eq!(registration.health_check.timeout_secs, 10);
    assert_eq!(registration.health_check.failure_threshold, 3);

    Ok(())
}

#[tokio::test]
async fn test_service_registration_endpoints_structure() -> Result<(), Box<dyn std::error::Error>> {
    let provider = create_test_provider().await?;
    let registration = EcosystemIntegration::create_service_registration(&provider);

    // Verify primary endpoint
    assert!(registration.endpoints.primary.starts_with("http://"));

    // Verify secondary endpoints include expected paths
    let secondary_paths = registration.endpoints.secondary.join(",");
    assert!(secondary_paths.contains("/metrics"));
    assert!(secondary_paths.contains("/admin"));
    assert!(secondary_paths.contains("/mcp"));
    assert!(secondary_paths.contains("/ai"));
    assert!(secondary_paths.contains("/mesh"));

    // Verify health endpoint
    let health = registration.endpoints.health.as_ref().unwrap();
    assert!(health.contains("/health"));

    Ok(())
}

#[tokio::test]
async fn test_multiple_registrations_independence() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("SERVER_BIND_ADDRESS", "192.168.1.100");
    env::set_var("SERVER_PORT", "8080");

    let provider1 = create_test_provider().await?;
    let registration1 = EcosystemIntegration::create_service_registration(&provider1);

    env::set_var("SERVER_BIND_ADDRESS", "192.168.1.200");
    env::set_var("SERVER_PORT", "9090");

    let provider2 = create_test_provider().await?;
    let registration2 = EcosystemIntegration::create_service_registration(&provider2);

    // Registrations should use their respective environment configs
    assert!(registration1
        .endpoints
        .primary
        .contains("192.168.1.100:8080"));
    assert!(registration2
        .endpoints
        .primary
        .contains("192.168.1.200:9090"));

    // Cleanup
    env::remove_var("SERVER_BIND_ADDRESS");
    env::remove_var("SERVER_PORT");

    Ok(())
}

#[tokio::test]
async fn test_service_registration_security_config() -> Result<(), Box<dyn std::error::Error>> {
    let provider = create_test_provider().await?;
    let registration = EcosystemIntegration::create_service_registration(&provider);

    // Default security config
    assert!(!registration.security_config.auth_required);
    assert_eq!(registration.security_config.encryption_level, "none");
    assert_eq!(registration.security_config.access_level, "public");

    Ok(())
}

#[tokio::test]
async fn test_service_registration_resource_requirements() -> Result<(), Box<dyn std::error::Error>>
{
    let provider = create_test_provider().await?;
    let registration = EcosystemIntegration::create_service_registration(&provider);

    assert_eq!(registration.resource_requirements.cpu, "1.0");
    assert_eq!(registration.resource_requirements.memory, "512");
    assert_eq!(registration.resource_requirements.storage, "10");
    assert_eq!(registration.resource_requirements.network, "100");

    Ok(())
}

#[tokio::test]
async fn test_service_registration_timestamps() -> Result<(), Box<dyn std::error::Error>> {
    let provider = create_test_provider().await?;
    let registration = EcosystemIntegration::create_service_registration(&provider);

    // Verify registered_at is set to a recent time
    let now = chrono::Utc::now();
    let diff = (now - registration.registered_at).num_seconds();
    assert!(diff < 5, "registered_at should be within 5 seconds of now");

    Ok(())
}
