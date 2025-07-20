//! Universal Adapter for Squirrel AI Primal
//!
//! This adapter implements the complete ecosystem integration patterns,
//! bringing together all components for seamless operation within the ecoPrimals ecosystem.
//!
//! ## Architecture
//!
//! The adapter follows the Songbird-centric communication model:
//! ```
//! biomeOS → Songbird (Service Mesh) → Squirrel Universal Adapter
//!                                           ↓
//!                        AI Coordination + MCP Protocol + Session Management
//! ```

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::ecosystem::{EcosystemConfig, EcosystemManager};
use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;
use crate::self_healing::SelfHealingManager;
use crate::shutdown::ShutdownManager;
use crate::universal::SecurityLevel;
use crate::universal_provider::UniversalSquirrelProvider;
use ecosystem_api::{traits::UniversalPrimalProvider, types::*};
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::api::ApiServer;

/// Universal Adapter for Squirrel AI Primal
///
/// This adapter orchestrates all components and provides a unified interface
/// for ecosystem integration following Songbird patterns.
pub struct SquirrelUniversalAdapter {
    /// Configuration for ecosystem integration
    config: EcosystemConfig,
    /// Universal provider implementation
    universal_provider: Arc<RwLock<UniversalSquirrelProvider>>,
    /// Legacy ecosystem manager for backward compatibility
    ecosystem_manager: Arc<EcosystemManager>,
    /// Metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// Self-healing manager
    self_healing_manager: Arc<SelfHealingManager>,
    /// Shutdown manager
    shutdown_manager: Arc<ShutdownManager>,
    /// API server
    api_server: Option<ApiServer>,
    /// Initialization state
    initialized: bool,
    /// Shutdown state
    shutdown: bool,
    /// Service registration information
    service_registration: Option<ecosystem_api::EcosystemServiceRegistration>,
    /// Start time for uptime calculation
    start_time: std::time::Instant,
}

impl SquirrelUniversalAdapter {
    /// Create a new universal adapter
    pub fn new(config: EcosystemConfig) -> Result<Self> {
        // Create primal context from environment
        let primal_context = PrimalContext {
            user_id: std::env::var("USER_ID").unwrap_or_else(|_| "default_user".to_string()),
            device_id: std::env::var("DEVICE_ID").unwrap_or_else(|_| "default_device".to_string()),
            network_location: NetworkLocation {
                ip_address: Some(std::env::var("NETWORK_IP").unwrap_or_else(|_| {
                    squirrel_mcp_config::core::network_defaults::DEFAULT_HOST.to_string()
                })),
                region: Some(
                    std::env::var("NETWORK_REGION").unwrap_or_else(|_| "local".to_string()),
                ),
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
            biome_id: config.biome_id.clone(),
            metadata: std::collections::HashMap::new(),
        };

        // Initialize universal provider
        let universal_provider = UniversalSquirrelProvider::new(config.clone(), primal_context)
            .map_err(|e| anyhow::anyhow!("Failed to create universal provider: {}", e))?;

        // Initialize components
        let metrics_collector = Arc::new(MetricsCollector::new());
        let self_healing_manager = Arc::new(SelfHealingManager::new(
            crate::self_healing::SelfHealingConfig::default(),
        ));
        let shutdown_manager = Arc::new(ShutdownManager::new(
            crate::shutdown::ShutdownConfig::default(),
        ));

        // Create legacy ecosystem manager for backward compatibility
        let legacy_config = crate::ecosystem::EcosystemConfig {
            biome_id: config.biome_id.clone(),
            registry_config: crate::ecosystem::EcosystemRegistryConfig {
                songbird_endpoint: config.songbird_endpoint.clone(),
                ..Default::default()
            },
            ..Default::default()
        };

        let ecosystem_manager = Arc::new(crate::ecosystem::EcosystemManager::new(
            legacy_config,
            metrics_collector.clone(),
        ));

        Ok(Self {
            config,
            universal_provider: Arc::new(RwLock::new(universal_provider)),
            ecosystem_manager,
            metrics_collector,
            self_healing_manager,
            shutdown_manager,
            api_server: None,
            initialized: false,
            shutdown: false,
            service_registration: None,
            start_time: std::time::Instant::now(),
        })
    }

    /// Initialize the universal adapter
    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        info!("Initializing Squirrel Universal Adapter");

        // Initialize ecosystem manager
        self.ecosystem_manager
            .initialize()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize ecosystem manager: {}", e))?;

        // Initialize universal provider
        let init_config = json!({
            "service_name": self.config.service_name,
            "service_host": self.config.service_host,
            "service_port": self.config.service_port,
            "songbird_endpoint": self.config.songbird_endpoint,
            "biome_id": self.config.biome_id,
            "security_level": "standard",
            "service_mesh_enabled": true
        });

        {
            let mut provider = self.universal_provider.write().await;
            provider
                .initialize(init_config)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize universal provider: {}", e))?;
        }

        // Initialize API server
        let api_server = ApiServer::new_with_host(
            self.config.service_host.clone(),
            self.config.service_port,
            self.ecosystem_manager.clone(),
            self.metrics_collector.clone(),
            self.shutdown_manager.clone(),
        );

        self.api_server = Some(api_server);
        self.initialized = true;

        info!("Squirrel Universal Adapter initialized successfully");
        Ok(())
    }

