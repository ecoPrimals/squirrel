//! Comprehensive Songbird Universal Patterns Integration Tests
//!
//! This test suite validates the integration between Songbird's orchestration
//! system and universal patterns, ensuring seamless primal management.

use tokio_test;
use std::collections::HashMap;
use std::sync::Arc;

use universal_patterns::traits::{
    PrimalProvider, PrimalContext, PrimalHealth, PrimalRequestType,
    PrimalResponseType, PrimalResult, DynamicPortInfo, 
    PortType, PortStatus, SecurityLevel
};
use universal_patterns::{
    UniversalPrimalConfig, PrimalInstanceConfig
};
use chrono::{Utc, Duration};
use uuid::Uuid;

/// Mock primal provider for testing
struct MockPrimalProvider {
    id: String,
    instance_id: String,
    healthy: bool,
    context: PrimalContext,
}

impl MockPrimalProvider {
    fn new(id: String, instance_id: String, healthy: bool) -> Self {
        Self {
            id,
            instance_id,
            healthy,
            context: PrimalContext::default(),
        }
    }
}

#[tokio::test]
async fn test_universal_primal_system_initialization() {
    let registry = universal_patterns::initialize_primal_system(None).await.unwrap();
    
    // Verify registry was created successfully
    let stats = registry.get_statistics().await;
    assert_eq!(stats.total_primals, 0);
    assert_eq!(stats.total_capabilities, 0);
}

#[tokio::test]
async fn test_development_config_creation() {
    let config = universal_patterns::create_development_config();
    
    // Verify development-specific settings
    assert_eq!(config.multi_instance.max_instances_per_type, 3);
    assert_eq!(config.multi_instance.max_instances_per_user, 2);
    assert!(!config.monitoring.metrics_enabled);
    assert_eq!(config.monitoring.tracing.level, "debug");
    assert_eq!(config.port_management.port_range.start, 8000);
    assert_eq!(config.port_management.port_range.end, 8100);
}

#[tokio::test]
async fn test_production_config_creation() {
    let config = universal_patterns::create_production_config();
    
    // Verify production-specific settings
    assert_eq!(config.multi_instance.max_instances_per_type, 20);
    assert_eq!(config.multi_instance.max_instances_per_user, 10);
    assert!(config.multi_instance.scaling.auto_scaling_enabled);
    assert!(config.multi_instance.failover.enabled);
    assert!(config.monitoring.metrics_enabled);
    assert_eq!(config.monitoring.tracing.level, "info");
    assert_eq!(config.port_management.port_range.start, 9000);
    assert_eq!(config.port_management.port_range.end, 10000);
}

#[tokio::test]
async fn test_primal_specific_config_creation() {
    let config = universal_patterns::create_primal_config(universal_patterns::PrimalType::Security, 5);
    
    // Verify primal-specific settings
    assert_eq!(config.multi_instance.max_instances_per_type, 5);
    assert_eq!(config.monitoring.tracing.level, "info");
    
    let ai_config = universal_patterns::create_primal_config(universal_patterns::PrimalType::AI, 3);
    assert_eq!(ai_config.multi_instance.max_instances_per_type, 3);
    assert_eq!(ai_config.multi_instance.scaling.scale_up_cpu_threshold, 60.0);
    assert_eq!(ai_config.monitoring.tracing.level, "debug");
}

#[tokio::test]
async fn test_primal_context_creation() {
    let context = universal_patterns::create_primal_context(
        "test_user".to_string(),
        "test_device".to_string(),
        SecurityLevel::High,
    );
    
    assert_eq!(context.user_id, "test_user");
    assert_eq!(context.device_id, "test_device");
    assert_eq!(context.security_level, SecurityLevel::High);
    assert_eq!(context.network_location.ip_address, "127.0.0.1");
}

#[tokio::test]
async fn test_primal_registration_and_discovery() {
    let registry = universal_patterns::UniversalPrimalRegistry::new();
    
    // Create a test context
    let context = universal_patterns::create_primal_context(
        "user123".to_string(),
        "device456".to_string(),
        SecurityLevel::Standard,
    );
    
    // Create a mock primal provider
    let capabilities = vec![
        universal_patterns::PrimalCapability::Authentication { 
            methods: vec!["oauth".to_string(), "jwt".to_string()] 
        },
        universal_patterns::PrimalCapability::Encryption { 
            algorithms: vec!["AES256".to_string()] 
        },
    ];
    
    let mock_primal = Arc::new(MockPrimalProvider::new(
        "beardog".to_string(),
        "beardog-user123".to_string(),
        true,
    ));
    
    // Register the primal
    let result = registry.register_primal_for_context(
        mock_primal.clone(),
        context.clone(),
        None,
    ).await;
    
    assert!(result.is_ok());
    
    // Verify registration
    let stats = registry.get_statistics().await;
    assert_eq!(stats.total_primals, 1);
    assert_eq!(stats.total_capabilities, 2);
    
    // Test context-based discovery
    let found_primals = registry.find_for_context(&context).await;
    assert_eq!(found_primals.len(), 1);
    assert_eq!(found_primals[0].instance_id, "beardog-user123");
}

