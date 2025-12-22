//! Additional comprehensive tests for service discovery
//!
//! These tests expand coverage of service discovery functionality

use super::*;
use crate::service_discovery::{
    HealthStatus, InMemoryServiceDiscovery, ServiceDefinition, ServiceDiscoveryClient,
    ServiceEndpoint, ServiceQuery, ServiceRegistry, ServiceType,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_service_lifecycle_complete() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());

    // Register
    let service = ServiceDefinition::new(
        "lifecycle-test".to_string(),
        "Lifecycle Test".to_string(),
        ServiceType::AI,
        vec![ServiceEndpoint::new(
            "http://localhost:8080".to_string(),
            "http".to_string(),
            8080,
        )],
    );

    discovery.register_service(service.clone()).await.unwrap();

    // Verify registered
    let found = discovery.get_service("lifecycle-test").await.unwrap();
    assert!(found.is_some());

    // Update heartbeat
    discovery
        .update_service_heartbeat("lifecycle-test")
        .await
        .unwrap();

    // Deregister
    discovery
        .deregister_service("lifecycle-test")
        .await
        .unwrap();

    // Verify removed
    let found = discovery.get_service("lifecycle-test").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_multiple_services_same_type() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());

    for i in 1..=5 {
        let service = ServiceDefinition::new(
            format!("ai-service-{}", i),
            format!("AI Service {}", i),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                format!("http://localhost:808{}", i),
                "http".to_string(),
                8080 + i,
            )],
        );
        discovery.register_service(service).await.unwrap();
    }

    let services = discovery.get_active_services().await.unwrap();
    assert_eq!(services.len(), 5);
}

#[tokio::test]
async fn test_capability_based_filtering() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());
    let client = ServiceDiscoveryClient::new(discovery.clone());

    // Service with chat capability
    let service1 = ServiceDefinition::new(
        "chat-1".to_string(),
        "Chat 1".to_string(),
        ServiceType::AI,
        vec![],
    )
    .with_capability("chat".to_string());

    // Service with vision capability
    let service2 = ServiceDefinition::new(
        "vision-1".to_string(),
        "Vision 1".to_string(),
        ServiceType::AI,
        vec![],
    )
    .with_capability("vision".to_string());

    discovery.register_service(service1).await.unwrap();
    discovery.register_service(service2).await.unwrap();

    // Find chat service
    let chat_service = client.find_service_by_capability("chat").await.unwrap();
    assert!(chat_service.is_some());
    assert_eq!(chat_service.unwrap().id, "chat-1");

    // Find vision service
    let vision_service = client.find_service_by_capability("vision").await.unwrap();
    assert!(vision_service.is_some());
    assert_eq!(vision_service.unwrap().id, "vision-1");
}

#[tokio::test]
async fn test_service_metadata() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());

    let service = ServiceDefinition::new(
        "metadata-test".to_string(),
        "Metadata Test".to_string(),
        ServiceType::AI,
        vec![],
    )
    .with_metadata("region".to_string(), "us-east-1".to_string())
    .with_metadata("version".to_string(), "1.0.0".to_string());

    discovery.register_service(service).await.unwrap();

    let found = discovery
        .get_service("metadata-test")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.metadata.get("region").unwrap(), "us-east-1");
    assert_eq!(found.metadata.get("version").unwrap(), "1.0.0");
}

#[tokio::test]
async fn test_service_query_complex() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());

    // Register various services
    for i in 1..=10 {
        let service = ServiceDefinition::new(
            format!("service-{}", i),
            format!("Service {}", i),
            if i % 2 == 0 {
                ServiceType::AI
            } else {
                ServiceType::Compute
            },
            vec![],
        )
        .with_capability(if i % 3 == 0 { "advanced" } else { "basic" }.to_string());

        discovery.register_service(service).await.unwrap();
    }

    // Query AI services with advanced capability
    let query = ServiceQuery::new()
        .with_service_type(ServiceType::AI)
        .with_capability("advanced".to_string())
        .with_health_status(HealthStatus::Healthy)
        .limit(5);

    let results = discovery.discover_services(query).await.unwrap();
    assert!(results.len() > 0);

    // Verify all results are AI and have advanced capability
    for service in results {
        assert_eq!(service.service_type, ServiceType::AI);
        assert!(service.capabilities.contains(&"advanced".to_string()));
    }
}

