//! Ecosystem Management - Refactored for Semantic Cohesion
//!
//! This module coordinates service discovery, health monitoring, lifecycle
//! management, and recovery within the ecoPrimals ecosystem. Following the
//! principle that "primal code has self-knowledge and discovers others at runtime",
//! this implementation uses capability-based discovery rather than hardcoded names.
//!
//! # Architecture
//!
//! The ecosystem manager is organized into semantic modules:
//! - `types`: Shared data structures
//! - `discovery`: Capability-based service discovery
//! - `health`: Health monitoring and status tracking
//! - `lifecycle`: Service startup, shutdown, and reload
//! - `recovery`: Circuit breakers and failover logic
//!
//! # Examples
//!
//! ```ignore
//! // Create and initialize ecosystem manager
//! let mut manager = EcosystemManager::new(config, metrics);
//! manager.initialize().await?;
//!
//! // Discover services by capability (NOT by name)
//! let security_services = manager.discover_by_capability("security").await?;
//!
//! // Register our service
//! manager.register_squirrel_service(&provider).await?;
//! ```

mod discovery;
mod health;
mod lifecycle;
mod recovery;
mod types;

// Re-export public types and functionality
pub use discovery::{DiscoveredService, ServiceDiscovery};
pub use health::HealthMonitor;
pub use lifecycle::LifecycleManager;
pub use recovery::RecoveryManager;
pub use types::*;

use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing;

use crate::error::PrimalError;
use crate::monitoring::MetricsCollector;
use crate::primal_provider::SquirrelPrimalProvider;
use crate::ecosystem::registry::EcosystemRegistryManager;
use crate::universal_primal_ecosystem::UniversalPrimalEcosystem;

/// Main ecosystem manager coordinating all ecosystem operations
///
/// This is the primary interface for ecosystem operations. It delegates
/// to specialized submodules for discovery, health, lifecycle, and recovery.
pub struct EcosystemManager {
    // Core dependencies
    registry_manager: Arc<EcosystemRegistryManager>,
    universal_ecosystem: UniversalPrimalEcosystem,
    config: EcosystemConfig,
    metrics_collector: Arc<MetricsCollector>,
    
    // Specialized coordinators
    discovery: ServiceDiscovery,
    health: HealthMonitor,
    lifecycle: LifecycleManager,
    recovery: RecoveryManager,
    
    // Manager status
    status: Arc<RwLock<EcosystemManagerStatus>>,
}

impl EcosystemManager {
    /// Create new ecosystem manager
    ///
    /// Initializes all subcomponents but does not start services.
    /// Call `initialize()` to complete setup.
    pub fn new(config: EcosystemConfig, metrics_collector: Arc<MetricsCollector>) -> Self {
        // Initialize registry manager
        let (registry_manager, _registry_receiver) =
            EcosystemRegistryManager::new(config.registry_config.clone());
        let registry_manager = Arc::new(registry_manager);

        // Initialize universal primal ecosystem
        let primal_context = crate::universal::PrimalContext {
            user_id: "squirrel".to_string(),
            device_id: uuid::Uuid::new_v4().to_string(),
            network_location: crate::universal::NetworkLocation {
                region: "local".to_string(),
                data_center: None,
                availability_zone: None,
                ip_address: Some("127.0.0.1".to_string()),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: crate::universal::SecurityLevel::Internal,
            biome_id: Some("squirrel-ecosystem".to_string()),
            session_id: Some(uuid::Uuid::new_v4().to_string()),
            metadata: std::collections::HashMap::new(),
        };
        let universal_ecosystem = UniversalPrimalEcosystem::new(primal_context);

        // Initialize specialized coordinators
        let discovery = ServiceDiscovery::new(
            Arc::clone(&registry_manager),
            Arc::clone(&metrics_collector),
        );
        
        let health = HealthMonitor::new(Arc::clone(&metrics_collector));
        let lifecycle = LifecycleManager::new(Arc::clone(&metrics_collector));
        let recovery = RecoveryManager::new(Arc::clone(&metrics_collector));

        // Initialize status
        let status = EcosystemManagerStatus {
            status: "initializing".to_string(),
            initialized_at: None,
            last_registration: None,
            active_registrations: Vec::new(),
            health_status: HealthStatus {
                health_score: 0.0,
                component_statuses: std::collections::HashMap::new(),
                last_check: Utc::now(),
                health_errors: Vec::new(),
            },
            error_count: 0,
            last_error: None,
        };

        Self {
            registry_manager,
            universal_ecosystem,
            config,
            metrics_collector,
            discovery,
            health,
            lifecycle,
            recovery,
            status: Arc::new(RwLock::new(status)),
        }
    }

    /// Initialize the ecosystem manager
    ///
    /// Completes initialization of all subcomponents and marks the
    /// manager as ready for operation.
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        tracing::info!("Initializing ecosystem manager with universal patterns");

        // Initialize registry manager
        self.registry_manager.initialize().await?;

        // Initialize universal primal ecosystem
        self.universal_ecosystem.initialize().await?;

        // Initialize lifecycle manager
        self.lifecycle.initialize().await?;

        // Update status
        let mut status = self.status.write().await;
        status.status = "initialized".to_string();
        status.initialized_at = Some(Utc::now());

        tracing::info!("Ecosystem manager initialized successfully");
        Ok(())
    }