#[tokio::test]
async fn test_capability_based_routing() {
    let registry = universal_patterns::UniversalPrimalRegistry::new();
    
    // Create test contexts
    let context1 = universal_patterns::create_primal_context(
        "user1".to_string(),
        "device1".to_string(),
        SecurityLevel::High,
    );
    
    let context2 = universal_patterns::create_primal_context(
        "user2".to_string(),
        "device2".to_string(),
        SecurityLevel::Standard,
    );
    
    // Create mock primals with different capabilities
    let auth_capability = universal_patterns::PrimalCapability::Authentication { 
        methods: vec!["oauth".to_string()] 
    };
    
    let encrypt_capability = universal_patterns::PrimalCapability::Encryption { 
        algorithms: vec!["AES256".to_string()] 
    };
    
    let primal1 = Arc::new(MockPrimalProvider::new(
        "beardog1".to_string(),
        "beardog-user1".to_string(),
        true,
    ));
    
    let primal2 = Arc::new(MockPrimalProvider::new(
        "beardog2".to_string(),
        "beardog-user2".to_string(),
        true,
    ));
    
    // Register both primals
    registry.register_primal_for_context(primal1, context1.clone(), None).await.unwrap();
    registry.register_primal_for_context(primal2, context2.clone(), None).await.unwrap();
    
    // Test capability-based routing
    let auth_primals = registry.find_by_capability_for_context(&auth_capability, &context1).await;
    assert_eq!(auth_primals.len(), 1);
    assert_eq!(auth_primals[0].instance_id, "beardog-user1");
    
    let encrypt_primals = registry.find_by_capability_for_context(&encrypt_capability, &context2).await;
    assert_eq!(encrypt_primals.len(), 1);
    assert_eq!(encrypt_primals[0].instance_id, "beardog-user2");
    
    // Test cross-context (should not find primals)
    let cross_context_primals = registry.find_by_capability_for_context(&auth_capability, &context2).await;
    assert_eq!(cross_context_primals.len(), 0);
}

#[tokio::test]
async fn test_multi_instance_support() {
    let registry = universal_patterns::UniversalPrimalRegistry::new();
    
    // Create multiple instances for the same user
    let context = universal_patterns::create_primal_context(
        "multi_user".to_string(),
        "device1".to_string(),
        SecurityLevel::Standard,
    );
    
    let capabilities = vec![
        universal_patterns::PrimalCapability::Authentication { 
            methods: vec!["oauth".to_string()] 
        }
    ];
    
    // Create multiple instances
    for i in 0..3 {
        let instance_id = format!("beardog-instance-{}", i);
        let primal = Arc::new(MockPrimalProvider::new(
            "beardog".to_string(),
            instance_id,
            true,
        ));
        
        registry.register_primal_for_context(primal, context.clone(), None).await.unwrap();
    }
    
    // Verify multiple instances
    let user_primals = registry.get_instances_for_user("multi_user").await;
    assert_eq!(user_primals.len(), 3);
    
    let security_primals = registry.get_instances_by_type(universal_patterns::PrimalType::Security).await;
    assert_eq!(security_primals.len(), 3);
}

