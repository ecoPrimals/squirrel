//! # Standalone Ecosystem Architecture Demo
//!
//! This example demonstrates the proper standalone architecture for ecoPrimals:
//! - Each primal is completely independent
//! - Communication happens through standardized APIs
//! - Service discovery is handled through Songbird service mesh
//! - No hard dependencies between primals
//! - Dynamic service registration and health monitoring
//!
//! ## Architecture Principles Demonstrated
//! 1. **Pure Service Discovery**: All primals discovered through Songbird
//! 2. **Standardized APIs**: Common HTTP/REST patterns for inter-primal communication
//! 3. **Dynamic Registration**: Services register themselves at runtime
//! 4. **Health Monitoring**: Continuous health checks without tight coupling
//! 5. **Graceful Degradation**: System works even when some primals are offline

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use squirrel_core::ecosystem::{
    initialize_ecosystem_integration, DiscoveredService, EcosystemConfig, EcosystemManager,
    EcosystemPrimalType, EcosystemRegistryConfig, EcosystemServiceRegistration, HealthCheckConfig,
    PrimalApiRequest, PrimalApiResponse, ResourceSpec, SecurityConfig, ServiceCapabilities,
    ServiceEndpoints,
};
use squirrel_core::error::PrimalError;
use squirrel_core::monitoring::metrics::MetricsCollector;
use squirrel_core::primal_provider::SquirrelPrimalProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌟 Standalone Ecosystem Architecture Demo");
    println!("=========================================");
    println!();

    // Step 1: Initialize metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new());

    // Step 2: Configure ecosystem manager (only needs Songbird for service discovery)
    let ecosystem_config = create_ecosystem_config();
    let ecosystem_manager =
        initialize_ecosystem_integration(ecosystem_config, metrics_collector).await?;

    // Step 3: Create and register Squirrel primal
    let squirrel_provider = create_squirrel_provider();
    ecosystem_manager
        .register_squirrel_service(&squirrel_provider)
        .await?;

    println!("✅ Squirrel primal registered with ecosystem");
    println!();

    // Step 4: Demonstrate service discovery
    demonstrate_service_discovery(&ecosystem_manager).await?;

    // Step 5: Demonstrate inter-primal communication
    demonstrate_inter_primal_communication(&ecosystem_manager).await?;

    // Step 6: Demonstrate coordination patterns
    demonstrate_coordination_patterns(&ecosystem_manager).await?;

    // Step 7: Demonstrate health monitoring
    demonstrate_health_monitoring(&ecosystem_manager).await?;

    // Step 8: Demonstrate graceful degradation
    demonstrate_graceful_degradation(&ecosystem_manager).await?;

    // Step 9: Cleanup
    ecosystem_manager.shutdown().await?;
    println!("✅ Ecosystem manager shutdown completed");
    println!();

    println!("🎉 Demo completed successfully!");
    println!("📋 Key takeaways:");
    println!("   - Each primal operates independently");
    println!("   - Communication happens through standardized APIs");
    println!("   - Service discovery eliminates hard dependencies");
    println!("   - System is resilient to individual primal failures");
    println!("   - Health monitoring happens without tight coupling");

    Ok(())
}

/// Create ecosystem configuration focused on service discovery
fn create_ecosystem_config() -> EcosystemConfig {
    EcosystemConfig {
        service_id: "primal-squirrel-demo".to_string(),
        instance_id: Uuid::new_v4().to_string(),
        biome_id: Some("demo-biome".to_string()),
        registry_config: EcosystemRegistryConfig {
            songbird_endpoint: "http://localhost:8000".to_string(), // Only need Songbird for discovery
            retry_config: Default::default(),
            health_config: Default::default(),
            discovery_config: Default::default(),
            security_config: Default::default(),
        },
        endpoints: ServiceEndpoints {
            health: "http://localhost:8080/health".to_string(),
            metrics: "http://localhost:8080/metrics".to_string(),
            admin: "http://localhost:8080/admin".to_string(),
            websocket: Some("ws://localhost:8080/ws".to_string()),
            mcp: "http://localhost:8080/mcp".to_string(),
            ai_coordination: "http://localhost:8080/ai".to_string(),
        },
        resource_requirements: ResourceSpec::default(),
        security_config: SecurityConfig::default(),
        health_check: HealthCheckConfig::default(),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("version".to_string(), "1.0.0".to_string());
            metadata.insert("environment".to_string(), "demo".to_string());
            metadata.insert("region".to_string(), "local".to_string());
            metadata
        },
    }
}