    /// Register Squirrel service with ecosystem
    ///
    /// Registers our own service capabilities with the ecosystem,
    /// making us discoverable by other primals.
    pub async fn register_squirrel_service(
        &self,
        provider: &SquirrelPrimalProvider,
    ) -> Result<(), PrimalError> {
        tracing::info!("Registering Squirrel service with ecosystem");

        // Create service registration
        let registration = self.discovery.create_registration(provider)?;

        // Register through discovery coordinator
        self.discovery.register_service(registration).await?;

        // Update status
        let mut status = self.status.write().await;
        status.last_registration = Some(Utc::now());
        status.active_registrations.push(provider.config().service_id.clone());

        Ok(())
    }

    /// Discover services by capability (NOT by primal name)
    ///
    /// This is the recommended way to find services - by what they can do,
    /// not by who provides them. Follows the self-knowledge principle.
    ///
    /// # Example
    /// ```ignore
    /// // CORRECT: Discover by capability
    /// let security_services = manager.discover_by_capability("security").await?;
    /// let best_security = manager.select_best_service(&security_services).await;
    ///
    /// // WRONG: Don't hardcode primal names
    /// // let beardog = manager.get_service("beardog").await?;
    /// ```
    pub async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>, PrimalError> {
        self.discovery.discover_by_capability(capability).await
    }

    /// Select best service from discovered options
    ///
    /// Uses health and load information to select the optimal service.
    pub async fn select_best_service(
        &self,
        services: &[DiscoveredService],
    ) -> Option<DiscoveredService> {
        self.discovery.select_best_service(services).await
    }

    /// Get current health status
    ///
    /// Returns aggregated health information for the entire ecosystem.
    pub async fn get_health_status(&self) -> HealthStatus {
        self.health.get_health_status().await
    }

    /// Update health for a component
    pub async fn update_component_health(
        &self,
        component_id: &str,
        status: String,
        error: Option<String>,
    ) -> Result<(), PrimalError> {
        self.health
            .update_component_health(component_id, status, error)
            .await
    }

    /// Get manager status
    pub async fn get_manager_status(&self) -> EcosystemManagerStatus {
        let status = self.status.read().await;
        status.clone()
    }

    /// Graceful shutdown
    ///
    /// Stops all services and cleans up resources.
    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        tracing::info!("Shutting down ecosystem manager");

        // Shutdown all services through lifecycle manager
        self.lifecycle.shutdown_all().await?;

        // Update status
        let mut status = self.status.write().await;
        status.status = "shutdown".to_string();

        Ok(())
    }

    // Backward compatibility methods - delegate to new structure
    // TODO: These can be deprecated in favor of direct module access

    /// Legacy method - use discover_by_capability instead
    pub async fn discover_services(&self) -> Result<Vec<DiscoveredService>, PrimalError> {
        // Return all registered services
        Ok(self.registry_manager
            .get_discovered_services()
            .await
            .into_iter()
            .map(|arc| (*arc).clone())
            .collect())
    }
}

// Re-export configuration for backward compatibility
pub use crate::ecosystem::registry::config::RegistryConfig;