#[tokio::test]
async fn test_request_routing() {
    let registry = universal_patterns::UniversalPrimalRegistry::new();
    
    // Create test context
    let context = universal_patterns::create_primal_context(
        "route_user".to_string(),
        "route_device".to_string(),
        SecurityLevel::Standard,
    );
    
    // Create and register a mock primal
    let mock_primal = Arc::new(MockPrimalProvider::new(
        "test_primal".to_string(),
        "test-instance".to_string(),
        true,
    ));
    
    registry.register_primal_for_context(mock_primal, context.clone(), None).await.unwrap();
    
    // Create a test request
    let request = universal_patterns::PrimalRequest {
        id: Uuid::new_v4(),
        request_type: universal_patterns::PrimalRequestType::Authenticate,
        payload: HashMap::new(),
        timestamp: Utc::now(),
        context: Some("test_context".to_string()),
        priority: Some(1),
        security_level: Some("standard".to_string()),
    };
    
    // Test context-based routing
    let response = registry.route_request_with_context(request.clone(), &context).await;
    assert!(response.is_ok());
    
    let response = response.unwrap();
    assert_eq!(response.request_id, request.id);
    assert_eq!(response.response_type, universal_patterns::PrimalResponseType::Authentication);
    assert!(response.success);
    
    // Test instance-specific routing
    let response = registry.route_request_to_instance(request.clone(), "test-instance").await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_health_monitoring() {
    let registry = universal_patterns::UniversalPrimalRegistry::new();
    
    // Create contexts for different health states
    let healthy_context = universal_patterns::create_primal_context(
        "healthy_user".to_string(),
        "device1".to_string(),
        SecurityLevel::Standard,
    );
    
    let degraded_context = universal_patterns::create_primal_context(
        "degraded_user".to_string(),
        "device2".to_string(),
        SecurityLevel::Standard,
    );
    
    // Create primals with different health states
    let mut healthy_primal = MockPrimalProvider::new(
        "healthy_primal".to_string(),
        "healthy-instance".to_string(),
        true,
    );
    
    let mut degraded_primal = MockPrimalProvider::new(
        "degraded_primal".to_string(),
        "degraded-instance".to_string(),
        false,
    );
    degraded_primal.healthy = false;
    
    // Register primals
    registry.register_primal_for_context(
        Arc::new(healthy_primal), 
        healthy_context, 
        None
    ).await.unwrap();
    
    registry.register_primal_for_context(
        Arc::new(degraded_primal), 
        degraded_context, 
        None
    ).await.unwrap();
    
    // Test health check
    let health_results = registry.health_check_all().await;
    assert_eq!(health_results.len(), 2);
    
    // Find healthy and degraded instances
    let healthy_result = health_results.iter()
        .find(|(id, _)| id == "healthy-instance")
        .unwrap();
    let degraded_result = health_results.iter()
        .find(|(id, _)| id == "degraded-instance")
        .unwrap();
    
    assert_eq!(healthy_result.1, universal_patterns::PrimalHealth::Healthy);
    assert!(matches!(degraded_result.1, universal_patterns::PrimalHealth::Degraded { .. }));
}

#[tokio::test]
async fn test_port_management() {
    let registry = universal_patterns::UniversalPrimalRegistry::new();
    
    let context = universal_patterns::create_primal_context(
        "port_user".to_string(),
        "port_device".to_string(),
        SecurityLevel::Standard,
    );
    
    let port_info = universal_patterns::DynamicPortInfo {
        assigned_port: 8080,
        port_type: universal_patterns::PortType::Http,
        status: universal_patterns::PortStatus::Active,
        assigned_at: Utc::now(),
        lease_duration: Duration::hours(1),
    };
    
    let mock_primal = Arc::new(MockPrimalProvider::new(
        "port_primal".to_string(),
        "port-instance".to_string(),
        true,
    ));
    
    // Register primal with port info
    registry.register_primal_for_context(
        mock_primal, 
        context, 
        Some(port_info.clone())
    ).await.unwrap();
    
    // Test port info retrieval
    let retrieved_port_info = registry.get_port_info("port-instance").await;
    assert!(retrieved_port_info.is_some());
    
    let retrieved_port_info = retrieved_port_info.unwrap();
    assert_eq!(retrieved_port_info.assigned_port, 8080);
    assert_eq!(retrieved_port_info.port_type, universal_patterns::PortType::Http);
    assert_eq!(retrieved_port_info.status, universal_patterns::PortStatus::Active);
    
    // Test port info update
    let updated_port_info = universal_patterns::DynamicPortInfo {
        assigned_port: 8081,
        port_type: universal_patterns::PortType::Https,
        status: universal_patterns::PortStatus::Active,
        assigned_at: Utc::now(),
        lease_duration: Duration::hours(2),
    };
    
    registry.update_port_info("port-instance", updated_port_info).await.unwrap();
    
    let final_port_info = registry.get_port_info("port-instance").await.unwrap();
    assert_eq!(final_port_info.assigned_port, 8081);
    assert_eq!(final_port_info.port_type, universal_patterns::PortType::Https);
}

#[tokio::test]
async fn test_enhanced_registry_statistics() {
    let registry = universal_patterns::UniversalPrimalRegistry::new();
    
    // Create multiple primals with different types and users
    let contexts = vec![
        universal_patterns::create_primal_context("user1".to_string(), "device1".to_string(), SecurityLevel::Standard),
        universal_patterns::create_primal_context("user1".to_string(), "device2".to_string(), SecurityLevel::High),
        universal_patterns::create_primal_context("user2".to_string(), "device1".to_string(), SecurityLevel::Standard),
    ];
    
    let primal_types = vec![
        universal_patterns::PrimalType::Security,
        universal_patterns::PrimalType::AI,
        universal_patterns::PrimalType::Storage,
    ];
    
    // Register primals
    for (i, (context, primal_type)) in contexts.iter().zip(primal_types.iter()).enumerate() {
        let instance_id = format!("instance-{}", i);
        let primal = Arc::new(MockPrimalProvider::new(
            format!("primal-{}", i),
            instance_id,
            true,
        ));
        
        registry.register_primal_for_context(primal, context.clone(), None).await.unwrap();
    }
    
    // Test enhanced statistics
    let stats = registry.get_enhanced_statistics().await;
    assert_eq!(stats.total_instances, 3);
    assert_eq!(stats.total_users, 2);
    assert_eq!(stats.instances_by_user.get("user1").unwrap(), &2);
    assert_eq!(stats.instances_by_user.get("user2").unwrap(), &1);
    assert_eq!(stats.instances_by_type.get(&universal_patterns::PrimalType::Security).unwrap(), &1);
    assert_eq!(stats.instances_by_type.get(&universal_patterns::PrimalType::AI).unwrap(), &1);
    assert_eq!(stats.instances_by_type.get(&universal_patterns::PrimalType::Storage).unwrap(), &1);
}

#[tokio::test]
async fn test_instance_unregistration() {
    let registry = universal_patterns::UniversalPrimalRegistry::new();
    
    let context = universal_patterns::create_primal_context(
        "unregister_user".to_string(),
        "unregister_device".to_string(),
        SecurityLevel::Standard,
    );
    
    let mock_primal = Arc::new(MockPrimalProvider::new(
        "unregister_primal".to_string(),
        "unregister-instance".to_string(),
        true,
    ));
    
    // Register primal
    registry.register_primal_for_context(mock_primal, context.clone(), None).await.unwrap();
    
    // Verify registration
    let stats = registry.get_statistics().await;
    assert_eq!(stats.total_primals, 1);
    assert_eq!(stats.total_capabilities, 1);
    
    // Unregister primal
    registry.unregister_instance("unregister-instance").await.unwrap();
    
    // Verify unregistration
    let stats = registry.get_statistics().await;
    assert_eq!(stats.total_primals, 0);
    assert_eq!(stats.total_capabilities, 0);
    
    // Verify context-based search returns empty
    let found_primals = registry.find_for_context(&context).await;
    assert_eq!(found_primals.len(), 0);
}

#[tokio::test]
async fn test_configuration_validation() {
    let mut config = universal_patterns::UniversalPrimalConfig::default();
    
    // Test valid configuration
    assert!(config.validate().is_ok());
    
    // Test invalid port range
    config.port_management.port_range.start = 9000;
    config.port_management.port_range.end = 8000;
    assert!(config.validate().is_err());
    
    // Fix port range
    config.port_management.port_range.start = 8000;
    config.port_management.port_range.end = 9000;
    
    // Test invalid instance limits
    config.multi_instance.max_instances_per_type = 0;
    assert!(config.validate().is_err());
    
    config.multi_instance.max_instances_per_type = 10;
    config.multi_instance.max_instances_per_user = 0;
    assert!(config.validate().is_err());
    
    // Fix instance limits
    config.multi_instance.max_instances_per_user = 5;
    
    // Test invalid scaling configuration
    config.multi_instance.scaling.auto_scaling_enabled = true;
    config.multi_instance.scaling.min_instances = 10;
    config.multi_instance.scaling.max_instances = 5;
    assert!(config.validate().is_err());
    
    // Fix scaling configuration
    config.multi_instance.scaling.min_instances = 2;
    config.multi_instance.scaling.max_instances = 10;
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_primal_instance_config_builder() {
    let config = universal_patterns::PrimalInstanceConfig::new(
        "http://localhost:8080".to_string(),
        "test-instance".to_string(),
        "test-user".to_string(),
        "test-device".to_string(),
    )
    .with_api_key("test-key".to_string())
    .with_security_level("high".to_string())
    .with_header("X-Custom-Header".to_string(), "test-value".to_string());
    
    assert_eq!(config.base_url, "http://localhost:8080");
    assert_eq!(config.instance_id, "test-instance");
    assert_eq!(config.user_id, "test-user");
    assert_eq!(config.device_id, "test-device");
    assert_eq!(config.api_key, Some("test-key".to_string()));
    assert_eq!(config.security_level, "high");
    assert_eq!(config.headers.get("X-Custom-Header"), Some(&"test-value".to_string()));
}

#[test]
fn test_version_info() {
    let version = universal_patterns::version();
    assert!(!version.is_empty());
    assert!(version.contains('.'));
} 