/// Create a mock Squirrel provider for demonstration
fn create_squirrel_provider() -> SquirrelPrimalProvider {
    // This would normally be a real provider with actual endpoints
    // For demo purposes, we'll use a minimal mock
    SquirrelPrimalProvider::new()
}

/// Demonstrate service discovery without hard dependencies
async fn demonstrate_service_discovery(
    ecosystem_manager: &EcosystemManager,
) -> Result<(), PrimalError> {
    println!("🔍 Service Discovery Demo");
    println!("-------------------------");

    // Discover all services in the ecosystem
    match ecosystem_manager.discover_services().await {
        Ok(services) => {
            println!("📡 Discovered {} services in ecosystem:", services.len());
            for service in &services {
                println!(
                    "   - {} ({}): {}",
                    service.service_id,
                    service.primal_type.as_str(),
                    service.endpoint
                );
                println!("     Capabilities: {:?}", service.capabilities);
                println!("     Health: {:?}", service.health_status);
            }
        }
        Err(e) => {
            println!("⚠️  Service discovery failed: {}", e);
            println!("   This is expected in demo mode without real Songbird");
        }
    }

    // Demonstrate discovering specific primal types
    let primal_types = vec![
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::BiomeOS,
    ];

    for primal_type in primal_types {
        let services = ecosystem_manager.find_services_by_type(primal_type).await;
        println!(
            "🔎 {} services found: {}",
            primal_type.as_str(),
            services.len()
        );

        if services.is_empty() {
            println!(
                "   No {} services available (expected in demo mode)",
                primal_type.as_str()
            );
        }
    }

    println!();
    Ok(())
}

/// Demonstrate inter-primal communication through standardized APIs
async fn demonstrate_inter_primal_communication(
    ecosystem_manager: &EcosystemManager,
) -> Result<(), PrimalError> {
    println!("🔄 Inter-Primal Communication Demo");
    println!("----------------------------------");

    // Example: Calling ToadStool for compute delegation
    let toadstool_request = PrimalApiRequest {
        request_id: Uuid::new_v4().to_string(),
        from_primal: EcosystemPrimalType::Squirrel,
        to_primal: EcosystemPrimalType::ToadStool,
        operation: "compute/delegate".to_string(),
        payload: serde_json::json!({
            "task_type": "ai_inference",
            "model": "gpt-4",
            "input": "Hello, world!",
            "priority": "high"
        }),
        headers: {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            headers.insert("X-Request-ID".to_string(), Uuid::new_v4().to_string());
            headers
        },
        timeout: Duration::from_secs(30),
    };

    match ecosystem_manager.call_primal_api(toadstool_request).await {
        Ok(response) => {
            println!("✅ ToadStool API call successful:");
            println!("   Request ID: {}", response.request_id);
            println!("   Success: {}", response.success);
            println!("   Processing time: {:?}", response.processing_time);
            if let Some(data) = response.data {
                println!("   Response data: {}", data);
            }
        }
        Err(e) => {
            println!("⚠️  ToadStool API call failed: {}", e);
            println!("   This is expected in demo mode without real ToadStool service");
        }
    }

    // Example: Calling NestGate for storage operations
    let nestgate_request = PrimalApiRequest {
        request_id: Uuid::new_v4().to_string(),
        from_primal: EcosystemPrimalType::Squirrel,
        to_primal: EcosystemPrimalType::NestGate,
        operation: "storage/store".to_string(),
        payload: serde_json::json!({
            "key": "session_context",
            "value": {"user_id": "demo_user", "context": "demo_context"},
            "ttl": 3600
        }),
        headers: HashMap::new(),
        timeout: Duration::from_secs(10),
    };

    match ecosystem_manager.call_primal_api(nestgate_request).await {
        Ok(response) => {
            println!("✅ NestGate API call successful:");
            println!("   Storage operation completed");
        }
        Err(e) => {
            println!("⚠️  NestGate API call failed: {}", e);
            println!("   This is expected in demo mode without real NestGate service");
        }
    }

    println!();
    Ok(())
}

