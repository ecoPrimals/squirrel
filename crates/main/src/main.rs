use anyhow::Result;
use ecosystem_api::types::{NetworkLocation, PrimalContext};
use squirrel::api::ApiServer;
use squirrel::ecosystem::{initialize_ecosystem_integration, EcosystemConfig};
use squirrel::shutdown::{ShutdownConfig, ShutdownManager};
use squirrel::universal_adapter::run_universal_adapter;
use squirrel::universal_provider::UniversalSquirrelProvider;
use squirrel::MetricsCollector;
use squirrel::UniversalProviderTrait;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("squirrel=info,debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    // Check if we should use the new universal adapter
    let use_universal_adapter = std::env::var("SQUIRREL_USE_UNIVERSAL_ADAPTER")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if use_universal_adapter {
        info!("Starting Squirrel Universal Adapter (new implementation)");
        return run_universal_adapter().await;
    }

    info!("Starting Squirrel Universal AI Primal with ecosystem-api integration");

    // Create ecosystem configuration from environment variables
    let ecosystem_api_config = EcosystemConfig::default();
    info!("Using default ecosystem configuration");

    // Create legacy ecosystem configuration for backward compatibility
    let mut ecosystem_config = EcosystemConfig::default();
    ecosystem_config.registry_config.songbird_endpoint =
        ecosystem_api_config.songbird_endpoint.clone();
    ecosystem_config.biome_id = ecosystem_api_config.biome_id.clone();

    info!("Configuration loaded:");
    info!("  Service name: {}", ecosystem_api_config.service_name);
    info!("  Service host: {}", ecosystem_api_config.service_host);
    info!("  Service port: {}", ecosystem_api_config.service_port);
    info!(
        "  Songbird endpoint: {}",
        ecosystem_api_config.songbird_endpoint
    );
    if let Some(biome_id) = &ecosystem_api_config.biome_id {
        info!("  Biome ID: {}", biome_id);
    }

    // Initialize metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new());

    // Initialize shutdown manager
    let shutdown_config = ShutdownConfig {
        graceful_timeout: Duration::from_secs(30),
        forceful_timeout: Duration::from_secs(10),
        persist_state: true,
        wait_for_operations: true,
        parallel_shutdown: true,
        max_concurrent_shutdowns: 10,
    };
    let shutdown_manager = Arc::new(ShutdownManager::new(shutdown_config));

    // Initialize ecosystem manager for backward compatibility
    info!("Initializing ecosystem manager with Songbird service mesh...");
    let ecosystem_manager = Arc::new(
        initialize_ecosystem_integration(ecosystem_config.clone(), metrics_collector.clone())
            .await?,
    );

    // Create primal context
    let primal_context = PrimalContext {
        user_id: std::env::var("USER_ID").unwrap_or_else(|_| "default_user".to_string()),
        device_id: std::env::var("DEVICE_ID").unwrap_or_else(|_| "default_device".to_string()),
        network_location: NetworkLocation {
            ip_address: Some(
                std::env::var("NETWORK_IP").unwrap_or_else(|_| "127.0.0.1".to_string()),
            ),
            region: Some(std::env::var("NETWORK_REGION").unwrap_or_else(|_| "local".to_string())),
            zone: Some(std::env::var("NETWORK_ZONE").unwrap_or_else(|_| "default".to_string())),
            segment: Some(
                std::env::var("NETWORK_SEGMENT").unwrap_or_else(|_| "default".to_string()),
            ),
        },
        session_id: std::env::var("SESSION_ID")
            .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
        security_level: match std::env::var("SECURITY_LEVEL").as_deref() {
            Ok("public") => ecosystem_api::SecurityLevel::Public,
            Ok("internal") => ecosystem_api::SecurityLevel::Internal,
            Ok("restricted") => ecosystem_api::SecurityLevel::Restricted,
            Ok("confidential") => ecosystem_api::SecurityLevel::Confidential,
            _ => ecosystem_api::SecurityLevel::Internal,
        },
        biome_id: ecosystem_api_config.biome_id.clone(),
        metadata: std::collections::HashMap::new(),
    };

    // Create universal Squirrel provider using ecosystem-api
    info!("Creating universal Squirrel provider...");
    let mut universal_provider =
        UniversalSquirrelProvider::new(ecosystem_api_config.clone(), primal_context)
            .map_err(|e| anyhow::anyhow!("Failed to create universal provider: {}", e))?;

    // Initialize the universal provider
    let init_config = serde_json::json!({
        "service_name": ecosystem_api_config.service_name,
        "service_host": ecosystem_api_config.service_host,
        "service_port": ecosystem_api_config.service_port,
        "songbird_endpoint": ecosystem_api_config.songbird_endpoint,
        "biome_id": ecosystem_api_config.biome_id,
        "security_level": "standard",
        "service_mesh_enabled": true
    });

    info!("Initializing universal Squirrel provider...");
    universal_provider
        .initialize(init_config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize universal provider: {}", e))?;

    // Register with ecosystem through service mesh
    info!("Registering Squirrel service with ecosystem...");
    match universal_provider.register_with_ecosystem().await {
        Ok(registration) => {
            info!("Successfully registered with ecosystem service mesh");
            info!("  Service ID: {}", registration.service_id);
            info!("  Service type: {:?}", registration.service_id);
            info!("  Capabilities: {:?}", registration.capabilities);
        }
        Err(e) => {
            warn!("Failed to register with ecosystem service mesh: {}", e);
            info!("Continuing in standalone mode...");
        }
    }

    // Create API server with ecosystem integration
    let api_server = ApiServer::new(
        ecosystem_api_config.service_port,
        ecosystem_manager.clone(),
        metrics_collector.clone(),
        shutdown_manager.clone(),
    );

    // Setup graceful shutdown
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    };

    info!("Squirrel primal ecosystem started successfully");
    info!(
        "API server available at http://{}:{}",
        ecosystem_api_config.service_host, ecosystem_api_config.service_port
    );
    info!(
        "Health check endpoint: http://{}:{}/health",
        ecosystem_api_config.service_host, ecosystem_api_config.service_port
    );
    info!(
        "Ecosystem status: http://{}:{}/api/v1/ecosystem/status",
        ecosystem_api_config.service_host, ecosystem_api_config.service_port
    );
    info!(
        "Service mesh status: http://{}:{}/api/v1/service-mesh/status",
        ecosystem_api_config.service_host, ecosystem_api_config.service_port
    );
    info!("Songbird integration: enabled");

    // Start heartbeat task
    let heartbeat_provider = Arc::new(tokio::sync::Mutex::new(universal_provider));
    let heartbeat_task = {
        let provider = heartbeat_provider.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Ok(mut provider) = provider.try_lock() {
                    if let Err(e) = provider.send_heartbeat().await {
                        warn!("Failed to send heartbeat: {}", e);
                    }
                }
            }
        })
    };

    // Run the API server with graceful shutdown
    tokio::select! {
        result = api_server.start() => {
            if let Err(e) = result {
                error!("API server error: {}", e);
            }
        }
        _ = shutdown_signal => {
            info!("Received shutdown signal, initiating graceful shutdown...");

            // Cancel heartbeat task
            heartbeat_task.abort();

            // Shutdown universal provider
            if let Ok(mut provider) = heartbeat_provider.try_lock() {
                if let Err(e) = provider.shutdown().await {
                    error!("Error shutting down universal provider: {}", e);
                }

                // Deregister from ecosystem
                if let Err(e) = provider.deregister_from_ecosystem().await {
                    error!("Error deregistering from ecosystem: {}", e);
                }
            }

            info!("Squirrel primal shutdown completed");
        }
    }

    Ok(())
}
