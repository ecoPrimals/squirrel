// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Universal System Integration Demo
//!
//! This example demonstrates how the universal system replaces hardcoded
//! primal endpoints with dynamic service discovery and configuration.
//!
//! ## What this demo shows:
//!
//! 1. Configuration loading from environment variables
//! 2. Dynamic service discovery and registration
//! 3. Universal API usage for querying primals
//! 4. Health checking and service monitoring
//! 5. Load balancing and failover capabilities
//! 6. Real-time service updates and heartbeats
//!
//! ## Running the demo:
//!
//! ```bash
//! # Set up environment variables
//! export SERVICE_AI_ENDPOINT="http://localhost:8080"
//! export SERVICE_AI_CAPABILITIES="chat,search,analysis"
//! export SERVICE_AI_WEIGHT="0.8"
//! export SERVICE_AI_REQUIRED="true"
//!
//! export SERVICE_COMPUTE_ENDPOINT="http://localhost:8081"
//! export SERVICE_COMPUTE_CAPABILITIES="execution,processing"
//! export SERVICE_COMPUTE_WEIGHT="0.6"
//!
//! export SERVICE_STORAGE_ENDPOINT="http://localhost:8082"
//! export SERVICE_STORAGE_CAPABILITIES="data,persistence"
//! export SERVICE_STORAGE_WEIGHT="0.9"
//!
//! # Run the demo
//! cargo run --example universal_system_demo
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, interval};
use tracing::{info, warn, error, debug};

// Import the universal system components
use squirrel::universal::{
    UniversalPrimalProvider, UniversalApi, PrimalQuery, PrimalInfo, HealthStatus
};
use squirrel_config::universal::{
    UniversalServiceConfig, ServiceConfig, ServiceConfigBuilder
};
use squirrel_core::service_discovery::{
    InMemoryServiceDiscovery, ServiceDiscovery, ServiceDefinition, ServiceType,
    ServiceEndpoint, HealthStatus as ServiceHealthStatus, ServiceRegistry,
    ServiceDiscoveryClient
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Universal System Integration Demo");
    
    // Demonstrate different configuration methods
    demonstrate_configuration_methods().await?;
    
    // Show service discovery capabilities
    demonstrate_service_discovery().await?;
    
    // Show universal API usage
    demonstrate_universal_api().await?;
    
    // Show health checking and monitoring
    demonstrate_health_monitoring().await?;
    
    // Show load balancing and failover
    demonstrate_load_balancing().await?;
    
    // Show real-time updates
    demonstrate_real_time_updates().await?;
    
    info!("✅ Universal System Integration Demo completed successfully!");
    Ok(())
}