/// Demonstrate coordination patterns across multiple primals
async fn demonstrate_coordination_patterns(
    ecosystem_manager: &EcosystemManager,
) -> Result<(), PrimalError> {
    println!("🤝 Coordination Patterns Demo");
    println!("-----------------------------");

    // Example: Multi-primal coordination for complex AI task
    let participants = vec![
        EcosystemPrimalType::Squirrel,  // Coordinator
        EcosystemPrimalType::ToadStool, // Compute
        EcosystemPrimalType::NestGate,  // Storage
        EcosystemPrimalType::BearDog,   // Security
    ];

    let context = {
        let mut context = HashMap::new();
        context.insert("task_type".to_string(), "complex_ai_inference".to_string());
        context.insert("user_id".to_string(), "demo_user".to_string());
        context.insert("priority".to_string(), "high".to_string());
        context.insert("security_level".to_string(), "encrypted".to_string());
        context
    };

    match ecosystem_manager
        .start_coordination(participants.clone(), context)
        .await
    {
        Ok(session_id) => {
            println!("✅ Coordination session started: {}", session_id);
            println!(
                "   Participants: {:?}",
                participants.iter().map(|p| p.as_str()).collect::<Vec<_>>()
            );

            // Simulate some work
            println!("   Simulating coordination work...");
            sleep(Duration::from_secs(2)).await;

            // Complete coordination
            match ecosystem_manager
                .complete_coordination(&session_id, true)
                .await
            {
                Ok(_) => {
                    println!("✅ Coordination session completed successfully");
                }
                Err(e) => {
                    println!("⚠️  Coordination completion failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Coordination failed: {}", e);
            println!("   This is expected in demo mode without real Songbird service");
        }
    }

    println!();
    Ok(())
}

/// Demonstrate health monitoring without tight coupling
async fn demonstrate_health_monitoring(
    ecosystem_manager: &EcosystemManager,
) -> Result<(), PrimalError> {
    println!("🏥 Health Monitoring Demo");
    println!("-------------------------");

    // Get ecosystem status
    let status = ecosystem_manager.get_ecosystem_status().await;
    println!("📊 Ecosystem Status:");
    println!("   Overall health: {:.2}%", status.overall_health * 100.0);
    println!("   Registered services: {}", status.registered_services);
    println!("   Discovery cache size: {}", status.discovery_cache_size);
    println!("   Active coordinations: {}", status.active_coordinations);

    if let Some(last_sync) = status.last_full_sync {
        println!(
            "   Last sync: {}",
            last_sync.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    println!("   Primal statuses:");
    for primal_status in &status.primal_statuses {
        println!(
            "   - {}: {:?} (health: {:.2}%)",
            primal_status.primal_type.as_str(),
            primal_status.status,
            primal_status.health_score * 100.0
        );
        println!("     Endpoint: {}", primal_status.endpoint);
        println!("     Capabilities: {:?}", primal_status.capabilities);
    }

    // Get manager status
    let manager_status = ecosystem_manager.get_manager_status().await;
    println!("🔧 Manager Status:");
    println!("   Status: {}", manager_status.status);
    println!("   Error count: {}", manager_status.error_count);
    println!(
        "   Active registrations: {:?}",
        manager_status.active_registrations
    );
    println!(
        "   Health score: {:.2}%",
        manager_status.health_status.health_score * 100.0
    );

    println!();
    Ok(())
}

/// Demonstrate graceful degradation when services are unavailable
async fn demonstrate_graceful_degradation(
    ecosystem_manager: &EcosystemManager,
) -> Result<(), PrimalError> {
    println!("🛡️  Graceful Degradation Demo");
    println!("-----------------------------");

    // Attempt to call a service that might not be available
    let unavailable_request = PrimalApiRequest {
        request_id: Uuid::new_v4().to_string(),
        from_primal: EcosystemPrimalType::Squirrel,
        to_primal: EcosystemPrimalType::ToadStool,
        operation: "compute/health".to_string(),
        payload: serde_json::json!({}),
        headers: HashMap::new(),
        timeout: Duration::from_secs(5),
    };

    match ecosystem_manager.call_primal_api(unavailable_request).await {
        Ok(response) => {
            println!("✅ Service available, response received");
        }
        Err(e) => {
            println!("⚠️  Service unavailable: {}", e);
            println!("   System continues to operate with degraded functionality");
            println!("   This demonstrates proper graceful degradation");
        }
    }

    // Show how the system can still function
    println!("🔄 System continues to operate:");
    println!("   - Local AI processing can continue");
    println!("   - Session management remains active");
    println!("   - Health monitoring continues");
    println!("   - Service discovery attempts retry");

    // Demonstrate alternative strategies
    println!("🔧 Alternative strategies:");
    println!("   - Use cached data when storage is unavailable");
    println!("   - Fall back to local compute when ToadStool is down");
    println!("   - Queue requests for retry when services recover");
    println!("   - Provide user feedback about reduced capabilities");

    println!();
    Ok(())
}

/// Example of how to mock ecosystem services for testing
#[allow(dead_code)]
fn create_mock_ecosystem_services() -> Vec<DiscoveredService> {
    vec![
        DiscoveredService {
            service_id: "toadstool-1".to_string(),
            primal_type: EcosystemPrimalType::ToadStool,
            endpoint: "http://localhost:8001".to_string(),
            health_endpoint: "http://localhost:8001/health".to_string(),
            api_version: "v1".to_string(),
            capabilities: vec!["compute".to_string(), "ai_inference".to_string()],
            metadata: HashMap::new(),
            discovered_at: chrono::Utc::now(),
            last_health_check: None,
            health_status: squirrel_core::ecosystem::ServiceHealthStatus::Healthy,
        },
        DiscoveredService {
            service_id: "nestgate-1".to_string(),
            primal_type: EcosystemPrimalType::NestGate,
            endpoint: "http://localhost:8002".to_string(),
            health_endpoint: "http://localhost:8002/health".to_string(),
            api_version: "v1".to_string(),
            capabilities: vec!["storage".to_string(), "persistence".to_string()],
            metadata: HashMap::new(),
            discovered_at: chrono::Utc::now(),
            last_health_check: None,
            health_status: squirrel_core::ecosystem::ServiceHealthStatus::Healthy,
        },
        DiscoveredService {
            service_id: "beardog-1".to_string(),
            primal_type: EcosystemPrimalType::BearDog,
            endpoint: "http://localhost:8003".to_string(),
            health_endpoint: "http://localhost:8003/health".to_string(),
            api_version: "v1".to_string(),
            capabilities: vec!["security".to_string(), "authentication".to_string()],
            metadata: HashMap::new(),
            discovered_at: chrono::Utc::now(),
            last_health_check: None,
            health_status: squirrel_core::ecosystem::ServiceHealthStatus::Healthy,
        },
    ]
}

/// Example configuration for different environments
#[allow(dead_code)]
fn create_production_config() -> EcosystemConfig {
    EcosystemConfig {
        service_id: "primal-squirrel-prod".to_string(),
        instance_id: Uuid::new_v4().to_string(),
        biome_id: Some("production-biome".to_string()),
        registry_config: EcosystemRegistryConfig {
            songbird_endpoint: "https://songbird.ecoprimals.com".to_string(),
            retry_config: Default::default(),
            health_config: Default::default(),
            discovery_config: Default::default(),
            security_config: squirrel_core::ecosystem::SecurityConfig {
                tls_enabled: true,
                mtls_required: true,
                auth_token: Some("production-token".to_string()),
                trust_domain: "ecoprimals.com".to_string(),
                certificate_path: Some("/etc/ssl/certs/squirrel.crt".to_string()),
                key_path: Some("/etc/ssl/private/squirrel.key".to_string()),
            },
        },
        endpoints: ServiceEndpoints {
            health: "https://squirrel.ecoprimals.com/health".to_string(),
            metrics: "https://squirrel.ecoprimals.com/metrics".to_string(),
            admin: "https://squirrel.ecoprimals.com/admin".to_string(),
            websocket: Some("wss://squirrel.ecoprimals.com/ws".to_string()),
            mcp: "https://squirrel.ecoprimals.com/mcp".to_string(),
            ai_coordination: "https://squirrel.ecoprimals.com/ai".to_string(),
        },
        resource_requirements: ResourceSpec {
            cpu_cores: 4.0,
            memory_mb: 8192,
            disk_mb: 10240,
            network_mbps: 1000,
            gpu: None,
        },
        security_config: SecurityConfig {
            tls_enabled: true,
            mtls_required: true,
            auth_method: "oauth2".to_string(),
            trust_domain: "ecoprimals.com".to_string(),
            cert_path: Some("/etc/ssl/certs/squirrel.crt".to_string()),
            key_path: Some("/etc/ssl/private/squirrel.key".to_string()),
        },
        health_check: HealthCheckConfig {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            retries: 3,
            grace_period: Duration::from_secs(60),
        },
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("environment".to_string(), "production".to_string());
            metadata.insert("region".to_string(), "us-east-1".to_string());
            metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
            metadata
        },
    }
}