    /// Register with the ecosystem
    pub async fn register_with_ecosystem(
        &mut self,
    ) -> Result<ecosystem_api::EcosystemServiceRegistration> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Adapter not initialized"));
        }

        info!("Registering with ecosystem service mesh");

        let registration = {
            let mut provider = self.universal_provider.write().await;
            provider
                .register_with_ecosystem()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to register with ecosystem: {}", e))?
        };

        self.service_registration = Some(registration.clone());

        info!("Successfully registered with ecosystem service mesh");
        info!("  Service ID: {}", registration.service_id);
        info!("  Primal type: {:?}", registration.primal_type);
        info!("  Capabilities: {:?}", registration.capabilities);

        Ok(registration)
    }

    /// Start the adapter services
    pub async fn start(&self) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Adapter not initialized"));
        }

        info!("Starting Squirrel Universal Adapter services");

        // Start heartbeat task
        let heartbeat_provider = self.universal_provider.clone();
        let heartbeat_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Ok(provider) = heartbeat_provider.try_read() {
                    if let Err(e) = provider.send_heartbeat().await {
                        warn!("Failed to send heartbeat: {}", e);
                    }
                }
            }
        });

        // Start API server
        if let Some(api_server) = &self.api_server {
            info!(
                "Starting API server on {}:{}",
                self.config.service_host, self.config.service_port
            );

            // Run API server in background
            let server_future = api_server.start();
            tokio::pin!(server_future);

            // Wait for shutdown signal
            let shutdown_signal = async {
                match tokio::signal::ctrl_c().await {
                    Ok(()) => {
                        info!("Received shutdown signal");
                    }
                    Err(e) => {
                        error!("Failed to install CTRL+C signal handler: {}", e);
                        // Fall back to timeout-based shutdown
                        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    }
                }
            };

            tokio::select! {
                result = server_future => {
                    if let Err(e) = result {
                        error!("API server error: {}", e);
                    }
                }
                _ = shutdown_signal => {
                    info!("Received shutdown signal, initiating graceful shutdown...");
                    heartbeat_task.abort();
                    // Shutdown will be handled by the shutdown method
                }
            }
        }

        Ok(())
    }

    /// Get adapter status
    pub async fn get_status(&self) -> AdapterStatus {
        let provider_status = if let Ok(provider) = self.universal_provider.try_read() {
            let health = provider.health_check().await;
            health.status
        } else {
            ecosystem_api::HealthStatus::Unhealthy
        };

        let ecosystem_health = {
            let status = self.ecosystem_manager.get_ecosystem_status().await;
            status.overall_health
        };

        AdapterStatus {
            initialized: self.initialized,
            provider_health: provider_status,
            ecosystem_health,
            service_registration: self.service_registration.clone(),
            api_server_running: self.api_server.is_some(),
            uptime: self.start_time.elapsed().as_secs(),
        }
    }

    /// Shutdown the adapter
    pub async fn shutdown(&mut self) -> Result<()> {
        if self.shutdown {
            return Ok(());
        }

        info!("Shutting down Squirrel Universal Adapter");

        self.shutdown = true;

        // Deregister from ecosystem
        if let Ok(mut provider) = self.universal_provider.try_write() {
            if let Err(e) = provider.deregister_from_ecosystem().await {
                error!("Error deregistering from ecosystem: {}", e);
            }

            if let Err(e) = provider.shutdown().await {
                error!("Error shutting down universal provider: {}", e);
            }
        }

        // Shutdown ecosystem manager
        if let Err(e) = self.ecosystem_manager.shutdown().await {
            error!("Error shutting down ecosystem manager: {}", e);
        }

        info!("Squirrel Universal Adapter shutdown completed");
        Ok(())
    }

    /// Get the universal provider
    pub fn get_universal_provider(&self) -> Arc<RwLock<UniversalSquirrelProvider>> {
        self.universal_provider.clone()
    }

    /// Get the ecosystem manager
    pub fn get_ecosystem_manager(&self) -> Arc<EcosystemManager> {
        self.ecosystem_manager.clone()
    }

    /// Get the metrics collector
    pub fn get_metrics_collector(&self) -> Arc<MetricsCollector> {
        self.metrics_collector.clone()
    }

    /// Get the configuration
    pub fn get_config(&self) -> &EcosystemConfig {
        &self.config
    }
}

