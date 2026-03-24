// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![forbid(unsafe_code)]

//! Squirrel MCP Server - Universal Swarm MCP Agent System
//!
//! This binary starts the Squirrel MCP server for multi-MCP coordination,
//! ecosystem participation, and federation.

use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use squirrel_core::{
    EcosystemConfig, EcosystemService, FederationService, McpRoutingService, MonitoringConfig,
    MonitoringService, Result, SongbirdConfig, api::ApiServer, federation::FederationConfig,
    routing::RoutingConfig,
};

/// Universal Swarm MCP Agent System
///
/// Squirrel MCP operates as:
/// - **Sovereign Multi-MCP Coordinator**: Routes AI tasks across multiple MCP endpoints
/// - **Ecosystem Participant**: Coordinates with Songbird, `NestGate`, `BearDog`, `ToadStool`
/// - **Federation Leader**: Spawns additional Squirrel instances for scaling
/// - **Universal Agent**: Can federate across nodes for distributed AI processing
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "squirrel_mcp=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🐿️  Starting Squirrel MCP - Universal Swarm MCP Agent System");
    info!("Version: {}", squirrel_core::SQUIRREL_MCP_VERSION);

    // Load configuration
    let config = load_configuration();
    info!("Configuration loaded successfully");

    // Initialize monitoring service first (delegates to Songbird when available)
    info!("Initializing universal monitoring service...");
    let monitoring_service = Arc::new(MonitoringService::new(config.monitoring));
    monitoring_service.initialize()?;
    info!("✅ Monitoring service initialized with delegation to external providers");

    // Initialize core services
    info!("Initializing core services...");

    // 1. Ecosystem Service - Handles sovereign operation and primal coordination
    let ecosystem_service = Arc::new(EcosystemService::new(
        config.ecosystem,
        monitoring_service.clone(),
    )?);
    info!("✅ Ecosystem service initialized");

    // 2. MCP Routing Service - Handles multi-MCP coordination and AI task routing
    let routing_service = Arc::new(McpRoutingService::new(config.routing)?);
    info!("✅ MCP routing service initialized");

    // 3. Federation Service - Handles scaling and node coordination
    let federation_service = Arc::new(FederationService::new(config.federation)?);
    info!("✅ Federation service initialized");

    // 4. API Server - Provides HTTP endpoints for coordination
    let api_server = ApiServer::new(
        ecosystem_service.clone(),
        routing_service.clone(),
        federation_service.clone(),
    );
    info!("✅ API server initialized");

    // Start services
    info!("Starting services...");

    // Start ecosystem service (primal coordination)
    ecosystem_service.start().await?;
    info!(
        "🌍 Ecosystem service started - {} mode",
        format!("{:?}", ecosystem_service.get_status())
    );

    // Start routing service (MCP coordination)
    routing_service.start()?;
    info!("🚀 MCP routing service started");

    // Start federation service (scaling & federation)
    federation_service.start().await?;
    info!("🤝 Federation service started");

    // Print startup summary
    print_startup_summary(
        &ecosystem_service,
        &routing_service,
        &federation_service,
        &monitoring_service,
    )
    .await;

    // Start API server
    let bind_addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| universal_constants::network::default_api_bind_addr());
    let bind_addr_clone = bind_addr.clone();
    info!("🌐 Starting API server on {}", bind_addr);

    // Run API server with graceful shutdown
    let api_task = tokio::spawn(async move {
        if let Err(e) = api_server.start(&bind_addr_clone).await {
            error!("API server failed: {}", e);
        }
    });

    // Wait for shutdown signal
    info!("🎯 Squirrel MCP ready for universal swarm coordination!");
    info!("📡 Endpoints:");
    info!("   Health:     GET  http://{}/health", bind_addr);
    info!("   MCP Route:  POST http://{}/api/v1/mcp/route", bind_addr);
    info!(
        "   Federation: GET  http://{}/api/v1/federation/info",
        bind_addr
    );
    info!(
        "   Discovery:  POST http://{}/api/v1/ecosystem/discover",
        bind_addr
    );
    info!(
        "   Monitoring: GET  http://{}/api/v1/monitoring/status",
        bind_addr
    );

    // Graceful shutdown handling
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("🛑 Received interrupt signal, starting graceful shutdown");
        }
        result = api_task => {
            if let Err(e) = result {
                error!("API server task failed: {:?}", e);
            }
        }
    }

    // Shutdown services in reverse order
    info!("Shutting down services...");

    if let Err(e) = federation_service.shutdown().await {
        warn!("Federation service shutdown error: {}", e);
    }

    if let Err(e) = ecosystem_service.shutdown().await {
        warn!("Ecosystem service shutdown error: {}", e);
    }

    info!("🏁 Squirrel MCP shutdown complete");
    Ok(())
}

/// Configuration structure combining all service configs
struct SquirrelConfig {
    ecosystem: EcosystemConfig,
    routing: RoutingConfig,
    federation: FederationConfig,
    monitoring: MonitoringConfig,
}

/// Load configuration from environment and files
fn load_configuration() -> SquirrelConfig {
    // Create monitoring configuration with Songbird delegation
    let monitoring_config = MonitoringConfig {
        enabled: std::env::var("MONITORING_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        require_provider: std::env::var("MONITORING_REQUIRE_PROVIDER")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false),
        songbird_config: std::env::var("SONGBIRD_ENDPOINT")
            .ok()
            .map(|songbird_endpoint| SongbirdConfig {
                endpoint: songbird_endpoint,
                service_name: "squirrel-mcp".to_string(),
                auth_token: std::env::var("SONGBIRD_AUTH_TOKEN").ok(),
                batch_size: std::env::var("SONGBIRD_BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                flush_interval: std::time::Duration::from_secs(
                    std::env::var("SONGBIRD_FLUSH_INTERVAL")
                        .unwrap_or_else(|_| "30".to_string())
                        .parse()
                        .unwrap_or(30),
                ),
            }),
        ..Default::default()
    };

    SquirrelConfig {
        ecosystem: EcosystemConfig::default(),
        routing: RoutingConfig::default(),
        federation: FederationConfig::default(),
        monitoring: monitoring_config,
    }
}

/// Print startup summary with monitoring status
async fn print_startup_summary(
    ecosystem: &EcosystemService,
    routing: &McpRoutingService,
    federation: &FederationService,
    monitoring: &MonitoringService,
) {
    info!("🎯 === Squirrel MCP Startup Summary ===");

    // Ecosystem status
    let ecosystem_status = ecosystem.get_status();
    let discovered_primals = ecosystem.get_discovered_primals();
    info!(
        "🌍 Ecosystem: {:?} ({} primals discovered)",
        ecosystem_status,
        discovered_primals.len()
    );

    // Routing status
    let routing_status = routing.get_stats();
    info!(
        "🚀 MCP Routing: {} agents registered",
        routing_status.registered_agents
    );

    // Federation status
    let federation_status = federation.get_federation_stats();
    info!(
        "🤝 Federation: {} instances, {} nodes",
        federation_status.local_instances, federation_status.federation_nodes
    );

    // Monitoring status
    let monitoring_status = monitoring.get_status().await;
    info!(
        "📊 Monitoring: {} providers active, fallback: {}",
        monitoring_status.provider_count, monitoring_status.fallback_active
    );
    for provider in &monitoring_status.providers {
        info!(
            "   📈 Provider: {} v{} - {:?}",
            provider.name, provider.version, provider.health
        );
    }

    info!("🎯 === Squirrel MCP Ready for Universal Coordination ===");
}