/// Demonstrate different configuration methods
async fn demonstrate_configuration_methods() -> Result<(), Box<dyn std::error::Error>> {
    info!("📋 Demonstrating configuration methods...");
    
    // Method 1: Load from environment variables
    println!("\n1. Loading from environment variables:");
    match UniversalServiceConfig::from_env() {
        Ok(config) => {
            let summary = config.summary();
            println!("   ✅ Loaded {} services from environment", summary.total_services);
            println!("   📊 Discovery endpoints: {}", summary.discovery_endpoints);
            println!("   🔒 TLS enabled: {}", summary.tls_enabled);
            println!("   ⚖️  Load balancing: {:?}", summary.load_balancing_strategy);
        }
        Err(e) => {
            println!("   ℹ️  No environment config found: {}", e);
        }
    }
    
    // Method 2: Programmatic configuration
    println!("\n2. Programmatic configuration:");
    
    // Discovery endpoint configurable via EXAMPLE_DISCOVERY_ENDPOINT or EXAMPLE_DISCOVERY_PORT
    let discovery_endpoint = std::env::var("EXAMPLE_DISCOVERY_ENDPOINT")
        .unwrap_or_else(|_| {
            let port = std::env::var("EXAMPLE_DISCOVERY_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(8500);
            format!("http://localhost:{}", port)
        });
    
    let config = UniversalServiceConfig::new()
        .add_discovery_endpoint(discovery_endpoint)?
        .with_default_timeout(Duration::from_secs(30))?
        .add_service(
            "ai-service".to_string(),
            ServiceConfigBuilder::new()
                .add_endpoint("http://localhost:8080".to_string())?
                .add_endpoint("http://localhost:8081".to_string())?
                .with_timeout(Duration::from_secs(10))?
                .add_capability("chat".to_string())
                .add_capability("search".to_string())
                .add_capability("analysis".to_string())
                .with_weight(0.8)?
                .add_tag("ai".to_string())
                .add_tag("ml".to_string())
                .with_priority(1)
                .with_required(true)
                .with_health_check_url("http://localhost:8080/health".to_string())?
                .add_metadata("version".to_string(), "1.0.0".to_string())
                .add_metadata("team".to_string(), "ai-team".to_string())
                .build()?,
        )?
        .add_service(
            "compute-service".to_string(),
            ServiceConfigBuilder::new()
                .add_endpoint("http://localhost:8082".to_string())?
                .add_capability("execution".to_string())
                .add_capability("processing".to_string())
                .with_weight(0.6)?
                .add_tag("compute".to_string())
                .with_priority(2)
                .build()?,
        )?
        .add_service(
            "storage-service".to_string(),
            ServiceConfigBuilder::new()
                .add_endpoint("http://localhost:8083".to_string())?
                .add_capability("data".to_string())
                .add_capability("persistence".to_string())
                .with_weight(0.9)?
                .add_tag("storage".to_string())
                .with_priority(3)
                .build()?,
        )?
        .build()?;
    
    let summary = config.summary();
    println!("   ✅ Created {} services programmatically", summary.total_services);
    println!("   🔧 Required services: {}", summary.required_services);
    println!("   🔵 Optional services: {}", summary.optional_services);
    
    // Validate configuration
    config.validate()?;
    println!("   ✅ Configuration validation passed");
    
    Ok(())
}

/// Demonstrate service discovery capabilities
async fn demonstrate_service_discovery() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Demonstrating service discovery...");
    
    // Create service discovery backend
    let discovery = Arc::new(InMemoryServiceDiscovery::new());
    
    // Register some services
    let ai_service = ServiceDefinition::new(
        "ai-service-1".to_string(),
        "AI Service Instance 1".to_string(),
        ServiceType::AI,
        vec![
            ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            ).as_primary().with_weight(0.8),
            ServiceEndpoint::new(
                "http://localhost:8081".to_string(),
                "http".to_string(),
                8081,
            ).with_weight(0.2),
        ],
    )
    .with_capability("chat".to_string())
    .with_capability("search".to_string())
    .with_metadata("version".to_string(), "1.0.0".to_string())
    .with_metadata("region".to_string(), "us-west".to_string());
    
    let compute_service = ServiceDefinition::new(
        "compute-service-1".to_string(),
        "Compute Service Instance 1".to_string(),
        ServiceType::Compute,
        vec![
            ServiceEndpoint::new(
                "http://localhost:8082".to_string(),
                "http".to_string(),
                8082,
            ).as_primary(),
        ],
    )
    .with_capability("execution".to_string())
    .with_capability("processing".to_string())
    .with_metadata("version".to_string(), "2.0.0".to_string());
    
    discovery.register_service(ai_service).await?;
    discovery.register_service(compute_service).await?;
    
    println!("\n📊 Service Discovery Status:");
    
    // Get all services
    let all_services = discovery.get_active_services().await?;
    println!("   📝 Total active services: {}", all_services.len());
    
    for service in &all_services {
        println!("   🔹 {} ({})", service.name, service.service_type.as_str());
        println!("      📍 Endpoints: {}", service.endpoints.len());
        println!("      🎯 Capabilities: {}", service.capabilities.join(", "));
        println!("      🏷️  Health: {}", service.health_status.as_str());
    }
    
    // Test service queries
    println!("\n🔍 Service Queries:");
    
    // Find AI services
    let ai_services = discovery.get_services_by_type(ServiceType::AI).await?;
    println!("   🤖 AI services found: {}", ai_services.len());
    
    // Find services with chat capability
    let chat_services = discovery.get_services_by_capability("chat").await?;
    println!("   💬 Chat-capable services: {}", chat_services.len());
    
    // Get service statistics
    let stats = discovery.get_service_stats().await?;
    println!("\n📈 Service Statistics:");
    println!("   📊 Total services: {}", stats.total_services);
    println!("   ✅ Healthy: {}", stats.healthy_services);
    println!("   ⚠️  Degraded: {}", stats.degraded_services);
    println!("   ❌ Unhealthy: {}", stats.unhealthy_services);
    println!("   🔝 Common capabilities: {:?}", stats.common_capabilities);
    
    Ok(())
}