/// Adapter status information
#[derive(Debug, Clone)]
pub struct AdapterStatus {
    /// Whether the adapter is initialized
    pub initialized: bool,
    /// Provider health status
    pub provider_health: ecosystem_api::HealthStatus,
    /// Ecosystem health score
    pub ecosystem_health: f64,
    /// Service registration information
    pub service_registration: Option<ecosystem_api::EcosystemServiceRegistration>,
    /// Whether the API server is running
    pub api_server_running: bool,
    /// Uptime in seconds
    pub uptime: u64,
}

/// Create a universal adapter from environment variables
pub async fn create_universal_adapter_from_env() -> Result<SquirrelUniversalAdapter> {
    let mut config = EcosystemConfig::default();

    // Load configuration from environment variables
    if let Ok(service_name) = std::env::var("SERVICE_NAME") {
        config.service_name = service_name;
    }
    if let Ok(service_host) = std::env::var("SERVICE_HOST") {
        config.service_host = service_host;
    }
    if let Ok(service_port) = std::env::var("SERVICE_PORT") {
        config.service_port = service_port.parse().unwrap_or(8080);
    }
    if let Ok(songbird_endpoint) = std::env::var("SONGBIRD_ENDPOINT") {
        config.songbird_endpoint = songbird_endpoint;
    }
    if let Ok(biome_id) = std::env::var("BIOME_ID") {
        config.biome_id = Some(biome_id);
    }

    SquirrelUniversalAdapter::new(config)
}

/// Create a universal adapter with default configuration
pub fn create_default_universal_adapter() -> Result<SquirrelUniversalAdapter> {
    let config = EcosystemConfig::default();
    SquirrelUniversalAdapter::new(config)
}

/// Initialize and start the universal adapter
pub async fn run_universal_adapter() -> Result<()> {
    let mut adapter = create_universal_adapter_from_env().await?;

    adapter.initialize().await?;
    adapter.register_with_ecosystem().await?;
    adapter.start().await?;

    Ok(())
}