#[tokio::test]
async fn test_service_registry_heartbeat() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());
    let registry = ServiceRegistry::new(discovery.clone());

    let service = ServiceDefinition::new(
        "heartbeat-test".to_string(),
        "Heartbeat Test".to_string(),
        ServiceType::AI,
        vec![],
    );

    registry.register_local_service(service).await.unwrap();

    // Send manual heartbeat
    registry.send_heartbeats().await.unwrap();

    // Verify service is still healthy
    let found = discovery.get_service("heartbeat-test").await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn test_service_deregistration_cascade() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());
    let registry = ServiceRegistry::new(discovery.clone());

    // Register multiple services
    for i in 1..=3 {
        let service = ServiceDefinition::new(
            format!("cascade-{}", i),
            format!("Cascade {}", i),
            ServiceType::AI,
            vec![],
        );
        registry.register_local_service(service).await.unwrap();
    }

    assert_eq!(registry.get_local_services().await.len(), 3);

    // Deregister one
    registry
        .deregister_local_service("cascade-1")
        .await
        .unwrap();
    assert_eq!(registry.get_local_services().await.len(), 2);

    // Deregister all
    registry.deregister_all_services().await.unwrap();
    assert_eq!(registry.get_local_services().await.len(), 0);
}

#[tokio::test]
async fn test_service_endpoint_validation() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());

    let endpoint1 = ServiceEndpoint::new(
        "http://localhost:8080".to_string(),
        "http".to_string(),
        8080,
    );

    let endpoint2 = ServiceEndpoint::new(
        "https://api.example.com".to_string(),
        "https".to_string(),
        443,
    );

    let service = ServiceDefinition::new(
        "multi-endpoint".to_string(),
        "Multi Endpoint".to_string(),
        ServiceType::Gateway,
        vec![endpoint1, endpoint2],
    );

    discovery.register_service(service).await.unwrap();

    let found = discovery
        .get_service("multi-endpoint")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.endpoints.len(), 2);
}

#[tokio::test]
async fn test_service_type_custom() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());

    let service = ServiceDefinition::new(
        "custom-service".to_string(),
        "Custom Service".to_string(),
        ServiceType::Custom("analytics".to_string()),
        vec![],
    );

    discovery.register_service(service).await.unwrap();

    let found = discovery
        .get_service("custom-service")
        .await
        .unwrap()
        .unwrap();
    match found.service_type {
        ServiceType::Custom(name) => assert_eq!(name, "analytics"),
        _ => panic!("Expected custom service type"),
    }
}

#[tokio::test]
async fn test_service_stats() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());
    let client = ServiceDiscoveryClient::new(discovery.clone());

    // Register services of different types
    for service_type in [ServiceType::AI, ServiceType::Compute, ServiceType::Storage] {
        let service = ServiceDefinition::new(
            format!("{:?}-service", service_type),
            format!("{:?} Service", service_type),
            service_type,
            vec![],
        );
        discovery.register_service(service).await.unwrap();
    }

    let stats = client.get_service_stats().await.unwrap();
    assert!(stats.total_services >= 3);
}

#[tokio::test]
async fn test_concurrent_registration() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());

    let mut handles = vec![];
    for i in 0..10 {
        let disc = discovery.clone();
        let handle = tokio::spawn(async move {
            let service = ServiceDefinition::new(
                format!("concurrent-{}", i),
                format!("Concurrent {}", i),
                ServiceType::AI,
                vec![],
            );
            disc.register_service(service).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let services = discovery.get_active_services().await.unwrap();
    assert_eq!(services.len(), 10);
}

#[tokio::test]
async fn test_service_query_limit() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());

    // Register 20 services
    for i in 1..=20 {
        let service = ServiceDefinition::new(
            format!("limit-test-{}", i),
            format!("Limit Test {}", i),
            ServiceType::AI,
            vec![],
        );
        discovery.register_service(service).await.unwrap();
    }

    // Query with limit of 5
    let query = ServiceQuery::new()
        .with_service_type(ServiceType::AI)
        .limit(5);

    let results = discovery.discover_services(query).await.unwrap();
    assert_eq!(results.len(), 5);
}

#[tokio::test]
async fn test_service_update_health_status() {
    let discovery = Arc::new(InMemoryServiceDiscovery::new());

    let service = ServiceDefinition::new(
        "health-test".to_string(),
        "Health Test".to_string(),
        ServiceType::AI,
        vec![],
    );

    discovery.register_service(service).await.unwrap();

    // Update heartbeat (marks as healthy)
    discovery
        .update_service_heartbeat("health-test")
        .await
        .unwrap();

    let found = discovery.get_service("health-test").await.unwrap().unwrap();
    assert_eq!(found.health_status, HealthStatus::Healthy);
}