/// Demonstrate universal API usage
async fn demonstrate_universal_api() -> Result<(), Box<dyn std::error::Error>> {
    info!("🌐 Demonstrating Universal API...");
    
    // Create configuration with multiple services
    let config = UniversalServiceConfig::new()
        .add_service(
            "ai-service".to_string(),
            ServiceConfigBuilder::new()
                .add_endpoint("http://localhost:8080".to_string())?
                .add_capability("chat".to_string())
                .add_capability("search".to_string())
                .with_weight(0.8)?
                .add_tag("ai".to_string())
                .add_metadata("type".to_string(), "ai".to_string())
                .build()?,
        )?
        .add_service(
            "compute-service".to_string(),
            ServiceConfigBuilder::new()
                .add_endpoint("http://localhost:8082".to_string())?
                .add_capability("execution".to_string())
                .with_weight(0.6)?
                .add_tag("compute".to_string())
                .add_metadata("type".to_string(), "compute".to_string())
                .build()?,
        )?
        .build()?;
    
    // Create universal primal provider
    let provider = Arc::new(UniversalPrimalProvider::new(config));
    
    // Create universal API
    let api = UniversalApi::new(provider.clone());
    
    println!("\n🌐 Universal API Operations:");
    
    // Get all primals
    let primals = api.get_primals().await?;
    println!("   📝 Total primals: {}", primals.len());
    
    for primal in &primals {
        println!("   🔹 {} ({})", primal.name, primal.service_type);
        println!("      📍 Endpoint: {}", primal.primary_endpoint);
        println!("      🎯 Capabilities: {}", primal.capabilities.join(", "));
        println!("      🏷️  Tags: {}", primal.tags.join(", "));
        if let Some(weight) = primal.weight {
            println!("      ⚖️  Weight: {:.1}", weight);
        }
    }
    
    // Get specific primal
    if let Some(ai_primal) = api.get_primal("ai-service").await? {
        println!("\n🤖 AI Service Details:");
        println!("   📛 Name: {}", ai_primal.name);
        println!("   🔗 Primary Endpoint: {}", ai_primal.primary_endpoint);
        println!("   📝 All Endpoints: {}", ai_primal.endpoints.join(", "));
        println!("   🎯 Capabilities: {}", ai_primal.capabilities.join(", "));
        println!("   🏷️  Tags: {}", ai_primal.tags.join(", "));
        println!("   📊 Metadata: {:?}", ai_primal.metadata);
    }
    
    // Test querying with different criteria
    println!("\n🔍 Query Tests:");
    
    // Query by capability
    let chat_query = PrimalQuery::new("".to_string())
        .with_capability("chat".to_string());
    
    let chat_primals = provider.query_primal(chat_query).await?;
    println!("   💬 Chat-capable primals: {}", chat_primals.len());
    
    // Query by metadata
    let ai_query = PrimalQuery::new("".to_string())
        .with_metadata("type".to_string(), "ai".to_string());
    
    let ai_primals = provider.query_primal(ai_query).await?;
    println!("   🤖 AI-type primals: {}", ai_primals.len());
    
    // Query by service name
    let specific_query = PrimalQuery::new("ai-service".to_string());
    let specific_primals = provider.query_primal(specific_query).await?;
    println!("   📍 Specific service primals: {}", specific_primals.len());
    
    Ok(())
}

/// Demonstrate health checking and monitoring
async fn demonstrate_health_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    info!("🏥 Demonstrating health monitoring...");
    
    let config = UniversalServiceConfig::new()
        .add_service(
            "monitored-service".to_string(),
            ServiceConfigBuilder::new()
                .add_endpoint("http://localhost:8080".to_string())?
                .with_health_check_url("http://localhost:8080/health".to_string())?
                .add_capability("monitoring".to_string())
                .build()?,
        )?
        .build()?;
    
    let provider = Arc::new(UniversalPrimalProvider::new(config));
    let api = UniversalApi::new(provider.clone());
    
    println!("\n🏥 Health Monitoring:");
    
    // Check service health
    if let Some(health) = api.check_health("monitored-service").await? {
        println!("   📊 Service: {}", health.service_name);
        println!("   🏥 Status: {}", health.status);
        println!("   🕐 Last Check: {}", health.last_check);
        println!("   📍 Endpoints: {}", health.endpoints.join(", "));
        
        if let Some(response_time) = health.response_time {
            println!("   ⏱️  Response Time: {}ms", response_time);
        }
        
        if let Some(error) = health.error {
            println!("   ❌ Error: {}", error);
        }
    }
    
    // Demonstrate health status changes
    println!("\n🔄 Health Status Simulation:");
    
    // Simulate different health statuses
    let statuses = vec![
        ("healthy", "All systems operational"),
        ("degraded", "Some issues detected"),
        ("unhealthy", "Service unavailable"),
        ("healthy", "Service recovered"),
    ];
    
    for (status, message) in statuses {
        println!("   🏥 Simulating status: {} - {}", status, message);
        sleep(Duration::from_millis(500)).await;
    }
    
    Ok(())
}

/// Demonstrate load balancing and failover
async fn demonstrate_load_balancing() -> Result<(), Box<dyn std::error::Error>> {
    info!("⚖️ Demonstrating load balancing...");
    
    let config = UniversalServiceConfig::new()
        .add_service(
            "load-balanced-service".to_string(),
            ServiceConfigBuilder::new()
                .add_endpoint("http://localhost:8080".to_string())?
                .add_endpoint("http://localhost:8081".to_string())?
                .add_endpoint("http://localhost:8082".to_string())?
                .add_capability("load-balancing".to_string())
                .with_weight(0.8)?
                .build()?,
        )?
        .add_service(
            "backup-service".to_string(),
            ServiceConfigBuilder::new()
                .add_endpoint("http://localhost:8083".to_string())?
                .add_capability("load-balancing".to_string())
                .with_weight(0.2)?
                .build()?,
        )?
        .build()?;
    
    let provider = Arc::new(UniversalPrimalProvider::new(config));
    
    println!("\n⚖️ Load Balancing Demonstration:");
    
    // Query services for load balancing
    let query = PrimalQuery::new("".to_string())
        .with_capability("load-balancing".to_string());
    
    let services = provider.query_primal(query).await?;
    println!("   📊 Available services: {}", services.len());
    
    for service in &services {
        println!("   🔹 {} (weight: {:?})", service.name, service.weight);
        println!("      📍 Endpoints: {}", service.endpoints.join(", "));
    }
    
    // Simulate load balancing requests
    println!("\n🔄 Load Balancing Simulation:");
    
    for i in 1..=10 {
        let selected = &services[i % services.len()];
        let endpoint = &selected.endpoints[i % selected.endpoints.len()];
        println!("   📤 Request {}: {} -> {}", i, selected.name, endpoint);
        sleep(Duration::from_millis(100)).await;
    }
    
    // Demonstrate failover
    println!("\n🔄 Failover Simulation:");
    println!("   ⚠️  Primary service failed, switching to backup...");
    
    let backup_query = PrimalQuery::new("backup-service".to_string());
    let backup_services = provider.query_primal(backup_query).await?;
    
    if let Some(backup) = backup_services.first() {
        println!("   ✅ Failover successful: {} -> {}", backup.name, backup.primary_endpoint);
    }
    
    Ok(())
}

/// Demonstrate real-time updates
async fn demonstrate_real_time_updates() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔄 Demonstrating real-time updates...");
    
    // Create service discovery with heartbeat
    let discovery = Arc::new(InMemoryServiceDiscovery::with_heartbeat_timeout(Duration::from_secs(5)));
    let registry = ServiceRegistry::new(discovery.clone());
    
    // Register a service
    let service = ServiceDefinition::new(
        "real-time-service".to_string(),
        "Real-time Service".to_string(),
        ServiceType::AI,
        vec![
            ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            ).as_primary(),
        ],
    )
    .with_capability("real-time".to_string());
    
    registry.register_local_service(service).await?;
    
    println!("\n🔄 Real-time Updates Demonstration:");
    
    // Start heartbeat
    registry.start_heartbeat_loop().await?;
    
    // Monitor service status
    let mut interval = interval(Duration::from_secs(1));
    let mut count = 0;
    
    while count < 10 {
        interval.tick().await;
        
        let services = discovery.get_active_services().await?;
        let service_count = services.len();
        
        println!("   📊 Time: {}s, Active services: {}", count + 1, service_count);
        
        if let Some(service) = services.first() {
            let time_since_heartbeat = chrono::Utc::now()
                .signed_duration_since(service.last_heartbeat)
                .num_seconds();
            
            println!("      💓 Last heartbeat: {}s ago", time_since_heartbeat);
        }
        
        count += 1;
    }
    
    // Simulate service going down
    println!("\n💥 Simulating service shutdown...");
    registry.shutdown().await?;
    
    // Wait for cleanup
    sleep(Duration::from_secs(2)).await;
    
    let services = discovery.get_active_services().await?;
    println!("   📊 Active services after shutdown: {}", services.len());
    
    Ok(())
}

/// Helper function to create a mock primal info
fn create_mock_primal(name: &str, service_type: &str, endpoint: &str) -> PrimalInfo {
    PrimalInfo {
        name: name.to_string(),
        service_type: service_type.to_string(),
        primary_endpoint: endpoint.to_string(),
        endpoints: vec![endpoint.to_string()],
        capabilities: vec!["mock".to_string()],
        metadata: HashMap::new(),
        tags: vec!["demo".to_string()],
        weight: Some(0.5),
        priority: Some(1),
        health_status: HealthStatus::Healthy,
    }
}

/// Helper function to create a mock health status
fn create_mock_health(service_name: &str, status: &str) -> squirrel::universal::HealthCheckResult {
    squirrel::universal::HealthCheckResult {
        service_name: service_name.to_string(),
        status: status.to_string(),
        last_check: chrono::Utc::now().to_rfc3339(),
        response_time: Some(50),
        endpoints: vec!["http://localhost:8080".to_string()],
        error: None,
    }
} 